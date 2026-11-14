# Prompt

You are generating code for a FrSky Ethos Lua widget.

## Goal

Create a widget named `{WIDGET_NAME}` that implements:

- Primary function: `{PRIMARY_FUNCTION}`
- Primary user value: `{USER_VALUE}`

## Environment

- Radio target(s): `{RADIO_TARGETS}` (example: `X20RS`)
- Ethos firmware target: `{ETHOS_VERSION}` (example: `26.1.0`; list exact
  builds used for validation if supporting a range)
- Project layout:
  - Script entry point: `scripts/{WIDGET_NAME}/main.lua`
  - Optional images: `scripts/{WIDGET_NAME}/images/`
  - Optional i18n: `scripts/{WIDGET_NAME}/i18n/`

## Ethos API Requirements (Critical)

- Use Ethos-native widget registration:
  - `init()` calling `system.registerWidget({...})`
  - `return { init = init }`
- Select callbacks for the target firmware and requested behavior. Check FrSky's
  Lua reference or examples at the matching release; for example, see
  [26.1.0 Lua docs](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/26.1.0/lua-doc.zip)
  under `system.registerWidget`.
- **Input widget:** In 26.1.0, register `event = event` and handle
  `event(widget, category, value, x, y)`. Match both category and value.
  Return `true` for input the widget handles; return `false` for unrelated
  input so Ethos can handle it. FrSky's [26.1.0 RSSI example](https://github.com/FrSkyRC/ETHOS-Feedback-Community/blob/26.1.0/lua/examples/widget-rssi/main.lua)
  matches `EVT_KEY` with `KEY_ENTER_LONG` for a long Enter press. For wheel
  turns, confirm the category and value delivered with the widget in focus on
  the requested target. [SensorList](../../../scripts/SensorList/main.lua)
  shows the same consume/pass-through pattern for touch.
- **Passive display:** Use `create` and `paint`, plus `wakeup` if data can
  change. Omit `event` when the display takes no input, as in FrSky's
  [26.1.0 cell-voltage widget](https://github.com/FrSkyRC/ETHOS-Feedback-Community/blob/26.1.0/lua/examples/widget-flvss/main.lua).
  Add `configure`, `read`, or `write` if the widget needs settings.
- Do not assume OpenTX/EdgeTX callback naming (`refresh`, `background`,
  etc.) unless explicitly requested.
- Callback safety:
  - Defensively handle unexpected `nil` callback args.
  - Never crash on startup or page navigation if context is incomplete.

## Functional Requirements

1. `{REQ_1}`
2. `{REQ_2}`
3. `{REQ_3}`
4. Empty-state behavior: `{EMPTY_STATE_TEXT_OR_RULE}`
5. Data ordering/grouping rules: `{SORT_OR_GROUP_RULES}`
6. Interaction model:
   - Default behavior when list/content exceeds visible area: `{OVERFLOW_BEHAVIOR}`
   - Input: `{CONTROL_AND_VISIBLE_RESPONSE}` (example: long Enter advances
     the selection one row; use `none` for a passive display).

## Data Access Requirements

- Primary data source(s): `{PRIMARY_DATA_APIS}`
- Fallback data source(s): `{FALLBACK_DATA_APIS}`
- Handle simulator-vs-radio API differences safely.
- If data handles may be table or userdata, support both where practical.
- Avoid expensive rescans every frame; use cached discovery where possible.

## Performance & Stability Requirements

- Keep `wakeup` lightweight; avoid heavy loops each call.
- Poll data on a defined interval: `{POLL_INTERVAL_SECONDS}`.
- Use periodic deep-rescan only when needed: `{DEEP_SCAN_POLICY}`.
- UI should be stable and deterministic (no forced auto-scroll unless requested).
- Avoid instruction-budget issues (`max instructions count reached`).

## UI Requirements

- Layout target: `{LAYOUT_TARGET}` (example: fullscreen widget)
- Visual style: `{VISUAL_STYLE}` (minimal, readable by default)
- Text/layout constraints:
  - `{TEXT_TRUNCATION_RULE}`
  - `{COLUMN_ALIGNMENT_RULE}`
  - `{COLOR_RULES}`

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
- Expected ZIP layout: `scripts/{WIDGET_NAME}/...`

## Validation Requirements

- Script compiles with `luac -p`.
- Widget appears in Ethos widget picker.
- Widget renders without runtime errors in simulator and/or target radio.
- Data display path and empty-state path both validated.
- For an input widget, focus it and exercise the specified control. For the
  selection example, start on row 1, hold Enter once, and expect row 2.
  Try an unhandled navigation key: the selection should stay put and the
  normal Ethos navigation action should still work.
- For a passive display, change the source value and check that it repaints
  without input (for a static display, verify the expected text or graphic).
- Report the API source separately from test results. For each runtime check,
  name the simulator or physical radio, firmware build, and observed result;
  mark checks not run as untested.

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

