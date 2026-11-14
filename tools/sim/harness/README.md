# Ethos Simulator Harness

This harness downloads cached Ethos WebSimulator runtimes, stages repository
scripts with the same install contract as `tools/build.py`, and runs repeatable
smoke checks against the staged project or project set.

From the repository folder, choose a radio/version (or a checked-in suite).
In PowerShell, give each independent preview a fresh scratch path:

```powershell
python tools/sim/harness/run.py download --radio X20RS-FCC --ethos-version latest-26.1
$smoke = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SensorList-X20RS-FCC.json --persist-dir "$smoke"
$preview = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
python tools/sim/harness/run.py gui --project SensorList --project BoundryMap --radio X20RS-FCC --ethos-version latest-26.1 --persist-dir "$preview"
```

Without `--persist-dir`, both modes use `persist/` inside the run directory;
the default run directory is new for each invocation. Use an empty directory
for an explicit scratch path: existing files there are included in the run
and staged scripts can overwrite files of the same name. Neither mode needs a
Suite profile, connected radio or deployment config. The JSON result names
the host directories (`runDir` and `persistDir`); the runtime sees the staged
files under `/persist/{Radio}`. Close the GUI server with Ctrl+C.

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

- `success`: simulator started with no detected script errors. Check
  `reloaded` to see whether this runtime supported a script reload. Startup
  does not establish what was drawn or what a physical receiver reported.
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
