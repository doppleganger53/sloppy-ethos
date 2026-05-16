# Ethos Simulator Harness

This harness downloads cached Ethos WebSimulator runtimes, stages repository
scripts with the same install contract as `tools/build.py`, and runs repeatable
smoke checks against the staged project or project set.

Common commands:

```powershell
python tools/sim/harness/run.py download --radio X20RS-FCC --ethos-version latest-26.1
python tools/sim/harness/run.py headless --project SensorList --radio X20RS-FCC --ethos-version latest-26.1
python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SensorList-X20RS-FCC.json
python tools/sim/harness/run.py gui --project SensorList --project BoundryMap --radio X20RS-FCC --ethos-version latest-26.1
```

Runtime ZIPs, extracted JavaScript/WASM files, run persist trees, GUI files,
and logs are stored under `tools/sim/radios/` and `tools/sim/runs/`. Those
directories are ignored by git because simulator payloads are downloaded release
artifacts, not source files.

The headless command exits with structured status:

- `success`: simulator started and `reloadScripts` completed without detected
  script errors.
- `missing_runtime`: the requested radio/version WebSimulator package is not
  available or is not cached when `--no-download` is used.
- `download_failure`: release metadata, download, or checksum validation failed.
- `startup_failure`: Node or the WebSimulator runtime failed before script
  reload.
- `script_failure`: the simulator surfaced Lua/script/runtime error output.
- `timeout`: the headless run did not finish within the requested timeout.

By default, the Node headless path does not call the WebSimulator export
`_writeDefaultSettingsAndModel()` because the X20RS-FCC `26.1.0-RC2` runtime
blocks in that call under Node. Use `--write-default-model` only when validating
that behavior against a runtime known not to block.
