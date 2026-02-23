from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest

from tests.helpers import REPO_ROOT, run_command


def _lua_executable() -> str | None:
    return shutil.which("lua") or shutil.which("lua54") or shutil.which("lua53")


def test_lua_executable_prefers_lua(monkeypatch):
    monkeypatch.setattr(shutil, "which", lambda name: "lua" if name == "lua" else None)
    assert _lua_executable() == "lua"


def test_lua_executable_falls_back_to_lua54(monkeypatch):
    monkeypatch.setattr(shutil, "which", lambda name: "lua54" if name == "lua54" else None)
    assert _lua_executable() == "lua54"


def test_lua_executable_returns_none_when_missing(monkeypatch):
    monkeypatch.setattr(shutil, "which", lambda _name: None)
    assert _lua_executable() is None


def test_sensorlist_lua_unit_tests_skips_when_no_lua(monkeypatch):
    monkeypatch.setattr("tests.test_sensorlist_widget._lua_executable", lambda: None)
    with pytest.raises(pytest.skip.Exception):
        test_sensorlist_lua_unit_tests()


def test_sensorlist_lua_unit_tests_invokes_expected_command(monkeypatch):
    expected_script = REPO_ROOT / "tests" / "lua" / "test_sensorlist.lua"
    called: dict[str, object] = {}

    def fake_run_command(command: list[str], cwd: Path):
        called["command"] = command
        called["cwd"] = cwd
        return subprocess.CompletedProcess(command, 0, "sensorlist lua tests passed\n", "")

    monkeypatch.setattr("tests.test_sensorlist_widget._lua_executable", lambda: "lua")
    monkeypatch.setattr("tests.test_sensorlist_widget.run_command", fake_run_command)
    test_sensorlist_lua_unit_tests()
    assert called["command"] == ["lua", str(expected_script)]
    assert called["cwd"] == REPO_ROOT


def test_sensorlist_lua_unit_tests():
    lua = _lua_executable()
    if not lua:
        pytest.skip("Lua interpreter not installed in this environment.")

    script = REPO_ROOT / "tests" / "lua" / "test_sensorlist.lua"
    result = run_command([lua, str(script)], cwd=REPO_ROOT)
    assert result.returncode == 0, result.stdout + "\n" + result.stderr
    assert "sensorlist lua tests passed" in result.stdout.lower()
