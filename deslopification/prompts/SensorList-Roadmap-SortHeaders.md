# Prompt

You are generating code for a FrSky Ethos Lua widget enhancement to `SensorList`.

## Goal

Add touchable column headers in `SensorList` to control sorting at runtime.

## Environment

- Widget: `SensorList`
- Script entry: `src/scripts/SensorList/main.lua`
- Target radio/simulator: `X20RS`
- Ethos version target: `1.6.4+`

## Functional Requirements

1. Header taps cycle sorting for each column (`Name`, `Physical ID`, `Application ID`).
2. Sort direction toggles ascending/descending on repeated tap of same header.
3. Active sort key and direction are visually indicated in the header row.
4. Default sort remains deterministic when no user interaction occurs.
5. Touch-scrolling continues to work and does not conflict with header tap detection.

## Ethos Callback Constraints

- Use current callback model already in `SensorList` (`create`, `paint`, `wakeup`, `event`).
- Keep `wakeup` lightweight and avoid per-frame full rescans.
- Do not introduce OpenTX/EdgeTX callback names.

## Non-Goals

- Do not add settings persistence across power cycles.
- Do not redesign the widget layout beyond minimal header indicators.
- Do not modify package/deploy tooling.

## Performance and Stability

- Preserve existing polling/deep-scan cadence unless a measurable reason exists.
- Keep scroll offset clamped after sort changes.
- Avoid runtime errors when touch events omit expected coordinates.

## Validation Checklist

1. `luac -p src/scripts/SensorList/main.lua` passes.
2. Simulator: header tap changes sort key and direction correctly.
3. Simulator: empty state remains readable and stable.
4. Regression: manual scrolling and conflict color-group rendering still work.

## Acceptance Criteria

1. User can tap any header to sort by that column.
2. Sort indicator is visible and accurate.
3. No regressions in scrolling, deep scan behavior, or empty state rendering.
