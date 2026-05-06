# Session Notes 2026-05-05 - Issue #76 Ethos 26.1 Baseline

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `docs`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Added `docs/ETHOS_26_1_COMPATIBILITY.md` as the repo-level baseline for the
  checked `ETHOS-Feedback-Community` `26.1` reference checkout, simulator
  validation targets, and the current compatibility matrix.
- Linked the new baseline doc from `README.md`, `docs/DEVELOPMENT.md`, and
  `docs/REPOSITORY_LAYOUT.md`.
- Extended docs tests so the baseline doc is required, linked, and checked for
  the core `26.1` reference facts and follow-up issue links.
- Updated `deslopification/memory/CURRENT_STATE.md` to point future sessions at
  the new compatibility baseline doc.
- Files touched:
  - `docs/ETHOS_26_1_COMPATIBILITY.md`
  - `README.md`
  - `docs/DEVELOPMENT.md`
  - `docs/REPOSITORY_LAYOUT.md`
  - `tests/test_docs_commands.py`
  - `tests/test_docs_contracts.py`
  - `deslopification/memory/CATALOG.md`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - Issue `#76` required a durable, reproducible Ethos `26.1` baseline that
    future compatibility work can reference without repeating the initial repo
    and issue triage.
- Scope guardrails:
  - Kept the change focused on the baseline documentation, docs tests, and
    memory sync.
  - Left script inventory cleanup and downstream compatibility fixes to their
    follow-up issues instead of expanding this issue into implementation work.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`143 passed, 17 skipped`)
- `python tools/update_memory_catalog.py`
  - result: pass (`Updated deslopification/memory/CATALOG.md`)

## Follow-up items

- Close or continue the downstream compatibility issues as each script or API
  probe is implemented, starting with `#77`, `#78`, `#79`, `#80`, `#81`, and
  `#82`.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
