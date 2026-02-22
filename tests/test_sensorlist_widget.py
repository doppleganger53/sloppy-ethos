from __future__ import annotations

import shutil
from pathlib import Path

import pytest

from tests.helpers import REPO_ROOT, run_command


def _lua_executable() -> str | None:
    return shutil.which("lua") or shutil.which("lua54") or shutil.which("lua53")


def test_sensorlist_lua_unit_tests():
    lua = _lua_executable()
    if not lua:
        pytest.skip("Lua interpreter not installed in this environment.")

    script = REPO_ROOT / "tests" / "lua" / "test_sensorlist.lua"
    result = run_command([lua, str(script)], cwd=REPO_ROOT)
    assert result.returncode == 0, result.stdout + "\n" + result.stderr
    assert "sensorlist lua tests passed" in result.stdout.lower()
