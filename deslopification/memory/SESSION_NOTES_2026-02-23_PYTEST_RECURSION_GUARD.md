# Session Notes 2026-02-23 - Pytest recursion guard

## What changed
- Diagnosed runaway Python process/memory behavior as recursive pytest spawning from docs command tests.
- Updated `tests/test_docs_commands.py` to treat documented `python -m pytest ...` commands as manual commands.
- Added a defensive skip for any `python -m pytest` command in docs command execution logic.
- Updated `tests/test_docs_contracts.py` to require pytest doc commands be classified as manual/environment-dependent.
- Added regression test coverage for pytest-command skip behavior.

## Validation run
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - `50 passed, 8 skipped`

## Follow-up
- If additional doc commands are introduced that can recursively invoke test runs, keep them in `MANUAL_PATTERNS` and avoid auto-execution in docs command tests.
