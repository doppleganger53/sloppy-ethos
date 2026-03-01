# Draft Prompt: Implement Issue #44 (Arduino FBus Integration Widget)

Use this prompt to implement GitHub issue `#44`:
`https://github.com/doppleganger53/sloppy-ethos/issues/44`

# Prompt

You are generating code for a FrSky Ethos Lua widget.

## Goal

Create a widget named `FBusTelemetry` that implements:

- Primary function: discover and display Arduino-backed FBus telemetry,
  especially accelerometer-oriented and other non-FrSky sensor inputs
- Primary user value: give pilots a stable, readable way to verify custom FBus
  sensor data directly on an Ethos screen without needing custom debug tools

## Environment

- Radio target(s): `X20RS` (and other Ethos radios that expose the same widget
  APIs)
- Ethos version target: `1.6.4+`
- Project layout:
  - Script entry point: `scripts/FBusTelemetry/main.lua`
  - Optional images: `scripts/FBusTelemetry/images/`
  - Optional i18n: `scripts/FBusTelemetry/i18n/`

## Ethos API Requirements (Critical)

- Use Ethos-native widget registration:
  - `init()` calling `system.registerWidget({...})`
  - `return { init = init }`
- Use Ethos callbacks only (as needed): `create`, `paint`, `wakeup`,
  `configure`, `read`, `write`
- Do not assume OpenTX/EdgeTX callback naming (`refresh`, `background`,
  etc.) unless explicitly requested.
- Callback safety:
  - Defensively handle unexpected `nil` callback args.
  - Never crash on startup or page navigation if context is incomplete.

## Functional Requirements

1. Discover telemetry sources exposed to Ethos and identify likely
   Arduino-backed FBus sensors using available source metadata, without
   depending on one exact sensor naming scheme.
2. Render a readable list of supported sensors, with accelerometer-oriented
   fields (for example X/Y/Z axes) shown first when available.
3. Display each row with a stable label, current value, and units/status text,
   while degrading gracefully when a sensor is present but unreadable.
4. Empty-state behavior: show `No supported Arduino FBus sensors detected.`
   when no matching sources are currently available.
5. Data ordering/grouping rules: group accelerometer-related sensors first,
   then show remaining supported sensors in case-insensitive alphabetical order
   by display label.
6. Interaction model:
   - Default behavior when list/content exceeds visible area: manual scrolling
     only; do not auto-scroll
   - Input navigation method(s): wheel up/down, touch swipe/drag if supported by
     the current radio

## Data Access Requirements

- Primary data source(s): Ethos telemetry source discovery and read APIs
  available to widgets, including source handles/IDs returned by the runtime and
  value reads performed through `system.getSource(...)` or equivalent widget-safe
  source access paths exposed by the current Ethos build
- Fallback data source(s): cached last-known supported sensor list and
  last-known-good display values when a source read temporarily fails
- Handle simulator-vs-radio API differences safely.
- If data handles may be table or userdata, support both where practical.
- Avoid expensive rescans every frame; use cached discovery where possible.

## Performance & Stability Requirements

- Keep `wakeup` lightweight; avoid heavy loops each call.
- Poll data on a defined interval: `0.5` seconds.
- Use periodic deep-rescan only when needed: full source rediscovery on startup,
  after manual refresh, and no more than once every `5` seconds while running if
  the cache is empty or source metadata appears to have changed.
- UI should be stable and deterministic (no forced auto-scroll unless requested).
- Avoid instruction-budget issues (`max instructions count reached`).

## UI Requirements

- Layout target: fullscreen widget
- Visual style: minimal, readable, diagnostics-first
- Text/layout constraints:
  - truncate long labels at the right edge before overlap; prefer clipping or
    ellipsis over line wrapping
  - align labels left and values right, with units/status text kept compact and
    visually secondary
  - use normal text color for valid readings, a muted color for unavailable
    values, and a warning/accent color only for clearly invalid or unsupported
    states

## Technical Constraints

- Keep implementation intentionally simple and maintainable.
- Prefer root-cause solutions over compatibility shims or temporary workarounds.
- Avoid unnecessary global state.
- Add brief comments only where behavior is non-obvious.
- Do not add unrelated features unless requested (alarms, logs, advanced menus).

## Packaging & Delivery Requirements

- Provide complete `main.lua`.
- If requested, include/update packaging script so output ZIP installs through
  Ethos Suite Lua import.
- Expected ZIP layout: `scripts/FBusTelemetry/...`

## Validation Requirements

- Script compiles with `luac -p`.
- Widget appears in Ethos widget picker.
- Widget renders without runtime errors in simulator and/or target radio.
- Data display path and empty-state path both validated.
- Interaction behavior validated (for example: manual scroll input works).

## Output Format

Return:

1. Complete `main.lua` implementation.
2. Short explanation of architecture and data flow.
3. Explicit test checklist (simulator + radio when applicable).
4. Any assumptions made due to API/version uncertainty.

## Optional Enhancements (Only if explicitly requested)

- Compact mode for smaller zones.
- User-configurable options (`configure/read/write`).
- Debug overlay toggle for API/event diagnostics.
