# FrSky Ethos Sensor Widget Prompt Template

Use this template to request generation of a simple FrSky Ethos Lua widget that displays configured sensor information.

## Prompt

You are generating code for a FrSky Ethos Lua widget.

### Goal

Create a simple widget named `{WIDGET_NAME}` that displays key information from configured sensors on a model.

### Environment

- Radio/Simulator target: `{ETHOS_TARGET}` (example: Ethos simulator, X20, X18)
- Ethos version: `{ETHOS_VERSION}`
- Project layout:
  - Script file: `src/scripts/{WIDGET_NAME}/main.lua`
  - Optional assets folder: `src/scripts/{WIDGET_NAME}/images/`

### Functional Requirements

1. Show a list of configured sensors with:
   - Sensor display name
   - Current value
   - Unit
2. If no sensors are configured, show a clear empty-state message.
3. Update values during refresh without blocking UI.
4. Keep logic simple and readable for beginners.

### UI Requirements

- Keep layout minimal and readable.
- Use consistent spacing and alignment.
- Ensure text remains legible on small widget zones.
- Prefer simple list formatting over complex graphics.

### Technical Requirements

- Use Ethos widget lifecycle functions appropriately (`create`, `update`, `refresh`, `background` as needed).
- Handle missing/invalid sensor handles safely.
- Avoid unnecessary global state.
- Add brief comments only where behavior is not obvious.

### Inputs

- Preferred widget title: `{WIDGET_TITLE}`
- Sensor selection approach: `{AUTO_OR_MANUAL}` (auto-discover configured sensors or user-selectable options)
- Max sensors to display: `{MAX_SENSORS}`
- Optional formatting rules:
  - Decimal places: `{DECIMAL_PLACES}`
  - Include min/max: `{SHOW_MIN_MAX}`

### Output Format

Return:

1. Complete `main.lua` implementation.
2. Short explanation of structure and sensor handling.
3. Setup notes for testing in Ethos simulator.

### Constraints

- Keep implementation intentionally simple.
- Do not add unrelated telemetry features (logging, alarms, menus) unless requested.
- Maintain compatibility with the specified Ethos version.

### Acceptance Criteria

- Widget loads without runtime errors.
- Widget displays sensor name/value/unit for available sensors.
- Empty-state path works when no sensors are available.
- Code is easy to modify for additional fields later.

### Optional Enhancements (Only if requested)

- Sort sensors alphabetically.
- Highlight stale/missing sensor values.
- Compact mode for smaller widget zones.
