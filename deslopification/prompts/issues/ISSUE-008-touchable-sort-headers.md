# Prompt: Implement Issue #8 (Touchable Sort Headers)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/8`
- Title: `[Enhancement] Make sensor table headers touchable for sorting`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`

## Mission

Implement runtime header-tap sorting in `SensorList` without breaking current
scroll behavior, long-press refresh, deep-scan cadence, or deterministic
defaults.

## Required Context

Read before editing:

- `AGENTS.md`
- `README.md`
- `docs/DEVELOPMENT.md`
- `docs/SensorList/SENSORLIST_ARCHITECTURE.md`
- `deslopification/memory/SensorList.md`
- `scripts/SensorList/main.lua`
- `tests/lua/test_sensorlist.lua`
- `tests/test_sensorlist_widget.py`

## Scope

- In scope:
  - Tap headers `Name`, `Physical ID`, `Application ID` to control sort key.
  - Repeated taps on active header toggle ascending/descending.
  - Active sort indicator renders in header row and stays readable.
  - Default ordering remains deterministic when no header is tapped.
- Out of scope:
  - Persistence across power cycles.
  - Redesigning entire widget layout.
  - Tooling/build workflow changes.

## Technical Constraints

- Keep Ethos callbacks defensive and lightweight.
- Preserve manual refresh on long-press.
- Guard against missing touch coordinates/events.
- Header hit-box logic must not steal normal row scrolling interactions.
- Prefer root-cause touch/event handling updates over ad-hoc event aliases.

## Suggested Execution Plan

1. Define explicit header geometry and touch hit testing.
2. Add per-widget sort state (`key`, `direction`) in `create()`.
3. Route header taps to sort-state updates; leave other touches in scroll path.
4. Update sensor normalization/sort pipeline to use active sort state.
5. Render sort indicator in header row.
6. Extend Lua unit tests for:
   - sort toggle behavior
   - untouched-default ordering
   - no regression in touch scroll activation rules

## Validation (Required)

- `luac -p scripts/SensorList/main.lua`
- `python -m pytest tests/test_sensorlist_widget.py -q`

## Done Criteria

1. All issue acceptance criteria are met.
2. No regressions in scroll, long-press refresh, empty-state rendering.
3. Required validation passes.
4. Session note added under `deslopification/memory/` with changes + validation.
