# Session Notes 2026-02-27 - Issue #16 Branch Gate Hardening

## What changed

- Added issue-session branch preflight tool:
  - `tools/session_preflight.py`
  - enforces issue-mode block on `main` with deterministic remediation output.
  - allows non-issue mode on `main` with explicit ask-before-mutate reminder.
- Updated `AGENTS.md` branch policy:
  - defined issue-linked work scope (issue URL/number or issue prompt-driven).
  - made issue preflight required before issue-linked edits.
  - codified non-issue-on-main allowance with mandatory user confirmation
    before mutations.
- Updated reusable issue prompt template:
  - `deslopification/prompts/templates/ISSUE_RESOLUTION_TEMPLATE.md`
  - removed `main` as issue default target branch.
  - added required preflight command and stop-on-block behavior.
- Normalized all open issue prompts to include preflight and non-main defaults:
  - `deslopification/prompts/issues/ISSUE-010-acceptable-conflict-definitions.md`
  - `deslopification/prompts/issues/ISSUE-014-sensor-values-column.md`
  - `deslopification/prompts/issues/ISSUE-016-memory-optimization.md`
  - `deslopification/prompts/issues/ISSUE-022-doit-migration-evaluation.md`
  - `deslopification/prompts/issues/ISSUE-026-ethos-events-ui-output.md`
  - `deslopification/prompts/issues/ISSUE-030-x20s-simulator-bug.md`
- Added regression tests:
  - `tests/test_session_preflight.py`
  - `tests/test_prompt_guardrails.py`

## Root Cause Analysis

- Immediate cause:
  - Issue-linked implementation work was started on `main` without a hard gate.
- Why it happened:
  - Existing policy documented branch conventions but did not provide an
    executable preflight check that fails fast on `main` for issue sessions.
  - Template wording still allowed issue defaults to `main` in several open
    issue prompts.
- Corrective controls implemented:
  - executable issue preflight tool (`tools/session_preflight.py`).
  - AGENTS startup requirement to run preflight for issue-linked sessions.
  - prompt/template normalization plus tests that detect regressions.

## Validation run(s)

- `python -m pytest tests/test_session_preflight.py -q`
  - result: pass (`6 passed`)
- `python -m pytest tests/test_build_py.py -q`
  - result: pass (`56 passed`)
- `python -m pytest tests/test_prompt_guardrails.py -q`
  - result: pass (`14 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`89 passed, 14 skipped`)
- `python -m pytest -q`
  - result: pass (`171 passed, 14 skipped`)

## Follow-up items

- Consider adding optional VS Code task wrapper for
  `python tools/session_preflight.py` to simplify manual use.
