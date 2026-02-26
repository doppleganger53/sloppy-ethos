# Prompt: Implement Issue #14 (Sensor Values Column)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/14`
- Title: `[Enhancement] Sensor Values`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`

## Mission

Add an optional sensor-value display mode to `SensorList`:

- `Display Value = No`: current 3-column behavior
- `Display Value = Yes`: add a 4th value column

## Required Context

- `AGENTS.md`
- `scripts/SensorList/main.lua`
- `scripts/SensorList/README.md`
- `docs/SensorList/SENSORLIST_ARCHITECTURE.md`
- `tests/lua/test_sensorlist.lua`
- `tests/test_sensorlist_widget.py`

## Scope

- In scope:
  - Add a configurable option for showing values.
  - Render a value column only when enabled.
  - Keep base behavior unchanged when disabled.
  - Handle missing/unsupported value reads gracefully.
- Out of scope:
  - Full formatting framework for every Ethos unit type.
  - Persistent advanced UI settings beyond this single toggle.
  - Unrelated layout redesign.

## Technical Constraints

- Keep refresh/poll cost bounded; avoid expensive per-frame source fetches.
- Maintain readability at current fullscreen geometry.
- Preserve scrolling/touch behavior and conflict display cues.
- Keep nil/API-variance handling defensive.

## Suggested Execution Plan

1. Add widget option state (`configure/read/write` callbacks if needed).
2. Extend normalized sensor row model with optional value text.
3. Update column layout math to support 3-column and 4-column modes.
4. Keep deterministic sort behavior and stable empty-state rendering.
5. Update docs if configuration surface changes.
6. Add/update tests for option-on/option-off rendering logic.

## Validation (Required)

- `luac -p scripts/SensorList/main.lua`
- `python -m pytest tests/test_sensorlist_widget.py -q`
- If docs change:  
  `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`

## Done Criteria

1. Toggle behavior matches issue outcome.
2. No regressions in existing widget behavior when toggle is off.
3. Required validation passes.
4. Session note includes change summary and follow-ups.
