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
HIGH_SIGNAL_CONCERN = {
    "workflow",
    "build",
    "docs",
    "release",
    "testing",
    "metadata",
}
RECENT_HIGH_SIGNAL_LIMIT = 12
RECENT_HIGH_SIGNAL_PER_CONCERN_LIMIT = 3
RECENT_ETHOS_PLATFORM_LIMIT = 6

ALLOWED_ARTIFACTS = {"session", "reference", "summary", "handoff"}
ALLOWED_SCOPES = {"repo", "memory", "ethos-platform", "sensorlist", "ethos-events", "handoff"}
ALLOWED_CONCERNS = {
    "implementation",
    "release",
    "build",
    "docs",
    "testing",
    "workflow",
    "prompts",
    "issue-admin",
    "metadata",
}


@dataclass(frozen=True)
class Entry:
    date: str
    artifact: str
    scope: str
    concern: str
    rel_path: str
    name: str
    title: str
    bytes_count: int
    lines_count: int


def read_scope_description(scope_dir: Path) -> str | None:
    desc_path = scope_dir / ".desc"
    if not desc_path.exists():
        return None
    text = desc_path.read_text(encoding="utf-8").strip()
    if not text:
        return None
    for line in text.splitlines():
        clean = line.strip()
        if clean:
            return clean
    return None


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


def normalized_byte_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    canonical = text.replace("\r\n", "\n").replace("\r", "\n")
    return len(canonical.encode("utf-8"))


def parse_artifact_scope_from_path(rel_path: Path) -> tuple[str, str] | None:
    parts = rel_path.parts
    if len(parts) >= 4 and parts[0] == "notes":
        return parts[1], parts[2]
    return None


def display_path(path: Path, memory_dir: Path = MEMORY_DIR) -> str:
    try:
        return path.relative_to(memory_dir).as_posix()
    except ValueError:
        return path.as_posix()


def parse_note_metadata(path: Path, memory_dir: Path = MEMORY_DIR) -> tuple[str, str, str]:
    text = path.read_text(encoding="utf-8")
    matches = {
        "artifact": re.search(r"^- Artifact: `([^`]+)`", text, flags=re.MULTILINE),
        "scope": re.search(r"^- Scope: `([^`]+)`", text, flags=re.MULTILINE),
        "concern": re.search(r"^- Concern: `([^`]+)`", text, flags=re.MULTILINE),
    }
    missing = [name for name, match in matches.items() if match is None]
    if missing:
        joined = ", ".join(missing)
        raise ValueError(f"{display_path(path, memory_dir)} missing note metadata field(s): {joined}")
    artifact = matches["artifact"].group(1)
    scope = matches["scope"].group(1)
    concern = matches["concern"].group(1)
    if artifact not in ALLOWED_ARTIFACTS:
        raise ValueError(f"{display_path(path, memory_dir)} has unknown artifact: {artifact}")
    if scope not in ALLOWED_SCOPES:
        raise ValueError(f"{display_path(path, memory_dir)} has unknown scope: {scope}")
    if concern not in ALLOWED_CONCERNS:
        raise ValueError(f"{display_path(path, memory_dir)} has unknown concern: {concern}")
    return artifact, scope, concern


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
        from_path = parse_artifact_scope_from_path(rel_path)
        if from_path is None:
            continue
        artifact, scope = from_path
        meta_artifact, meta_scope, concern = parse_note_metadata(path, memory_dir=memory_dir)
        if meta_artifact != artifact or meta_scope != scope:
            raise ValueError(
                f"{rel_path.as_posix()} metadata/path mismatch: "
                f"path={artifact}/{scope}, metadata={meta_artifact}/{meta_scope}"
            )
        entries.append(
            Entry(
                date=extract_date(path.name),
                artifact=artifact,
                scope=scope,
                concern=concern,
                rel_path=rel_path.as_posix(),
                name=path.name,
                title=title,
                bytes_count=normalized_byte_count(path),
                lines_count=line_count(path),
            )
        )
    return entries


def select_recent_high_signal(entries: list[Entry]) -> list[Entry]:
    candidates = [
        item
        for item in entries
        if item.artifact == "session"
        and item.date != "-"
        and item.concern in HIGH_SIGNAL_CONCERN
    ]
    by_concern: dict[str, list[Entry]] = {}
    for item in candidates:
        by_concern.setdefault(item.concern, []).append(item)

    selected: list[Entry] = []
    for concern_entries in by_concern.values():
        ranked = sorted(
            concern_entries,
            key=lambda x: (x.date, x.name, x.rel_path),
            reverse=True,
        )
        selected.extend(ranked[:RECENT_HIGH_SIGNAL_PER_CONCERN_LIMIT])

    return sorted(
        selected,
        key=lambda x: (x.date, x.name, x.rel_path),
        reverse=True,
    )[:RECENT_HIGH_SIGNAL_LIMIT]


def select_recent_ethos_platform(entries: list[Entry]) -> list[Entry]:
    candidates = [
        item
        for item in entries
        if item.scope == "ethos-platform" and item.artifact in {"session", "reference"}
    ]
    ranked = sorted(
        candidates,
        key=lambda x: (x.date != "-", x.date, x.name, x.rel_path),
        reverse=True,
    )
    return ranked[:RECENT_ETHOS_PLATFORM_LIMIT]


def format_slug_list(values: set[str]) -> str:
    ordered = [f"`{value}`" for value in sorted(values)]
    if not ordered:
        return "(none)"
    if len(ordered) == 1:
        return ordered[0]
    if len(ordered) == 2:
        return f"{ordered[0]} or {ordered[1]}"
    return f"{', '.join(ordered[:-1])}, or {ordered[-1]}"


def collect_scope_descriptions(entries: list[Entry], memory_dir: Path = MEMORY_DIR) -> dict[str, str]:
    by_scope: dict[str, set[str]] = {}
    for item in entries:
        scope_dir = memory_dir / Path(item.rel_path).parent
        desc = read_scope_description(scope_dir)
        if desc is None:
            continue
        by_scope.setdefault(item.scope, set()).add(desc)

    descriptions: dict[str, str] = {}
    for scope, values in by_scope.items():
        ordered = sorted(values)
        descriptions[scope] = ordered[0] if len(ordered) == 1 else " / ".join(ordered)
    return descriptions


def render_catalog(entries: list[Entry], memory_dir: Path = MEMORY_DIR) -> str:
    total_files = len(entries)
    total_bytes = sum(item.bytes_count for item in entries)
    total_lines = sum(item.lines_count for item in entries)

    by_artifact: dict[str, int] = {}
    by_scope: dict[str, int] = {}
    by_concern: dict[str, int] = {}
    for item in entries:
        by_artifact[item.artifact] = by_artifact.get(item.artifact, 0) + 1
        by_scope[item.scope] = by_scope.get(item.scope, 0) + 1
        by_concern[item.concern] = by_concern.get(item.concern, 0) + 1

    artifact_order = ["session", "handoff", "reference", "summary"]
    artifact_lines = []
    for artifact in artifact_order:
        count = by_artifact.get(artifact)
        if count is None:
            continue
        label = {
            "session": "session notes",
            "handoff": "handoff/restart notes",
            "reference": "reference notes",
            "summary": "summary notes",
        }[artifact]
        artifact_lines.append(f"- {label}: {count}")

    scope_descriptions = collect_scope_descriptions(entries, memory_dir)
    scope_lines = []
    for scope, count in sorted(by_scope.items(), key=lambda pair: (-pair[1], pair[0])):
        description = scope_descriptions.get(scope, "description missing")
        scope_lines.append(f"- {count} -- {scope} ( {description} )")

    concern_lines = []
    for concern, count in sorted(by_concern.items(), key=lambda pair: (-pair[1], pair[0])):
        concern_lines.append(f"- {count} -- {concern}")

    rows = []
    for item in sorted(entries, key=lambda x: (x.date, x.rel_path)):
        title = item.title.replace("|", "/")
        rows.append(
            f"| {item.date} | {item.artifact} | {item.scope} | {item.concern} | "
            f"[{item.rel_path}]({item.rel_path}) | {title} |"
        )

    recent_high_signal = select_recent_high_signal(entries)
    high_signal_concern_text = format_slug_list(HIGH_SIGNAL_CONCERN)
    recent_lines = []
    for item in recent_high_signal:
        title = item.title.replace("|", "/")
        recent_lines.append(
            f"- {item.date} | {item.concern} | {item.scope} | "
            f"[{item.rel_path}]({item.rel_path}) | {title}"
        )

    recent_ethos_platform = select_recent_ethos_platform(entries)
    ethos_lines = []
    for item in recent_ethos_platform:
        title = item.title.replace("|", "/")
        ethos_lines.append(
            f"- {item.date} | {item.artifact} | {item.concern} | "
            f"[{item.rel_path}]({item.rel_path}) | {title}"
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
        "- Distribution by artifact:",
    ]
    lines.extend(f"  {item}" for item in artifact_lines)
    lines.extend(
        [
            "",
            "- Distribution by scope:",
        ]
    )
    lines.extend(f"  {item}" for item in scope_lines)
    lines.extend(
        [
            "",
            "- Distribution by concern:",
        ]
    )
    lines.extend(f"  {item}" for item in concern_lines)
    lines.extend(
        [
            "",
            "## Recent High-Signal Notes (Auto-generated)",
            "",
            (
                "- Selection: newest session notes where `Concern` is one of "
                f"{high_signal_concern_text}; keep up to "
                f"{RECENT_HIGH_SIGNAL_PER_CONCERN_LIMIT} per concern, then keep "
                f"newest {RECENT_HIGH_SIGNAL_LIMIT} overall."
            ),
        ]
    )
    lines.extend(recent_lines if recent_lines else ["- None."])
    lines.extend(
        [
            "",
            "## Recent Ethos Platform Notes",
            "",
            (
                "- Selection: newest `session` and `reference` notes where "
                "`Scope` is `ethos-platform`; keep newest "
                f"{RECENT_ETHOS_PLATFORM_LIMIT} overall."
            ),
        ]
    )
    lines.extend(ethos_lines if ethos_lines else ["- None."])
    lines.extend(
        [
            "",
            "## Entries",
            "",
            "| Date | Artifact | Scope | Concern | File | Title |",
            "| --- | --- | --- | --- | --- | --- |",
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
