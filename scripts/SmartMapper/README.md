# SmartMapper

SmartMapper is an Ethos `26.1+` widget for reviewing available control
sources and the model assignments that Lua can read safely.

## Current Behavior

- Registers a fullscreen Ethos widget named `SmartMapper`.
- Inventories controls through `system.getSources(categoryNumber)`.
- Reads accessible mixes, logical switches, trims, special functions, inputs,
  and channels through defensive model API enumeration.
- Normalizes discovered assignments into a compact list of assigned controls
  followed by unused switch-like controls.
- Labels special-function assignments with play-audio filenames first, then
  play-text text, then the normal model object name fallback.
- Reports unavailable or empty API surfaces in a trailing `Status` section
  instead of fabricating mappings.

SmartMapper does not create, edit, or delete model configuration.

## User Guide

See [SmartMapper User Guide](../../docs/SmartMapper/SMARTMAPPER_USER_GUIDE.md)
for installation, widget setup, screenshots, and display interpretation notes.

## Simulator Validation

Run the automated WebSimulator smoke suite from the repo root:

```powershell
python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SmartMapper-X20RS-FCC.json
```

The suite stages SmartMapper and verifies that the Ethos `26.1` WebSimulator
starts and reloads scripts without detected runtime errors. The harness also
includes a `SmartMapperApiProbe` that can emit a structured
`[SimProbe:SmartMapperApiProbe]` report when a simulator model or manual GUI
workflow activates the probe script; plain headless `reloadScripts` does not
open standalone tools, widgets, or tasks by itself.

## Build And Deploy

```powershell
luac -p scripts/SmartMapper/main.lua
python -m pytest scripts/SmartMapper/tests -q
python tools/build.py --project SmartMapper --dist
python tools/build.py --project SmartMapper --deploy
```

Use manual simulator or radio confirmation after deploy when evaluating real
model assignments, because simulator model data can differ from a physical
radio or a configured aircraft model.
