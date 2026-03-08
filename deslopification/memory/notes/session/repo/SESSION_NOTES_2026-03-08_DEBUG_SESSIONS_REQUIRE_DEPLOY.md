# Session Notes 2026-03-08 - Debug Sessions Require Deploy

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `workflow`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Added a repository workflow rule that simulator debugging sessions for Lua behavior must deploy the touched script before session closeout.
- Documented the rule in repo policy and contributor-facing docs, with `python tools/build.py --project SensorList --deploy` as the minimum SensorList debug-session closeout command.
- Updated current-state memory because this is now a durable workflow baseline.
- Files touched:
  - `AGENTS.md`
  - `README.md`
  - `docs/DEVELOPMENT.md`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - Debugging sessions were otherwise easy to close out with validated code but stale simulator state, which weakens runtime verification and makes regressions harder to catch quickly.
- Scope guardrails:
  - This rule is workflow-only; it does not change `tools/build.py` semantics or require `--dist` for every debugging session.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass

## Follow-up items

- none

## Current State Sync

- `CURRENT_STATE.md` updated: yes
- If `no`, reason: n/a
