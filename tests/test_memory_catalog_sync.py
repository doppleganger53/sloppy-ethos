from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path


def load_catalog_tool_module():
    repo_root = Path(__file__).resolve().parents[1]
    tool_path = repo_root / "tools" / "update_memory_catalog.py"
    spec = importlib.util.spec_from_file_location("memory_catalog_tool", tool_path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


tool = load_catalog_tool_module()


def test_catalog_generation_matches_repo_file():
    expected = tool.render_catalog(tool.collect_entries())
    actual = tool.CATALOG_PATH.read_text(encoding="utf-8")
    assert expected == actual


def test_catalog_includes_focus_dimension():
    entries = tool.collect_entries()
    focuses = {item.focus for item in entries}
    categories = {item.category for item in entries}
    assert len(focuses) > len(categories)
    rendered = tool.render_catalog(entries)
    assert "- Distribution by focus:" in rendered
    assert "- Distribution by category:" in rendered
    assert "| Date | Category | Focus | File | Title |" in rendered


def test_catalog_uses_relative_paths_in_file_column():
    rendered = tool.render_catalog(tool.collect_entries())
    assert "[notes/session-note/" in rendered


def test_catalog_includes_recent_high_signal_section():
    rendered = tool.render_catalog(tool.collect_entries())
    assert "## Recent High-Signal Notes (Auto-generated)" in rendered
    assert "- Selection: newest session notes where `Focus` is one of " in rendered


def test_recent_high_signal_section_is_descending_by_date():
    rendered = tool.render_catalog(tool.collect_entries())
    lines = rendered.splitlines()
    start = lines.index("## Recent High-Signal Notes (Auto-generated)")
    end = lines.index("## Entries")
    section = lines[start:end]
    date_lines = [
        line for line in section if re.match(r"^- \d{4}-\d{2}-\d{2} \| ", line)
    ]
    dates = [line.split("|", 1)[0].replace("- ", "").strip() for line in date_lines]
    assert dates == sorted(dates, reverse=True)


def test_catalog_check_mode_passes_when_synced(capsys):
    result = tool.main(["--check"])
    output = capsys.readouterr().out
    assert result == 0
    assert "up to date" in output


def test_session_note_template_has_current_state_sync_field():
    template = (tool.MEMORY_DIR / "SESSION_NOTE_TEMPLATE.md").read_text(encoding="utf-8")
    assert "## Current State Sync" in template
    assert "`CURRENT_STATE.md` updated: {yes|no}" in template


def test_current_state_has_no_manual_session_note_list():
    current_state = (tool.MEMORY_DIR / "CURRENT_STATE.md").read_text(encoding="utf-8")
    assert "## High-Value Recent Notes" not in current_state
    assert "SESSION_NOTES_" not in current_state


def test_memory_root_contains_only_index_control_markdown_files():
    allowed = {"README.md", "CURRENT_STATE.md", "CATALOG.md", "SESSION_NOTE_TEMPLATE.md"}
    root_markdown = {
        path.name
        for path in tool.MEMORY_DIR.glob("*.md")
    }
    assert root_markdown == allowed
