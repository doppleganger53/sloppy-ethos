from __future__ import annotations

import shutil
import subprocess

import pytest

from tests.helpers import REPO_ROOT, run_command


def _lua_executable() -> str | None:
    return shutil.which("lua") or shutil.which("lua54") or shutil.which("lua53")


def test_boundrymap_lua_unit_tests_skips_when_no_lua(monkeypatch):
    monkeypatch.setattr("tests.test_boundrymap_widget._lua_executable", lambda: None)
    with pytest.raises(pytest.skip.Exception):
        test_boundrymap_lua_unit_tests()


def test_boundrymap_lua_unit_tests_invokes_expected_command(monkeypatch):
    expected_script = REPO_ROOT / "tests" / "lua" / "test_boundrymap.lua"
    called: dict[str, object] = {}

    def fake_run_command(command: list[str], cwd):
        called["command"] = command
        called["cwd"] = cwd
        return subprocess.CompletedProcess(command, 0, "boundrymap lua tests passed\n", "")

    monkeypatch.setattr("tests.test_boundrymap_widget._lua_executable", lambda: "lua")
    monkeypatch.setattr("tests.test_boundrymap_widget.run_command", fake_run_command)
    test_boundrymap_lua_unit_tests()
    assert called["command"] == ["lua", str(expected_script)]
    assert called["cwd"] == REPO_ROOT


def test_boundrymap_lua_unit_tests():
    lua = _lua_executable()
    if not lua:
        pytest.skip("Lua interpreter not installed in this environment.")

    script = REPO_ROOT / "tests" / "lua" / "test_boundrymap.lua"
    result = run_command([lua, str(script)], cwd=REPO_ROOT)
    assert result.returncode == 0, result.stdout + "\n" + result.stderr
    assert "boundrymap lua tests passed" in result.stdout.lower()
