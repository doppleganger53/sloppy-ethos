# Session Notes 2026-02-27 - Issue #16 Repo Focus Unification

## Note Placement

- Category: `session-note`
- Focus: `memory-ops`
- Focus guidance:
  - prefer a specific focus classifier and avoid `general` when possible.
- Store this file under:
  - `deslopification/memory/notes/{category}/{focus}/`

## What changed

- Collapsed two focus classifiers into one:
  - `repo-governance` + `repo-metadata` -> `repo`.
- Moved notes into unified folder:
  - from `notes/session-note/repo-governance/` to `notes/session-note/repo/`
  - from `notes/session-note/repo-metadata/` to `notes/session-note/repo/`
- Updated catalog generator in `tools/update_memory_catalog.py`:
  - `HIGH_SIGNAL_FOCUS` now includes `repo`.
  - recent high-signal selection text now references `repo`.
- Updated memory guidance examples:
  - `deslopification/memory/README.md`
  - `deslopification/memory/CURRENT_STATE.md`
- Added guard test in `tests/test_memory_catalog_sync.py`:
  - `repo` must be present;
  - `repo-governance` and `repo-metadata` must be absent.

## Why

- Root cause or objective:
  - reduce unnecessary taxonomy split and improve focus cardinality quality.
- Scope guardrails:
  - only memory taxonomy/path/indexing and related documentation/test guardrails
    were changed.

## Validation run(s)

- `python -m pytest tests/test_memory_catalog_sync.py -q`
  - result: pass (14 passed)
- `python -m pytest tests/test_build_py.py -q`
  - result: pass (56 passed)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (89 passed, 14 skipped)
- `python -m pytest -q`
  - result: pass (198 passed, 14 skipped)

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
- If `no`, reason: n/a
