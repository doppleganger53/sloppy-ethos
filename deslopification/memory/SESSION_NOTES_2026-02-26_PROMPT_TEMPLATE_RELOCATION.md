# Session Notes 2026-02-26 - Prompt Template Relocation

## What changed

- Relocated reusable prompt templates into `deslopification/prompts/templates/`:
  - moved `deslopification/prompts/issues/ISSUE_RESOLUTION_TEMPLATE.md`
    to `deslopification/prompts/templates/ISSUE_RESOLUTION_TEMPLATE.md`
  - moved `deslopification/prompts/RELEASE_RESOLUTION_TEMPLATE.md`
    to `deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md`
- Updated prompt references to new template paths:
  - `deslopification/prompts/issues/README.md`
  - `deslopification/prompts/issues/ISSUE-022-doit-migration-evaluation.md`
  - `deslopification/prompts/issues/ISSUE-029-milestone-versioning-policy.md`
- Preserved and included user refactor that moved completed prompts to:
  - `deslopification/prompts/issues/done/ISSUE-008-touchable-sort-headers.md`
  - `deslopification/prompts/issues/done/ISSUE-009-conflict-severity-cues.md`
  - `deslopification/prompts/issues/done/ISSUE-017-refresh-haptic-feedback.md`
  - updated `deslopification/prompts/issues/README.md` to reflect active vs done prompt locations.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `78 passed, 14 skipped`

## Follow-up items

- If additional prompt indices are added later, keep all reusable template links scoped under `deslopification/prompts/templates/`.
