from __future__ import annotations

from pathlib import Path

import pytest

from tests.helpers import REPO_ROOT


ISSUE_PROMPTS_DIR = REPO_ROOT / "deslopification" / "prompts" / "issues"
ISSUE_TEMPLATE_PATH = REPO_ROOT / "deslopification" / "prompts" / "templates" / "ISSUE_RESOLUTION_TEMPLATE.md"

OPEN_ISSUE_PROMPTS = sorted(path for path in ISSUE_PROMPTS_DIR.glob("ISSUE-*.md"))


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


@pytest.mark.parametrize("path", OPEN_ISSUE_PROMPTS)
def test_open_issue_prompts_do_not_default_to_main(path: Path):
    text = _read(path)
    assert "Target branch (default): `main`" not in text
    assert "Target branch (default):" in text


@pytest.mark.parametrize("path", OPEN_ISSUE_PROMPTS)
def test_open_issue_prompts_require_issue_preflight(path: Path):
    text = _read(path)
    assert "python tools/session_preflight.py --mode issue" in text


def test_issue_template_does_not_default_to_main():
    text = _read(ISSUE_TEMPLATE_PATH)
    assert "Target branch (default: `main`)" not in text
    assert "Target branch (default by issue kind, never `main`):" in text


def test_issue_template_requires_issue_preflight():
    text = _read(ISSUE_TEMPLATE_PATH)
    assert "python tools/session_preflight.py --mode issue" in text
