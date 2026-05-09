# Session Notes 2026-05-09 - SmartMapper API Probe

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `implementation`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Added `SmartMapper Probe` as an Ethos system tool for issue `#83`.
- The probe prints and writes a report covering candidate `model.*` APIs,
  `system.getSources(...)`, `system.getSource(...)`, and the read/enumeration
  surfaces needed by issue `#45`: mixes, logical switches, trims, special
  functions, and switch/input assignments.
- Added script-local Lua and Python tests under `scripts/SmartMapper/tests/`.
- Documented the probe run path and updated the Ethos `26.1` compatibility
  baseline to track SmartMapper as probe-added with runtime capture pending.
- Files touched:
  - `scripts/SmartMapper/main.lua`
  - `scripts/SmartMapper/README.md`
  - `scripts/SmartMapper/VERSION`
  - `scripts/SmartMapper/tests/test_smartmapper_probe.py`
  - `scripts/SmartMapper/tests/lua/test_smartmapper.lua`
  - `docs/ETHOS_26_1_COMPATIBILITY.md`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - Issue `#45` should not resume from the stale SmartMapper branch until the
    Ethos `26.1` runtime proves whether model read/enumeration APIs exist for
    the mapping feature.
- Scope guardrails:
  - The stale `feature/45-smartmapper-function-mapping-script` branch was used
    only for scaffold context; its deferred-widget behavior and old root-level
    test layout were not reused.
  - The probe does not create or mutate model configuration, even though it
    records whether `model.createMix` is present.

## Validation run(s)

- `luac -p scripts/SmartMapper/main.lua`
  - result: pass
- `python -m pytest scripts/SmartMapper/tests -q`
  - result: pass (`3 passed`)
- `python tools/build.py --project SmartMapper --dist`
  - result: pass; packaged `dist/SmartMapper-0.1.0.zip`
- `python tools/build.py --project SmartMapper --deploy --sim-radio X20RS`
  - result: pass; deployed to configured `1.6.4` X20RS simulator path
- `python tools/build.py --project SmartMapper --deploy --sim-radio X20RS --config <temp 26.1 config>`
  - result: pass; deployed to local `26.1.0-RC1` X20RS simulator path
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`169 passed, 21 skipped`)

## Follow-up items

- Open `SmartMapper Probe` in the Ethos `26.1` simulator or target radio,
  capture the `[SmartMapperProbe]` report, and add the runtime result to issue
  `#45` before resuming the full mapping implementation.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
