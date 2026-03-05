from __future__ import annotations

import re
import subprocess
from pathlib import Path

import pytest

from tests.helpers import REPO_ROOT, command_exists, run_command


DOC_FILES = [
    REPO_ROOT / "README.md",
    REPO_ROOT / "docs" / "DEVELOPMENT.md",
    REPO_ROOT / "scripts" / "SensorList" / "README.md",
    REPO_ROOT / "scripts" / "ethos_events" / "README.md",
    REPO_ROOT / "CONTRIBUTING.md",
    REPO_ROOT / ".github" / "PULL_REQUEST_TEMPLATE.md",
]

MANUAL_COMMANDS = (
    "luac -p",
    "python -m pip install -r requirements/dev.txt",
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


def is_manual_command(command: str) -> bool:
    if command in MANUAL_COMMANDS:
        return True
    if "--dist" in command or "--deploy" in command or "--clean" in command:
        return True
    if command.startswith("python -m pytest"):
        return True
    if command.startswith("powershell "):
        return True
    return False


def test_documented_commands_discovered():
    commands = discover_commands()
    assert commands, "Expected at least one command snippet in docs."


def test_extract_backtick_commands_filters_supported_prefixes():
    text = "Run `python tools/build.py --project SensorList --dist` and `echo nope`."
    assert _extract_backtick_commands(text) == ["python tools/build.py --project SensorList --dist"]


def test_extract_fenced_commands_parses_supported_lines():
    text = "```powershell\npython tools/build.py --project SensorList --dist\nnot-a-command\n```"
    assert _extract_fenced_commands(text) == ["python tools/build.py --project SensorList --dist"]


@pytest.mark.parametrize("command", discover_commands())
def test_command_references_existing_scripts(command: str):
    for script in ("tools/build.py",):
        if script in command:
            assert (REPO_ROOT / script).exists(), f"Missing referenced script: {script}"


@pytest.mark.parametrize("command", discover_commands())
def test_documented_command_syntax_or_execution(command: str):
    if is_manual_command(command):
        pytest.skip("Manual or environment-dependent command.")

    if command.startswith("luac "):
        if not command_exists("luac"):
            pytest.skip("luac not installed in this environment.")
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
        # Only run lightweight non-manual doc commands.
        result = run_command(command.split())
        assert result.returncode == 0, result.stderr
        return

    assert False, f"Unsupported documented command prefix: {command}"


def test_documented_command_luac_skips_when_missing(monkeypatch):
    monkeypatch.setattr("tests.test_docs_commands.command_exists", lambda _name: False)
    with pytest.raises(pytest.skip.Exception):
        test_documented_command_syntax_or_execution("luac -p scripts/SensorList/main.lua")

@pytest.mark.parametrize(
    "command",
    [
        "python tools/build.py --project WidgetX --dist",
        "python -m pytest tests/test_any.py",
        "python -m pip install -r requirements/dev.txt",
        "powershell -File script.ps1",
    ],
)
def test_is_manual_command_identifies_environment_dependent_commands(command: str):
    assert is_manual_command(command) is True


def test_documented_command_stylua_runs_with_check(monkeypatch):
    called: dict[str, object] = {}

    def fake_run(command: list[str]):
        called["command"] = command
        return subprocess.CompletedProcess(command, 0, "", "")

    monkeypatch.setattr("tests.test_docs_commands.command_exists", lambda _name: True)
    monkeypatch.setattr("tests.test_docs_commands.run_command", fake_run)
    test_documented_command_syntax_or_execution("stylua --config-path tools/config/stylua.toml scripts")
    assert called["command"] == ["stylua", "--config-path", "tools/config/stylua.toml", "scripts", "--check"]


@pytest.mark.parametrize(
    "command",
    [
        "luac -p",
        "python tools/build.py --project WidgetX --dist",
        "python -m pytest tests/test_any.py",
        "python -m pip install -r requirements/dev.txt",
        "powershell -File script.ps1",
    ],
)
def test_documented_manual_commands_skip(command: str):
    with pytest.raises(pytest.skip.Exception):
        test_documented_command_syntax_or_execution(command)
