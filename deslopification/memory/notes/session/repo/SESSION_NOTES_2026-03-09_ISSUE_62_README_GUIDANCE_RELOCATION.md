# Session Notes 2026-03-09 - Issue #62 README Guidance Relocation

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `docs`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Removed contributor-maintenance guidance text from `README.md` download section to keep user-facing docs concise.
- Added equivalent maintenance guidance to `CONTRIBUTING.md` so release-link upkeep instructions live in contributor policy docs instead of end-user README content.
- Files touched:
  - `README.md`
  - `CONTRIBUTING.md`

## Why

- Root cause or objective:
  - Follow-up request on issue #62 asked to keep process guidance out of README and place it in contributor/agent workflow docs.
- Scope guardrails:
  - Limited changes to documentation wording and did not alter release links or test logic.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`97 passed, 15 skipped`)

## Follow-up items

- none

## Current State Sync

- `CURRENT_STATE.md` updated: `no`
- If `no`, reason: no durable workflow policy change; this is placement refinement for existing issue-specific docs guidance.
