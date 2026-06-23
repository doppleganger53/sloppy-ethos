# Session Notes 2026-06-23 - Issue #45 SmartMapper Implementation

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `implementation`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Replaced the `SmartMapper Probe` system tool with a production
  `SmartMapper` Ethos widget for issue `#45`.
- The widget inventories controls through `system.getSources(categoryNumber)`,
  defensively reads accessible model assignment surfaces, labels
  special-function audio/text mappings, and shows unused switch-like controls.
- Added harness-only simulator probe support and a SmartMapper X20RS-FCC smoke
  suite. Probe reports are parsed when an activated simulator script emits
  `[SimProbe:NAME]` JSON; plain headless `reloadScripts` does not open
  standalone tools, widgets, or tasks.
- Files touched:
  - `scripts/SmartMapper/main.lua`
  - `tools/sim/harness/run.py`
  - `tools/sim/harness/probes/SmartMapperApiProbe/main.lua`
  - `tools/sim/harness/suites/SmartMapper-X20RS-FCC.json`
  - `docs/ETHOS_26_1_COMPATIBILITY.md`

## Why

- Root cause or objective:
  - Issue `#45` required SmartMapper to move from API discovery into a usable
    function/source mapping widget while staying safe on incomplete Ethos model
    API surfaces.
- Scope guardrails:
  - The stale `feature/45-smartmapper-function-mapping-script` branch was not
    revived. The implementation restarted from current `main`.
  - SmartMapper does not mutate model configuration and does not fabricate
    mappings when a subsystem has no readable assignment fields.

## Validation run(s)

- `luac -p scripts/SmartMapper/main.lua`
  - result: pass
- `luac -p tools/sim/harness/probes/SmartMapperApiProbe/main.lua`
  - result: pass
- `python -m pytest scripts/SmartMapper/tests -q`
  - result: pass (`3 passed`)
- `python -m pytest tests/test_sim_harness.py -q`
  - result: pass (`31 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py tests/test_memory_catalog_sync.py -q`
  - result: pass (`206 passed, 26 skipped`)
- `python tools/build.py --project SmartMapper --dist`
  - result: pass; packaged `dist/SmartMapper-0.2.0.zip`
- `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SmartMapper-X20RS-FCC.json`
  - result: pass on X20RS-FCC Ethos `26.1.0-RC2`; staged SmartMapper and completed `reloadScripts` without detected runtime errors
- `python tools/build.py --project SmartMapper --deploy`
  - result: pass after escalation; deployed to configured Ethos Suite simulator path `26.1.0-RC1\persist\X20RS\scripts\SmartMapper`
- `python -m pytest -q`
  - result: pass (`358 passed, 26 skipped`)

## Runtime finding

- The generated plan assumed a staged probe would run during WebSimulator
  `reloadScripts`. Direct 26.1 runtime experiments showed that standalone Lua
  tools/widgets/tasks are not opened by `reloadScripts` alone; a model or GUI
  interaction has to activate the script before a probe report can be expected.

## Current State Sync

- `CURRENT_STATE.md` updated: yes

## 2026-06-23 User Guide And GUI Capture Addendum

- Added `docs/SmartMapper/SMARTMAPPER_USER_GUIDE.md` with simulator-captured
  screenshots under `docs/SmartMapper/assets/`.
- Captured the guide screenshots from the GUI harness using the
  `SmartMapper-X20RS-FCC` suite on Ethos `26.1.0-RC2`.
- The GUI harness now mirrors staged files into `/persist/<radio>/...` inside
  the WebSimulator virtual filesystem. Before that fix, GUI mode wrote files
  under `/scripts/...`, while Ethos looked under `/persist/X20RS/scripts/...`,
  so SmartMapper could be staged but absent from the widget picker.
- Reduced SmartMapper bounded fallback scan limits and compacted section
  header rendering in narrow widget zones after GUI activation exposed a
  `Max instructions count reached` create-time failure and cramped section
  text.
- Guide capture evidence: SmartMapper appeared in the widget picker, selected
  as a widget, rendered in the configured screen, and the GUI log tail did not
  contain `create() error` or `Max instructions`.
