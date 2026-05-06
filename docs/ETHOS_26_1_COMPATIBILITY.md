# Ethos 26.1 Compatibility Baseline

This document is the repository's working baseline for Ethos `26.1`
compatibility checks. It records the exact reference checkout, the simulator
targets to use for repeatable validation, and the currently known upgrade risks
across the repo.

## Reference Checkout

- Repository:
  `https://github.com/FrSkyRC/ETHOS-Feedback-Community.git`
- Local path: `..\ETHOS-Feedback-Community`
- Resolved Windows path:
  `C:\Users\kurtk\Documents\Workspaces\EthosLua\ETHOS-Feedback-Community`
- Branch: `26.1`
- Commit: `e9d7afb7`
- Baseline rule:
  the original request mentioned Ethos `21.1`, but the concrete checked and
  requested reference branch was `26.1`. Keep `26.1` as the evidence source
  unless a later issue explicitly changes the target version.

## Recreate Or Refresh

```powershell
git clone --branch 26.1 --single-branch https://github.com/FrSkyRC/ETHOS-Feedback-Community.git ..\ETHOS-Feedback-Community
git -C ..\ETHOS-Feedback-Community fetch origin 26.1 --depth 1
git -C ..\ETHOS-Feedback-Community checkout 26.1
git -C ..\ETHOS-Feedback-Community pull --ff-only origin 26.1
git -C ..\ETHOS-Feedback-Community rev-parse --short HEAD
```

Use the clone command when the reference checkout is missing. Use the
fetch/checkout/pull sequence when the repo already exists and you want to
confirm that it is still pinned to the tracked `26.1` branch.

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
- Treat simulator results as the default reproducible baseline, but require
  physical-radio confirmation for issues that depend on touch events, icon
  loading, or source-handle behavior that may diverge from the simulator.

## Observed 26.1 Reference Patterns

- Widget registration: `system.registerWidget(...)`
- System tools and background tasks:
  `system.registerSystemTool(...)`, `system.registerTask(...)`
- Form fields seen in reference examples:
  `form.addSensorField(...)`, `form.addChoiceField(...)`,
  `form.addNumberField(...)`, `form.addTextButton(...)`,
  `form.addStaticText(...)`
- Bitmap loading patterns: `lcd.loadMask(...)`, `lcd.loadBitmap(...)`
- Sensor/source access patterns:
  `system.getSource(...)`, then `source:value(...)` or
  `source:stringValue(...)`
- Model surfaces confirmed during triage:
  `model.createMix(...)`, `model.name()`

## Compatibility Matrix

| Area | Status | Current 26.1 risk | Validation target | Follow-up |
| --- | --- | --- | --- | --- |
| Shared Lua API stubs | Baseline defined, coverage missing | Local Lua and Python test doubles can drift from `26.1` registration, form, and source-handle behavior. | Extend repo tests and script-local harnesses before relying on the stubs for more compatibility work. | [#77](https://github.com/doppleganger53/sloppy-ethos/issues/77) |
| `SensorList` | Partially aligned, runtime coverage incomplete | Existing defensive source discovery may still miss method-vs-field or table-vs-userdata drift on `26.1`, especially in simulator startup paths. | Validate on `X20RS`; recheck `X20S` whenever the reproduction overlaps the existing simulator bug surface. | [#82](https://github.com/doppleganger53/sloppy-ethos/issues/82), [#30](https://github.com/doppleganger53/sloppy-ethos/issues/30) |
| `BoundryMap` | Highest active compatibility risk | GPS sub-value reads currently rely on `system.getSource({ ..., options = ... })`, boolean config fields assume `form.addBooleanField(...)`, and asset loading needs a `26.1` install/deploy check. | Validate packaged and deployed assets on `X20RS`, then confirm GPS/config behavior on hardware if simulator results are ambiguous. | [#78](https://github.com/doppleganger53/sloppy-ethos/issues/78), [#79](https://github.com/doppleganger53/sloppy-ethos/issues/79), [#80](https://github.com/doppleganger53/sloppy-ethos/issues/80) |
| `ethos_events` | Best local runtime probe, `26.1` capture pending | The repo can already log raw events, but the current docs still point at an older upstream path and the `26.1` event map has not been recorded yet. | Deploy to both simulator targets and capture current raw touch/key constants before treating local event assumptions as settled. | [#81](https://github.com/doppleganger53/sloppy-ethos/issues/81) |
| `SmartMapper` | Blocked on API proof | Reference examples confirm `model.createMix(...)` and `model.name()`, but do not yet prove the read/enumeration APIs needed to inspect existing mixes, switches, trims, or special functions. | Probe the reference checkout and runtime APIs before reviving implementation on the stale feature branch. | [#83](https://github.com/doppleganger53/sloppy-ethos/issues/83), [#45](https://github.com/doppleganger53/sloppy-ethos/issues/45) |
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
