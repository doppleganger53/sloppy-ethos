# Prompt: Implement Issue #17 (Refresh Haptic Feedback)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/17`
- Title: `[Enhancement] SensorList user feedback enhancement`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`
- Target branch (default): `enhancements` (or as user-directed for current workflow)

## Mission

When a long-press refresh completes in `SensorList`, provide positive haptic
feedback on compatible radios without breaking simulator/runtime safety.

## Required Context

- `AGENTS.md`
- `scripts/SensorList/main.lua`
- `docs/SensorList/SENSORLIST_ARCHITECTURE.md`
- `tests/lua/test_sensorlist.lua`
- `tests/test_sensorlist_widget.py`
- Issue acceptance note requiring physical radio verification.

## Branch/Worktree Gate (Required Before Editing)

1. Confirm target branch and current branch:
   - `git branch --show-current`
   - `git status --short --branch`
2. If branch mismatch or dirty worktree is present, stop and confirm stash/commit/switch strategy.
3. After switching branches, sync before editing:
   - `git pull --ff-only origin {target-branch}`

## Interaction Contract

- Haptic trigger should be bound to refresh completion (not refresh start).
- One long-press gesture should yield at most one haptic signal.
- No haptic calls from paint/wakeup loops.
- Navigation and touch/key interaction semantics should remain unchanged.

## Scope

- In scope:
  - Trigger haptic feedback only after refresh completion path.
  - Keep behavior safe when haptic API is absent.
  - Avoid duplicate buzzes from one long-press gesture.
- Out of scope:
  - New general notification framework.
  - Audio alarms or unrelated UI changes.
  - Build tooling/doc churn unless required for clarity.

## Technical Constraints

- Defensive API use (`safeCall`) for runtime differences.
- No regression to long-press refresh semantics.
- Avoid coupling haptic signal to paint/wakeup loops.

## Suggested Execution Plan

1. Identify refresh-success completion points for long-press path.
2. Add optional haptic helper with API guard.
3. Ensure single-trigger behavior per user gesture.
4. Add/adjust tests for "API available" and "API missing" scenarios.
5. Add manual validation checklist for physical radio.

## Manual Acceptance Scenarios (Required)

1. On hardware with haptic support, complete long-press refresh and verify exactly one positive haptic signal.
2. Repeat long-press gesture and verify one signal per completed gesture (no duplicate buzzes).
3. In simulator or API-missing environment, verify no runtime error and no haptic-call crash.
4. Verify long-press refresh, scroll, and navigation behaviors remain unchanged.

## Validation (Required)

- `luac -p scripts/SensorList/main.lua`
- `python -m pytest tests/test_sensorlist_widget.py -q`
- Manual (human): verify haptic confirmation on physical radio.

## Done Criteria

1. Long-press refresh causes one positive haptic signal when supported.
2. No runtime errors when haptic API is unavailable.
3. Required automated validation passes.
4. Manual radio verification is explicitly recorded as complete/pending.
