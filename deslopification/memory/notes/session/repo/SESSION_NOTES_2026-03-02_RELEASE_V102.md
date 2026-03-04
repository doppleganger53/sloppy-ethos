# Session Notes 2026-03-02 - Release v1.0.2

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `release`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Bumped the root `VERSION` to `1.0.2` and promoted the current `CHANGELOG.md` `Unreleased` repo release notes into the `v1.0.2` entry.
- Built the repo release assets, including the required consolidated multi-script bundle `dist/sloppy-ethos_scripts.zip` plus the single-script ZIP artifacts.
- Updated release workflow policy/docs so future script-scoped releases use `release/{ProjectName}-v{VERSION}` instead of the repo-scoped `release/v{VERSION}` branch naming.
- Published the `v1.0.2` GitHub release from the generated release notes file sourced from `CHANGELOG.md`.
- Files touched:
  - `VERSION`
  - `CHANGELOG.md`
  - `AGENTS.md`
  - `README.md`
  - `docs/DEVELOPMENT.md`
  - `deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md`
  - `deslopification/memory/CURRENT_STATE.md`
  - `deslopification/memory/notes/session-note/release-versioning/SESSION_NOTES_2026-03-02_RELEASE_V102.md`
  - `deslopification/memory/CATALOG.md`

## Why

- Root cause or objective:
  - Ship the next repo-level release so the documented consolidated script bundle deliverable is attached to a published release along with the current release-notes-file workflow.
  - Close the branch-naming gap left by the earlier script release so future script release-prep work uses a script-scoped branch name.
- Scope guardrails:
  - No Lua/runtime behavior changes.
  - Script artifact versions remain unchanged (`scripts/SensorList/VERSION` stays `1.0.0`; `scripts/ethos_events/VERSION` stays `0.1.0`).

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `90 passed, 14 skipped`
- `python -m pytest tests/test_build_py.py -q`
  - result: `56 passed`
- `python tools/build.py --project SensorList --dist`
  - result: pass; generated `dist/SensorList-1.0.0.zip`
- `python tools/build.py --project ethos_events --dist`
  - result: pass; generated `dist/ethos_events-0.1.0.zip`
- `python tools/build.py --project SensorList --project ethos_events --dist`
  - result: pass; generated `dist/sloppy-ethos_scripts.zip`
- `python -m zipfile -l dist/sloppy-ethos_scripts.zip`
  - result: pass; archive contains both `scripts/SensorList/` and `scripts/ethos_events/`

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
