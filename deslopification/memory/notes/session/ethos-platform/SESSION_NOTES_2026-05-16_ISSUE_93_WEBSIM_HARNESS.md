# Session Notes 2026-05-16 - Issue #93 WebSimulator Harness

## Note Placement

- Artifact: `session`
- Scope: `ethos-platform`
- Concern: `testing`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Added `tools/sim/harness/run.py` and `websim_runner.js` as the first
  automated Ethos WebSimulator harness.
- Added `tools/sim/harness/README.md` with download, headless, GUI, status,
  and cache guidance.
- Added a simple SensorList smoke-suite definition:
  - `tools/sim/harness/suites/SensorList-X20RS-FCC.json`
- Added ignored cache/run roots:
  - `tools/sim/radios/`
  - `tools/sim/runs/`
- Added focused harness coverage in `tests/test_sim_harness.py`.
- Updated docs and policy surfaces:
  - `AGENTS.md`
  - `README.md`
  - `CONTRIBUTING.md`
  - `docs/DEVELOPMENT.md`
  - `docs/ETHOS_26_1_COMPATIBILITY.md`
  - docs command/contract tests

## Why

- Root cause or objective:
  - Issue `#93` required a committed simulator harness that can download and
    cache WebSimulator runtimes, stage scripts through the repository's existing
    install contract, and run repeatable real-runtime smoke checks.
- Scope guardrails:
  - Kept `tools/build.py` as the packaging/staging source of truth.
  - Kept downloaded simulator payloads, extracted WebSimulator assets, run
    logs, generated GUI files, and persist trees out of git.

## Runtime Evidence

- The latest `26.1` release resolved on 2026-05-16 to `26.1.0-RC2`.
- X20RS-FCC package used:
  - `X20RS-FCC-WebSimulator.zip`
  - release digest:
    `sha256:73d7e33c5c399707d54a462e2138427ce30402d23965839abbb843a285335581`
- Live smoke command:
  - `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SensorList-X20RS-FCC.json`
- Result:
  - `status=success`
  - `ethosVersion=26.1.0-RC2`
  - `started=true`
  - `reloaded=true`
  - no detected Lua/script errors
- Important runtime note:
  - The X20RS-FCC `26.1.0-RC2` WebSimulator blocks under Node when calling
    `_writeDefaultSettingsAndModel()`, so the harness does not call it by
    default. The optional `--write-default-model` flag exists for runtimes known
    not to block in that export.

## Validation run(s)

- `python -m pytest tests/test_sim_harness.py -q`
  - result: pass (`9 passed`)
- `luac -p scripts/SensorList/main.lua`
  - result: pass
- `python -m pytest scripts/SensorList/tests -q`
  - result: pass (`6 passed`)
- `python -m pytest tests/test_build_py.py -q`
  - result: pass (`70 passed`)
- `python -m pytest tests/test_sim_harness.py tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`185 passed, 24 skipped`)
- `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SensorList-X20RS-FCC.json`
  - result: pass (`status=success`, `ethosVersion=26.1.0-RC2`)
- `python tools/sim/harness/run.py gui --project SensorList --radio X20RS-FCC --ethos-version latest-26.1 --dry-run --run-dir tools/sim/runs/gui-dry-run`
  - result: pass (`status=gui_ready`)

## Follow-up items

- Consider adding a dedicated model fixture once the WebSimulator model storage
  path is understood well enough to avoid relying on runtime defaults.
- Consider adding browser automation around the generated GUI view after the
  headless path is stable in CI.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
