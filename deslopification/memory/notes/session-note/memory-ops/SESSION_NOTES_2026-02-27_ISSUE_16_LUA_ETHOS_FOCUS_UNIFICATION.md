# Session Notes 2026-02-27 - Issue #16 Lua-Ethos Focus Unification

## Note Placement

- Category: `session-note`
- Focus: `memory-ops`
- Focus guidance:
  - prefer a specific focus classifier and avoid `general` when possible.
  - use `lua-ethos` for Ethos Lua script/widget/tool notes.
- Store this file under:
  - `deslopification/memory/notes/{category}/{focus}/`

## What changed

- Collapsed two script-specific focus classifiers into one:
  - `ethos-events` + `sensorlist` -> `lua-ethos`.
- Moved note artifacts:
  - `notes/session-note/ethos-events/*` -> `notes/session-note/lua-ethos/*`
  - `notes/session-note/sensorlist/*` -> `notes/session-note/lua-ethos/*`
  - `notes/domain-note/sensorlist/SensorList.md` ->
    `notes/domain-note/lua-ethos/SensorList.md`
- Updated guidance to classify future Ethos Lua script notes as `lua-ethos`:
  - `AGENTS.md`
  - `deslopification/memory/README.md`
  - `deslopification/memory/CURRENT_STATE.md`
  - `deslopification/memory/SESSION_NOTE_TEMPLATE.md`
  - `docs/DEVELOPMENT.md`
  - `CONTRIBUTING.md`
- Updated direct path references:
  - `deslopification/prompts/issues/ISSUE-030-x20s-simulator-bug.md`
  - `tools/create_todo_issues.py`
- Updated catalog generator:
  - `tools/update_memory_catalog.py` now includes `lua-ethos` in high-signal
    focus selection.
- Added guard test:
  - `tests/test_memory_catalog_sync.py` enforces `lua-ethos` exists and legacy
    script focus names are absent.

## Why

- Root cause or objective:
  - unify overlapping script focus buckets to improve taxonomy quality and
    future note consistency for Ethos Lua projects.
- Scope guardrails:
  - only memory taxonomy, note placement, references, and guardrails changed.

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
