# Handoff 2026-02-22 - TODO Execution

## What Changed

- Normalized `TODO.md` into dependency-aware checklist IDs (`TODO-01`..`TODO-09`) with done criteria.
- Added issue templates:
  - `.github/ISSUE_TEMPLATE/enhancement.md`
  - `.github/ISSUE_TEMPLATE/refactor.md`
- Added roadmap prompt artifacts:
  - `deslopification/prompts/SensorList-Roadmap-SortHeaders.md`
  - `deslopification/prompts/SensorList-Roadmap-ConflictDisplay.md`
  - `deslopification/prompts/SensorList-Roadmap-AcceptableConflicts.md`
- Added pytest harness and tests:
  - `requirements-dev.txt`
  - `tests/helpers.py`
  - `tests/test_build_py.py`
  - `tests/test_docs_commands.py`
  - `tests/test_sensorlist_widget.py`
  - `tests/lua/test_sensorlist.lua`
  - `tests/__init__.py`
- Added guarded test exports in `src/scripts/SensorList/main.lua` via `__SENSORLIST_TEST__`.
- Refactored `AGENTS.md` to repo-wide guidance and moved SensorList operational specifics to:
  - `deslopification/memory/SensorList.md`
- Updated `src/scripts/SensorList/README.md` to current behavior and Python-first workflow.
- Added issue creation automation script:
  - `tools/create-todo-issues.ps1`

## Validation Run

- `pytest -q` -> `23 passed, 5 skipped`
- `luac -p src/scripts/SensorList/main.lua` -> pass
- `python tools/build.py --project SensorList --dist` -> pass, produced `dist/SensorList-0.1.0.zip`

## Blockers / Follow-Ups

- GitHub issues for TODO tracking have been created:
  - Parent tracker: `https://github.com/doppleganger53/sloppy-ethos/issues/7`
  - Child issues: `#1` through `#6`
- `TODO.md` item `TODO-03` is now complete.
