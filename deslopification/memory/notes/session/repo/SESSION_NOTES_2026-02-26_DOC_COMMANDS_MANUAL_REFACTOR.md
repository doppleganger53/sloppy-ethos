# Session Notes 2026-02-26 - Docs Command Manual Classification Refactor

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `docs`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`


## What changed

- Refactored docs command execution tests to remove redundant hardcoded command paths.
- Replaced `MANUAL_PATTERNS` with:
  - a small explicit `MANUAL_COMMANDS` tuple for exact manual-only commands
  - shared `is_manual_command(command)` rule logic for environment-dependent/manual commands
- Updated `tests/test_docs_commands.py` to use `is_manual_command` for skip decisions.
- Updated `tests/test_docs_contracts.py` to assert manual classification via `is_manual_command` instead of tuple membership.
- Adjusted manual-skip tests to avoid unnecessary hardcoded repository-specific pytest paths.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `73 passed, 10 skipped`

## Follow-up items

- None.