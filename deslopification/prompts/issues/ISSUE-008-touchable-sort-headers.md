# Prompt: Implement Issue #8 (Touchable Sort Headers)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/8`
- Title: `[Enhancement] Make sensor table headers touchable for sorting`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`
- Target branch (default): `enhancements` (or as user-directed for current workflow)

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

## Branch/Worktree Gate (Required Before Editing)

1. Confirm target branch and current branch:
   - `git branch --show-current`
   - `git status --short --branch`
2. If branch mismatch or dirty worktree is present, stop and confirm stash/commit/switch strategy.
3. After switching branches, sync before editing:
   - `git pull --ff-only origin {target-branch}`

## Interaction Contract

- Header-tap sorting must be touch-compatible with Ethos touch event phases (`start`/`move`/`end`).
- Header taps must not break row scrolling/drag behavior outside header hit zones.
- Existing long-press refresh semantics must remain unchanged.
- Navigation key behavior (e.g., `RTN/EXIT`) must remain system-managed.

## UI Output Contract

- Active sort indicator should be visible on the header row without obscuring labels.
- Indicator should remain legible at current row height and font settings.
- No new prefix/noise text should reduce row readability.

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

## Manual Acceptance Scenarios (Required)

1. Tap each header and verify sort key changes correctly.
2. Re-tap active header and verify ascending/descending toggle.
3. Touch-drag in table body and verify scrolling still works (no accidental sort trigger).
4. Long-press refresh still works and sort state remains consistent.
5. `RTN/EXIT` navigation behavior remains unchanged.

## Validation (Required)

- `luac -p scripts/SensorList/main.lua`
- `python -m pytest tests/test_sensorlist_widget.py -q`

## Done Criteria

1. All issue acceptance criteria are met.
2. No regressions in scroll, long-press refresh, empty-state rendering.
3. Required validation passes.
4. Session note added under `deslopification/memory/` with changes + validation.
