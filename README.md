# sloppy-ethos

`<human>`
What started as a simple experiment to play around with agentic AI coding for a little utility widget has morphed itself into an idea to start a community repo with two main goals:
- Make some handy Ethos utilities
- Spend more time flying than coding

So, if you're here to just grab a widget, script, or be amused by the slop - by all means enjoy yourself !!

If you want to play around with  finding repeatable, low-maintenance ways to deslopify and generate decent scripts to make our RC hobbist lives easier, let see those PRs :)

`</human>`

Ethos Lua widget workspace. Current active project: `SensorList`.

## Current Status

- Widget loads in Ethos simulator and appears in fullscreen widget picker.
- Sensor discovery works in simulator for the tested model.
- Packaging flow produces Ethos-installable ZIP archives.
- Manual list scrolling is implemented via wheel/button events.

## Quick Start

- Build or deploy from repo root:

```powershell
python tools/build.py --project SensorList --dist
python tools/build.py --project SensorList --deploy
```

- Install the dist ZIP inside Ethos Suite for radio deployment.
- Use the PowerShell helpers (`tools/build-package.ps1`, `tools/deploy-ethos-sim.ps1`) as Windows-only fallbacks.
- Configure `tools/deploy.config.json` (copy the example) or set `ETHOS_SIM_PATH` before running `--deploy`.
- Package version is read from `VERSION`; ZIP name format is `dist/{ProjectName}-{version}.zip`.

### 30-Second First Run

1. Run:
   `python tools/build.py --project SensorList --dist`
2. Confirm output contains both:
   - `Checking Lua syntax:`
   - `Packaged widget ZIP:`
3. Install the generated archive from `dist/` via Ethos Suite.

## Project Layout

- `src/scripts/SensorList/main.lua`: widget implementation
- `src/scripts/SensorList/README.md`: widget-focused usage notes
- `tools/build-package.ps1`: syntax-check + packaging script
- `tools/deploy-ethos-sim.ps1`: simulator deploy helper (placeholder)
- `deslopification/prompts/SensorList.md`: original implementation prompt
- `docs/`: development notes and handoff documents
- `docs/REPOSITORY_LAYOUT.md`: reference for `tools/`, `tests/`, `deslopification/`, and root artifacts

## Development

- Recommended development environment: [Visual Studio Code](https://code.visualstudio.com/).
- Recommended coding agent: [OpenAI Codex](https://openai.com/codex/).
- Format Lua: VS Code task `Format Lua (stylua)` or run `stylua src`
- Build package: VS Code task `Build Ethos Install ZIP`
- Lua parse check:

```powershell
luac -p src/scripts/SensorList/main.lua
```

### Running the Python tests

- Select a Python 3.9+ interpreter in VS Code via `Python: Select Interpreter` (or ensure `python` resolves to your preferred interpreter in terminal).
- Install test dependencies once per interpreter: `python -m pip install -r requirements-dev.txt`.
- Execute the sensor-list test file: `python -m pytest tests/test_sensorlist_widget.py`.
- VS Code Test Explorer coverage runs require `pytest-cov`, which is included in `requirements-dev.txt`.

## Collaboration

- See `CONTRIBUTING.md` for contribution flow.
- Use `.github/PULL_REQUEST_TEMPLATE.md` when opening PRs.
- See `CODE_OF_CONDUCT.md` for collaboration expectations.
- See `SECURITY.md` for responsible vulnerability reporting.
- See `deslopification/memory/HANDOFF_2026-02-21.md` for latest session notes.

## Troubleshooting

- `Required command 'luac' not found on PATH.`:
  install Lua tooling and verify `luac` resolves in your shell.
- `Simulator path not configured.`:
  set `ETHOS_SIM_PATH` or create `tools/deploy.config.json` from `tools/deploy.config.example.json`.
- Deploy fails with permission denied:
  close Ethos Suite, then retry deployment from a shell with write access.
- Stale simulator behavior after script changes:
  remove cached `main.luac`, reinstall widget, then reset Ethos Info errors.

## Roadmap

### sloppy-ethos

- Script idea: smart switch map that pre-populates mapped switches and identifies unused switches
- optimize agentic and human workflows in the repo
- genericize build and test scripts for multiple scripts

### SensorList

- Touch column headings to change sorting.
- Refine conflict display behavior for `Application ID` vs `Physical ID` cases.
- Allow defining acceptable conflicts (for example, multiple values per device such as receivers or servos).

Roadmap implementation work is currently tracked in the following enhancement issues:

- [#8](https://github.com/doppleganger53/sloppy-ethos/issues/8) Touchable column headers for sort control.
- [#9](https://github.com/doppleganger53/sloppy-ethos/issues/9) Conflict display severity refinement.
- [#10](https://github.com/doppleganger53/sloppy-ethos/issues/10) Acceptable conflict definition model.
