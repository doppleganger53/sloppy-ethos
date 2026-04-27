# Session Notes 2026-04-27 - PR71 Exclude Source Fix

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `build`

## What changed

- Fixed generic build asset staging so `assets.excludeSource=false` preserves matched source files inside `scripts/{ProjectName}` while still installing copies to radio-root destinations.
- Added regression coverage for install-spec exclusions and ZIP contents when asset source retention is requested.
- Files touched:
  - `tools/build.py`
  - `tests/test_build_py.py`

## Why

- Root cause or objective:
  - PR #71 review found that discovered asset files were still added to `script_exclusions` when their asset group set `excludeSource=false`, so the documented source-retention behavior could not work.
- Scope guardrails:
  - BoundryMap's current `build.json` default asset-source exclusion behavior was left unchanged.

## Validation run(s)

- `python -m pytest tests/test_build_py.py -q`
  - result: pass (`68 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`107 passed, 15 skipped`)
- `python -m pytest -q`
  - result: pass (`250 passed, 15 skipped`)
- `python tools/build.py --project BoundryMap --dist --version codex-pr71-review-check`
  - result: pass; packaged `dist/BoundryMap-codex-pr71-review-check.zip`
- ZIP inspection for `dist/BoundryMap-codex-pr71-review-check.zip`
  - result: pass; `scripts/BoundryMap/maps/` count was `0`, `scripts/BoundryMap/build.json` was absent, and `scripts/BoundryMap/main.lua` was present.

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: bug fix to an existing durable build-manifest decision; no new workflow or behavior policy.
