# SmartMapper

SmartMapper is currently an Ethos `26.1` API validation probe for issue
[#83](https://github.com/doppleganger53/sloppy-ethos/issues/83), before the
full function-mapping feature in issue
[#45](https://github.com/doppleganger53/sloppy-ethos/issues/45) resumes.

## Current Behavior

- Registers an Ethos system tool named `SmartMapper Probe`.
- Prints available `model.*` APIs, documented source category constant values,
  and `system.getSources(categoryNumber)` probe results with the
  `[SmartMapperProbe]` log prefix.
- Attempts to write the same report to the first writable path:
  /documents/SmartMapper-api-probe.txt,
  /scripts/SmartMapper/smartmapper-api-probe.txt, or
  smartmapper-api-probe.txt.
- Explicitly marks whether candidate read/enumeration support appears present
  for mixes, logical switches, trims, special functions, and switch/input
  assignments.

The probe does not create or mutate model configuration. It intentionally
reports `model.createMix` as availability evidence only.

## Simulator Or Radio Run

1. Deploy the probe:

   ```powershell
   python tools/build.py --project SmartMapper --deploy
   ```

2. Open the Ethos `SmartMapper Probe` system tool on the `26.1` simulator or
   target radio.
3. Capture the printed `[SmartMapperProbe]` lines or copy the generated report
   file when the runtime exposes a writable path.
4. Add the observed results to issue
   [#45](https://github.com/doppleganger53/sloppy-ethos/issues/45) before
   reviving implementation work from any stale branch.

## Validation

```powershell
luac -p scripts/SmartMapper/main.lua
python -m pytest scripts/SmartMapper/tests -q
python tools/build.py --project SmartMapper --dist
```
