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


def make_entry(
    *,
    date: str,
    focus: str,
    name: str,
    rel_path: str,
    category: str = "session-note",
) -> object:
    return tool.Entry(
        date=date,
        category=category,
        focus=focus,
        rel_path=rel_path,
        name=name,
        title="# Title",
        bytes_count=1,
        lines_count=1,
    )


def first_high_signal_focus() -> str:
    focuses = sorted(tool.HIGH_SIGNAL_FOCUS)
    assert focuses, "HIGH_SIGNAL_FOCUS must contain at least one focus"
    return focuses[0]


def test_catalog_generation_matches_repo_file():
    expected = tool.render_catalog(tool.collect_entries())
    actual = tool.CATALOG_PATH.read_text(encoding="utf-8")
    assert expected == actual


def test_catalog_includes_focus_dimension():
    entries = [
        make_entry(
            date="2026-01-01",
            focus="focus-a",
            name="a.md",
            rel_path="notes/session-note/focus-a/a.md",
        ),
        make_entry(
            date="2026-01-02",
            focus="focus-b",
            name="b.md",
            rel_path="notes/session-note/focus-b/b.md",
        ),
    ]
    focuses = {item.focus for item in entries}
    categories = {item.category for item in entries}
    assert len(focuses) > len(categories)
    rendered = tool.render_catalog(entries)
    assert "- Distribution by focus:" in rendered
    assert "- Distribution by category:" in rendered
    assert "| Date | Category | Focus | File | Title |" in rendered


def test_focus_distribution_uses_count_focus_description_format():
    entries = [
        make_entry(
            date="2026-01-01",
            focus="focus-a",
            name="a.md",
            rel_path="notes/session-note/focus-a/a.md",
        )
    ]
    rendered = tool.render_catalog(entries)
    assert "- 1 -- focus-a ( description missing )" in rendered


def test_catalog_uses_relative_paths_in_file_column():
    entries = [
        make_entry(
            date="2026-01-01",
            focus="focus-a",
            name="a.md",
            rel_path="notes/session-note/focus-a/a.md",
        )
    ]
    rendered = tool.render_catalog(entries)
    assert "[notes/session-note/" in rendered


def test_catalog_lists_control_files_in_static_block():
    rendered = tool.render_catalog(tool.collect_entries())
    assert "Control files (not indexed in entries):" in rendered
    assert "- [README.md](README.md)" in rendered
    assert "- [CURRENT_STATE.md](CURRENT_STATE.md)" in rendered
    assert "- [SESSION_NOTE_TEMPLATE.md](SESSION_NOTE_TEMPLATE.md)" in rendered


def test_catalog_entries_table_excludes_memory_control_files():
    rendered = tool.render_catalog(tool.collect_entries())
    lines = rendered.splitlines()
    start = lines.index("## Entries")
    entries_section = "\n".join(lines[start:])
    assert "[README.md](README.md)" not in entries_section
    assert "[CURRENT_STATE.md](CURRENT_STATE.md)" not in entries_section
    assert "[SESSION_NOTE_TEMPLATE.md](SESSION_NOTE_TEMPLATE.md)" not in entries_section


def test_catalog_includes_recent_high_signal_section():
    rendered = tool.render_catalog(tool.collect_entries())
    assert "## Recent High-Signal Notes (Auto-generated)" in rendered
    selection_line = next(
        line for line in rendered.splitlines() if line.startswith("- Selection:")
    )
    assert f"up to {tool.RECENT_HIGH_SIGNAL_PER_FOCUS_LIMIT} per focus" in selection_line
    assert f"newest {tool.RECENT_HIGH_SIGNAL_LIMIT} overall" in selection_line
    for focus in sorted(tool.HIGH_SIGNAL_FOCUS):
        assert f"`{focus}`" in selection_line


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


def test_no_session_notes_use_general_focus():
    entries = tool.collect_entries()
    general_session_notes = [
        item for item in entries if item.category == "session-note" and item.focus == "general"
    ]
    assert general_session_notes == []


def test_select_recent_high_signal_caps_each_focus_at_three():
    focus = first_high_signal_focus()
    entries = [
        make_entry(
            date=f"2026-02-{day:02d}",
            focus=focus,
            name=f"HS-{day:02d}.md",
            rel_path=f"notes/session-note/{focus}/HS-{day:02d}.md",
        )
        for day in range(1, 6)
    ]

    selected = tool.select_recent_high_signal(entries)
    selected_dates = [item.date for item in selected if item.focus == focus]

    assert len(selected_dates) == tool.RECENT_HIGH_SIGNAL_PER_FOCUS_LIMIT
    assert selected_dates == ["2026-02-05", "2026-02-04", "2026-02-03"]


def test_select_recent_high_signal_applies_global_limit_after_per_focus_cap():
    entries = []
    day = 1
    for focus in sorted(tool.HIGH_SIGNAL_FOCUS):
        for index in range(tool.RECENT_HIGH_SIGNAL_PER_FOCUS_LIMIT):
            name = f"{focus}-{index}.md"
            entries.append(
                make_entry(
                    date=f"2026-01-{day:02d}",
                    focus=focus,
                    name=name,
                    rel_path=f"notes/session-note/{focus}/{name}",
                )
            )
            day += 1

    selected = tool.select_recent_high_signal(entries)
    expected_count = min(
        tool.RECENT_HIGH_SIGNAL_LIMIT,
        len(tool.HIGH_SIGNAL_FOCUS) * tool.RECENT_HIGH_SIGNAL_PER_FOCUS_LIMIT,
    )
    assert len(selected) == expected_count
    for focus in tool.HIGH_SIGNAL_FOCUS:
        count = sum(1 for item in selected if item.focus == focus)
        assert count <= tool.RECENT_HIGH_SIGNAL_PER_FOCUS_LIMIT


def test_select_recent_high_signal_sorting_is_deterministic_with_tie_breakers():
    focus = first_high_signal_focus()
    rel_paths = [
        f"notes/session-note/{focus}/A.md",
        f"notes/session-note/{focus}/B.md",
        f"notes/session-note/{focus}/C.md",
    ]
    entries = [
        make_entry(
            date="2026-02-20",
            focus=focus,
            name="SAME.md",
            rel_path=rel_path,
        )
        for rel_path in rel_paths
    ]

    selected = tool.select_recent_high_signal(entries)
    assert [item.rel_path for item in selected] == sorted(rel_paths, reverse=True)


def test_select_recent_high_signal_filters_non_matching_entries():
    focus = first_high_signal_focus()
    entries = [
        make_entry(
            date="2026-02-10",
            category="session-note",
            focus=focus,
            name="valid.md",
            rel_path=f"notes/session-note/{focus}/valid.md",
        ),
        make_entry(
            date="-",
            category="session-note",
            focus=focus,
            name="no-date.md",
            rel_path=f"notes/session-note/{focus}/no-date.md",
        ),
        make_entry(
            date="2026-02-11",
            category="handoff",
            focus=focus,
            name="wrong-category.md",
            rel_path="notes/handoff/handoff/wrong-category.md",
        ),
        make_entry(
            date="2026-02-12",
            category="session-note",
            focus="__non_high_signal__",
            name="wrong-focus.md",
            rel_path="notes/session-note/__non_high_signal__/wrong-focus.md",
        ),
    ]

    selected = tool.select_recent_high_signal(entries)
    assert [item.name for item in selected] == ["valid.md"]


def test_each_active_focus_folder_has_nonempty_desc_file():
    entries = tool.collect_entries()
    focus_dirs = {Path(item.rel_path).parent for item in entries}
    for focus_dir in focus_dirs:
        desc_file = tool.MEMORY_DIR / focus_dir / ".desc"
        assert desc_file.exists(), f"Missing .desc for focus directory: {focus_dir}"
        assert desc_file.read_text(encoding="utf-8").strip(), f"Empty .desc file: {focus_dir}"
