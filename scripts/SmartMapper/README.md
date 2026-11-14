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

## Run the probe in an isolated simulator

From the repository folder, start an isolated X20RS-FCC preview:

```powershell
$preview = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
python tools/sim/harness/run.py gui --project SmartMapper --radio X20RS-FCC --ethos-version latest-26.1 --persist-dir "$preview"
```

Open **SmartMapper Probe** in the simulator's system tools. Capture the
`[SmartMapperProbe]` lines or the generated report if the runtime exposes the
file. Record the radio and resolved firmware from the harness result when
reporting observations for [#45](https://github.com/doppleganger53/sloppy-ethos/issues/45).

If the task specifically calls for updating a live Suite installation,
verify its configured path and select that radio explicitly. For the X20RS
entry, use `python tools/build.py --project SmartMapper --deploy --sim-radio X20RS`.
For a requested physical-radio check, use the install ZIP below and record
the probe output on that radio.

## Validation

```powershell
luac -p scripts/SmartMapper/main.lua
python -m pytest scripts/SmartMapper/tests -q
python tools/build.py --project SmartMapper --dist
```
