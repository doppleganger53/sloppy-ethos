# Session Notes 2026-03-08 - Build Clean Order Fix

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `build`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Reordered `tools/build.py` so `--clean` executes before any `--deploy` or `--dist` work when multiple action flags are supplied together.
- Added regression coverage proving `python tools/build.py --project SensorList --clean --deploy --dist` runs in the order: clean simulator, clean dist, deploy, then dist packaging.
- Files touched:
  - `tools/build.py`
  - `tests/test_build_py.py`

## Why

- Root cause or objective:
  - The previous main flow executed `--dist` before the clean block, which could immediately delete freshly packaged artifacts when `--clean --deploy --dist` was used in one command.
- Scope guardrails:
  - Kept the existing single-action behavior and simulator path resolution unchanged.

## Validation run(s)

- `python -m pytest tests/test_build_py.py -q`
  - result: pass (`56 passed`)

## Follow-up items

- none

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: no durable workflow/policy baseline changed
