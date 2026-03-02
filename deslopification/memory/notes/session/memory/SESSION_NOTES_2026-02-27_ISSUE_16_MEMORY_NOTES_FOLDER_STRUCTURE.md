# Session Notes 2026-02-27 - Issue #16 Memory Notes Folder Structure

## Note Placement

- Artifact: `session`
- Scope: `memory`
- Concern: `workflow`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Migrated memory note artifacts from a flat `deslopification/memory/` layout to
  `deslopification/memory/notes/{category}/{focus}/`.
- Kept root memory control files in place:
  - `deslopification/memory/README.md`
  - `deslopification/memory/CURRENT_STATE.md`
  - `deslopification/memory/CATALOG.md`
  - `deslopification/memory/SESSION_NOTE_TEMPLATE.md`
- Updated catalog generator (`tools/update_memory_catalog.py`) to:
  - recurse through memory markdown files;
  - derive category/focus from `notes/{category}/{focus}` when present;
  - index relative paths in `CATALOG.md` links;
  - exclude `CATALOG.md` from indexed snapshot totals for deterministic output.
- Updated process docs/policies/templates for the new note path convention:
  - `AGENTS.md`
  - `deslopification/memory/README.md`
  - `docs/DEVELOPMENT.md`
  - `CONTRIBUTING.md`
  - `deslopification/prompts/templates/ISSUE_RESOLUTION_TEMPLATE.md`
  - `deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md`
- Updated related path references:
  - `deslopification/prompts/issues/ISSUE-030-x20s-simulator-bug.md`
  - `tools/create_todo_issues.py`
- Extended catalog drift tests in `tests/test_memory_catalog_sync.py` to enforce:
  - relative note paths in catalog rows;
  - root memory markdown files limited to index/control files.

## Why

- Root cause or objective:
  - improve human browsing/navigation by reflecting memory classification
    directly in folder structure.
- Scope guardrails:
  - no deletion of historical notes and no unrelated source-code behavior
    changes outside memory/process artifacts.

## Validation run(s)

- `python tools/update_memory_catalog.py --check`
  - result: pass (`CATALOG.md is up to date.`)
- `python -m pytest tests/test_memory_catalog_sync.py -q`
  - result: pass (9 passed)
- `python -m pytest tests/test_prompt_guardrails.py -q`
  - result: pass (14 passed)
- `python -m pytest tests/test_build_py.py -q`
  - result: pass (56 passed)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (89 passed, 14 skipped)
- `python -m pytest -q`
  - result: pass (193 passed, 14 skipped)

## Follow-up items

- Consider adding a helper script for note creation that prompts for
  `category`/`focus` and writes to the correct path automatically.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
- If `no`, reason: n/a
