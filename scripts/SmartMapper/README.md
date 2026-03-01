# SmartMapper

Deferred placeholder widget for the SmartMapper concept.

The original goal is to inspect model configuration and build a compact
control-to-function map, but current Ethos `1.6.4` widget runtime access is not
sufficient to implement that feature reliably.

## Current Behavior

- Registers as an Ethos widget named `SmartMapper`.
- Displays a clear defer message instead of attempting unsupported runtime
  probing.
- Documents that full implementation is blocked until Ethos `1.7.x` final API
  validation confirms richer model access in the target script context.

## Why It Is Deferred

- Ethos `1.6.4` widget runtime exposes `model.getChannel()` and a few
  output-oriented helpers, but not the model-enumeration APIs needed to inspect:
  - mixes
  - special functions
  - logical switches
  - trims
  - switch inventory
- Available channel objects expose output properties like channel name and min/max
  values, not the routing metadata needed to reconstruct switch/input mappings.
- Continuing to probe in-widget would add noise without closing the API gap.

## Build and Install

1. Build the install ZIP from repository root:
   `python tools/build.py --project SmartMapper --dist`
2. In Ethos Suite, run the Lua install/import action and select the ZIP from
   `dist/`.
3. Transfer/sync to the radio.

Installed path on radio: `scripts/SmartMapper`.
Artifact version source: `scripts/SmartMapper/VERSION`.

## Simulator Deploy

- Deploy script files into the configured simulator scripts directory:
  `python tools/build.py --project SmartMapper --deploy`
- Configure simulator paths via `tools/deploy.config.json` `ETHOS_SIM_PATHS`,
  with one entry marked `"default": true`.

## Validation Checklist

- `luac -p scripts/SmartMapper/main.lua`
- `python -m pytest tests/test_smartmapper_widget.py -q`
- `python tools/build.py --project SmartMapper --dist`
- `python tools/build.py --project SmartMapper --deploy`

## Next Target

- Retest against Ethos `1.7.x` final runtime before resuming implementation.
- Start with a small capability probe to confirm whether the required model APIs
  are exposed in the intended script context.
