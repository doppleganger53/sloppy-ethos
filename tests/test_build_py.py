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
    assert str(result) == "C:\\from\\env" or str(result) == "C:/from/env"


def test_resolve_simulator_path_from_config(tmp_path: Path):
    config = tmp_path / "deploy.json"
    config.write_text('{"ETHOS_SIM_PATH":"C:/simulator"}', encoding="utf-8")
    result = build.resolve_simulator_path(config, None)
    assert result is not None
    assert "simulator" in str(result)


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


def test_main_requires_action(monkeypatch):
    monkeypatch.setattr(
        build,
        "parse_args",
        lambda: Namespace(project="SensorList", dist=False, deploy=False, config=None, no_zip=False, version=None),
    )
    monkeypatch.setattr(build, "ensure_luac_available", lambda: "luac")
    monkeypatch.setattr(build, "run_lua_check", lambda *_args, **_kwargs: None)
    monkeypatch.setattr(build, "resolve_version", lambda *_args, **_kwargs: "1.0.0")
    with pytest.raises(SystemExit) as exc:
        build.main()
    assert "Nothing to do" in str(exc.value)
