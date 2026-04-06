# Session Notes 2026-03-28 - Optional Radio File Build Manifest

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `build`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Extended `tools/build.py` with an optional project-local build manifest (`build.json`) that declares installable `radioFiles` outside `/scripts`.
- Updated dist packaging, multi-project bundle staging, simulator deploy, and simulator clean so manifest-declared files are copied to radio-root destinations and removed from the script payload itself.
- Added `scripts/BoundryMap/build.json` so the example `maps/WJRC` bitmap and metadata JSON now install to the radio's `bitmaps/GPS/` and `documents/user/` paths.
- Bumped `scripts/BoundryMap/VERSION` to `0.1.1` for the installable asset change.
- Expanded regression and docs coverage for the new manifest workflow, including auto-discovery of all script README files in the docs command tests.
- Files touched:
  - `tools/build.py`
  - `tools/build_help.txt`
  - `tests/test_build_py.py`
  - `tests/test_docs_commands.py`
  - `README.md`
  - `docs/DEVELOPMENT.md`
  - `CONTRIBUTING.md`
  - `scripts/BoundryMap/build.json`
  - `scripts/BoundryMap/README.md`
  - `scripts/BoundryMap/VERSION`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - `BoundryMap` requires installable assets that live on the radio outside `/scripts`, but the existing build/deploy flow only knew how to package and copy `scripts/{ProjectName}`.
- Scope guardrails:
  - Kept the new workflow manifest-driven and optional instead of hard-coding `BoundryMap` paths into the build script.

## Validation run(s)

- `python -m pytest tests/test_build_py.py -q`
  - result: pass (`61 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`107 passed, 15 skipped`)
- `python tools/build.py --project BoundryMap --clean --deploy --sim-radio X20RS`
  - result: pass after rerunning with elevated filesystem access; deployed `scripts/BoundryMap/main.lua`, `bitmaps/GPS/WJRC.bmp`, and `documents/user/WJRC.json`
- `python tools/build.py --project BoundryMap --dist`
  - result: pass; produced `dist/BoundryMap-0.1.1.zip`

## Follow-up items

- If additional BoundryMap sample maps are added, declare each installable bitmap/metadata file pair in `scripts/BoundryMap/build.json`.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
