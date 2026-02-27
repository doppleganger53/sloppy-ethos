#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
MEMORY_DIR = REPO_ROOT / "deslopification" / "memory"
CATALOG_PATH = MEMORY_DIR / "CATALOG.md"

PRE_OPT_BASELINE = {
    "files": 52,
    "bytes": 69293,
    "lines": 1249,
    "date": "2026-02-26",
}

CONTROL_FILES = ("README.md", "CURRENT_STATE.md", "SESSION_NOTE_TEMPLATE.md")
NOTES_ROOT = MEMORY_DIR / "notes"
HIGH_SIGNAL_FOCUS = {
    "memory-ops",
    "workflow-policy",
    "build-tooling",
    "docs-process",
    "release-versioning",
    "testing",
    "prompts",
    "repo-metadata",
    "issue-lifecycle",
    "repo-governance",
}
RECENT_HIGH_SIGNAL_LIMIT = 12


@dataclass(frozen=True)
class Entry:
    date: str
    category: str
    focus: str
    rel_path: str
    name: str
    title: str
    bytes_count: int
    lines_count: int


def extract_date(name: str) -> str:
    match = re.search(r"(\d{4}-\d{2}-\d{2})", name)
    return match.group(1) if match else "-"


def read_first_line(path: Path) -> str:
    text = path.read_text(encoding="utf-8")
    if not text:
        return "(no title)"
    first = text.splitlines()[0].strip()
    return first if first else "(no title)"


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def parse_category_focus_from_path(rel_path: Path) -> tuple[str, str] | None:
    parts = rel_path.parts
    if len(parts) >= 4 and parts[0] == "notes":
        return parts[1], parts[2]
    return None


def collect_entries(memory_dir: Path = MEMORY_DIR) -> list[Entry]:
    entries: list[Entry] = []
    notes_root = memory_dir / "notes"
    if not notes_root.exists():
        return entries

    for path in sorted(
        notes_root.rglob("*.md"),
        key=lambda p: p.relative_to(memory_dir).as_posix(),
    ):
        rel_path = path.relative_to(memory_dir)
        if rel_path.parts and rel_path.parts[0] == "temp":
            continue

        title = read_first_line(path)
        from_path = parse_category_focus_from_path(rel_path)
        if from_path is None:
            continue
        category, focus = from_path
        entries.append(
            Entry(
                date=extract_date(path.name),
                category=category,
                focus=focus,
                rel_path=rel_path.as_posix(),
                name=path.name,
                title=title,
                bytes_count=path.stat().st_size,
                lines_count=line_count(path),
            )
        )
    return entries


def select_recent_high_signal(entries: list[Entry]) -> list[Entry]:
    candidates = [
        item
        for item in entries
        if item.category == "session-note"
        and item.date != "-"
        and item.focus in HIGH_SIGNAL_FOCUS
    ]
    return sorted(candidates, key=lambda x: (x.date, x.name), reverse=True)[
        :RECENT_HIGH_SIGNAL_LIMIT
    ]


def render_catalog(entries: list[Entry]) -> str:
    total_files = len(entries)
    total_bytes = sum(item.bytes_count for item in entries)
    total_lines = sum(item.lines_count for item in entries)

    by_category: dict[str, int] = {}
    by_focus: dict[str, int] = {}
    for item in entries:
        by_category[item.category] = by_category.get(item.category, 0) + 1
        by_focus[item.focus] = by_focus.get(item.focus, 0) + 1

    category_order = ["session-note", "handoff", "domain-note", "monthly-summary"]
    category_lines = []
    for category in category_order:
        count = by_category.get(category)
        if count is None:
            continue
        label = {
            "session-note": "session notes",
            "handoff": "handoff/restart notes",
            "domain-note": "domain notes",
            "monthly-summary": "monthly summaries",
        }[category]
        category_lines.append(f"- {label}: {count}")

    focus_lines = []
    for focus, count in sorted(by_focus.items(), key=lambda pair: (-pair[1], pair[0])):
        focus_lines.append(f"- {focus}: {count}")

    rows = []
    for item in sorted(entries, key=lambda x: (x.date, x.rel_path)):
        title = item.title.replace("|", "/")
        rows.append(
            f"| {item.date} | {item.category} | {item.focus} | [{item.rel_path}]({item.rel_path}) | {title} |"
        )

    recent_high_signal = select_recent_high_signal(entries)
    recent_lines = []
    for item in recent_high_signal:
        title = item.title.replace("|", "/")
        recent_lines.append(
            f"- {item.date} | {item.focus} | [{item.rel_path}]({item.rel_path}) | {title}"
        )

    lines = [
        "# Memory Catalog",
        "",
        "Index of all memory artifacts in this folder.",
        "",
        "Control files (not indexed in entries):",
        *(f"- [{name}]({name})" for name in CONTROL_FILES),
        "",
        f"Pre-optimization baseline (before Issue #16 on {PRE_OPT_BASELINE['date']}):",
        "",
        f"- Files: {PRE_OPT_BASELINE['files']}",
        f"- Total size: {PRE_OPT_BASELINE['bytes']:,} bytes",
        f"- Total lines: {PRE_OPT_BASELINE['lines']:,}",
        "",
        "Current snapshot (auto-generated, excludes `CATALOG.md`):",
        "",
        f"- Files: {total_files}",
        f"- Total size: {total_bytes:,} bytes",
        f"- Total lines: {total_lines:,}",
        "- Distribution by category:",
    ]
    lines.extend(f"  {item}" for item in category_lines)
    lines.extend(
        [
            "",
            "- Distribution by focus:",
        ]
    )
    lines.extend(f"  {item}" for item in focus_lines)
    lines.extend(
        [
            "",
            "## Recent High-Signal Notes (Auto-generated)",
            "",
            (
                "- Selection: newest session notes where `Focus` is one of "
                "`memory-ops`, `workflow-policy`, `build-tooling`, "
                "`docs-process`, `release-versioning`, `testing`, `prompts`, "
                "`repo-metadata`, `issue-lifecycle`, or `repo-governance`."
            ),
        ]
    )
    lines.extend(recent_lines if recent_lines else ["- None."])
    lines.extend(
        [
            "",
            "## Entries",
            "",
            "| Date | Category | Focus | File | Title |",
            "| --- | --- | --- | --- | --- |",
            *rows,
            "",
        ]
    )
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Regenerate deslopification/memory/CATALOG.md")
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit non-zero if CATALOG.md is out of date.",
    )
    args = parser.parse_args(argv)

    generated = render_catalog(collect_entries())
    current = CATALOG_PATH.read_text(encoding="utf-8") if CATALOG_PATH.exists() else ""

    if args.check:
        if generated != current:
            print("CATALOG.md is out of date. Run: python tools/update_memory_catalog.py")
            return 1
        print("CATALOG.md is up to date.")
        return 0

    if generated == current:
        print("CATALOG.md already up to date.")
        return 0

    CATALOG_PATH.write_text(generated, encoding="utf-8")
    print("Updated deslopification/memory/CATALOG.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
