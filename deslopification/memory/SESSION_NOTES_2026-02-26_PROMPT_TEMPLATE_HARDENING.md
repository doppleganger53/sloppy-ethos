# Session Notes 2026-02-26 - Prompt/Template Hardening

## What changed

- Updated `deslopification/prompts/issues/ISSUE_RESOLUTION_TEMPLATE.md` to add:
  - explicit target-branch field
  - branch/worktree gate before editing
  - interaction contract section for input/event issues
  - UI output contract section for visible text/readability expectations
  - fallback parity constraint for helper fallback loaders
  - manual acceptance scenarios section for UI/input changes
  - delivery contract now includes manual acceptance results
- Updated `deslopification/prompts/issues/ISSUE-026-ethos-events-ui-output.md` to encode concrete learnings from implementation:
  - branch/worktree gate
  - touch event interaction contract with expected raw values
  - touch-only toggle guardrail (avoid key conflicts)
  - UI output readability/prefix policy
  - explicit manual acceptance scenarios including navigation interoperability checks
- Expanded hardening patterns to additional issue prompts:
  - `deslopification/prompts/issues/ISSUE-008-touchable-sort-headers.md`
  - `deslopification/prompts/issues/ISSUE-009-conflict-severity-cues.md`
  - `deslopification/prompts/issues/ISSUE-014-sensor-values-column.md`
  - `deslopification/prompts/issues/ISSUE-017-refresh-haptic-feedback.md`
  - `deslopification/prompts/issues/ISSUE-022-doit-migration-evaluation.md`
  - `deslopification/prompts/issues/ISSUE-029-milestone-versioning-policy.md`
  - `deslopification/prompts/issues/ISSUE-030-x20s-simulator-bug.md`
  - Added branch/worktree gates and, where applicable, interaction contracts, UI output contracts, manual acceptance scenarios, and release-template cross-links.
- Updated `.github/PULL_REQUEST_TEMPLATE.md` to align verification checklist with `AGENTS.md` validation matrix instead of SensorList-only checks.
- Added a new dedicated release prompt template:
  - `deslopification/prompts/RELEASE_RESOLUTION_TEMPLATE.md`
  - includes release preflight, branch/worktree guardrails, tag/release existence checks, packaging/publish steps, and delivery contract
- Updated prompt-pack index:
  - `deslopification/prompts/issues/README.md` now references the release template.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `78 passed, 14 skipped`

## Follow-up items

- Consider adding a lightweight test to ensure reusable prompt templates include required guardrail headings (branch gate, validation, delivery contract).
