import hashlib
import json
import shutil
import subprocess
import sys
from zipfile import ZipFile

import pytest

from tests.helpers import REPO_ROOT
from tools.prepare_public_zip import audit, prepare, tracked_path


def git(repo, *args):
    return subprocess.check_output(["git", *args], cwd=repo, text=True).strip()


@pytest.fixture
def release(tmp_path):
    repo = tmp_path / "checkout"
    repo.mkdir()
    git(repo, "init", "-q")
    git(repo, "config", "user.email", "fixture@example.invalid")
    git(repo, "config", "user.name", "Fixture")
    git(repo, "config", "core.autocrlf", "false")
    (repo / "tools").mkdir()
    shutil.copyfile(REPO_ROOT / "tools/build.py", repo / "tools/build.py")
    (repo / "VERSION").write_text("1.0.3\n")
    for name, ver in (("SensorList", "1.0.1"), ("ethos_events", "0.1.0"), ("BoundryMap", "0.2.0")):
        script = repo / "scripts" / name
        script.mkdir(parents=True)
        (script / "VERSION").write_text(ver + "\n")
        (script / "main.lua").write_bytes(b"-- committed script\n")
        (script / "icon.png").write_bytes(b"\x89PNG\r\n\x00binary marker")
    boundry = repo / "scripts/BoundryMap"
    for name in ("build.json", ".gitignore"):
        shutil.copyfile(REPO_ROOT / "scripts/BoundryMap" / name, boundry / name)
    git(repo, "add", ".")
    git(repo, "commit", "-qm", "fixture release")
    commit = git(repo, "rev-parse", "HEAD")
    # Move HEAD forward: provenance must come from the chosen commit, not current HEAD/index.
    (boundry / "main.lua").write_text("-- newer committed source\n")
    git(repo, "commit", "-qam", "newer commit")
    for name in ("SensorList", "BoundryMap"):
        path = repo / "scripts" / name
        (path / "VERSION").write_text("9.9.9\n")
        (path / "main.lua").write_text("-- dirty source\n")
    maps = boundry / "assets/maps/fake"
    maps.mkdir(parents=True)
    for name in ("fixture.png", "fixture.json", "fixture.boundries.json"):
        (maps / name).write_bytes(b"harmless ignored marker")
        assert git(repo, "check-ignore", str(maps / name))
    return repo, commit


@pytest.mark.parametrize(("project", "expected", "versions", "name"), [
    (None, "1.0.3", {"SensorList": "1.0.1", "ethos_events": "0.1.0"}, "sloppy-ethos_scripts.zip"),
    ("BoundryMap", "0.2.0", {"BoundryMap": "0.2.0"}, "BoundryMap-0.2.0.zip"),
])
def test_public_build_uses_selected_commit_and_never_overwrites(release, tmp_path, project, expected, versions, name):
    repo, commit = release
    # These local settings would omit an icon and rewrite Lua in a Git archive.
    (repo / ".git/info/attributes").write_bytes(b"*.png export-ignore\n*.lua text eol=crlf\n")
    git(repo, "config", "core.autocrlf", "true")
    dist = repo / "dist"
    dist.mkdir()
    old = dist / name
    with ZipFile(old, "w") as archive:
        archive.writestr("stale-marker", b"old bytes")
    stale_bytes = old.read_bytes()
    tags = git(repo, "tag")
    (repo / "tools/build.py").write_text('raise SystemExit("wrong working-tree builder")\n')
    # Exercise a bundle member too: an ordinary working-tree build would copy these inputs.
    (repo / ".git/info/exclude").write_text("*.private-marker\n")
    marker = repo / "scripts/SensorList/fixture.private-marker"
    marker.write_bytes(b"local bundle marker")
    assert git(repo, "check-ignore", str(marker))
    out = dist / "new-public"
    prepare(repo, commit, project, expected, out)
    fresh = out / name
    receipt = json.loads((out / "receipt.json").read_text())
    assert receipt["source_commit"] == commit != git(repo, "rev-parse", "HEAD")
    assert receipt["repository_version"] == "1.0.3"
    assert receipt["projects"] == versions
    assert receipt["archive"] == name
    assert receipt["sha256"] == hashlib.sha256(fresh.read_bytes()).hexdigest()
    with ZipFile(fresh) as archive:
        assert sorted(archive.namelist()) == receipt["entries"]
        assert "stale-marker" not in archive.namelist()
        assert not any("fixture" in n or "maps/" in n for n in archive.namelist())
        installed = {n.split("/")[1] for n in archive.namelist() if n.endswith("/main.lua")}
        assert installed == set(versions)
        for p, v in versions.items():
            assert archive.read(f"scripts/{p}/VERSION").decode().strip() == v
            for file in ("main.lua", "VERSION", "icon.png"):
                tracked = f"scripts/{p}/{file}"
                blob = subprocess.check_output(["git", "cat-file", "blob", f"{commit}:{tracked}"], cwd=repo)
                assert archive.read(tracked) == blob
    with pytest.raises(ValueError, match="already exists"):
        prepare(repo, commit, project, expected, dist)
    assert old.read_bytes() == stale_bytes
    assert git(repo, "tag") == tags
    assert (repo / "scripts/BoundryMap/assets/maps/fake/fixture.png").exists()


def test_personal_build_still_installs_ignored_maps(release, tmp_path):
    repo, _ = release
    out = tmp_path / "personal"
    subprocess.run([sys.executable, "tools/build.py", "--project", "BoundryMap", "--dist",
                    "--out-dir", str(out)], cwd=repo, check=True)
    with ZipFile(out / "BoundryMap-9.9.9.zip") as package:
        for name in ("fixture.png", "fixture.json", "fixture.boundries.json"):
            assert package.read(f"scripts/BoundryMap/assets/maps/{name}") == b"harmless ignored marker"


def test_forced_tracked_map_is_refused(release, tmp_path):
    repo, _ = release
    git(repo, "add", "-f", "scripts/BoundryMap/assets/maps/fake/fixture.png")
    git(repo, "commit", "-qm", "force-add fake map")
    out = tmp_path / "rejected"
    with pytest.raises(ValueError, match="Private map or sidecar in release inputs"):
        prepare(repo, "HEAD", "BoundryMap", "0.2.0", out)
    assert not out.exists()


def test_wrong_requested_version_is_refused(release, tmp_path):
    repo, commit = release
    out = tmp_path / "wrong-version"
    with pytest.raises(ValueError, match="Expected release version"):
        prepare(repo, commit, None, "9.9.9", out)
    assert not out.exists()


def test_tracked_symlink_is_refused_without_following_target(release, tmp_path):
    repo, _ = release
    # Install a link in the Git index without requiring OS symlink privileges.
    oid = subprocess.check_output(["git", "hash-object", "-w", "--stdin"], cwd=repo,
                                  input=b"../../outside").decode().strip()
    git(repo, "update-index", "--add", "--cacheinfo", "120000", oid, "scripts/BoundryMap/link")
    git(repo, "commit", "-qm", "synthetic link")
    out = tmp_path / "rejected-link"
    with pytest.raises(ValueError, match="regular files"):
        prepare(repo, "HEAD", "BoundryMap", "0.2.0", out)
    assert not out.exists()


@pytest.mark.parametrize("tail", ["../outside", "bad\\path", "C:/outside", "stream:ads", "aux.png"])
def test_nonportable_tree_paths_are_rejected(tail):
    with pytest.raises(ValueError, match="Unsafe release input"):
        tracked_path("scripts/BoundryMap/" + tail)


@pytest.mark.parametrize(("extra", "value", "error"), [
    ("scripts/BoundryMap/fixture.boundries.json", "{}", "Private map or sidecar"),
    ("scripts/Other/main.lua", "return {}", "Unexpected public ZIP entry"),
    ("scripts/BoundryMap/VERSION", "9.9.9", "Incorrect embedded version"),
])
def test_archive_audit_fails_closed(tmp_path, extra, value, error):
    path = tmp_path / "wrong.zip"
    files = {"scripts/BoundryMap/main.lua": "--\n", "scripts/BoundryMap/README.md": "# Test",
             "scripts/BoundryMap/VERSION": "0.2.0", extra: value}
    with ZipFile(path, "w") as archive:
        for name, contents in files.items():
            archive.writestr(name, contents)
    with pytest.raises(ValueError, match=error):
        audit(path, {"BoundryMap": "0.2.0"}, "1.0.3", False)
