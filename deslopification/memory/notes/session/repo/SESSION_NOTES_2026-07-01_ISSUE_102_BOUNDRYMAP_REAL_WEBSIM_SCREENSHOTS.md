# Session Notes 2026-07-01 - Issue #102 BoundryMap Real WebSimulator Screenshots

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `testing`

## What changed

- Replaced the BoundryMap README images with frames captured from the actual
  Ethos 1.6.6 `X20RS-FCC` WebSimulator canvas using a temporary ignored
  synthetic `GuideField` map.
- Fixed the WebSimulator harness persist staging path so headless and GUI runs
  write staged files under `/persist/{Radio}`, matching the runtime user
  directory.
- Fixed BoundryMap touch controls for 1.6.6 WebSimulator end-only duplicate
  touch delivery by trying raw control coordinates before the legacy offset
  path and debouncing duplicate `Draw`/`Delete` end-only events. `Save` remains
  start-armed.
- Shortened successful sidecar diagnostics to the map stem so the status fits
  on the 800px simulator screen.
- Files touched:
  - `scripts/BoundryMap/README.md`
  - `scripts/BoundryMap/docs/images/*.png`
  - `scripts/BoundryMap/main.lua`
  - `scripts/BoundryMap/tests/lua/test_boundrymap.lua`
  - `tools/sim/harness/run.py`
  - `tools/sim/harness/websim_runner.js`
  - `tools/sim/harness/README.md`
  - `tests/test_sim_harness.py`

## Why

- Root cause or objective:
  - The existing README images were browser-drawn SVG guide captures, not real
    simulator output.
  - GUI manifest staging previously wrote files at the virtual filesystem root,
    while 1.6.6 uses `/persist/X20RS` as `USER:/`.
  - The real 1.6.6 simulator delivered visible widget taps as duplicated
    touch-end events without a touch-start, so BoundryMap controls did not
    toggle unless the script accepted that event shape.
- Scope guardrails:
  - Sibling reference repos were not touched.
  - Private/local map assets were not used for captures.
  - The `GuideField` map and simulator run directories stayed ignored.
  - The boundary-warning screenshot used a run-manifest-only simulated GPS
    state because the stock simulator model exposed no GPS source.

## Validation run(s)

- `luac -p .\scripts\BoundryMap\main.lua`
  - result: pass
- `python -m pytest .\scripts\BoundryMap\tests -q`
  - result: pass (`3 passed`)
- `python -m pytest tests\test_sim_harness.py -q`
  - result: pass (`33 passed`)
- `python -m pytest .\tests\test_docs_commands.py .\tests\test_docs_contracts.py -q`
  - result: pass (`189 passed, 26 skipped`)
- `python tools\sim\harness\run.py headless --suite tools\sim\harness\suites\BoundryMap-X20RS-FCC-1.6.6.json --no-download --run-dir tools\sim\runs\issue-102-boundrymap-x20rs-166-final-validation --persist-dir tools\sim\runs\issue-102-boundrymap-x20rs-166-final-validation-persist --timeout-ms 20000`
  - result: pass; `status=success`, `persistMount=/persist/X20RS`, `messages[].code=reloadScripts_unavailable`

## Follow-up items

- Consider a first-class simulator telemetry fixture if future docs or UAT need
  warning-state captures without a run-manifest-only state shim.

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: issue-local harness/script behavior and docs update; no new
  durable workflow policy.
