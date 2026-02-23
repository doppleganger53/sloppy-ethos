from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest

from tests.helpers import REPO_ROOT, run_command


def _lua_executable() -> str | None:
    return shutil.which("lua") or shutil.which("lua54") or shutil.which("lua53")


@pytest.mark.parametrize(
    ("which_map", "expected"),
    [
        ({"lua": "lua"}, "lua"),
        ({"lua54": "lua54"}, "lua54"),
        ({}, None),
    ],
)
def test_lua_executable_resolution(monkeypatch, which_map: dict[str, str], expected: str | None):
    monkeypatch.setattr(shutil, "which", lambda name: which_map.get(name))
    assert _lua_executable() == expected


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
