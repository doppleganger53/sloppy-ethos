# Session Notes 2026-02-24 - Root-Cause Strategy Policy

## What changed

- Updated repository-level agent policy in `AGENTS.md` with a new `Root-Cause Strategy` section:
  - prioritize root-cause fixes over compatibility shims/legacy aliases
  - treat stale local metadata/cache issues as an operational fix first when appropriate
  - require explicit request or confirmed compatibility need before adding shims
  - require documenting compatibility removal conditions when shims are approved
- Updated `deslopification/prompts/templates/EthosWidgetPromptTemplate.md` technical constraints to reinforce root-cause-first implementation behavior.

## Audit result

- Current worktree is not yet "refactor-only".
- Legacy compatibility alias code is still present in:
  - `tests/test_docs_commands.py`
  - `tests/test_docs_contracts.py`

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `61 passed, 11 skipped`

## Follow-up

- If desired, remove legacy alias test logic to align strictly with the new root-cause strategy and keep only the direct refactor changes.
