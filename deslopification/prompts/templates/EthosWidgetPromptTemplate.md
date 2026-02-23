# Prompt

You are generating code for a FrSky Ethos Lua widget.

## Goal

Create a widget named `{WIDGET_NAME}` that implements:

- Primary function: `{PRIMARY_FUNCTION}`
- Primary user value: `{USER_VALUE}`

## Environment

- Radio target(s): `{RADIO_TARGETS}` (example: `X20RS`)
- Ethos version target: `{ETHOS_VERSION}` (example: `1.6.4+`)
- Project layout:
  - Script entry point: `src/scripts/{WIDGET_NAME}/main.lua`
  - Optional images: `src/scripts/{WIDGET_NAME}/images/`
  - Optional i18n: `src/scripts/{WIDGET_NAME}/i18n/`

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

1. `{REQ_1}`
2. `{REQ_2}`
3. `{REQ_3}`
4. Empty-state behavior: `{EMPTY_STATE_TEXT_OR_RULE}`
5. Data ordering/grouping rules: `{SORT_OR_GROUP_RULES}`
6. Interaction model:
   - Default behavior when list/content exceeds visible area: `{OVERFLOW_BEHAVIOR}`
   - Input navigation method(s): `{INPUT_METHODS}` (example: wheel up/down)

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
