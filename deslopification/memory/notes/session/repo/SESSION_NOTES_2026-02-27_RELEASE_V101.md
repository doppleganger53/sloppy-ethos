# Session Notes 2026-02-27 - Release v1.0.1

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `release`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Closed Issue #39 by capturing the release-scope clarity decision in `CHANGELOG.md` and marking `v1.0.0` as a bad release that `v1.0.1` supersedes while keeping the script artifact versions at `0.1.1`.
- Bumped the root `VERSION` to `1.0.1` and added this release entry, then generated the corresponding session note and refreshed the catalog index so the release history is discoverable.
- Files touched:
  - `VERSION`
  - `CHANGELOG.md`
  - `deslopification/memory/notes/session-note/release-versioning/SESSION_NOTES_2026-02-27_RELEASE_V101.md`
  - `deslopification/memory/CATALOG.md`

## Why

- Root cause or objective:
  - Finish the repo-level release flow by shipping `v1.0.1` that explains the Issue #39 policy clarity work and explicitly records that `v1.0.0` was a bad release.
- Scope guardrails:
  - No script or widget behavior changes; `scripts/SensorList/VERSION` and `scripts/ethos_events/VERSION` remain `0.1.1`.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: 90 passed, 14 skipped
- `python -m pytest tests/test_build_py.py -q`
  - result: 56 passed

## Follow-up items

- None; release branch merged/removed, `v1.0.1` tagged/published, and `v1.0.0` marked bad on GitHub.

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: release metadata only; no durable workflow/behavior change.
