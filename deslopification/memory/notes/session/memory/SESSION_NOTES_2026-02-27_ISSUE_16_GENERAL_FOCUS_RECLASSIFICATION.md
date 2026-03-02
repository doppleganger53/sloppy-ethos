# Session Notes 2026-02-27 - Issue #16 General Focus Reclassification

## Note Placement

- Artifact: `session`
- Scope: `memory`
- Concern: `workflow`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Reclassified all notes under `notes/session-note/general/` based on full note
  content review:
  - moved 11 notes into existing focus buckets.
  - created and used 2 new focus buckets:
    - `issue-lifecycle`
    - `repo-governance`
- Updated memory policy/docs to discourage `general` for session notes:
  - `AGENTS.md`
  - `deslopification/memory/README.md`
  - `deslopification/memory/SESSION_NOTE_TEMPLATE.md`
  - `deslopification/memory/CURRENT_STATE.md`
  - `docs/DEVELOPMENT.md`
  - `CONTRIBUTING.md`
- Updated `tools/update_memory_catalog.py`:
  - added `issue-lifecycle` and `repo-governance` to recent high-signal focus
    selection.
  - expanded fallback focus classifier keyword checks for the two new buckets.
  - updated recent high-signal selection text accordingly.
- Extended regression tests in `tests/test_memory_catalog_sync.py`:
  - assert `notes/session-note/general/` has no markdown notes.
  - assert no `session-note` entries use `general` focus.

## Why

- Root cause or objective:
  - `general` had low informational value and risked increasing over time,
    reducing catalog filtering effectiveness.
- Scope guardrails:
  - no historical note deletion; only reclassification/path moves and memory
    workflow guardrails.

## Validation run(s)

- `python -m pytest tests/test_memory_catalog_sync.py -q`
  - result: pass (11 passed)
- `python -m pytest tests/test_build_py.py -q`
  - result: pass (56 passed)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (89 passed, 14 skipped)
- `python -m pytest -q`
  - result: pass (195 passed, 14 skipped)

## Follow-up items

- Consider a small note-creation helper that requires explicit `focus` input
  and blocks `general` unless overridden with an explicit flag.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
- If `no`, reason: n/a
