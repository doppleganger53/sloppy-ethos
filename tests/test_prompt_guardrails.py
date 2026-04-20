from __future__ import annotations

from pathlib import Path

from tests.helpers import REPO_ROOT


ISSUE_PROMPTS_DIR = REPO_ROOT / "deslopification" / "prompts" / "issues"
ISSUE_PROMPTS_ARCHIVE_DIR = ISSUE_PROMPTS_DIR / "archive"
ISSUE_TEMPLATE_PATH = REPO_ROOT / "deslopification" / "prompts" / "templates" / "ISSUE_RESOLUTION_TEMPLATE.md"
ISSUE_PROMPTS_INDEX_PATH = ISSUE_PROMPTS_DIR / "README.md"

ACTIVE_ISSUE_PROMPTS = sorted(path for path in ISSUE_PROMPTS_DIR.glob("ISSUE-*.md"))
ARCHIVED_ISSUE_PROMPTS = sorted(ISSUE_PROMPTS_ARCHIVE_DIR.glob("ISSUE-*.md"))
DONE_ISSUE_PROMPTS = sorted((ISSUE_PROMPTS_DIR / "done").glob("ISSUE-*.md"))


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def test_active_issue_prompt_snapshots_are_archived():
    assert ACTIVE_ISSUE_PROMPTS == []


def test_archived_issue_prompt_snapshots_exist():
    assert ARCHIVED_ISSUE_PROMPTS or DONE_ISSUE_PROMPTS


def test_issue_template_does_not_default_to_main():
    text = _read(ISSUE_TEMPLATE_PATH)
    assert "Target branch (default: `main`)" not in text
    assert "Target branch (default by issue kind, never `main`):" in text


def test_issue_template_requires_guided_and_explicit_preflight_paths():
    text = _read(ISSUE_TEMPLATE_PATH)
    assert "python tools/session_start.py issue" in text
    assert "python tools/session_preflight.py --mode issue" in text


def test_issue_prompt_index_documents_template_first_archive_workflow():
    text = _read(ISSUE_PROMPTS_INDEX_PATH)
    assert "Template-first workflow" in text
    assert "issues/archive/" in text
