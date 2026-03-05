# Session Notes 2026-02-26 - Issue #22 PR Closure Guardrail

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `issue-admin`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`


## What changed

- Reopened issue `#22` to restore expected PR-driven closure flow.
- Added contributor workflow guardrail in `CONTRIBUTING.md`:
  - linked issues must not be manually closed before PR merge;
  - use PR closing keywords and allow GitHub auto-close on merge.
- Added matching agent prompt guardrail in:
  - `deslopification/prompts/templates/ISSUE_RESOLUTION_TEMPLATE.md`
  - explicit issue-closure sequencing in delivery contract.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`84 passed, 14 skipped`)

## Follow-up items

- Open PR for `feature/22-doit-migration-evaluation` with `Closes #22` in PR body and close issue via merge.