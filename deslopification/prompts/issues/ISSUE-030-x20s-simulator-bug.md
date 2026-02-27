# Prompt: Implement Issue #30 (SensorList X20S Simulator Startup Bug)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/30`
- Title: `[Bug] SensorList not functioning in X20S simulator`
- Labels: `bug`
- Snapshot state: open on `2026-02-26`
- Target branch (default): `fix/30-x20s-simulator-bug` (or as user-directed for current workflow)
- Attached screenshot evidence:
  `https://github.com/user-attachments/assets/e79aed7e-da30-48b7-810d-c612e0fdf406`

## Mission

Reproduce and fix the X20S simulator startup failure that prevents SensorList
from loading in widget selection/runtime, using a root-cause fix.

## Required Context

- `AGENTS.md`
- `.github/ISSUE_TEMPLATE/bug_report.md`
- `scripts/SensorList/main.lua`
- `docs/SensorList/SENSORLIST_ARCHITECTURE.md`
- `deslopification/memory/notes/domain-note/lua-ethos/SensorList.md`
- `tests/lua/test_sensorlist.lua`
- `tests/test_sensorlist_widget.py`
- `tools/deploy.config.example.json`
- `tools/build.py`

## Branch/Worktree Gate (Required Before Editing)

1. Run issue preflight:
   - `python tools/session_preflight.py --mode issue --issue-number 30 --issue-kind bug --slug x20s-simulator-bug`
2. Confirm target branch and current branch:
   - `git branch --show-current`
   - `git status --short --branch`
3. If preflight blocks due to `main`, create/switch to the recommended branch before editing.
4. If branch mismatch or dirty worktree is present, stop and confirm stash/commit/switch strategy.
5. After switching branches, sync before editing:
   - `git pull --ff-only origin {target-branch}`

## Scope

- In scope:
  - Reproduce issue on X20S simulator path/environment.
  - Identify exact crash/failure point.
  - Implement minimal robust fix.
  - Add regression coverage where feasible.
- Out of scope:
  - Broad behavior redesign.
  - Compatibility shims without confirmed need.
  - Unrelated feature additions.

## Debug Strategy

1. Reproduce with explicit simulator target:
   - `python tools/build.py --project SensorList --deploy --sim-radio X20S`
2. Capture exact runtime error text and call path.
3. Compare runtime assumptions between X20RS and X20S (callbacks/constants/window).
4. Patch root cause with defensive handling where needed.
5. Remove temporary debug instrumentation after confirming fix.

Temporary helper scripts/log files may be created only under:
`deslopification/memory/temp/`.

## Technical Constraints

- Keep widget registration and callback lifecycle valid for Ethos.
- Avoid startup crashes when optional APIs/constants are missing.
- Preserve current SensorList behavior outside bug fix.
- Do not regress known-good X20RS behavior while fixing X20S.
- Preserve system navigation/interoperability (`RTN/EXIT`, touch handling semantics).

## Interaction Contract

- Widget must initialize safely when simulator-specific constants/APIs differ.
- Missing optional runtime APIs/constants should degrade safely without crash.
- Event/touch/key handling should not consume system navigation unexpectedly.

## Validation (Required)

- `luac -p scripts/SensorList/main.lua`
- `python -m pytest tests/test_sensorlist_widget.py -q`
- If touching broad logic: `python -m pytest -q`
- Manual simulator check on X20S after deploy.

## Manual Acceptance Scenarios (Required)

1. X20S simulator: deploy and confirm SensorList opens without startup error.
2. X20RS simulator: confirm no regression in startup/runtime behavior.
3. Verify core interactions still work: scroll, long-press refresh, and navigation (`RTN/EXIT`).
4. Confirm no new simulator fallback/unhandled-event errors are introduced.

## Done Criteria

1. SensorList loads and runs in X20S simulator without startup error.
2. Fix is root-cause based and minimally scoped.
3. Regression risk is covered by tests and/or explicit manual checklist.
4. Required validation passes and session notes capture evidence.
