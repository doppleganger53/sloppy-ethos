# Session Notes 2026-04-27 - Generic Build Assets

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `build`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Replaced BoundryMap-specific `mapAssets` support with generic `assets` manifest entries in `tools/build.py`.
- `assets` entries now define source directories, include glob patterns, radio-root destinations, flattening behavior, optional missing-source behavior, and source exclusion behavior entirely in each project's `build.json`.
- Updated BoundryMap's `build.json` so map image and JSON metadata routing is project-defined instead of hardcoded in the build tool.
- Updated root docs, development docs, contributing guidance, BoundryMap README, and build help to describe the generic asset model.
- Files touched:
  - `CONTRIBUTING.md`
  - `README.md`
  - `docs/DEVELOPMENT.md`
  - `scripts/BoundryMap/README.md`
  - `scripts/BoundryMap/build.json`
  - `tests/test_build_py.py`
  - `tools/build.py`
  - `tools/build_help.txt`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - Build asset handling should be script-agnostic; maps are only one possible asset type, and all asset patterns/destinations should live in the project manifest.
- Scope guardrails:
  - Existing explicit `radioFiles` behavior remains available for one-off files.
  - BoundryMap Lua runtime behavior was not changed.

## Validation run(s)

- `python -m pytest tests/test_build_py.py -q`
  - result: pass (`66 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`108 passed, 15 skipped`)
- `python tools/build.py --project BoundryMap --dist --version codex-generic-assets-check`
  - result: pass; packaged `dist/BoundryMap-codex-generic-assets-check.zip`
- ZIP inspection for `dist/BoundryMap-codex-generic-assets-check.zip`
  - result: pass; no `scripts/BoundryMap/maps/` entries leaked into the package, while local map assets installed under radio-root destinations.

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
