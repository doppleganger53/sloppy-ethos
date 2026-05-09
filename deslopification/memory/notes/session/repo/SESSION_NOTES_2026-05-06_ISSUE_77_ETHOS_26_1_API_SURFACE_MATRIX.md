# Session Notes 2026-05-06 - Issue #77 Ethos 26.1 API Surface Matrix

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `testing`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Expanded `docs/ETHOS_26_1_COMPATIBILITY.md` with an API surface matrix that
  ties Ethos `26.1` widget, tool, task, source, form, and touch APIs to the
  current repo scripts and smoke targets.
- Aligned `scripts/BoundryMap/tests/lua/test_boundrymap.lua` with the checked
  `26.1` form surface by adding `addSourceField` support and removing the
  non-observed `addBooleanField` stub.
- Extended docs contract coverage so the matrix and smoke targets stay
  enforced.
- Files touched:
  - `docs/ETHOS_26_1_COMPATIBILITY.md`
  - `scripts/BoundryMap/tests/lua/test_boundrymap.lua`
  - `tests/test_docs_contracts.py`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - Issue `#77` needed a concrete matrix tying Ethos `26.1` surfaces to repo
    scripts and smoke targets so the local stubs stop drifting silently.
- Scope guardrails:
  - Kept runtime behavior changes out of scope; this change only updated docs,
    tests, and local stubs.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`161 passed, 18 skipped`)
- `python -m pytest scripts/SensorList/tests scripts/BoundryMap/tests -q`
  - result: pass (`9 passed`)

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
