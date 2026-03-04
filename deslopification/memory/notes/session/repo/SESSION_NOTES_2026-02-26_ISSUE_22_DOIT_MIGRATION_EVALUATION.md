# Session Notes 2026-02-26 - Issue #22 `build.py` to `doit` Migration Evaluation

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `build`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`


## What changed

- Implemented issue #22 as a decision-first tooling evaluation.
- Added a decision record at:
  - `docs/decisions/ISSUE-022-doit-migration-evaluation.md`
- Decision outcome:
  - no-go for migrating to `doit` at current repository scale;
  - retain `tools/build.py` as the canonical build/deploy workflow.
- Documented discoverability pointer in:
  - `docs/DEVELOPMENT.md` under `## Tooling Decision Records`.
- No command-surface or behavior changes were introduced.

## Validation run(s)

- `python -m pytest tests/test_build_py.py -q`
  - result: pass (`56 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - initial run: fail (`Missing documented path: build.py` from docs token parsing)
  - fix: removed backticks from `build.py` text in `docs/DEVELOPMENT.md`
  - rerun result: pass (`84 passed, 14 skipped`)

## Follow-up items

- Revisit migration only if the triggers listed in
  `docs/decisions/ISSUE-022-doit-migration-evaluation.md` are met.