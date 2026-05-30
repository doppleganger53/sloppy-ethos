# Session Notes 2026-05-18 - WebSimulator Runtime Cache Reuse

## Note Placement

- Artifact: `session`
- Scope: `ethos-platform`
- Concern: `testing`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Refactored GUI harness serving so run directories no longer receive copied
  runtime JS/WASM files.
- GUI mode now serves `/runtime/{file}` from the cached WebSimulator package
  under `tools/sim/radios/` while serving generated GUI wrapper files from the
  run directory.
- Preserved per-run `index.html` and `persist_manifest.json` as generated GUI
  artifacts, with logs/results still under the run directory.
- Deleted pre-2026-05-18 directories under `tools/sim/runs/` after resolving
  each target inside the run root. The `.gitkeep` file and 2026-05-18 run
  directories were preserved.

## Why

- Runtime packages are cache artifacts, not run-specific evidence. Reusing the
  cached runtime keeps `tools/sim/runs/` focused on run artifacts and avoids
  duplicate JS/WASM payloads.

## Validation run(s)

- `python -m pytest tests/test_sim_harness.py -q`
  - result: pass (`19 passed`)
- `python -m pytest tests/test_sim_harness.py tests/test_build_py.py tests/test_docs_commands.py tests/test_docs_contracts.py tests/test_memory_catalog_sync.py -q`
  - result: pass (`294 passed, 25 skipped`)

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
