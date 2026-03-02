# Session Notes 2026-02-27 - Issue #16 Catalog Control-File Deindex

## Note Placement

- Artifact: `session`
- Scope: `memory`
- Concern: `workflow`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Updated `tools/update_memory_catalog.py` so `CATALOG.md` entries/distributions
  index only note artifacts under `deslopification/memory/notes/**`.
- Added a static control-files section in `CATALOG.md` for:
  - `README.md`
  - `CURRENT_STATE.md`
  - `SESSION_NOTE_TEMPLATE.md`
- Updated memory sync tests in `tests/test_memory_catalog_sync.py` to enforce:
  - control-files block is present;
  - control files are excluded from the `## Entries` table.
- Updated wording in:
  - `deslopification/memory/README.md`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - reduce catalog noise by removing control files from searchable historical
    note listings while preserving quick discoverability of entrypoint files.
- Scope guardrails:
  - no note-content rewrites or behavior changes outside memory indexing/docs.

## Validation run(s)

- `python -m pytest tests/test_memory_catalog_sync.py -q`
  - result: pass (13 passed)
- `python -m pytest tests/test_build_py.py -q`
  - result: pass (56 passed)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (89 passed, 14 skipped)
- `python -m pytest -q`
  - result: pass (197 passed, 14 skipped)

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
- If `no`, reason: n/a
