# Prompt

You are generating code for a FrSky Ethos Lua widget enhancement to `SensorList`.

## Goal

Refine conflict display behavior so users can quickly distinguish:
- likely problematic conflicts
- acceptable shared-ID scenarios
based on `Physical ID` and `Application ID` combinations.

## Environment

- Widget: `SensorList`
- Script entry: `src/scripts/SensorList/main.lua`
- Target radio/simulator: `X20RS`
- Ethos version target: `1.6.4+`

## Functional Requirements

1. Keep current table view (`Name`, `Physical ID`, `Application ID`).
2. Improve visual conflict signaling so same-physical-id rows are not treated as uniformly severe.
3. Differentiate at least two states:
   - duplicate physical ID with duplicate application ID (high conflict risk)
   - duplicate physical ID with distinct application IDs (possible acceptable case)
4. Make state cues readable without relying only on color.
5. Preserve deterministic sorting and stable row rendering.

## Ethos Callback Constraints

- Maintain existing callback lifecycle (`create`, `paint`, `wakeup`, `event`).
- Keep rendering and polling lightweight; no expensive scans per frame.
- Handle simulator/radio API differences defensively.

## Non-Goals

- Do not add user configuration menus in this change.
- Do not alter packaging/deploy scripts.
- Do not remove duplicate-name rows from display.

## Performance and Stability

- Keep deep scan behavior bounded to avoid instruction-budget issues.
- Ensure empty-state and debug information still render cleanly.
- Preserve touch/wheel scrolling interactions.

## Validation Checklist

1. `luac -p src/scripts/SensorList/main.lua` passes.
2. Simulator data with duplicate physical+application IDs is clearly marked high risk.
3. Simulator data with duplicate physical IDs but distinct application IDs is shown with lower-severity cue.
4. Regression: no runtime errors in empty-state path and no scroll regressions.

## Acceptance Criteria

1. Conflict display conveys severity using observable row cues.
2. Users can distinguish likely-bad and potentially-acceptable duplicates at a glance.
3. Widget remains stable and readable under normal and empty data conditions.
