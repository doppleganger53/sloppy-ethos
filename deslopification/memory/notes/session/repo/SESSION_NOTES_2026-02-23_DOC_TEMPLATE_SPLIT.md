# Session Notes 2026-02-23 - Documentation Template Split

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `docs`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`


## What changed
- Refactored `.github/ISSUE_TEMPLATE/enhancement.md` to remove docs-specific handling:
  - updated `about` text to scope to widget/tooling/workflow enhancements.
  - removed `Documentation` option from enhancement type checklist.
  - removed docs-only scoped prompt fields.
- Added new docs-specific issue template:
  - `.github/ISSUE_TEMPLATE/documentation.md`
  - includes correction/enhancement type selection, affected docs, source-of-truth reference, acceptance criteria, and validation notes.

## Validation run(s)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q` -> `50 passed, 8 skipped in 0.08s`

## Follow-up items
- If desired, migrate existing documentation enhancements to use the new template title prefix (`[Docs]`).