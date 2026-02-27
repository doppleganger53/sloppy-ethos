# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Changed

- Documented Issue #42 repo release deliverables so GitHub repo releases explicitly include `dist/sloppy-ethos_scripts.zip` alongside single-script ZIP assets, including the build step `python tools/build.py --project SensorList --project ethos_events --dist`.

## [1.0.1] - 2026-02-27

### Changed

- Closed Issue #39 with a clearer repo release-scope workflow and clarified that the release-conflating script gates are not blockers for repo-only work.
- Marked `v1.0.0` as a bad release and documented that `v1.0.1` supersedes it without touching script artifacts (`scripts/SensorList/VERSION` and `scripts/ethos_events/VERSION` remain at `0.1.1`).

### Testing

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- `python -m pytest tests/test_build_py.py -q`

## [1.0.0] - 2026-02-27

### Changed

- Root repository version bumped to `1.0.0` to close the Issue #16 memory/catalog milestone while leaving the `scripts/SensorList/VERSION` and `scripts/ethos_events/VERSION` artifacts at `0.1.1`.
- Documented the completed memory/catalog automation and focus taxonomy updates, plus the decision record from Issue #22 confirming that `tools/build.py` remains the canonical build/deploy workflow.

### Testing

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- `python -m pytest tests/test_build_py.py -q`

## [0.3.0] - 2026-02-26

### Changed

- Adopted a main-only branch strategy:
  - `main` is the only long-lived branch.
  - short-lived issue branch conventions are now explicit (`feature/`, `fix/`, `docs/`, `chore/`).
  - release-prep branches now use `release/v{VERSION}`.
- Updated repository workflow and contributor docs for the new branching/release model:
  - `AGENTS.md`
  - `docs/DEVELOPMENT.md`
  - `CONTRIBUTING.md`
  - `README.md`
- Updated issue/release prompt templates and open issue prompt defaults to align with `main` as the default target branch.
- Clarified release/version policy:
  - root `VERSION` is bumped on release branches.
  - script `VERSION` files remain independent and are bumped only when script artifacts/behavior change.
  - optional `-rc.N` prerelease usage is defined for release branch stabilization.

## [0.1.1] - 2026-02-23

### Added

- New `ethos_events` utility integration:
  - `src/scripts/ethos_events/main.lua` entrypoint and local wrapper files.
  - Deployable system-tool script path support via `src/scripts/tools/ethos_events.lua`.
  - Shared library export in `src/scripts/lib/ethos_events.lua`.
  - Ethos-compatible tool icon assets.
- New event tracer documentation in `src/scripts/ethos_events/README.md`.

### Changed

- `SensorList` touch handling now uses the verified Ethos touch event
  contract (`EVT_TOUCH` values `16640/16642/16641`).
- `SensorList` refresh behavior is manual-first:
  - refresh on widget create.
  - refresh on long-press interaction.
  - no periodic refresh polling in `wakeup()`.
- Documentation and docs-command tests updated for the new `ethos_events` commands and paths.

### Fixed

- Added fallback long-press detection on touch end when native
  `EVT_TOUCH_LONG` is unavailable.
- Resized and aligned the `ethos_events` icon to avoid Ethos loader failures.

## [0.1.0] - 2026-02-21

### Added

- Initial public release of `SensorList`.
- Fullscreen Ethos widget for sensor listing with sortable inspection workflow.
- Build and deploy tooling for simulator and install ZIP packaging.
