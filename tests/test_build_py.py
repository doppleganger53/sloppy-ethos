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


def test_normalize_version_valid():
    assert build.normalize_version("1.2.3") == "1.2.3"
    assert build.normalize_version("release_candidate-1") == "release_candidate-1"


@pytest.mark.parametrize("raw", ["", " has spaces ", "!!", "#tag"])
def test_normalize_version_invalid(raw: str):
    with pytest.raises(SystemExit):
        build.normalize_version(raw)


def test_resolve_version_and_project_version(tmp_path: Path):
    (tmp_path / "VERSION").write_text("9.9.9\n", encoding="utf-8")
    project_dir = tmp_path / "scripts" / "WidgetX"
    project_dir.mkdir(parents=True)
    (project_dir / "VERSION").write_text("1.0.0\n", encoding="utf-8")
    assert build.resolve_version(tmp_path, None) == "9.9.9"
    assert build.resolve_project_version(project_dir, None) == "1.0.0"


def test_resolve_project_version_missing_file(tmp_path: Path):
    project_dir = tmp_path / "scripts" / "WidgetX"
    project_dir.mkdir(parents=True)
    with pytest.raises(SystemExit) as exc:
        build.resolve_project_version(project_dir, None)
    assert "Project version file not found" in str(exc.value)


def test_normalize_project_names_rules():
    assert build.normalize_project_names(None) == ["SensorList"]
    assert build.normalize_project_names(["SensorList", "ethos_events"]) == ["SensorList", "ethos_events"]
    with pytest.raises(SystemExit):
        build.normalize_project_names(["SensorList", "SensorList"])


def test_resolve_simulator_path_default(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text(
        '{"ETHOS_SIM_PATHS":[{"radio":"X20RS","path":"C:/sim/X20RS","default":true},{"radio":"X20S","path":"C:/sim/X20S"}]}',
        encoding="utf-8",
    )
    assert build.resolve_simulator_path(config) == Path("C:/sim/X20RS")


def test_run_lua_checks_exits_when_main_missing(tmp_path: Path):
    project_dir = tmp_path / "WidgetX"
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
    (project_dir / "VERSION").write_text("0.1.1\n", encoding="utf-8")

    expected_archive = dist_dir / "SensorList-0.1.1.zip"
    monkeypatch.setattr(build.shutil, "make_archive", lambda *_args: str(expected_archive))
    archive = build.build_zip(project_dir, "SensorList", "0.1.1", dist_dir, repo_root)
    assert archive == expected_archive
    assert not (repo_root / ".build-staging").exists()


def test_build_multi_project_zip_creates_archive(monkeypatch, tmp_path: Path):
    repo_root = tmp_path / "repo"
    dist_dir = repo_root / "dist"
    sensor_dir = repo_root / "scripts" / "SensorList"
    events_dir = repo_root / "scripts" / "ethos_events"
    sensor_dir.mkdir(parents=True)
    events_dir.mkdir(parents=True)
    (sensor_dir / "main.lua").write_text("print('sensor')\n", encoding="utf-8")
    (events_dir / "main.lua").write_text("print('events')\n", encoding="utf-8")
    (sensor_dir / "VERSION").write_text("0.1.1\n", encoding="utf-8")
    (events_dir / "VERSION").write_text("0.1.1\n", encoding="utf-8")
    (repo_root / "VERSION").write_text("0.1.1\n", encoding="utf-8")

    expected_archive = dist_dir / "sloppy-ethos_scripts.zip"
    monkeypatch.setattr(build.shutil, "make_archive", lambda *_args: str(expected_archive))
    archive = build.build_multi_project_zip(["SensorList", "ethos_events"], dist_dir, repo_root)
    assert archive == expected_archive
    assert not (repo_root / ".build-staging").exists()


def test_parse_args_defaults(monkeypatch):
    monkeypatch.setattr(build.sys, "argv", ["build.py"])
    args = build.parse_args()
    assert args.project is None
    assert args.dist is False
    assert args.deploy is False
    assert args.clean is False


def test_parse_args_repeated_project(monkeypatch):
    monkeypatch.setattr(
        build.sys,
        "argv",
        ["build.py", "--project", "SensorList", "--project", "ethos_events", "--dist"],
    )
    args = build.parse_args()
    assert args.project == ["SensorList", "ethos_events"]
    assert args.dist is True


def _set_main_stubs(monkeypatch, args: Namespace):
    calls: dict[str, object] = {}
    monkeypatch.setattr(build, "parse_args", lambda: args)
    monkeypatch.setattr(build, "ensure_luac_available", lambda: "luac")
    monkeypatch.setattr(build, "run_lua_checks", lambda project_dir, luac_exec: calls.setdefault("checks", []).append((project_dir, luac_exec)))

    def fake_resolve_project_version(project_dir, explicit):
        calls["project_version"] = (project_dir, explicit)
        return "0.1.1"

    monkeypatch.setattr(build, "resolve_project_version", fake_resolve_project_version)
    return calls


def test_main_single_project_dist_calls_build_zip(monkeypatch):
    calls = _set_main_stubs(
        monkeypatch,
        Namespace(
            help=False,
            project=["SensorList"],
            dist=True,
            deploy=False,
            clean=False,
            sim_radio=None,
            config=None,
            no_zip=False,
            version=None,
            out_dir=None,
        ),
    )
    monkeypatch.setattr(
        build,
        "build_zip",
        lambda project_dir, project_name, version, dist_dir, repo_root: calls.setdefault(
            "build_zip", (project_dir, project_name, version, dist_dir, repo_root)
        ),
    )
    build.main()
    assert "build_zip" in calls
    _project_dir, project_name, _version, _dist_dir, _repo_root = calls["build_zip"]
    assert project_name == "SensorList"


def test_main_multi_project_dist_calls_bundle_builder(monkeypatch):
    calls = _set_main_stubs(
        monkeypatch,
        Namespace(
            help=False,
            project=["SensorList", "ethos_events"],
            dist=True,
            deploy=False,
            clean=False,
            sim_radio=None,
            config=None,
            no_zip=False,
            version=None,
            out_dir=None,
        ),
    )
    monkeypatch.setattr(build, "build_multi_project_zip", lambda projects, dist_dir, repo_root: calls.setdefault("bundle", (projects, dist_dir, repo_root)))
    build.main()
    assert calls["bundle"][0] == ["SensorList", "ethos_events"]


def test_main_rejects_multi_with_deploy_or_version(monkeypatch):
    monkeypatch.setattr(
        build,
        "parse_args",
        lambda: Namespace(
            help=False,
            project=["SensorList", "ethos_events"],
            dist=True,
            deploy=True,
            clean=False,
            sim_radio=None,
            config=None,
            no_zip=False,
            version=None,
            out_dir=None,
        ),
    )
    with pytest.raises(SystemExit):
        build.main()

    monkeypatch.setattr(
        build,
        "parse_args",
        lambda: Namespace(
            help=False,
            project=["SensorList", "ethos_events"],
            dist=True,
            deploy=False,
            clean=False,
            sim_radio=None,
            config=None,
            no_zip=False,
            version="9.9.9",
            out_dir=None,
        ),
    )
    with pytest.raises(SystemExit):
        build.main()


def test_main_deploy_and_clean_paths(monkeypatch, tmp_path: Path):
    sim_path = tmp_path / "simulator" / "X20RS"
    sim_path.mkdir(parents=True)
    calls: dict[str, object] = {}

    monkeypatch.setattr(
        build,
        "parse_args",
        lambda: Namespace(
            help=False,
            project=["SensorList"],
            dist=False,
            deploy=True,
            clean=True,
            sim_radio="X20RS",
            config=None,
            no_zip=False,
            version=None,
            out_dir=None,
        ),
    )
    monkeypatch.setattr(build, "ensure_luac_available", lambda: "luac")
    monkeypatch.setattr(build, "run_lua_checks", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(build, "resolve_simulator_path", lambda *_args, **_kwargs: sim_path)
    monkeypatch.setattr(build, "clean_from_simulator", lambda project_name, resolved: calls.setdefault("clean", (project_name, resolved)))
    monkeypatch.setattr(build, "deploy_to_simulator", lambda project_dir, project_name, resolved: calls.setdefault("deploy", (project_name, resolved)))
    build.main()
    assert calls["clean"] == ("SensorList", sim_path)
    assert calls["deploy"] == ("SensorList", sim_path)
