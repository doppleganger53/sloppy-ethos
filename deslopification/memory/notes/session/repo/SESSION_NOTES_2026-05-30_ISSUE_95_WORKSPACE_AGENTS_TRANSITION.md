# Session Notes 2026-05-30 - Issue #95 workspace AGENTS transition

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `workflow`
- Store this file under:
  - `deslopification/memory/notes/session/repo/`

## What changed

- Documented the parent `EthosLua` workspace boundary for agents and
  contributors.
- Clarified that `sloppy-ethos/AGENTS.md` remains authoritative once a session
  enters this repository.
- Added guardrails for sibling reference checkouts:
  - treat them as read-only evidence by default
  - update them only with checkout-scoped Git commands
  - verify license and attribution before copying reference material
- Files touched:
  - `AGENTS.md`
  - `README.md`
  - `CONTRIBUTING.md`
  - `docs/DEVELOPMENT.md`
  - `deslopification/prompts/templates/ISSUE_RESOLUTION_TEMPLATE.md`
  - `deslopification/memory/CURRENT_STATE.md`
  - `deslopification/memory/CATALOG.md`

## Why

- Root cause or objective:
  - Agents may now start from the parent `EthosLua` workspace, while
    implementation and repo workflow must remain scoped to `sloppy-ethos/`.
- Scope guardrails:
  - No runtime API, Lua behavior, build behavior, or release versioning changed.
  - Reference projects were not modified.

## Validation run(s)

- `python tools/update_memory_catalog.py`
  - result: pass; regenerated `deslopification/memory/CATALOG.md`
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py tests/test_memory_catalog_sync.py tests/test_prompt_guardrails.py -q`
  - result: pass; 205 passed, 21 skipped

## Follow-up items

- Convert or replace the parent workspace's local `GPSAccuMap2.0/` snapshot with
  a pullable Git checkout only after its upstream remote is confirmed.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
