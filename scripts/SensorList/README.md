# SensorList

Ethos widget that lists configured sensors in a sortable table with:

- Name
- Physical ID
- Application ID

Conflicting sensors that share the same Physical ID are color-grouped to make
potential conflicts easier to identify.

## Current Behavior

- Displays columns for `Name`, `Physical ID`, `Application ID`.
- Sort order is deterministic by `Physical ID`, then `Application ID`, then name.
- Duplicate physical IDs are color-grouped to help identify conflicts.
- List navigation is manual via wheel/button/touch scrolling (no forced auto-scroll).
- Sensor discovery runs on initial widget load and explicit long-press refresh only (no periodic polling).

## Build and Install

1. Build the install ZIP from repository root:
   `python tools/build.py --project SensorList --dist`
2. In Ethos Suite, run the Lua install/import action and select the ZIP from `dist/`.
3. Transfer/sync to the radio.

Installed path on radio: `scripts/SensorList`.
Artifact version source: `scripts/SensorList/VERSION`.

## Simulator Deploy

- Deploy script files into simulator scripts directory:
  `python tools/build.py --project SensorList --deploy`
- Configure simulator paths via `tools/deploy.config.json` `ETHOS_SIM_PATHS`, with one entry marked `"default": true`.
