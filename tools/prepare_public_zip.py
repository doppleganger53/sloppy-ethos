"""Build and audit one public install ZIP from a committed Git snapshot."""

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path, PureWindowsPath
from zipfile import BadZipFile, ZipFile


# The repository release ships this bundle; a script release names just one project.
BUNDLE_PROJECTS = ("SensorList", "ethos_events")


def private_map(name):
    parts = name.casefold().strip("/").split("/")
    return "maps" in parts or parts[-1].endswith((".boundries.json", ".boundaries.json"))


def tracked_path(name):
    parts = name.split("/")
    if any(p in {"", ".", ".."} or p.casefold() == ".git" or p.endswith((" ", "."))
           or re.search(r'[\\:<>"|?*\x00-\x1f]', p) or PureWindowsPath(p).is_reserved()
           for p in parts):
        raise ValueError(f"Unsafe release input path: {name!r}")
    return Path(*parts)


def snapshot(repo, ref, projects, dest):
    git = ["git", "--no-replace-objects"]
    commit = subprocess.check_output(
        [*git, "rev-parse", "--verify", "--end-of-options", ref + "^{commit}"],
        cwd=repo, text=True,
    ).strip()
    tree = subprocess.check_output(
        [*git, "ls-tree", "--full-tree", "-rz", commit,
         "--", "VERSION", "tools/build.py", *(f"scripts/{p}" for p in projects)],
        cwd=repo,
    )
    # git archive/checkout honors machine-local attributes and EOL filters. Read
    # raw blobs into a new private tree so only the named commit chooses bytes.
    dest.mkdir()
    for entry in tree.split(b"\0")[:-1]:  # ls-tree -z terminates each record with NUL.
        metadata, path_bytes = entry.split(b"\t", 1)
        mode, object_type, oid = metadata.split()
        name = path_bytes.decode("utf-8")
        if object_type != b"blob" or mode not in (b"100644", b"100755"):
            raise ValueError(f"Public inputs must be regular files: {name}")
        if private_map(name):
            raise ValueError(f"Private map or sidecar in release inputs: {name}")
        target = dest / tracked_path(name)
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as stream:
            subprocess.run([*git, "cat-file", "blob", oid.decode("ascii")], cwd=repo, stdout=stream, check=True)
    return commit


def audit(package, versions, repo_version, bundle):
    with ZipFile(package) as archive:
        entries = sorted(archive.namelist())
        if archive.testzip() or len({n.rstrip("/").casefold() for n in entries}) != len(entries):
            raise ValueError("Corrupt ZIP or duplicate install path.")
        for name in entries:
            parts = name.rstrip("/").split("/")
            if private_map(name):
                raise ValueError(f"Private map or sidecar in public ZIP: {name}")
            if name == "scripts/" or (bundle and name == "README.md"):
                continue
            if (len(parts) < 2 or parts[0] != "scripts" or parts[1] not in versions
                    or "\\" in name or set(parts) & {"", "..", ".", "tests", "__pycache__", "build.json"}
                    or name.lower().endswith(".pyc")):
                raise ValueError(f"Unexpected public ZIP entry: {name}")
        for project, version in versions.items():
            root = f"scripts/{project}/"
            required = {root + path for path in ("main.lua", "README.md", "VERSION")}
            if not required.issubset(entries):
                raise ValueError(f"Missing required {project} install files: {required.difference(entries)}")
            if archive.read(root + "VERSION").decode().strip() != version:
                raise ValueError(f"Incorrect embedded version for {project}; expected {version}")
        if bundle:
            if "README.md" not in entries:
                raise ValueError("Missing bundle manifest.")
            manifest = archive.read("README.md").decode().splitlines()
            expected = [f"- scripts/{p} (version {v})" for p, v in versions.items()]
            if ([line for line in manifest if line.startswith("- scripts/")] != expected
                    or [line for line in manifest if line.startswith("Repository version:")]
                    != [f"Repository version: {repo_version}"]):
                raise ValueError("Incorrect project list or versions in bundle manifest.")
    return entries


def prepare(repo, ref, project, expect_version, out_dir):
    bundle = project is None
    if not bundle and not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", project):
        raise ValueError("Use a single script directory name for --project.")
    if out_dir.exists():
        raise ValueError(f"Output directory already exists; choose a new path: {out_dir}")
    projects = BUNDLE_PROJECTS if bundle else (project,)
    with tempfile.TemporaryDirectory() as tmp:
        source, dist = Path(tmp) / "source", Path(tmp) / "dist"
        commit = snapshot(repo, ref, projects, source)
        repo_version = (source / "VERSION").read_text(encoding="utf-8").strip()
        versions = {p: (source / "scripts" / p / "VERSION").read_text(encoding="utf-8").strip() for p in projects}
        actual = repo_version if bundle else versions[project]
        if actual != expect_version:
            raise ValueError(f"Expected release version {expect_version}, found {actual} at {commit}")
        flags = [arg for p in projects for arg in ("--project", p)]
        subprocess.run(
            [sys.executable, "-B", "tools/build.py", *flags, "--dist", "--out-dir", str(dist)],
            cwd=source, check=True,
        )
        # The builder runs in the export. Nothing from a developer's dist/ is consulted.
        name = "sloppy-ethos_scripts.zip" if bundle else f"{project}-{actual}.zip"
        package = dist / name
        entries = audit(package, versions, repo_version, bundle)
        digest = hashlib.sha256(package.read_bytes()).hexdigest()
        receipt = dict(source_commit=commit, repository_version=repo_version, projects=versions,
                       archive=name, sha256=digest, entries=entries)
        out_dir.mkdir(parents=True, exist_ok=False)
        shutil.copyfile(package, out_dir / name)
        (out_dir / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
        print(f"Audited {out_dir / name}\nSource: {commit}\nSHA-256: {digest}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    scope = parser.add_mutually_exclusive_group(required=True)
    scope.add_argument("--bundle", action="store_true", help="Prepare the repository release bundle.")
    scope.add_argument("--project", help="Prepare one script release.")
    parser.add_argument("--ref", required=True, help="Local commit or ref to export.")
    parser.add_argument("--expect-version", required=True, help="Expected repository (bundle) or script version.")
    parser.add_argument("--out-dir", required=True, type=Path, help="New directory for the audited ZIP and receipt.")
    args = parser.parse_args()
    try:
        prepare(Path(__file__).resolve().parents[1], args.ref, args.project, args.expect_version, args.out_dir)
    except (OSError, ValueError, BadZipFile, subprocess.CalledProcessError) as exc:
        parser.exit(1, f"{exc}\n")


if __name__ == "__main__":
    main()
