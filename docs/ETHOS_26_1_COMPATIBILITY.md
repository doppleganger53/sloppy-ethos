# Ethos 26.1 Compatibility Baseline

This document is the repository's working baseline for Ethos `26.1`
compatibility checks. It records the exact reference checkout, the simulator
targets to use for repeatable validation, and the currently known upgrade risks
across the repo.

## Reference Checkout

- Repository:
  `https://github.com/FrSkyRC/ETHOS-Feedback-Community.git`
- Local path: `..\ETHOS-Feedback-Community-prerelease`
- Branch: `26.1`
- Commit: `9a89bdae`
- Baseline rule:
  the original request mentioned Ethos `21.1`, but the concrete checked and
  requested reference branch was `26.1`. Keep `26.1` as the evidence source
  unless a later issue explicitly changes the target version.

## Recreate Or Refresh

```powershell
git clone --branch 26.1 --single-branch https://github.com/FrSkyRC/ETHOS-Feedback-Community.git ..\ETHOS-Feedback-Community-prerelease
git -C ..\ETHOS-Feedback-Community-prerelease fetch origin 26.1 --depth 1
git -C ..\ETHOS-Feedback-Community-prerelease checkout 26.1
git -C ..\ETHOS-Feedback-Community-prerelease pull --ff-only origin 26.1
git -C ..\ETHOS-Feedback-Community-prerelease rev-parse --short HEAD
```

Use the clone command when the reference checkout is missing. Use the
fetch/checkout/pull sequence when the repo already exists and you want to
confirm that it is still pinned to the tracked `26.1` branch.

## Release Assets

- The `26.1.0-RC1` GitHub release includes additional assets that are not part
  of the public repo checkout, including the
  [Lua documentation ZIP](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/26.1.0-RC1/lua-doc.zip).
- For Ethos API compatibility work, check release assets in addition to the
  branch checkout whenever the repo examples do not expose a newly announced
  API.

## Validation Targets

- Reference API and example surface: the `26.1` checkout above.
- Simulator radios:
  `X20RS` primary, `X20S` secondary.
- Local simulator paths stay private in `tools/deploy.config.json`; only the
  radio keys belong in committed docs.
- Debugging sessions with Lua behavior changes must deploy the touched script to
  the simulator before closeout, using
  `python tools/build.py --project {ProjectName} --deploy` and `--sim-radio X20S`
  when reproducing `X20S`-specific behavior.
- Automated WebSimulator smoke tests now provide the repeatable first simulator
  layer when a matching WebSimulator package exists. For SensorList on the
  current `26.1` line, use
  `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SensorList-X20RS-FCC.json`.
- Treat simulator results as the default reproducible baseline, but require
  physical-radio confirmation for issues that depend on touch events, icon
  loading, or source-handle behavior that may diverge from the simulator.

## Observed 26.1 Reference Patterns

- Widget registration: `system.registerWidget(...)`
- System tools and background tasks:
  `system.registerSystemTool(...)`, `system.registerTask(...)`,
  `system.getMemoryUsage(...)`
- Form fields seen in reference examples:
  `form.addSensorField(...)`, `form.addSourceField(...)`,
  `form.addChoiceField(...)`, `form.addNumberField(...)`,
  `form.addTextButton(...)`, `form.addBitmapField(...)`,
  `form.addStaticText(...)`
  - `form.addBooleanField(...)` was not surfaced in the checked `26.1`
    examples, so keep it optional in local code and stubs.
- Bitmap loading patterns: `lcd.loadMask(...)`, `lcd.loadBitmap(...)`
- Sensor/source access patterns:
  `system.getSource(...)`, then `source:value(...)` or
  `source:stringValue(...)`
- Source enumeration:
  `system.getSources(categoryNumber)` is documented in the `26.1.0-RC1` Lua
  docs and is now used by `SmartMapper` for source/control inventory.
- Touch constants and categories:
  `EVT_TOUCH*`, raw values `16640`, `16641`, `16642`
- Model surfaces confirmed during triage:
  `model.createMix(...)`, `model.name()`

## Compatibility Matrix

### API Surface Matrix

| Surface | Used by | 26.1 evidence | Smoke target | Notes |
| --- | --- | --- | --- | --- |
| `system.registerWidget` / widget callbacks | SensorList: yes; BoundryMap: yes; ethos_events: no; SmartMapper: yes; FBus: no | `lua/templates/widget/main.lua`, `lua/examples/widget-rssi/main.lua`, `lua/examples/widget-sinus/main.lua` | `python -m pytest scripts/SensorList/tests -q`, `python tools/sim/harness/run.py headless --project SensorList --radio X20RS-FCC --ethos-version latest-26.1`, `python -m pytest scripts/BoundryMap/tests -q`, `python -m pytest scripts/SmartMapper/tests -q`, `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SmartMapper-X20RS-FCC.json` | callback set includes `create`, `configure`, `paint`, `wakeup`, `event`, `read`, `write`, `menu`, `persistent` |
| `system.registerSystemTool` / `lcd.loadMask` / `lcd.loadBitmap` | SensorList: no; BoundryMap: no; ethos_events: yes; SmartMapper: no; FBus: no | `lua/tests/emergency-test/main.lua`, `lua/tests/latency-test/main.lua`, `lua/examples/bluetooth/main.lua`, `lua/examples/widget-lcddemo/main.lua` | `python tools/build.py --project ethos_events --deploy` | `ethos_events` probes events and touch logging; SmartMapper is now a widget, with harness-only API probing handled outside the installable script |
| `system.registerTask` / `system.getMemoryUsage` | SensorList: no; BoundryMap: no; ethos_events: no; SmartMapper: no; FBus: no | `lua/examples/task/main.lua`, `lua/examples/memstatus/main.lua` | reference-only until a task-based script lands | budget-sensitive helper APIs, no current local consumer |
| `system.getSource` / `system.getSources` / `system.getSensors` / `model.getSensors` | SensorList: yes; BoundryMap: yes; ethos_events: no; SmartMapper: yes; FBus: no | `lua/examples/widget-rssi/main.lua`, `lua/examples/widget-jet/main.lua`, `lua/examples/wizard/main.lua`; `26.1.0-RC1` Lua docs for `system.getSources(categoryNumber)` | `python -m pytest scripts/SensorList/tests -q`, `python -m pytest scripts/BoundryMap/tests -q`, `python -m pytest scripts/SmartMapper/tests -q`, `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SmartMapper-X20RS-FCC.json` | `SensorList` also exercises method-vs-field drift; `BoundryMap` uses source-handle reads for GPS and altitude; `SmartMapper` uses source enumeration for control inventory |
| `form.addSensorField` / `form.addSourceField` / `form.addChoiceField` / `form.addNumberField` / `form.addTextButton` / `form.addBitmapField` / `form.addStaticText` / `form.addBooleanField` | SensorList: yes; BoundryMap: yes; ethos_events: no; SmartMapper: no; FBus: no | `lua/examples/tool-form/main.lua`, `lua/examples/widget-jet/main.lua`, `lua/modules/crossfire/main.lua`, `lua/modules/elrs/main.lua` | `python -m pytest scripts/SensorList/tests -q`, `python -m pytest scripts/BoundryMap/tests -q` | `form.addBooleanField` was not surfaced in the checked `26.1` examples, so BoundryMap keeps that row optional and the local test harness omits the stub |
| `EVT_TOUCH*` and raw touch values `16640`, `16641`, `16642` | SensorList: yes; BoundryMap: yes; ethos_events: yes; SmartMapper: no; FBus: no | `scripts/SensorList/main.lua`, `scripts/BoundryMap/main.lua`, `scripts/ethos_events/main.lua` | `python -m pytest scripts/SensorList/tests -q`, `python -m pytest scripts/BoundryMap/tests -q`, `python tools/build.py --project ethos_events --deploy` | local tests lock the raw-value mapping, while `ethos_events` remains the manual simulator probe |

### Workstream Matrix

| Area | Status | Current 26.1 risk | Validation target | Follow-up |
| --- | --- | --- | --- | --- |
| Shared Lua API stubs | Compatibility matrix established; the local stubs now track the 26.1 source-field surface and keep boolean fields optional. | Local Lua and Python test doubles can still drift on unexercised task and memory-budget APIs. | Extend repo tests and script-local harnesses before relying on the stubs for more compatibility work. | [#78](https://github.com/doppleganger53/sloppy-ethos/issues/78), [#79](https://github.com/doppleganger53/sloppy-ethos/issues/79), [#80](https://github.com/doppleganger53/sloppy-ethos/issues/80) |
| `SensorList` | Automated X20RS-FCC WebSimulator harness added; broader runtime coverage still incomplete | Existing defensive source discovery may still miss method-vs-field or table-vs-userdata drift on `26.1`, especially in simulator startup paths. | Run `python tools/sim/harness/run.py headless --project SensorList --radio X20RS-FCC --ethos-version latest-26.1`; recheck `X20S` whenever the reproduction overlaps the existing simulator bug surface. | [#82](https://github.com/doppleganger53/sloppy-ethos/issues/82), [#30](https://github.com/doppleganger53/sloppy-ethos/issues/30) |
| `BoundryMap` | Highest active compatibility risk | GPS sub-value reads currently rely on `system.getSource({ ..., options = ... })`, and the `26.1` surface keeps boolean config fields optional rather than guaranteed. | Validate packaged and deployed assets on `X20RS`, then confirm GPS/config behavior on hardware if simulator results are ambiguous. | [#78](https://github.com/doppleganger53/sloppy-ethos/issues/78), [#79](https://github.com/doppleganger53/sloppy-ethos/issues/79), [#80](https://github.com/doppleganger53/sloppy-ethos/issues/80) |
| `ethos_events` | Best local runtime probe, `26.1` capture pending | The repo can already log raw events, but the current docs still point at an older upstream path and the `26.1` event map has not been recorded yet. | Deploy to both simulator targets and capture current raw touch/key constants before treating local event assumptions as settled. | [#81](https://github.com/doppleganger53/sloppy-ethos/issues/81) |
| `SmartMapper` | Widget implemented for issue #45 | `system.getSources(categoryNumber)` provides the source/control inventory path; mix, trim, channel, input, and special-function assignment reads remain defensive because model read/enumeration support can vary by runtime/model. | `python -m pytest scripts/SmartMapper/tests -q`, `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SmartMapper-X20RS-FCC.json`, then deploy with `python tools/build.py --project SmartMapper --deploy` for manual simulator or radio confirmation. | [#45](https://github.com/doppleganger53/sloppy-ethos/issues/45) |
| `Arduino FBus` | Feasibility unresolved | The surviving remote branch appears prompt-only, and the `26.1` reference scan did not expose a proven widget pattern for this work. | Decide whether to revive or retire the branch after reviewing the telemetry use case against current Ethos surfaces. | [#84](https://github.com/doppleganger53/sloppy-ethos/issues/84) |

## Repo-Level Follow-Ups

- [#85](https://github.com/doppleganger53/sloppy-ethos/issues/85):
  reconcile stale branches before stacking more compatibility work onto old
  refs.
- [#86](https://github.com/doppleganger53/sloppy-ethos/issues/86):
  governance drift around PR `#65` was closed during the baseline triage; keep
  that closure linked because the compatibility backlog was created with that
  dependency in mind.
- [#87](https://github.com/doppleganger53/sloppy-ethos/issues/87):
  update the contributor-facing script inventory and public compatibility status
  without changing the published release-link policy.

## Related Existing Issues

- [#10](https://github.com/doppleganger53/sloppy-ethos/issues/10):
  `SensorList` acceptable conflict definitions.
- [#67](https://github.com/doppleganger53/sloppy-ethos/issues/67),
  [#68](https://github.com/doppleganger53/sloppy-ethos/issues/68),
  [#69](https://github.com/doppleganger53/sloppy-ethos/issues/69),
  [#74](https://github.com/doppleganger53/sloppy-ethos/issues/74):
  active `BoundryMap` feature work that should stay separate from the baseline
  compatibility fixes above.
