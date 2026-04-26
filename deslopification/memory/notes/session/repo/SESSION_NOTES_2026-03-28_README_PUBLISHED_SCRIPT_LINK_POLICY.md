# Session Notes 2026-03-28 - README Published Script Link Policy

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `docs`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Corrected the README download-link policy to cover only currently published single-script GitHub release assets, not every unreleased script directory under `scripts/`.
- Updated the docs contract test so it validates the scripts explicitly listed in `README.md` instead of requiring every `scripts/{ProjectName}/VERSION` entry to already be published.
- Synchronized the workflow wording across:
  - `AGENTS.md`
  - `README.md`
  - `CONTRIBUTING.md`
  - `docs/DEVELOPMENT.md`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - Adding `scripts/BoundryMap/VERSION` caused the old docs contract to fail even though `BoundryMap` has not been released on GitHub yet.
- Scope guardrails:
  - Kept the fix focused on release-link policy wording and docs contract behavior.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`98 passed, 14 skipped`)
- `python -m pytest tests/test_boundrymap_widget.py -q`
  - result: pass (`3 passed`)

## Follow-up items

- When `BoundryMap` gets its first published script release, add the release ZIP link to the README download section in the same release-prep session.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
