# Prompt: Implement Issue #9 (Conflict Severity Cues)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/9`
- Title: `[Enhancement] Refine conflict display severity`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`
- Target branch (default): `enhancements` (or as user-directed for current workflow)

## Mission

Differentiate high-severity vs lower-severity duplicate conflicts in
`SensorList` while preserving table readability and interaction stability.

## Required Context

- `AGENTS.md`
- `docs/SensorList/SENSORLIST_ARCHITECTURE.md`
- `deslopification/memory/SensorList.md`
- `scripts/SensorList/main.lua`
- `deslopification/prompts/SensorList-Roadmap-ConflictDisplay.md`
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

- Severity cues must be distinguishable without relying only on color.
- Severity markers must remain readable at current row height/font sizes.
- Cue text/symbols must avoid clutter that hides sensor identity fields.
- Empty-state and non-conflict rows should retain current visual clarity.

## Scope

- In scope:
  - High severity: duplicate `Physical ID` + duplicate `Application ID`.
  - Lower severity: duplicate `Physical ID` with distinct `Application ID`.
  - Non-color-only cue (text marker/icon/prefix/suffix) for accessibility.
  - Keep all rows visible.
- Out of scope:
  - User-editable severity policy UI.
  - Changes to packaging/deploy tooling.
  - Removing existing duplicate grouping concept.

## Technical Constraints

- Handle missing/unknown `Application ID` safely.
- Do not break sorting, scrolling, or empty-state path.
- Keep paint path lightweight; precompute severity markers in refresh pipeline.

## Suggested Execution Plan

1. Build conflict classification during refresh (not inside tight paint loops).
2. Store severity class on normalized rows.
3. Update row rendering with both color and non-color cue.
4. Confirm cues remain legible at current row height/font sizes.
5. Add tests for classification logic and regression tests for core behaviors.

## Manual Acceptance Scenarios (Required)

1. Verify high-severity duplicates are clearly distinguishable from lower-severity duplicates.
2. Verify non-color cues are visible and meaningful with color cues ignored.
3. Verify non-conflict rows remain readable and not visually over-marked.
4. Verify scrolling/sorting/empty-state behavior remains unchanged.

## Validation (Required)

- `luac -p scripts/SensorList/main.lua`
- `python -m pytest tests/test_sensorlist_widget.py -q`

## Done Criteria

1. High vs lower conflict severity is clearly distinguishable.
2. Existing widget behavior remains stable.
3. Required validation passes.
4. Session note added with validation and follow-up notes.

