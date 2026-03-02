# Session Notes 2026-03-02 - Issue #45 Prompt Guardrail Fix

## Note Placement

- Category: `session-note`
- Focus: `prompts`
- Focus guidance:
  - prefer a specific focus classifier and avoid `general` when possible.
  - use `lua-ethos` for Ethos Lua script/widget/tool notes.
- Store this file under:
  - `deslopification/memory/notes/{category}/{focus}/`

## What changed

- Added the required target-branch guardrail to the open issue prompt for Issue #45.
- Added the required `tools/session_preflight.py --mode issue` preflight step to the same prompt so it satisfies repository guardrail tests.
- Aligned the canonical issue title in the prompt with the live GitHub issue title.
- Files touched:
  - `deslopification/prompts/issues/ISSUE-045-smartmapper-function-mapping-script.md`

## Why

- Root cause or objective:
  - PR CI failed because the Issue #45 open-issue prompt omitted required branch-target and issue-preflight instructions enforced by `tests/test_prompt_guardrails.py`.
- Scope guardrails:
  - Kept the rest of the SmartMapper prompt content intact and limited changes to the missing guardrail metadata.

## Validation run(s)

- `python -m pytest tests/test_prompt_guardrails.py -q`
  - result: pass (`10 passed`)
- `python -m pytest -q`
  - result: pass (`206 passed, 14 skipped`)

## Follow-up items

- none

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: this fixes a single prompt artifact and does not change durable repository-wide workflow policy.
