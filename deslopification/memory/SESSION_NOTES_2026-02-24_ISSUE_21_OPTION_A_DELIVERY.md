# Session Notes 2026-02-24 - Issue #21 Option A Delivery

## What changed

Implemented issue #21 by extending the existing `tools/build.py` workflow (Option A) instead of replacing it.

Code changes:

- `tools/build.py`
  - Added `--clean` to remove deployed `scripts/{ProjectName}` from simulator path.
  - Added `--sim-radio` to resolve simulator paths from `ETHOS_SIM_PATHS` in deploy config.
  - Added model-specific error output when selected radio simulator path is missing.
  - Added `--help` handling that prints a plain-text build reference and exits.
  - Kept existing `--dist` and `--deploy` behavior intact.
- `tools/build_help.txt`
  - New plain-text command reference for terminal/editor viewing.
- `tools/deploy.config.example.json`
  - Added `ETHOS_SIM_PATHS` example mapping for radio-model-specific paths.

Test updates:

- `tests/test_build_py.py`
  - Added coverage for `--clean`, `--sim-radio`, `--help`, model-map resolution, and clean operation behavior.
- `tests/test_docs_commands.py`
  - Marked new `--clean` docs command as manual/environment-dependent.
- `tests/test_docs_contracts.py`
  - Updated manual command classifier to treat `--clean` commands as environment-dependent.

Documentation updates:

- `README.md`
  - Added `--clean --sim-radio` and `--help` command examples.
  - Documented `ETHOS_SIM_PATHS` + `--sim-radio` usage.
- `docs/DEVELOPMENT.md`
  - Added workflow references for `--clean`, `--sim-radio`, and `--help` behavior.

## Validation run(s)

- `python -m pytest tests/test_build_py.py -q`
  - result: `45 passed`
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `61 passed, 10 skipped`
- `python -m pytest -q`
  - result: `112 passed, 10 skipped`

## Follow-up

- Opened enhancement issue for Option B migration path (doit replacement):
  - https://github.com/doppleganger53/sloppy-ethos/issues/22
