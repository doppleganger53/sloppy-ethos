# Session Notes 2026-03-02 - Memory Taxonomy Refactor

## Note Placement

- Artifact: `session`
- Scope: `memory`
- Concern: `workflow`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Refactored the memory taxonomy from `Category` + `Focus` to `Artifact` + `Scope` + `Concern`.
- Migrated the full memory note corpus into `notes/{artifact}/{scope}/`, including new `ethos-platform`, `sensorlist`, and `ethos-events` scope splits.
- Added extracted companion notes for mixed Ethos/script findings so reusable Ethos runtime knowledge and script-local impacts can be retrieved independently.
- Added the reusable Ethos reference note `deslopification/memory/notes/reference/ethos-platform/EthosPlatform.md` and narrowed `SensorList.md` to script-specific operating guidance.
- Updated the catalog generator, memory catalog tests, contributor docs, prompt templates, and helper references to enforce the new memory contract.
- Files touched:
  - `tools/update_memory_catalog.py`
  - `tests/test_memory_catalog_sync.py`
  - `AGENTS.md`
  - `deslopification/memory/README.md`
  - `deslopification/memory/CURRENT_STATE.md`
  - `deslopification/memory/SESSION_NOTE_TEMPLATE.md`
  - `docs/DEVELOPMENT.md`
  - `CONTRIBUTING.md`

## Why

- Root cause or objective:
  - The prior `Category` axis had become almost non-discriminating and the `Focus` axis mixed subject and concern, which increased cold-start context cost and made reusable Ethos knowledge harder to retrieve consistently.
- Scope guardrails:
  - Preserved historical note content, avoided compatibility shims, and limited structural changes to the memory system, related docs, and supporting tooling/tests.

## Validation run(s)

- `python tools/update_memory_catalog.py`
  - result: pass (`Updated deslopification/memory/CATALOG.md`)
- `python tools/update_memory_catalog.py --check`
  - result: pass (`CATALOG.md is up to date.`)
- `python -m pytest tests/test_memory_catalog_sync.py -q`
  - result: pass (`27 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`91 passed, 14 skipped`)
- `python -m pytest tests/test_build_py.py -q`
  - result: pass (`56 passed`)
- `python -m pytest -q`
  - result: pass (`225 passed, 14 skipped`)

## Follow-up items

- If a future script line grows beyond `sensorlist` and `ethos-events`, keep `ethos-platform` as the reusable default and add new script scopes only when script-local retrieval clearly benefits.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
- If `no`, reason: n/a
