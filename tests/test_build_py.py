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

    def fake_run_lua_checks(project_dir: Path, luac_exec: str):
        calls["run_lua_checks"] = (project_dir, luac_exec)

    monkeypatch.setattr(build, "resolve_version", fake_resolve_version)
    monkeypatch.setattr(build, "ensure_luac_available", fake_ensure_luac_available)
    monkeypatch.setattr(build, "run_lua_checks", fake_run_lua_checks)
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


def test_resolve_simulator_path_uses_default_when_sim_radio_unspecified(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text(
        '{"ETHOS_SIM_PATHS":[{"radio":"X20RS","path":"C:/sim/X20RS","default":true},{"radio":"X20S","path":"C:/sim/X20S"}]}',
        encoding="utf-8",
    )
    result = build.resolve_simulator_path(config)
    assert result == Path("C:/sim/X20RS")


def test_resolve_simulator_path_sim_radio_uses_model_array(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text(
        '{"ETHOS_SIM_PATHS":[{"radio":"X20RS","path":"C:/sim/X20RS"},{"radio":"X20S","path":"C:/sim/X20S","default":true}]}',
        encoding="utf-8",
    )
    result = build.resolve_simulator_path(config, "X20RS")
    assert result == Path("C:/sim/X20RS")


def test_resolve_simulator_path_sim_radio_exits_when_array_entry_invalid(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text('{"ETHOS_SIM_PATHS":[{"radio":"X20RS"}]}', encoding="utf-8")
    with pytest.raises(SystemExit) as exc:
        build.resolve_simulator_path(config, "X20RS")
    assert "must include non-empty 'radio' and 'path'" in str(exc.value)


def test_resolve_simulator_path_exits_when_model_paths_type_invalid(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text('{"ETHOS_SIM_PATHS":"bad"}', encoding="utf-8")
    with pytest.raises(SystemExit) as exc:
        build.resolve_simulator_path(config, "X20RS")
    assert "must be a JSON array" in str(exc.value)


def test_resolve_simulator_path_exits_when_model_paths_missing(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text("{}", encoding="utf-8")
    with pytest.raises(SystemExit) as exc:
        build.resolve_simulator_path(config, "X20RS")
    assert "ETHOS_SIM_PATHS" in str(exc.value)


def test_resolve_simulator_path_exits_when_multiple_defaults(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text(
        '{"ETHOS_SIM_PATHS":[{"radio":"X20RS","path":"C:/sim/X20RS","default":true},{"radio":"X20S","path":"C:/sim/X20S","default":true}]}',
        encoding="utf-8",
    )
    with pytest.raises(SystemExit) as exc:
        build.resolve_simulator_path(config)
    assert "Only one ETHOS_SIM_PATHS entry may set default=true" in str(exc.value)


def test_resolve_simulator_path_sim_radio_exits_when_model_missing(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text('{"ETHOS_SIM_PATHS":[{"radio":"X18RS","path":"C:/sim/X18RS","default":true}]}', encoding="utf-8")
    with pytest.raises(SystemExit) as exc:
        build.resolve_simulator_path(config, "X20RS")
    assert "radio model 'X20RS'" in str(exc.value)


def test_resolve_simulator_path_exits_when_default_missing_for_unspecified_radio(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text('{"ETHOS_SIM_PATHS":[{"radio":"X20RS","path":"C:/sim/X20RS"}]}', encoding="utf-8")
    with pytest.raises(SystemExit) as exc:
        build.resolve_simulator_path(config)
    assert "No default simulator path configured" in str(exc.value)


def test_resolve_simulator_path_exits_when_duplicate_radios(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text(
        '{"ETHOS_SIM_PATHS":[{"radio":"X20RS","path":"C:/sim/X20RS","default":true},{"radio":"X20RS","path":"C:/sim/X20RS-2"}]}',
        encoding="utf-8",
    )
    with pytest.raises(SystemExit) as exc:
        build.resolve_simulator_path(config)
    assert "Duplicate simulator radio 'X20RS'" in str(exc.value)


def test_resolve_simulator_path_exits_when_default_not_bool(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text(
        '{"ETHOS_SIM_PATHS":[{"radio":"X20RS","path":"C:/sim/X20RS","default":"yes"}]}',
        encoding="utf-8",
    )
    with pytest.raises(SystemExit) as exc:
        build.resolve_simulator_path(config)
    assert "default' must be true/false" in str(exc.value)


def test_ensure_luac_available_returns_path(monkeypatch):
    monkeypatch.setattr(build.shutil, "which", lambda _name: "luac")
    assert build.ensure_luac_available() == "luac"


def test_ensure_luac_available_exits_when_missing(monkeypatch):
    monkeypatch.setattr(build.shutil, "which", lambda _name: None)
    with pytest.raises(SystemExit) as exc:
        build.ensure_luac_available()
    assert "Required command 'luac' not found" in str(exc.value)


def test_run_lua_checks_invokes_luac_for_all_lua_files(monkeypatch, tmp_path: Path):
    project_dir = tmp_path / "SensorList"
    nested_dir = project_dir / "nested"
    nested_dir.mkdir(parents=True)
    (project_dir / "main.lua").write_text("print('ok')\n", encoding="utf-8")
    (nested_dir / "module.lua").write_text("print('nested')\n", encoding="utf-8")
    called: list[list[str]] = []

    def fake_run(command: list[str], check: bool):
        assert check is True
        called.append(command)
        return None

    monkeypatch.setattr(build.subprocess, "run", fake_run)
    build.run_lua_checks(project_dir, "luac")
    assert called == [
        ["luac", "-p", str(project_dir / "main.lua")],
        ["luac", "-p", str(nested_dir / "module.lua")],
    ]


def test_run_lua_checks_exits_when_project_missing(tmp_path: Path):
    with pytest.raises(SystemExit) as exc:
        build.run_lua_checks(tmp_path / "missing", "luac")
    assert "Project directory not found" in str(exc.value)


def test_run_lua_checks_exits_when_main_missing(tmp_path: Path):
    project_dir = tmp_path / "SensorList"
    project_dir.mkdir()
    with pytest.raises(SystemExit) as exc:
        build.run_lua_checks(project_dir, "luac")
    assert "Widget entrypoint not found" in str(exc.value)


def test_build_zip_creates_archive_and_cleans_staging(monkeypatch, tmp_path: Path):
    repo_root = tmp_path / "repo"
    project_dir = repo_root / "scripts" / "SensorList"
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
    project_dir = repo_root / "scripts" / "SensorList"
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
    project_dir = repo_root / "scripts" / "SensorList"
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


def test_ensure_packaged_readme_noop_when_present(tmp_path: Path):
    destination = tmp_path / "scripts" / "WidgetX"
    destination.mkdir(parents=True)
    readme = destination / "README.md"
    readme.write_text("existing\n", encoding="utf-8")
    build.ensure_packaged_readme(destination, "WidgetX")
    assert readme.read_text(encoding="utf-8") == "existing\n"


def test_ensure_packaged_readme_writes_default_when_missing(tmp_path: Path):
    destination = tmp_path / "scripts" / "WidgetX"
    destination.mkdir(parents=True)
    build.ensure_packaged_readme(destination, "WidgetX")
    readme = destination / "README.md"
    assert readme.exists()
    content = readme.read_text(encoding="utf-8")
    assert "# WidgetX" in content
    assert "scripts/WidgetX" in content


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


def test_clean_from_simulator_removes_existing_target(tmp_path: Path, capsys):
    sim_path = tmp_path / "sim"
    target = sim_path / "scripts" / "SensorList"
    target.mkdir(parents=True)
    (target / "main.lua").write_text("print('ok')\n", encoding="utf-8")
    build.clean_from_simulator("SensorList", sim_path)
    assert not target.exists()
    assert "Cleaned simulator target:" in capsys.readouterr().out


def test_clean_from_simulator_noop_when_missing(tmp_path: Path, capsys):
    sim_path = tmp_path / "sim"
    sim_path.mkdir()
    build.clean_from_simulator("SensorList", sim_path)
    assert "Clean skip:" in capsys.readouterr().out


def test_print_help_text_reads_file(monkeypatch, capsys, tmp_path: Path):
    help_path = tmp_path / "build_help.txt"
    help_path.write_text("hello\n", encoding="utf-8")
    monkeypatch.setattr(build, "HELP_FILE", help_path)
    build.print_help_text()
    assert "hello" in capsys.readouterr().out


def test_parse_args_defaults(monkeypatch):
    monkeypatch.setattr(build.sys, "argv", ["build.py"])
    args = build.parse_args()
    assert args.help is False
    assert args.project == "SensorList"
    assert args.dist is False
    assert args.deploy is False
    assert args.clean is False
    assert args.sim_radio is None
    assert args.config is None
    assert args.no_zip is False
    assert args.version is None
    assert args.out_dir is None


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
            "--clean",
            "--sim-radio",
            "X20RS",
            "--config",
            "path/to/deploy.json",
            "--no-zip",
            "--help",
            "--version",
            "9.9.9",
            "--out-dir",
            "custom-dist",
        ],
    )
    args = build.parse_args()
    assert args.help is True
    assert args.project == "WidgetX"
    assert args.dist is True
    assert args.deploy is True
    assert args.clean is True
    assert args.sim_radio == "X20RS"
    assert args.config == "path/to/deploy.json"
    assert args.no_zip is True
    assert args.version == "9.9.9"
    assert args.out_dir == "custom-dist"


def test_main_requires_action(monkeypatch):
    _set_main_prerequisites(
        monkeypatch,
        Namespace(help=False, project="SensorList", dist=False, deploy=False, clean=False, sim_radio=None, config=None, no_zip=False, version=None, out_dir=None),
    )
    with pytest.raises(SystemExit) as exc:
        build.main()
    assert "Nothing to do" in str(exc.value)


def test_main_dist_calls_build_zip_when_nozip_false(monkeypatch):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(help=False, project="SensorList", dist=True, deploy=False, clean=False, sim_radio=None, config=None, no_zip=False, version=None, out_dir=None),
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
    assert project_dir == repo_root / "scripts" / "SensorList"
    assert dist_dir == repo_root / "dist"


def test_main_dist_skips_build_zip_when_nozip_true(monkeypatch):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(help=False, project="SensorList", dist=True, deploy=False, clean=False, sim_radio=None, config=None, no_zip=True, version=None, out_dir=None),
    )
    monkeypatch.setattr(build, "build_zip", lambda *_args, **_kwargs: calls.setdefault("build_zip", "called"))
    build.main()
    assert "build_zip" not in calls


def test_main_dist_uses_explicit_out_dir(monkeypatch):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(help=False, project="SensorList", dist=True, deploy=False, clean=False, sim_radio=None, config=None, no_zip=False, version=None, out_dir="custom-dist"),
    )

    def fake_build_zip(project_dir: Path, project_name: str, version: str, dist_dir: Path, repo_root: Path):
        calls["build_zip"] = (project_dir, project_name, version, dist_dir, repo_root)
        return dist_dir / "SensorList-1.0.0.zip"

    monkeypatch.setattr(build, "build_zip", fake_build_zip)
    build.main()
    _project_dir, _project_name, _version, dist_dir, _repo_root = calls["build_zip"]
    assert dist_dir == Path("custom-dist")


def test_main_deploy_uses_default_config_path_when_config_missing(monkeypatch):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(help=False, project="SensorList", dist=False, deploy=True, clean=False, sim_radio=None, config=None, no_zip=False, version=None, out_dir=None),
    )
    sim_path = Path("C:/simulator")

    def fake_resolve_simulator_path(config_path: Path, sim_radio: str | None):
        calls["resolve_simulator_path"] = (config_path, sim_radio)
        return sim_path

    def fake_deploy_to_simulator(project_dir: Path, project_name: str, resolved_sim_path: Path):
        calls["deploy"] = (project_dir, project_name, resolved_sim_path)

    monkeypatch.setattr(build, "resolve_simulator_path", fake_resolve_simulator_path)
    monkeypatch.setattr(build, "deploy_to_simulator", fake_deploy_to_simulator)
    build.main()
    config_path, sim_radio = calls["resolve_simulator_path"]
    assert config_path == Path(build.__file__).resolve().parent.parent / "tools" / "deploy.config.json"
    assert sim_radio is None
    project_dir, project_name, resolved_sim_path = calls["deploy"]
    assert project_name == "SensorList"
    assert resolved_sim_path == sim_path
    assert project_dir == Path(build.__file__).resolve().parent.parent / "scripts" / "SensorList"


def test_main_deploy_uses_explicit_config_path(monkeypatch):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(
            help=False,
            project="SensorList",
            dist=False,
            deploy=True,
            clean=False,
            sim_radio=None,
            config="C:/custom/deploy.json",
            no_zip=False,
            version=None,
            out_dir=None,
        ),
    )
    def fake_resolve_simulator_path(config_path: Path, sim_radio: str | None):
        calls["resolve_simulator_path"] = (config_path, sim_radio)
        return Path("C:/simulator")

    monkeypatch.setattr(build, "resolve_simulator_path", fake_resolve_simulator_path)
    monkeypatch.setattr(build, "deploy_to_simulator", lambda *_args, **_kwargs: None)
    build.main()
    config_path, _sim_radio = calls["resolve_simulator_path"]
    assert config_path == Path("C:/custom/deploy.json")


def test_main_deploy_exits_when_sim_path_unconfigured(monkeypatch):
    _set_main_prerequisites(
        monkeypatch,
        Namespace(help=False, project="SensorList", dist=False, deploy=True, clean=False, sim_radio=None, config=None, no_zip=False, version=None, out_dir=None),
    )
    monkeypatch.setattr(build, "resolve_simulator_path", lambda *_args, **_kwargs: None)
    with pytest.raises(SystemExit) as exc:
        build.main()
    assert "Configure ETHOS_SIM_PATHS" in str(exc.value)


def test_main_help_exits_before_other_work(monkeypatch):
    calls: dict[str, object] = {}
    monkeypatch.setattr(
        build,
        "parse_args",
        lambda: Namespace(help=True, project="SensorList", dist=False, deploy=False, clean=False, sim_radio=None, config=None, no_zip=False, version=None, out_dir=None),
    )
    monkeypatch.setattr(build, "print_help_text", lambda: calls.setdefault("help", True))
    monkeypatch.setattr(build, "ensure_luac_available", lambda: calls.setdefault("luac", True))
    build.main()
    assert "help" in calls
    assert "luac" not in calls


def test_main_clean_calls_clean_from_simulator(monkeypatch, tmp_path: Path):
    calls = _set_main_prerequisites(
        monkeypatch,
        Namespace(help=False, project="SensorList", dist=False, deploy=False, clean=True, sim_radio="X20RS", config=None, no_zip=False, version=None, out_dir=None),
    )
    sim_path = tmp_path / "simulator" / "X20RS"
    sim_path.mkdir(parents=True)
    monkeypatch.setattr(build, "resolve_simulator_path", lambda *_args, **_kwargs: sim_path)
    monkeypatch.setattr(build, "clean_from_simulator", lambda project_name, resolved: calls.setdefault("clean", (project_name, resolved)))
    build.main()
    assert "ensure_luac_available" not in calls
    assert calls["clean"] == ("SensorList", sim_path)


