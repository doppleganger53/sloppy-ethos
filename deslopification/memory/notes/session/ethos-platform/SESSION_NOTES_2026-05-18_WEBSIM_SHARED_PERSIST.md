# Session Notes 2026-05-18 - WebSimulator Shared Persist

## Note Placement

- Artifact: `session`
- Scope: `ethos-platform`
- Concern: `testing`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Updated `tools/sim/harness/run.py` so headless and GUI WebSimulator modes
  stage projects into the Ethos Suite persist directory for the selected
  runtime version and radio:
  - `{Ethos Suite data}/.simulator/{EthosVersion}/persist/{Radio}`
- Added `--persist-dir` for nonstandard simulator persist roots.
- Kept per-run logs and generated GUI wrapper files under `tools/sim/runs/`.
- Removed the best-effort headless `_stop()` call after reload/settle because
  the shared-persist smoke reached shutdown and then hung inside that runtime
  export before structured JSON could be emitted.
- Updated focused harness coverage and docs for the shared-persist behavior.

## Why

- The harness should exercise the same simulator state that Ethos Suite uses
  instead of creating an isolated per-run persist tree.
- The staging path no longer deletes the persist root; it overlays the selected
  project files through the existing `tools/build.py` install-spec helpers.

## Validation run(s)

- `python -m pytest tests/test_sim_harness.py -q`
  - result: pass (`18 passed`)
- `python -m pytest tests/test_sim_harness.py tests/test_build_py.py tests/test_docs_commands.py tests/test_docs_contracts.py tests/test_memory_catalog_sync.py -q`
  - result: pass (`293 passed, 25 skipped`)
- `python tools/sim/harness/run.py headless --project SensorList --radio X20RS-FCC --ethos-version 26.1.0-RC2 --no-download --run-dir tools/sim/runs/shared-persist-headless-validation --timeout-ms 12000`
  - result: timeout after `12000 ms`; result used `persistDir=C:\Users\kurtk\AppData\Roaming\Ethos Suite\.simulator\26.1.0-RC2\persist\X20RS`
- `python tools/sim/harness/run.py headless --project SensorList --radio X20RS-FCC --ethos-version 26.1.0-RC2 --no-download --run-dir tools/sim/runs/shared-persist-headless-validation-retry --timeout-ms 30000`
  - result: timeout after `30000 ms`; stderr showed the runner reached `stopping simulator`
- `python tools/sim/harness/run.py headless --project SensorList --radio X20RS-FCC --ethos-version 26.1.0-RC2 --no-download --run-dir tools/sim/runs/shared-persist-headless-validation-final --timeout-ms 30000`
  - result: pass (`status=success`, `started=true`, `reloaded=true`, shared Ethos Suite `persistDir`)

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
