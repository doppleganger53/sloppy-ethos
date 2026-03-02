# Session Notes 2026-02-26 - Dist Cleanup

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `build`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`


## What changed

- Extended `python tools/build.py --clean` so it removes simulator deployments and clears the `dist/` (`--out-dir`) output directory.
- Documented the new cleanup behavior in `README.md`, `docs/DEVELOPMENT.md`, and `tools/build_help.txt`.
- Added `clean_dist_dir` coverage and main-flow expectations in `tests/test_build_py.py`.

## Validation run(s)

- `python -m pytest tests/test_build_py.py -q`

## Follow-up items

- None.