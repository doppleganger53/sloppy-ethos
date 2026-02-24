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
]

MANUAL_PATTERNS = (
    "luac -p",
    "python tools/build.py --project SensorList --dist",
    "python tools/build.py --project SensorList --deploy",
    "python tools/build.py --project SensorList --clean --sim-radio X20RS",
    "python tools/build.py --project ethos_events --dist",
    "python tools/build.py --project ethos_events --deploy",
    "python -m pip install -r requirements/dev.txt",
    "python -m pytest tests/test_sensorlist_widget.py",
    "python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q",
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
        if "--dist" in command or "--deploy" in command or "--clean" in command:
            pytest.skip("Build/deploy commands are documented but environment-dependent.")
        if command.startswith("python -m pytest"):
            pytest.skip("pytest doc commands are manual to avoid recursive test process spawning.")
        result = run_command(command.split())
        assert result.returncode == 0, result.stderr
        return

    if command.startswith("powershell "):
        pytest.skip("PowerShell doc command treated as manual fallback.")


def test_documented_command_luac_skips_when_missing(monkeypatch):
    monkeypatch.setattr("tests.test_docs_commands.command_exists", lambda _name: False)
    with pytest.raises(pytest.skip.Exception):
        test_documented_command_syntax_or_execution("luac -p scripts/SensorList/main.lua")


def test_documented_command_luac_fragment_skips(monkeypatch):
    monkeypatch.setattr("tests.test_docs_commands.command_exists", lambda _name: True)
    monkeypatch.setattr("tests.test_docs_commands.MANUAL_PATTERNS", ())
    with pytest.raises(pytest.skip.Exception):
        test_documented_command_syntax_or_execution("luac -p")


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
        "python tools/build.py --project WidgetX --dist",
        "python -m pytest tests/test_sensorlist_widget.py",
        "powershell -File script.ps1",
    ],
)
def test_documented_manual_commands_skip(command: str):
    with pytest.raises(pytest.skip.Exception):
        test_documented_command_syntax_or_execution(command)
