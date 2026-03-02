from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path

import pytest


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
    scope: str,
    concern: str,
    name: str,
    rel_path: str,
    artifact: str = "session",
) -> object:
    return tool.Entry(
        date=date,
        artifact=artifact,
        scope=scope,
        concern=concern,
        rel_path=rel_path,
        name=name,
        title="# Title",
        bytes_count=1,
        lines_count=1,
    )


def write_note(path: Path, *, artifact: str, scope: str, concern: str, title: str = "# Title") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "\n".join(
            [
                title,
                "",
                "## Note Placement",
                "",
                f"- Artifact: `{artifact}`",
                f"- Scope: `{scope}`",
                f"- Concern: `{concern}`",
                "- Store this file under:",
                "  - `deslopification/memory/notes/{artifact}/{scope}/`",
                "",
            ]
        ),
        encoding="utf-8",
    )


def test_normalized_byte_count_is_platform_stable(tmp_path):
    lf_file = tmp_path / "lf.md"
    crlf_file = tmp_path / "crlf.md"
    lf_file.write_bytes(b"line1\nline2\n")
    crlf_file.write_bytes(b"line1\r\nline2\r\n")
    assert tool.normalized_byte_count(lf_file) == tool.normalized_byte_count(crlf_file)


def first_high_signal_concern() -> str:
    concerns = sorted(tool.HIGH_SIGNAL_CONCERN)
    assert concerns, "HIGH_SIGNAL_CONCERN must contain at least one concern"
    return concerns[0]


def test_catalog_generation_matches_repo_file():
    expected = tool.render_catalog(tool.collect_entries())
    actual = tool.CATALOG_PATH.read_text(encoding="utf-8")
    assert expected == actual


def test_catalog_includes_scope_and_concern_dimensions():
    entries = [
        make_entry(
            date="2026-01-01",
            scope="scope-a",
            concern="implementation",
            name="a.md",
            rel_path="notes/session/scope-a/a.md",
        ),
        make_entry(
            date="2026-01-02",
            scope="scope-b",
            concern="workflow",
            name="b.md",
            rel_path="notes/session/scope-b/b.md",
        ),
    ]
    scopes = {item.scope for item in entries}
    artifacts = {item.artifact for item in entries}
    assert len(scopes) > len(artifacts)
    rendered = tool.render_catalog(entries)
    assert "- Distribution by scope:" in rendered
    assert "- Distribution by artifact:" in rendered
    assert "- Distribution by concern:" in rendered
    assert "| Date | Artifact | Scope | Concern | File | Title |" in rendered


def test_scope_distribution_uses_count_scope_description_format():
    entries = [
        make_entry(
            date="2026-01-01",
            scope="scope-a",
            concern="implementation",
            name="a.md",
            rel_path="notes/session/scope-a/a.md",
        )
    ]
    rendered = tool.render_catalog(entries)
    assert "- 1 -- scope-a ( description missing )" in rendered


def test_catalog_uses_relative_paths_in_file_column():
    entries = [
        make_entry(
            date="2026-01-01",
            scope="scope-a",
            concern="implementation",
            name="a.md",
            rel_path="notes/session/scope-a/a.md",
        )
    ]
    rendered = tool.render_catalog(entries)
    assert "[notes/session/" in rendered


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
    assert f"up to {tool.RECENT_HIGH_SIGNAL_PER_CONCERN_LIMIT} per concern" in selection_line
    assert f"newest {tool.RECENT_HIGH_SIGNAL_LIMIT} overall" in selection_line
    for concern in sorted(tool.HIGH_SIGNAL_CONCERN):
        assert f"`{concern}`" in selection_line


def test_recent_high_signal_section_is_descending_by_date():
    rendered = tool.render_catalog(tool.collect_entries())
    lines = rendered.splitlines()
    start = lines.index("## Recent High-Signal Notes (Auto-generated)")
    end = lines.index("## Recent Ethos Platform Notes")
    section = lines[start:end]
    date_lines = [
        line for line in section if re.match(r"^- \d{4}-\d{2}-\d{2} \| ", line)
    ]
    dates = [line.split("|", 1)[0].replace("- ", "").strip() for line in date_lines]
    assert dates == sorted(dates, reverse=True)


def test_catalog_includes_recent_ethos_platform_section():
    rendered = tool.render_catalog(tool.collect_entries())
    assert "## Recent Ethos Platform Notes" in rendered


def test_recent_ethos_platform_section_keeps_undated_reference_after_dated_sessions():
    entries = [
        make_entry(
            date="-",
            artifact="reference",
            scope="ethos-platform",
            concern="implementation",
            name="EthosPlatform.md",
            rel_path="notes/reference/ethos-platform/EthosPlatform.md",
        ),
        make_entry(
            date="2026-03-02",
            scope="ethos-platform",
            concern="implementation",
            name="SESSION_NOTES_2026-03-02_FOO.md",
            rel_path="notes/session/ethos-platform/SESSION_NOTES_2026-03-02_FOO.md",
        ),
    ]
    selected = tool.select_recent_ethos_platform(entries)
    assert [item.name for item in selected] == [
        "SESSION_NOTES_2026-03-02_FOO.md",
        "EthosPlatform.md",
    ]


def test_catalog_check_mode_passes_when_synced(capsys):
    result = tool.main(["--check"])
    output = capsys.readouterr().out
    assert result == 0
    assert "up to date" in output


def test_session_note_template_has_concern_field():
    template = (tool.MEMORY_DIR / "SESSION_NOTE_TEMPLATE.md").read_text(encoding="utf-8")
    assert "- Concern: `{implementation|release|build|docs|testing|workflow|prompts|issue-admin|metadata}`" in template


def test_current_state_has_no_manual_session_note_list():
    current_state = (tool.MEMORY_DIR / "CURRENT_STATE.md").read_text(encoding="utf-8")
    assert "## High-Value Recent Notes" not in current_state
    assert "SESSION_NOTES_" not in current_state


def test_memory_root_contains_only_index_control_markdown_files():
    allowed = {"README.md", "CURRENT_STATE.md", "CATALOG.md", "SESSION_NOTE_TEMPLATE.md"}
    root_markdown = {path.name for path in tool.MEMORY_DIR.glob("*.md")}
    assert root_markdown == allowed


def test_collect_entries_rejects_missing_concern(tmp_path):
    memory_dir = tmp_path / "memory"
    note_path = memory_dir / "notes" / "session" / "repo" / "bad.md"
    note_path.parent.mkdir(parents=True, exist_ok=True)
    note_path.write_text(
        "\n".join(
            [
                "# Title",
                "",
                "## Note Placement",
                "",
                "- Artifact: `session`",
                "- Scope: `repo`",
            ]
        ),
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="missing note metadata field"):
        tool.collect_entries(memory_dir=memory_dir)


def test_collect_entries_rejects_metadata_path_mismatch(tmp_path):
    memory_dir = tmp_path / "memory"
    note_path = memory_dir / "notes" / "session" / "repo" / "bad.md"
    write_note(note_path, artifact="session", scope="memory", concern="workflow")
    with pytest.raises(ValueError, match="metadata/path mismatch"):
        tool.collect_entries(memory_dir=memory_dir)


def test_collect_entries_rejects_unknown_concern(tmp_path):
    memory_dir = tmp_path / "memory"
    note_path = memory_dir / "notes" / "session" / "repo" / "bad.md"
    write_note(note_path, artifact="session", scope="repo", concern="nope")
    with pytest.raises(ValueError, match="unknown concern"):
        tool.collect_entries(memory_dir=memory_dir)


def test_no_notes_use_legacy_category_or_focus_metadata():
    note_files = list((tool.MEMORY_DIR / "notes").rglob("*.md"))
    assert note_files
    for path in note_files:
        text = path.read_text(encoding="utf-8")
        assert "- Category:" not in text
        assert "- Focus:" not in text


def test_no_lua_ethos_scope_remains():
    entries = tool.collect_entries()
    legacy = [item for item in entries if item.scope == "lua-ethos"]
    assert legacy == []


def test_ethos_platform_scope_exists():
    entries = tool.collect_entries()
    scopes = {item.scope for item in entries}
    assert "ethos-platform" in scopes


def test_ethos_platform_reference_exists():
    path = tool.MEMORY_DIR / "notes" / "reference" / "ethos-platform" / "EthosPlatform.md"
    assert path.exists()


def test_select_recent_high_signal_caps_each_concern_at_three():
    concern = first_high_signal_concern()
    entries = [
        make_entry(
            date=f"2026-02-{day:02d}",
            scope="repo",
            concern=concern,
            name=f"HS-{day:02d}.md",
            rel_path=f"notes/session/repo/HS-{day:02d}.md",
        )
        for day in range(1, 6)
    ]

    selected = tool.select_recent_high_signal(entries)
    selected_dates = [item.date for item in selected if item.concern == concern]

    assert len(selected_dates) == tool.RECENT_HIGH_SIGNAL_PER_CONCERN_LIMIT
    assert selected_dates == ["2026-02-05", "2026-02-04", "2026-02-03"]


def test_select_recent_high_signal_applies_global_limit_after_per_concern_cap():
    entries = []
    day = 1
    for concern in sorted(tool.HIGH_SIGNAL_CONCERN):
        for index in range(tool.RECENT_HIGH_SIGNAL_PER_CONCERN_LIMIT):
            name = f"{concern}-{index}.md"
            entries.append(
                make_entry(
                    date=f"2026-01-{day:02d}",
                    scope="repo",
                    concern=concern,
                    name=name,
                    rel_path=f"notes/session/repo/{name}",
                )
            )
            day += 1

    selected = tool.select_recent_high_signal(entries)
    expected_count = min(
        tool.RECENT_HIGH_SIGNAL_LIMIT,
        len(tool.HIGH_SIGNAL_CONCERN) * tool.RECENT_HIGH_SIGNAL_PER_CONCERN_LIMIT,
    )
    assert len(selected) == expected_count
    for concern in tool.HIGH_SIGNAL_CONCERN:
        count = sum(1 for item in selected if item.concern == concern)
        assert count <= tool.RECENT_HIGH_SIGNAL_PER_CONCERN_LIMIT


def test_select_recent_high_signal_sorting_is_deterministic_with_tie_breakers():
    concern = first_high_signal_concern()
    rel_paths = [
        "notes/session/repo/A.md",
        "notes/session/repo/B.md",
        "notes/session/repo/C.md",
    ]
    entries = [
        make_entry(
            date="2026-02-20",
            scope="repo",
            concern=concern,
            name="SAME.md",
            rel_path=rel_path,
        )
        for rel_path in rel_paths
    ]

    selected = tool.select_recent_high_signal(entries)
    assert [item.rel_path for item in selected] == sorted(rel_paths, reverse=True)


def test_select_recent_high_signal_filters_non_matching_entries():
    concern = first_high_signal_concern()
    entries = [
        make_entry(
            date="2026-02-10",
            artifact="session",
            scope="repo",
            concern=concern,
            name="valid.md",
            rel_path="notes/session/repo/valid.md",
        ),
        make_entry(
            date="-",
            artifact="session",
            scope="repo",
            concern=concern,
            name="no-date.md",
            rel_path="notes/session/repo/no-date.md",
        ),
        make_entry(
            date="2026-02-11",
            artifact="handoff",
            scope="handoff",
            concern=concern,
            name="wrong-artifact.md",
            rel_path="notes/handoff/handoff/wrong-artifact.md",
        ),
        make_entry(
            date="2026-02-12",
            artifact="session",
            scope="repo",
            concern="implementation",
            name="wrong-concern.md",
            rel_path="notes/session/repo/wrong-concern.md",
        ),
    ]

    selected = tool.select_recent_high_signal(entries)
    assert [item.name for item in selected] == ["valid.md"]


def test_each_active_scope_folder_has_nonempty_desc_file():
    entries = tool.collect_entries()
    scope_dirs = {Path(item.rel_path).parent for item in entries}
    for scope_dir in scope_dirs:
        desc_file = tool.MEMORY_DIR / scope_dir / ".desc"
        assert desc_file.exists(), f"Missing .desc for scope directory: {scope_dir}"
        assert desc_file.read_text(encoding="utf-8").strip(), f"Empty .desc file: {scope_dir}"
