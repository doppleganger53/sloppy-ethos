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

By default, headless and GUI runs stage projects into the same Ethos Suite
persist directory for the selected radio and runtime version:
`{Ethos Suite data}/.simulator/{EthosVersion}/persist/{Radio}`. Use
`--persist-dir` only when you need to point the WebSimulator at a nonstandard
persist tree.

Runtime ZIPs and extracted JavaScript/WASM files are stored under
`tools/sim/radios/`. GUI mode serves those cached runtime files directly
instead of copying JS/WASM into each run directory. Run logs and generated GUI
wrapper files are stored under `tools/sim/runs/`. Those directories are ignored
by git because simulator payloads and run artifacts are not source files.
When `latest-26.1` already has a matching cached `26.1` runtime, the harness
uses the cached package before calling GitHub so repeated smoke runs can work
offline. The `download` command still checks GitHub for latest aliases so you
can refresh the latest-release lookup explicitly.

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

By default, the headless and GUI paths do not call the WebSimulator export
`_writeDefaultSettingsAndModel()` because some X20RS-FCC runtimes can block or
abort in that call during automated startup. Use `--write-default-model` only
when validating that behavior against a runtime known not to block.
