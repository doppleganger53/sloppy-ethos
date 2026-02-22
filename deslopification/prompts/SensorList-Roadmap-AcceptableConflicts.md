# Prompt

You are generating code for a FrSky Ethos Lua widget enhancement to `SensorList`.

## Goal

Allow defining acceptable conflict patterns so known-safe combinations are de-emphasized in the conflict display.

## Environment

- Widget: `SensorList`
- Script entry: `src/scripts/SensorList/main.lua`
- Target radio/simulator: `X20RS`
- Ethos version target: `1.6.4+`

## Functional Requirements

1. Add a lightweight, maintainable model for acceptable conflicts.
2. At minimum, support whitelist rules based on:
   - `Physical ID`
   - optional `Application ID` match set
3. Rows matching acceptable rules must remain visible but visually distinguished from unresolved conflicts.
4. Unknown or non-whitelisted duplicates must continue to be highlighted.
5. Keep default behavior safe when no whitelist rules are defined.

## Ethos Callback Constraints

- Preserve Ethos widget registration and callback lifecycle.
- Keep `wakeup` and `paint` performant; evaluate whitelist via cached lookup structures.
- Avoid hard failures if rule config is malformed or incomplete.

## Non-Goals

- Do not add complex UI editors for whitelist management in this iteration.
- Do not introduce file I/O configuration persistence in widget runtime.
- Do not alter tooling in `tools/build.py`.

## Data and Interface Expectations

- Define whitelist rules in a clearly documented local Lua table near top-level constants.
- Keep rule structure simple for future extension.
- Add comments only for non-obvious behavior.

## Validation Checklist

1. `luac -p src/scripts/SensorList/main.lua` passes.
2. Rule hit: whitelisted duplicates appear as acceptable (not critical conflict).
3. Rule miss: non-whitelisted duplicates remain highlighted as conflicts.
4. Regression: list sorting, scrolling, and empty-state behavior remain correct.

## Acceptance Criteria

1. Acceptable conflict behavior is configurable via maintainable rule data.
2. Widget clearly differentiates acceptable vs unresolved conflicts.
3. No regressions in existing sensor discovery and display behavior.
