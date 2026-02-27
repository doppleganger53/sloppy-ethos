# Prompt: Implement Issue #26 (ethos_events UI Output + Toggle)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/26`
- Title: `[Enhancement] Add event output to UI`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`
- Target branch (default): `feature/26-ethos-events-ui-output` (or as user-directed for current workflow)

## Mission

Enhance `ethos_events` standalone system tool so captured events render in UI,
and make `throttleSame` behavior configurable through runtime interaction.

## Required Context

- `AGENTS.md`
- `scripts/ethos_events/main.lua`
- `scripts/ethos_events/ethos_events.lua`
- `scripts/ethos_events/README.md`
- `tools/build.py` (only if packaging behavior needs updates)

## Branch/Worktree Gate (Required Before Editing)

1. Run issue preflight:
   - `python tools/session_preflight.py --mode issue --issue-number 26 --issue-kind enhancement --slug ethos-events-ui-output`
2. Confirm target branch and current branch:
   - `git branch --show-current`
   - `git status --short --branch`
3. If preflight blocks due to `main`, create/switch to the recommended branch before editing.
4. If branch mismatch or dirty worktree is present, stop and confirm stash/commit/switch strategy.
5. After switching branches, sync before editing:
   - `git pull --ff-only origin {target-branch}`

## Scope

- In scope:
  - Display recent formatted event lines in tool UI.
  - Add a touch-driven UI control to toggle `throttleSame` on/off.
  - Preserve existing console/debug behavior.
  - Keep runtime robust when touch/key APIs vary.
- Out of scope:
  - Rewriting upstream helper module behavior unnecessarily.
  - Complex multi-page UI redesign.
  - Unrelated changes to SensorList.

## Technical Constraints

- Keep event handling lightweight (bounded in-memory buffer/ring).
- Avoid nil crashes in paint/event paths.
- Preserve compatibility with helper fallback loader.
- Make toggle state obvious in UI text.
- Avoid key bindings that can conflict with system navigation/back actions.
- Ensure handled touch flows do not fall through to simulator/system fallback behavior.
- Keep event lines readable (compact line rendering, no redundant visible prefixes).

## Interaction Contract

- Expected touch mapping (simulator-observed):
  - category `EVT_TOUCH (1)`
  - `TOUCH_START (16640)`
  - `TOUCH_MOVE (16642)`
  - `TOUCH_END (16641)`
- Toggle interaction contract:
  - touch-only on status row
  - non-toggle touches must not clear/reset UI
- Navigation contract:
  - `RTN/EXIT` behavior must remain system-managed (do not remap for toggle).

## UI Output Contract

- UI event lines should not show redundant tag prefix (`[ethos_events]`) when already in Ethos Events tool context.
- Event row font/line spacing should prioritize readability for long category/value strings.
- Empty state text should remain visible and stable after touch interactions.

## Suggested Execution Plan

1. Add per-tool instance state for log buffer + throttle toggle.
2. Route event callback through helper with options table.
3. Store returned formatted lines and render in `paint()`.
4. Add touch-only toggle action with clear on-screen indicator.
5. Update `scripts/ethos_events/README.md` for new interaction model.

## Manual Acceptance Scenarios (Required)

1. Positive:
   - Tap toggle row and verify `throttleSame` state flips with UI status update.
2. Guardrail:
   - Tap outside toggle row and verify no toggle and no display clear/reset.
3. Regression:
   - Generate repeated events and verify `throttleSame` ON suppresses adjacent duplicates; OFF restores them.
4. Navigation/system interoperability:
   - Press `RTN/EXIT` and verify normal system-tool navigation behavior (no extra required presses caused by script handling).
5. Readability:
   - Verify long event lines remain readable with compact font and without redundant prefix noise.

## Validation (Required)

- `luac -p scripts/ethos_events/main.lua`
- `python tools/build.py --project ethos_events --dist`
- If docs changed:
  `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`

## Done Criteria

1. Standalone tool shows event output in UI.
2. `throttleSame` can be toggled at runtime.
3. Existing behavior remains stable (no startup/runtime crashes).
4. Required validation passes and session note is recorded.
