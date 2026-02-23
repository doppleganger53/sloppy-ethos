from __future__ import annotations

import importlib.util
from argparse import Namespace
from pathlib import Path

import pytest


def load_build_module():
    repo_root = Path(__file__).resolve().parents[1]
    build_path = repo_root / "tools" / "build.py"
    spec = importlib.util.spec_from_file_location("build_module", build_path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


build = load_build_module()


def _set_main_prerequisites(monkeypatch, args: Namespace):
    calls: dict[str, object] = {}

    monkeypatch.setattr(build, "parse_args", lambda: args)

    def fake_resolve_version(repo_root: Path, explicit_version: str | None):
        calls["resolve_version"] = (repo_root, explicit_version)
        return "1.0.0"

    def fake_ensure_luac_available():
        calls["ensure_luac_available"] = True
        return "luac"

    def fake_run_lua_check(project_dir: Path, luac_exec: str):
        calls["run_lua_check"] = (project_dir, luac_exec)

    monkeypatch.setattr(build, "resolve_version", fake_resolve_version)
    monkeypatch.setattr(build, "ensure_luac_available", fake_ensure_luac_available)
    monkeypatch.setattr(build, "run_lua_check", fake_run_lua_check)
    return calls


def test_normalize_version_valid():
    assert build.normalize_version("1.2.3") == "1.2.3"
    assert build.normalize_version("release_candidate-1") == "release_candidate-1"
    assert build.normalize_version("  2026.02.22  ") == "2026.02.22"


@pytest.mark.parametrize("raw", ["", " has spaces ", "!!", "#tag"])
def test_normalize_version_invalid(raw: str):
    with pytest.raises(SystemExit):
        build.normalize_version(raw)


def test_resolve_version_prefers_explicit(tmp_path: Path):
    (tmp_path / "VERSION").write_text("9.9.9\n", encoding="utf-8")
    assert build.resolve_version(tmp_path, "1.0.0") == "1.0.0"


def test_resolve_version_from_file(tmp_path: Path):
    (tmp_path / "VERSION").write_text("2.4.6\nextra\n", encoding="utf-8")
    assert build.resolve_version(tmp_path, None) == "2.4.6"


def test_resolve_version_missing_file(tmp_path: Path):
    with pytest.raises(SystemExit) as exc:
        build.resolve_version(tmp_path, None)
    assert "Version file not found" in str(exc.value)


def test_load_config_missing_file(tmp_path: Path):
    assert build.load_config(tmp_path / "missing.json") == {}


def test_load_config_invalid_json(tmp_path: Path):
    config = tmp_path / "broken.json"
    config.write_text("{ bad", encoding="utf-8")
    with pytest.raises(SystemExit):
        build.load_config(config)


def test_resolve_simulator_path_env_wins(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text('{"ETHOS_SIM_PATH":"C:/from/config"}', encoding="utf-8")
    result = build.resolve_simulator_path(config, "C:/from/env")
    assert result == Path("C:/from/env")


def test_resolve_simulator_path_from_config(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text('{"ETHOS_SIM_PATH":"C:/simulator"}', encoding="utf-8")
    result = build.resolve_simulator_path(config, None)
    assert result == Path("C:/simulator")


def test_resolve_simulator_path_missing_key_returns_none(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text("{}", encoding="utf-8")
    assert build.resolve_simulator_path(config, None) is None


def test_ensure_luac_available_returns_path(monkeypatch):
    monkeypatch.setattr(build.shutil, "which", lambda _name: "luac")
    assert build.ensure_luac_available() == "luac"


def test_ensure_luac_available_exits_when_missing(monkeypatch):
    monkeypatch.setattr(build.shutil, "which", lambda _name: None)
    with pytest.raises(SystemExit) as exc:
        build.ensure_luac_available()
    assert "Required command 'luac' not found" in str(exc.value)


def test_run_lua_check_invokes_luac(monkeypatch, tmp_path: Path):
    project_dir = tmp_path / "SensorList"
    project_dir.mkdir()
    (project_dir / "main.lua").write_text("print('ok')\n", encoding="utf-8")
    called: dict[str, object] = {}

    def fake_run(command: list[str], check: bool):
        called["command"] = command
        called["check"] = check
        return None

    monkeypatch.setattr(build.subprocess, "run", fake_run)
    build.run_lua_check(project_dir, "luac")
    assert called["command"] == ["luac", "-p", str(project_dir / "main.lua")]
    assert called["check"] is True


def test_run_lua_check_exits_when_main_missing(tmp_path: Path):
    project_dir = tmp_path / "SensorList"
    project_dir.mkdir()
    with pytest.raises(SystemExit) as exc:
        build.run_lua_check(project_dir, "luac")
    assert "Widget entrypoint not found" in str(exc.value)


def test_build_zip_creates_archive_and_cleans_staging(monkeypatch, tmp_path: Path):
    repo_root = tmp_path / "repo"
    project_dir = repo_root / "src" / "scripts" / "SensorList"
    dist_dir = repo_root / "dist"
    project_dir.mkdir(parents=True)
    (project_dir / "main.lua").write_text("print('ok')\n", encoding="utf-8")

    expected_archive = dist_dir / "SensorList-1.2.3.zip"
    monkeypatch.setattr(build.shutil, "make_archive", lambda *_args: str(expected_archive))
    archive = build.build_zip(project_dir, "SensorList", "1.2.3", dist_dir, repo_root)
    assert archive == expected_archive
    assert not (repo_root / ".build-staging").exists()


def test_build_zip_removes_existing_staging_before_copy(monkeypatch, tmp_path: Path):
    repo_root = tmp_path / "repo"
    project_dir = repo_root / "src" / "scripts" / "SensorList"
    dist_dir = repo_root / "dist"
    staging_root = repo_root / ".build-staging"
    project_dir.mkdir(parents=True)
    (project_dir / "main.lua").write_text("print('ok')\n", encoding="utf-8")
    (staging_root / "stale").mkdir(parents=True)
    calls: list[Path] = []
    real_rmtree = build.shutil.rmtree

    def tracked_rmtree(path: Path, ignore_errors: bool = False):
        calls.append(Path(path))
        return real_rmtree(path, ignore_errors=ignore_errors)

    monkeypatch.setattr(build.shutil, "rmtree", tracked_rmtree)
    monkeypatch.setattr(build.shutil, "make_archive", lambda *_args: str(dist_dir / "SensorList-2.0.0.zip"))
    build.build_zip(project_dir, "SensorList", "2.0.0", dist_dir, repo_root)
    assert calls[0] == staging_root


def test_build_zip_cleans_staging_on_copy_failure(monkeypatch, tmp_path: Path):
    repo_root = tmp_path / "repo"
    project_dir = repo_root / "src" / "scripts" / "SensorList"
    dist_dir = repo_root / "dist"
    staging_root = repo_root / ".build-staging"
    project_dir.mkdir(parents=True)
    cleanup_calls: list[Path] = []
    real_rmtree = build.shutil.rmtree

    def fail_copytree(*_args, **_kwargs):
        raise OSError("copy failed")

    def tracked_rmtree(path: Path, ignore_errors: bool = False):
        cleanup_calls.append(Path(path))
        return real_rmtree(path, ignore_errors=ignore_errors)

    monkeypatch.setattr(build.shutil, "copytree", fail_copytree)
    monkeypatch.setattr(build.shutil, "rmtree", tracked_rmtree)
    with pytest.raises(OSError):
        build.build_zip(project_dir, "SensorList", "2.1.0", dist_dir, repo_root)
    assert staging_root in cleanup_calls


def test_add_project_extras_to_scripts_root_noop_for_other_projects(tmp_path: Path):
    project_dir = tmp_path / "src" / "scripts" / "SensorList"
    scripts_root = tmp_path / "staging" / "scripts"
    project_dir.mkdir(parents=True)
    scripts_root.mkdir(parents=True)
    build.add_project_extras_to_scripts_root(project_dir, "SensorList", scripts_root)
    assert not (scripts_root / "tools" / "ethos_events.png").exists()


def test_add_project_extras_to_scripts_root_copies_ethos_events_icon(tmp_path: Path):
    project_dir = tmp_path / "src" / "scripts" / "ethos_events"
    scripts_root = tmp_path / "staging" / "scripts"
    project_dir.mkdir(parents=True)
    scripts_root.mkdir(parents=True)
    (project_dir / "ethos_events.png").write_bytes(b"pngdata")
    build.add_project_extras_to_scripts_root(project_dir, "ethos_events", scripts_root)
    assert (scripts_root / "tools" / "ethos_events.png").read_bytes() == b"pngdata"


def test_format_deploy_error_permission():
    target = Path("C:/sim/scripts/SensorList")
    message = build.format_deploy_error(target, PermissionError("denied"))
    assert "Permission denied" in message
    assert "Close Ethos Suite and retry." in message


def test_format_deploy_error_generic():
    target = Path("C:/sim/scripts/SensorList")
    message = build.format_deploy_error(target, OSError("io"))
    assert "Unexpected filesystem error" in message
    assert "Details:" in message


def test_deploy_to_simulator_exits_when_sim_path_missing(tmp_path: Path):
    project_dir = tmp_path / "project"
    project_dir.mkdir()
    with pytest.raises(SystemExit) as exc:
        build.deploy_to_simulator(project_dir, "SensorList", tmp_path / "does-not-exist")
    assert "does not exist" in str(exc.value)


def test_deploy_to_simulator_success_copies_and_prints(monkeypatch, capsys, tmp_path: Path):
    project_dir = tmp_path / "project"
    sim_path = tmp_path / "sim"
    project_dir.mkdir()
    sim_path.mkdir()
    called: dict[str, object] = {}

    def fake_copytree(src: Path, dst: Path, dirs_exist_ok: bool):
        called["src"] = src
        called["dst"] = dst
        called["dirs_exist_ok"] = dirs_exist_ok
        return None

    monkeypatch.setattr(build.shutil, "copytree", fake_copytree)
    build.deploy_to_simulator(project_dir, "SensorList", sim_path)
    target = sim_path / "scripts" / "SensorList"
    assert called["src"] == project_dir
    assert called["dst"] == target
    assert called["dirs_exist_ok"] is True
    assert target.parent.exists()
    assert "Deployed SensorList ->" in capsys.readouterr().out


def test_deploy_to_simulator_wraps_oserror_with_formatted_message(monkeypatch, tmp_path: Path):
    project_dir = tmp_path / "project"
    sim_path = tmp_path / "sim"
    project_dir.mkdir()
    sim_path.mkdir()
    monkeypatch.setattr(
        build.shutil,
        "copytree",
        lambda *_args, **_kwargs: (_ for _ in ()).throw(PermissionError("denied")),
    )
    with pytest.raises(SystemExit) as exc:
        build.deploy_to_simulator(project_dir, "SensorList", sim_path)
    assert "Permission denied while writing to the simulator folder." in str(exc.value)
    assert "Failed to deploy 'SensorList'" in str(exc.value)


def test_parse_args_defaults(monkeypatch):
    monkeypatch.setattr(build.sys, "argv", ["build.py"])
    args = build.parse_args()
    assert args.project == "SensorList"
    assert args.dist is False
    assert args.deploy is False
    assert args.config is None
    assert args.no_zip is False
    assert args.version is None


def test_parse_args_flags(monkeypatch):
    monkeypatch.setattr(
        build.sys,
        "argv",
        [
            "build.py",
            "--project",
            "WidgetX",
            "--dist",
            "--deploy",
            "--config",
            "path/to/deploy.json",
            "--no-zip",
            "--version",
            "9.9.9",
        ],
    )
    args = build.parse_args()
    assert args.project == "WidgetX"
    assert args.dist is True
    assert args.deploy is True
    assert args.config == "path/to/deploy.json"
    assert args.no_zip is True
    assert args.version == "9.9.9"


def test_main_requires_action(monkeypatch):
    _set_main_prerequisites(
        monkeypatch,
        Namespace(project="SensorList", dist=False, deploy=False, config=None, no_zip=False, version=None),
    )
    with pytest.raises(SystemExit) as exc:
        build.main()
    assert "Nothing to do" in str(exc.value)


def test_main_dist_calls_build_zip_when_nozip_false(monkeypatch):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(project="SensorList", dist=True, deploy=False, config=None, no_zip=False, version=None),
    )

    def fake_build_zip(project_dir: Path, project_name: str, version: str, dist_dir: Path, repo_root: Path):
        calls["build_zip"] = (project_dir, project_name, version, dist_dir, repo_root)
        return dist_dir / "SensorList-1.0.0.zip"

    monkeypatch.setattr(build, "build_zip", fake_build_zip)
    build.main()
    assert "build_zip" in calls
    project_dir, project_name, version, dist_dir, repo_root = calls["build_zip"]
    assert project_name == "SensorList"
    assert version == "1.0.0"
    assert project_dir == repo_root / "src" / "scripts" / "SensorList"
    assert dist_dir == repo_root / "dist"


def test_main_dist_skips_build_zip_when_nozip_true(monkeypatch):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(project="SensorList", dist=True, deploy=False, config=None, no_zip=True, version=None),
    )
    monkeypatch.setattr(build, "build_zip", lambda *_args, **_kwargs: calls.setdefault("build_zip", "called"))
    build.main()
    assert "build_zip" not in calls


def test_main_deploy_uses_default_config_path_when_config_missing(monkeypatch):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(project="SensorList", dist=False, deploy=True, config=None, no_zip=False, version=None),
    )
    sim_path = Path("C:/simulator")
    monkeypatch.delenv("ETHOS_SIM_PATH", raising=False)

    def fake_resolve_simulator_path(config_path: Path, override_env: str | None):
        calls["resolve_simulator_path"] = (config_path, override_env)
        return sim_path

    def fake_deploy_to_simulator(project_dir: Path, project_name: str, resolved_sim_path: Path):
        calls["deploy"] = (project_dir, project_name, resolved_sim_path)

    monkeypatch.setattr(build, "resolve_simulator_path", fake_resolve_simulator_path)
    monkeypatch.setattr(build, "deploy_to_simulator", fake_deploy_to_simulator)
    build.main()
    config_path, override_env = calls["resolve_simulator_path"]
    assert config_path == Path(build.__file__).resolve().parent.parent / "tools" / "deploy.config.json"
    assert override_env is None
    project_dir, project_name, resolved_sim_path = calls["deploy"]
    assert project_name == "SensorList"
    assert resolved_sim_path == sim_path
    assert project_dir == Path(build.__file__).resolve().parent.parent / "src" / "scripts" / "SensorList"


def test_main_deploy_uses_explicit_config_path(monkeypatch):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(
            project="SensorList",
            dist=False,
            deploy=True,
            config="C:/custom/deploy.json",
            no_zip=False,
            version=None,
        ),
    )
    def fake_resolve_simulator_path(config_path: Path, override_env: str | None):
        calls["resolve_simulator_path"] = (config_path, override_env)
        return Path("C:/simulator")

    monkeypatch.setattr(build, "resolve_simulator_path", fake_resolve_simulator_path)
    monkeypatch.setattr(build, "deploy_to_simulator", lambda *_args, **_kwargs: None)
    build.main()
    config_path, _override_env = calls["resolve_simulator_path"]
    assert config_path == Path("C:/custom/deploy.json")


def test_main_deploy_exits_when_sim_path_unconfigured(monkeypatch):
    _set_main_prerequisites(
        monkeypatch,
        Namespace(project="SensorList", dist=False, deploy=True, config=None, no_zip=False, version=None),
    )
    monkeypatch.delenv("ETHOS_SIM_PATH", raising=False)
    monkeypatch.setattr(build, "resolve_simulator_path", lambda *_args, **_kwargs: None)
    with pytest.raises(SystemExit) as exc:
        build.main()
    assert "Simulator path not configured" in str(exc.value)


