# SensorList

Ethos widget that lists configured sensors in a sortable table with:

- Name
- PhysID
- AppID
- SubID
- optional Value

Conflicting sensors that share the same `PhysID`, `AppID`, and `SubID` are
color-grouped to make true duplicates easier to identify.

## Current Behavior

- Displays columns for `Name`, `PhysID`, `AppID`, `SubID`.
- Optional `Display Value` widget setting adds a fifth `Value` column without changing the default layout.
- Default sort is deterministic by `PhysID`, then `AppID`, then `SubID`, then name.
- Tapping `Name`, `PhysID`, or `AppID` changes sort key and toggles ascending/descending for that column (`^` / `v` in header).
- `SubID` is display-only and acts as the final tie-breaker for stable sort order.
- `Value` is display-only and shows the best available formatted/string/numeric sensor value, falling back to `--` when unavailable.
- With `Display Value = Yes`, visible sensor values are refreshed during `wakeup()` at roughly 5 Hz.
- Duplicate `PhysID` + `AppID` + `SubID` rows are color-grouped to help identify conflicts.
- Alternating row bands improve left-to-right readability across the visible columns.
- List navigation is manual via wheel/button/touch scrolling (no forced auto-scroll).
- Sensor discovery uses staged background expansion after initial load so large sensor lists can populate without tripping the Ethos callback instruction budget.
- Manual long-press queues a fresh staged refresh and triggers best-effort completion feedback (`system.playHaptic` fallback to `system.playTone`).

## Widget Option

- `Display Value = No`: default four-column layout (`Name`, `PhysID`, `AppID`, `SubID`)
- `Display Value = Yes`: compressed five-column layout with `Value` shown at the right edge

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
