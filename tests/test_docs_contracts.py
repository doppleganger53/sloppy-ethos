from __future__ import annotations

import json
import re
from pathlib import Path

import pytest

from tests.helpers import REPO_ROOT
from tests.test_docs_commands import DOC_FILES, MANUAL_PATTERNS, discover_commands


README_PATH = REPO_ROOT / "README.md"
DEVELOPMENT_PATH = REPO_ROOT / "docs" / "DEVELOPMENT.md"


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _extract_backtick_tokens(text: str) -> list[str]:
    return [token.strip() for token in re.findall(r"`([^`\n]+)`", text) if token.strip()]


def _looks_like_local_file_reference(token: str) -> bool:
    if "://" in token or " " in token:
        return False
    if token == "VERSION":
        return True
    if token.startswith(("dist/", "dist\\", "${", "<")):
        return False
    return token.endswith((".md", ".py", ".ps1", ".lua", ".json", ".txt"))


def _iter_documented_file_refs() -> list[str]:
    refs: list[str] = []
    for path in DOC_FILES:
        for token in _extract_backtick_tokens(_read(path)):
            if _looks_like_local_file_reference(token):
                refs.append(token)
    return sorted(set(refs))


@pytest.mark.parametrize(
    ("path", "sections"),
    [
        (README_PATH, ("## Quick Start", "## Development", "### Running the Python tests")),
        (DEVELOPMENT_PATH, ("## Prerequisites", "## Core Commands", "## Running the Python tests")),
    ],
)
def test_docs_include_required_sections(path: Path, sections: tuple[str, ...]):
    text = _read(path)
    for section in sections:
        assert section in text, f"Missing required section '{section}' in {path}."


@pytest.mark.parametrize("ref", _iter_documented_file_refs())
def test_documented_local_file_references_exist(ref: str):
    assert (REPO_ROOT / ref).exists(), f"Missing documented path: {ref}"


def test_python_version_guidance_consistent_across_docs():
    assert "Python 3.9+" in _read(README_PATH)
    assert "Python 3.9+" in _read(DEVELOPMENT_PATH)


def test_test_dependency_guidance_consistent_across_docs():
    readme_text = _read(README_PATH)
    development_text = _read(DEVELOPMENT_PATH)
    assert "requirements-dev.txt" in readme_text
    assert "requirements-dev.txt" in development_text
    assert "pytest-cov" in readme_text
    assert "pytest-cov" in development_text


def test_zip_naming_policy_documented():
    assert "dist/{ProjectName}-{version}.zip" in _read(README_PATH)
    assert "dist/{ProjectName}-{version}.zip" in _read(DEVELOPMENT_PATH)


def test_version_file_exists_and_nonempty():
    version_text = (REPO_ROOT / "VERSION").read_text(encoding="utf-8").strip()
    assert version_text, "VERSION file must contain a non-empty version string."


def test_vscode_pytest_enabled_in_repo_settings():
    settings = json.loads((REPO_ROOT / ".vscode" / "settings.json").read_text(encoding="utf-8"))
    assert settings.get("python.testing.pytestEnabled") is True


def test_environment_dependent_doc_commands_are_manual():
    def should_be_manual(command: str) -> bool:
        return (
            "--dist" in command
            or "--deploy" in command
            or command.startswith("powershell ")
            or command == "python -m pip install -r requirements-dev.txt"
        )

    for command in discover_commands():
        if should_be_manual(command):
            assert command in MANUAL_PATTERNS, f"Environment-dependent command missing in MANUAL_PATTERNS: {command}"
