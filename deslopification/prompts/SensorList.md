# Prompt

You are generating code for a FrSky Ethos Lua widget.

### Goal

Create a simple widget named `SensorList` (`WIDGET_NAME`) that displays a list
of all configured sensors in a tabular format with 3 columns: `Name`,
`Physical ID`, and `Application ID`.

### Environment

- Radio/Simulator target: X20RS
- Ethos version: `1.6.4+`
- Project layout:
  - Script file: `scripts/{WIDGET_NAME}/main.lua`
  - Optional assets folder: `scripts/{WIDGET_NAME}/images/`

### Functional Requirements

1. Show a list of configured sensors with:
   - Sensor display name
   - Physical ID
   - Application ID
2. If no sensors are configured, show a clear empty-state message.
3. Update values during refresh without blocking UI.
4. The list should be sorted by Physical ID and Application ID (ascending).
5. If conflicting sensors exist, matching sensors should be color-coded with
   unique, readable text colors.
   - Sensor 1 - 00 - 01
   - Sensor 2 - 00 - 00
   - Sensor 3 - 0A - 6800
   - Sensor 4 - 0A - 6801
   - Sensor 1 and Sensor 2 lines should share a distinguishable text color.
   - Sensor 3 and Sensor 4 lines should share a different distinguishable
     text color.

### UI Requirements

- Keep layout minimal and readable.
- Use consistent spacing and alignment.
- Target usage in a full screen widget.
- Prefer simple list formatting over complex graphics.

### Technical Requirements

- Use Ethos widget lifecycle functions appropriately (`create`, `update`,
  `refresh`, `background` as needed).
- Handle missing/invalid sensor handles safely.
- Avoid unnecessary global state.
- Add brief comments only where behavior is not obvious.

### Inputs

- Preferred widget title: SensorList
- Sensor selection approach: AUTO (auto-discover configured sensors or
  user-selectable options)
- Max sensors to display: no maximum, list should scroll or be paged
- Optional formatting rules:
  - No additional formatting rules

### Output Format

Return:

1. Complete `main.lua` implementation.
2. Short explanation of structure and sensor handling.
3. Setup notes for testing in Ethos simulator.

### Constraints

- Keep implementation intentionally simple.
- Do not add unrelated telemetry features (logging, alarms, menus) unless
  requested.
- Maintain compatibility with the specified Ethos version.

### Acceptance Criteria

- Widget loads without runtime errors.
- Widget displays sensor name/value/unit for available sensors.
- Empty-state path works when no sensors are available.
- Code is easy to modify for additional fields later.

### Optional Enhancements (Only if requested)

- Compact mode for smaller widget zones.

## Domain Knowledge

- In Ethos, Physical ID and Application ID must be unique for discovered
  sensors, otherwise a `Sensor Conflict` audio warning message is played by
  the OS.
- Conflicting sensors result in incorrect behavior/readings or complete loss
  of functionality.
- Certain base Physical IDs can be shared; for example, sensors of a receiver
  or redundancy bus may share a Physical ID but have unique Application IDs.
  This does not cause a sensor conflict message.
- All option add-on sensors must have unique Physical and Application IDs in
  order to function correctly, for example GPS, speed sensors, voltage
  sensors, XAct serial servos, and others.

## Use cases / desire for functionality

- Many add-on sensors can share Physical and Application IDs by default.
  For example, XAct servos default to a particular Physical and Application ID.
  The user must configure them one by one to be unique before installation and
  connection to the receiver. In complex aircraft builds, it can be difficult
  to keep track of sensor IDs, especially as the build continues or sensors
  are added over time.
- An easy way to identify and resolve sensor conflicts is highly desirable.

