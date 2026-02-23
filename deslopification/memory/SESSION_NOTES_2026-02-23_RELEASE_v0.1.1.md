# Session Notes 2026-02-23 - Release v0.1.1

## What Changed

- Synced local `main` to `origin/main` after merge of PR #15 (`fix/open-bugs` -> `main`).
- Added `CHANGELOG.md` as release-history source of truth.
- Updated release documentation:
  - `README.md`:
    - added `Releases` section with changelog/version/release links.
    - generalized session notes pointer to `deslopification/memory/`.
  - `docs/DEVELOPMENT.md`:
    - added `Release Workflow` section documenting version/changelog/tag/release steps.

## Validation Runs

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `59 passed, 10 skipped`
- `python tools/build.py --project SensorList --dist`
  - result: produced `dist/SensorList-0.1.1.zip`

## Follow-up

- Tag and publish `v0.1.1` GitHub release with `dist/SensorList-0.1.1.zip`.
- Keep `CHANGELOG.md` updated for each future tagged release.
