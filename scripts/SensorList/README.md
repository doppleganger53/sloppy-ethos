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

## Try the widget in the simulator

From the repository folder, run the isolated smoke suite:

```powershell
$smoke = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SensorList-X20RS-FCC.json --persist-dir "$smoke"
```

For a visible check, use the [WebSimulator GUI](../../tools/sim/harness/README.md)
and add SensorList to a widget area. If a task calls for updating a live Suite
installation, verify its path in `tools/deploy.config.json` and select its
radio explicitly, for example
`python tools/build.py --project SensorList --deploy --sim-radio X20RS`.

## Lua Coverage In VS Code

- Install the recommended `Coverage Gutters` extension in VS Code.
- Install Lua coverage tooling with LuaRocks:
  - `luarocks install luacov`
  - `luarocks install luacov-reporter-lcov`
- Generate coverage with `lua -lluacov scripts/SensorList/tests/lua/test_sensorlist.lua` followed by `luacov -r lcov`; Coverage Gutters can then display `coverage/lua/luacov.report.out`.
