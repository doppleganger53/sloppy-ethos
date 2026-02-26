# Session Notes 2026-02-26 - Multi-script ZIP Naming Update

## What changed

- Updated multi-script dist naming convention to the fixed filename:
  - `dist/sloppy-ethos_scripts.zip`
- Updated `tools/build.py` multi-project archive basename to `sloppy-ethos_scripts`.
- Synced related docs/contracts/tests:
  - `README.md`
  - `docs/DEVELOPMENT.md`
  - `tools/build_help.txt`
  - `AGENTS.md`
  - `tests/test_build_py.py`
  - `tests/test_docs_contracts.py`

## Validation run(s)

- `python -m pytest -q`
  - result: `96 passed, 11 skipped`
- manual command:
  - `python tools/build.py --project SensorList --project ethos_events --dist`
  - result: `Packaged multi-script ZIP: ...\\dist\\sloppy-ethos_scripts.zip`

## Follow-up items

- None.
