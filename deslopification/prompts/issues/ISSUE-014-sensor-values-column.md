# Prompt: Implement Issue #14 (Sensor Values Column)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/14`
- Title: `[Enhancement] Sensor Values`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`
- Target branch (default): `enhancements` (or as user-directed for current workflow)

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

## Branch/Worktree Gate (Required Before Editing)

1. Confirm target branch and current branch:
   - `git branch --show-current`
   - `git status --short --branch`
2. If branch mismatch or dirty worktree is present, stop and confirm stash/commit/switch strategy.
3. After switching branches, sync before editing:
   - `git pull --ff-only origin {target-branch}`

## UI Output Contract

- `Display Value = No`: preserve current 3-column readability and spacing.
- `Display Value = Yes`: value column must be legible without collapsing key identifiers.
- Long names/values should degrade gracefully (truncate/clip consistently, no overlap).
- Empty-state and conflict cues must remain readable in both modes.

## Interaction Contract

- Touch scrolling and long-press refresh behavior must remain unchanged in both display modes.
- Any config/toggle interaction must not conflict with existing navigation keys/system behavior.

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

## Manual Acceptance Scenarios (Required)

1. Toggle `Display Value` OFF and verify baseline 3-column behavior is unchanged.
2. Toggle `Display Value` ON and verify 4-column layout remains readable for long names and values.
3. Verify scrolling and long-press refresh still behave normally in both modes.
4. Verify conflict cues and empty-state rendering remain readable in both modes.

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
