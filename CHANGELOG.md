# Changelog

All notable changes to this project are documented in this file.

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
