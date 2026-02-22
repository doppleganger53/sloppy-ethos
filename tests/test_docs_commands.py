from __future__ import annotations

import re
from pathlib import Path

import pytest

from tests.helpers import REPO_ROOT, command_exists, run_command


DOC_FILES = [
    REPO_ROOT / "README.md",
    REPO_ROOT / "docs" / "DEVELOPMENT.md",
    REPO_ROOT / "src" / "scripts" / "SensorList" / "README.md",
    REPO_ROOT / "CONTRIBUTING.md",
]

MANUAL_PATTERNS = (
    "luac -p",
    "python tools/build.py --project SensorList --dist",
    "python tools/build.py --project SensorList --deploy",
    "powershell -ExecutionPolicy Bypass -File tools/build-package.ps1 -ProjectName SensorList",
    "powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-package.ps1 -ProjectName SensorList",
)


def _extract_backtick_commands(text: str) -> list[str]:
    return [
        cmd.strip()
        for cmd in re.findall(r"`([^`\n]+)`", text)
        if cmd.strip().startswith(("python ", "luac ", "stylua ", "powershell "))
    ]


def _extract_fenced_commands(text: str) -> list[str]:
    commands: list[str] = []
    for block in re.findall(r"```(?:powershell|bash)?\n(.*?)```", text, flags=re.DOTALL):
        for line in block.splitlines():
            line = line.strip()
            if line.startswith(("python ", "luac ", "stylua ", "powershell ")):
                commands.append(line)
    return commands


def discover_commands() -> list[str]:
    commands: list[str] = []
    for path in DOC_FILES:
        text = path.read_text(encoding="utf-8")
        commands.extend(_extract_backtick_commands(text))
        commands.extend(_extract_fenced_commands(text))
    return sorted(set(commands))


def test_documented_commands_discovered():
    commands = discover_commands()
    assert commands, "Expected at least one command snippet in docs."


@pytest.mark.parametrize("command", discover_commands())
def test_command_references_existing_scripts(command: str):
    for script in ("tools/build.py", "tools/build-package.ps1"):
        if script in command:
            assert (REPO_ROOT / script).exists(), f"Missing referenced script: {script}"


@pytest.mark.parametrize("command", discover_commands())
def test_documented_command_syntax_or_execution(command: str):
    if command in MANUAL_PATTERNS:
        pytest.skip("Manual or environment-dependent command.")

    if command.startswith("luac "):
        if not command_exists("luac"):
            pytest.skip("luac not installed in this environment.")
        if command.strip() == "luac -p":
            pytest.skip("Command fragment requires file argument in source docs.")
        result = run_command(command.split())
        assert result.returncode == 0, result.stderr
        return

    if command.startswith("stylua "):
        if not command_exists("stylua"):
            pytest.skip("stylua not installed in this environment.")
        result = run_command(command.split() + ["--check"])
        assert result.returncode == 0, result.stderr
        return

    if command.startswith("python "):
        # Only run lightweight doc commands by default.
        if "--dist" in command or "--deploy" in command:
            pytest.skip("Build/deploy commands are documented but environment-dependent.")
        result = run_command(command.split())
        assert result.returncode == 0, result.stderr
        return

    if command.startswith("powershell "):
        pytest.skip("PowerShell doc command treated as manual fallback.")
