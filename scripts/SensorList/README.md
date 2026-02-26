# SensorList

Ethos widget that lists configured sensors in a sortable table with:

- Name
- Physical ID
- Application ID

Conflicting sensors are highlighted with severity cues so likely collisions are
easy to distinguish from lower-risk shared-ID cases.

## Current Behavior

- Displays columns for `Name`, `Physical ID`, `Application ID`.
- Default sort is deterministic by `Physical ID`, then `Application ID`, then name.
- Tapping a column header changes sort key and toggles ascending/descending for that column (`^` / `v` in header).
- Conflict severity is shown with both color and text markers in the `Name` column:
  - `[!]` high severity: duplicate `Physical ID` + duplicate `Application ID`, or any duplicate `Physical ID` group containing unknown `Application ID` values (`--`).
  - `[~]` lower severity: duplicate `Physical ID` with distinct known `Application ID` values.
- List navigation is manual via wheel/button/touch scrolling (no forced auto-scroll).
- Sensor discovery runs on initial widget load and explicit long-press refresh only (no periodic polling).
- Manual long-press refresh triggers best-effort completion feedback (`system.playHaptic` fallback to `system.playTone`).

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
