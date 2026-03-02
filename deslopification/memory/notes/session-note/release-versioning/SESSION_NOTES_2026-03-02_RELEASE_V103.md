# Session Notes 2026-03-02 - Release v1.0.3

## Note Placement

- Category: `session-note`
- Focus: `release-versioning`

## What changed

- Bumped the root `VERSION` to `1.0.3` and promoted the current `CHANGELOG.md` `Unreleased` repo release notes into the `v1.0.3` entry.
- Built the repo release asset `dist/sloppy-ethos_scripts.zip`.
- Generated a standalone release notes file from `CHANGELOG.md` and published the `v1.0.3` GitHub release from that file.
- Files touched:
  - `VERSION`
  - `CHANGELOG.md`
  - `deslopification/memory/notes/session-note/release-versioning/SESSION_NOTES_2026-03-02_RELEASE_V103.md`
  - `deslopification/memory/CATALOG.md`

## Why

- Root cause or objective:
  - Ship the next repo-level release so the current documented repo-release asset policy is reflected in a published release.
- Scope guardrails:
  - No Lua/runtime behavior changes.
  - Script artifact versions remain unchanged (`scripts/SensorList/VERSION` stays `1.0.0`; `scripts/ethos_events/VERSION` stays `0.1.0`).

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- `python tools/build.py --project SensorList --project ethos_events --dist`

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: no
