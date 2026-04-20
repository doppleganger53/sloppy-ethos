# Prompt: Implement Issue #10 (Acceptable Conflict Definitions)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/10`
- Title: `[Enhancement] Support acceptable conflict definitions`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`
- Target branch (default): `feature/10-acceptable-conflict-definitions` (or as user-directed for current workflow)

## Mission

Add a maintainable local whitelist for acceptable conflict combinations so
known-safe duplicates are de-emphasized without being hidden.

## Required Context

- `AGENTS.md`
- `docs/SensorList/SENSORLIST_ARCHITECTURE.md`
- `deslopification/prompts/SensorList-Roadmap-AcceptableConflicts.md`
- `scripts/SensorList/main.lua`
- `tests/lua/test_sensorlist.lua`
- `tests/test_sensorlist_widget.py`

## Branch/Worktree Gate (Required Before Editing)

1. Run issue preflight:
   - `python tools/session_preflight.py --mode issue --issue-number 10 --issue-kind enhancement --slug acceptable-conflict-definitions`
2. Confirm target branch and current branch:
   - `git branch --show-current`
   - `git status --short --branch`
3. If preflight blocks due to `main`, create/switch to the recommended branch before editing.
4. If worktree is dirty, stop and confirm stash/commit/continue strategy.
5. After switching branches, sync before editing:
   - `git pull --ff-only origin {target-branch}`

## Scope

- In scope:
  - Define whitelist near top-level constants in `main.lua`.
  - Match at minimum by `Physical ID`, with optional `Application ID` subsets.
  - Visually distinguish acceptable duplicates from unresolved conflicts.
  - Safe default when whitelist is empty/malformed.
- Out of scope:
  - External config files or persistent storage.
  - Runtime editor UI for whitelist management.
  - Tooling/build workflow changes.

## Technical Constraints

- Whitelist lookup should be efficient and easy to maintain.
- Unknown/malformed entries must default to unresolved-conflict behavior.
- Keep sorting/scrolling/deep-scan behavior unchanged.

## Suggested Execution Plan

1. Define whitelist data schema and helper lookup functions.
2. Integrate lookup into conflict classification flow.
3. Render acceptable conflicts with a clear but muted cue.
4. Add targeted tests for whitelist match/miss and malformed rule fallbacks.

## Validation (Required)

- `luac -p scripts/SensorList/main.lua`
- `python -m pytest tests/test_sensorlist_widget.py -q`

## Done Criteria

1. Acceptable conflicts are configurable and visible.
2. Non-whitelisted conflicts remain prominently flagged.
3. Existing behavior remains stable outside intended change.
4. Required validation passes and session notes are recorded.
