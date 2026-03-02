from __future__ import annotations

import argparse
import sys
from pathlib import Path


def normalize_version(raw_version: str) -> str:
    version = raw_version.strip()
    if version.startswith("v") and len(version) > 1 and version[1].isdigit():
        return version[1:]
    return version


def build_release_label(version: str, project: str | None) -> str:
    normalized_version = normalize_version(version)
    if project:
        return f"{project} v{normalized_version}"
    return normalized_version


def extract_release_notes(changelog_text: str, version: str, project: str | None = None, keep_heading: bool = False) -> str:
    target_label = build_release_label(version, project)
    lines = changelog_text.splitlines()
    start_index: int | None = None
    end_index = len(lines)

    for index, line in enumerate(lines):
        if line.startswith("## ["):
            if start_index is None:
                if line.startswith(f"## [{target_label}] - "):
                    start_index = index
            else:
                end_index = index
                break

    if start_index is None:
        raise ValueError(f"Release entry not found in CHANGELOG.md for '{target_label}'.")

    section_lines = lines[start_index:end_index]
    if not keep_heading:
        section_lines[0] = section_lines[0][3:]
    return "\n".join(section_lines).rstrip() + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Extract one release entry from CHANGELOG.md into a standalone markdown file for gh release --notes-file.",
    )
    parser.add_argument(
        "--version",
        required=True,
        help="Release version to extract. Accepts either '1.0.1' or 'v1.0.1'.",
    )
    parser.add_argument(
        "--project",
        help="Project name for script-scoped releases, for example 'SensorList'.",
    )
    parser.add_argument(
        "--changelog",
        default="CHANGELOG.md",
        help="Path to the changelog source file. Defaults to CHANGELOG.md.",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Path to the output markdown file for gh release --notes-file.",
    )
    parser.add_argument(
        "--keep-heading",
        action="store_true",
        help="Keep the leading '##' changelog heading instead of flattening it for release body use.",
    )
    return parser.parse_args(sys.argv[1:])


def main() -> int:
    args = parse_args()
    changelog_path = Path(args.changelog)
    output_path = Path(args.output)

    if not changelog_path.exists():
        raise SystemExit(f"Changelog file not found: {changelog_path}")

    changelog_text = changelog_path.read_text(encoding="utf-8")
    try:
        notes_text = extract_release_notes(
            changelog_text,
            version=args.version,
            project=args.project,
            keep_heading=args.keep_heading,
        )
    except ValueError as exc:
        raise SystemExit(str(exc)) from exc

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(notes_text, encoding="utf-8")
    print(f"Wrote release notes: {output_path}")
    return 0


if __name__ == "__main__":
    main()
