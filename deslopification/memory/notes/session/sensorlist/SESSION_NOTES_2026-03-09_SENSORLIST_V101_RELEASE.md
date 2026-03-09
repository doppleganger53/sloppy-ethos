# Session Notes 2026-03-09 - SensorList-v1.0.1 Release

## Note Placement

- Artifact: `session`
- Scope: `sensorlist`
- Concern: `release`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Released the `SensorList` script artifact at `1.0.1`.
- Added a script-scoped `CHANGELOG.md` entry for `SensorList v1.0.1` anchored on Issue #14 and explicitly including the Issue #58 visible-value refresh stabilization.
- Rebuilt the script artifact from the release branch for publication:
  - `dist/SensorList-1.0.1.zip`
- Generated standalone release notes for GitHub publication with `tools/write_release_notes.py`.
- Published tag and GitHub release:
  - tag: `SensorList-v1.0.1`
  - release URL: `https://github.com/doppleganger53/sloppy-ethos/releases/tag/SensorList-v1.0.1`
- Files touched:
  - `CHANGELOG.md`
  - `deslopification/memory/notes/session/sensorlist/SESSION_NOTES_2026-03-09_SENSORLIST_V101_RELEASE.md`

## Why

- Root cause or objective:
  - Publish the deferred `SensorList v1.0.1` script release with correct changelog metadata, validated packaging, and a GitHub release body sourced directly from `CHANGELOG.md`.
- Scope guardrails:
  - Root repository version remained `1.0.3`.
  - No repository-wide release policy changed.

## Validation run(s)

- `python tools/session_preflight.py --mode issue --issue-number 14 --issue-kind enhancement --slug sensor-values-column --release-kind script --project SensorList --script-gate-issue 32`
  - result: `PASS_WITH_WARNING` on `release/SensorList-v1.0.1` (release branch differs from the recommended issue branch)
- `luac -p scripts/SensorList/main.lua`
  - result: passed
- `python -m pytest tests/test_sensorlist_widget.py -q`
  - result: `6 passed`
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `95 passed, 14 skipped`
- `python tools/build.py --project SensorList --dist`
  - result: passed
  - artifact: `dist/SensorList-1.0.1.zip`
- `python tools/write_release_notes.py --version 1.0.1 --project SensorList --output .tmp/release-notes-SensorList-v1.0.1.md`
  - result: passed
  - artifact: `.tmp/release-notes-SensorList-v1.0.1.md`
- `gh release view SensorList-v1.0.1 --json tagName,name,url,isDraft,isPrerelease,publishedAt`
  - result: passed
  - release: `https://github.com/doppleganger53/sloppy-ethos/releases/tag/SensorList-v1.0.1`

## Follow-up items

- None. The GitHub release is published and the local/remote release branch no longer exists.

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: release execution only; no durable workflow or behavior policy changed.
