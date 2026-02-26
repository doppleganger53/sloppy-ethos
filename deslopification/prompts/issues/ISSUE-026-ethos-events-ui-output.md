# Prompt: Implement Issue #26 (ethos_events UI Output + Toggle)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/26`
- Title: `[Enhancement] Add event output to UI`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`

## Mission

Enhance `ethos_events` standalone system tool so captured events render in UI,
and make `throttleSame` behavior configurable through runtime interaction.

## Required Context

- `AGENTS.md`
- `scripts/ethos_events/main.lua`
- `scripts/ethos_events/ethos_events.lua`
- `scripts/ethos_events/README.md`
- `tools/build.py` (only if packaging behavior needs updates)

## Scope

- In scope:
  - Display recent formatted event lines in tool UI.
  - Add a UI control to toggle `throttleSame` on/off.
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

## Suggested Execution Plan

1. Add per-tool instance state for log buffer + throttle toggle.
2. Route event callback through helper with options table.
3. Store returned formatted lines and render in `paint()`.
4. Add key/touch-driven toggle action with clear on-screen indicator.
5. Update `scripts/ethos_events/README.md` for new interaction model.

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
