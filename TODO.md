# TODO List

Execution order dependencies:
- `TODO-01` and `TODO-02` must be done before `TODO-03`.
- `TODO-05` depends on test harness setup in `TODO-04`.
- `TODO-08` and `TODO-09` are done after tests to prevent stale command guidance.

- [x] `TODO-01` Add enhancement issue template
  Done when `.github/ISSUE_TEMPLATE/enhancement.md` exists with required sections and frontmatter.
- [x] `TODO-02` Add refactoring issue template
  Done when `.github/ISSUE_TEMPLATE/refactor.md` exists with required sections and frontmatter.
- [x] `TODO-03` Create appropriate issues for remainder of TODO list
  Done when backlog child issues and a parent tracker issue exist in GitHub and are cross-linked. Completed via issues `#1` through `#7`.
- [x] `TODO-04` Add pytest harness setup
  Done when `requirements-dev.txt`, `tests/`, and shared test helpers exist and run under `pytest`.
- [x] `TODO-05` Unit tests for SensorList
  Done when Lua-driven SensorList behavior tests run through pytest orchestration.
- [x] `TODO-06` Unit tests for `build.py`
  Done when core build/deploy/version/config behaviors are covered with isolated pytest unit tests.
- [x] `TODO-07` Unit tests for documentation commands
  Done when docs commands are parsed and validated for syntax/path existence with explicit manual skip handling.
- [x] `TODO-08` Refactor `AGENTS.md` to streamline context and move script-specific notes to memory
  Done when repo-wide guidance remains in `AGENTS.md` and SensorList details are in `deslopification/memory/SensorList.md`.
- [x] `TODO-09` Update `src/scripts/SensorList/README.md` for current state and Python-first commands
  Done when README reflects scroll/conflict behavior and uses `python tools/build.py` as primary flow.
