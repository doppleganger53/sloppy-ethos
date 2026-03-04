# Session Notes - 2026-02-21 (Release + Packaging)

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `release`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`


## What Changed

- Removed hardcoded visible-row cap in `SensorList` and switched to dynamic rows based on window height.
- Removed duplicate-name filtering in fallback sensor discovery (`system.getSource` scan path), so sensors with identical names are now shown.
- Added version source-of-truth file: `VERSION` (currently `0.1.0`).
- Updated packaging naming from timestamped ZIPs to:
  - `dist/{ProjectName}-{version}.zip`
- Updated both build paths:
  - `tools/build.py`
  - `tools/build-package.ps1`
- Updated docs:
  - `README.md`
  - `docs/DEVELOPMENT.md`
- Added high-level roadmap section in `README.md`.

## Release State

- Commit: `9c45844` (`Release v0.1.0`)
- Tag: `v0.1.0` (pushed)
- Dist artifact: `dist/SensorList-0.1.0.zip`

## Notes for Next Session

- If GitHub Release entry is not yet created, publish release `v0.1.0` and attach `dist/SensorList-0.1.0.zip`.
- For next version bump:
  1. Update `VERSION`
  2. Run `python tools/build.py --project SensorList --dist`
  3. Tag using `v{VERSION}`