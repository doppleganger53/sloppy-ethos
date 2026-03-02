from __future__ import annotations

import importlib.util
import runpy
import sys
from pathlib import Path

import pytest


def load_release_notes_module():
    repo_root = Path(__file__).resolve().parents[1]
    script_path = repo_root / "tools" / "write_release_notes.py"
    spec = importlib.util.spec_from_file_location("write_release_notes_module", script_path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


release_notes = load_release_notes_module()


SAMPLE_CHANGELOG = """# Changelog

All notable changes to this project are documented in this file.

## [SensorList v1.0.0] - 2026-03-02

### Changed

- Released the SensorList script artifact.

### Testing

- `python tools/build.py --project SensorList --dist`

## [1.0.1] - 2026-02-27

### Changed

- Closed Issue #39.
"""


def test_normalize_version_strips_optional_v_prefix():
    assert release_notes.normalize_version("v1.0.1") == "1.0.1"
    assert release_notes.normalize_version("1.0.1") == "1.0.1"


def test_build_release_label_supports_script_release():
    assert release_notes.build_release_label("1.0.0", "SensorList") == "SensorList v1.0.0"


def test_extract_release_notes_repo_strips_section_heading():
    notes = release_notes.extract_release_notes(SAMPLE_CHANGELOG, version="1.0.1")
    assert notes.startswith("[1.0.1] - 2026-02-27\n")
    assert "### Changed" in notes
    assert "## [1.0.1]" not in notes


def test_extract_release_notes_script_finds_project_scoped_entry():
    notes = release_notes.extract_release_notes(SAMPLE_CHANGELOG, version="v1.0.0", project="SensorList")
    assert notes.startswith("[SensorList v1.0.0] - 2026-03-02\n")
    assert "### Testing" in notes


def test_extract_release_notes_preserves_heading_when_requested():
    notes = release_notes.extract_release_notes(SAMPLE_CHANGELOG, version="1.0.1", keep_heading=True)
    assert notes.startswith("## [1.0.1] - 2026-02-27\n")


def test_extract_release_notes_raises_when_release_missing():
    with pytest.raises(ValueError, match="Release entry not found"):
        release_notes.extract_release_notes(SAMPLE_CHANGELOG, version="9.9.9")


def test_main_writes_requested_output_file(monkeypatch, tmp_path: Path, capsys):
    changelog_path = tmp_path / "CHANGELOG.md"
    output_path = tmp_path / ".tmp" / "release-notes.md"
    changelog_path.write_text(SAMPLE_CHANGELOG, encoding="utf-8")
    monkeypatch.setattr(
        release_notes.sys,
        "argv",
        [
            "write_release_notes.py",
            "--version",
            "1.0.1",
            "--changelog",
            str(changelog_path),
            "--output",
            str(output_path),
        ],
    )

    assert release_notes.main() == 0
    assert output_path.read_text(encoding="utf-8").startswith("[1.0.1] - 2026-02-27\n")
    assert "Wrote release notes:" in capsys.readouterr().out


def test_main_exits_when_changelog_missing(monkeypatch, tmp_path: Path):
    output_path = tmp_path / "release-notes.md"
    monkeypatch.setattr(
        release_notes.sys,
        "argv",
        [
            "write_release_notes.py",
            "--version",
            "1.0.1",
            "--changelog",
            str(tmp_path / "missing.md"),
            "--output",
            str(output_path),
        ],
    )

    with pytest.raises(SystemExit, match="Changelog file not found"):
        release_notes.main()


def test_module_entrypoint_invokes_main(monkeypatch, tmp_path: Path):
    repo_root = Path(__file__).resolve().parents[1]
    script_path = repo_root / "tools" / "write_release_notes.py"
    changelog_path = tmp_path / "CHANGELOG.md"
    output_path = tmp_path / "release-notes.md"
    changelog_path.write_text(SAMPLE_CHANGELOG, encoding="utf-8")
    monkeypatch.setattr(
        sys,
        "argv",
        [
            str(script_path),
            "--version",
            "1.0.1",
            "--changelog",
            str(changelog_path),
            "--output",
            str(output_path),
        ],
    )

    runpy.run_path(str(script_path), run_name="__main__")
