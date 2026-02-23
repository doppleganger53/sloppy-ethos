# sloppy-ethos

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

## Project Layout

- `src/scripts/SensorList/main.lua`: widget implementation
- `src/scripts/SensorList/README.md`: widget-focused usage notes
- `tools/build-package.ps1`: syntax-check + packaging script
- `tools/deploy-ethos-sim.ps1`: simulator deploy helper (placeholder)
- `deslopification/prompts/SensorList.md`: original implementation prompt
- `docs/`: development notes and handoff documents

## Development

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
- See `deslopification/memory/HANDOFF_2026-02-21.md` for latest session notes.

## Roadmap

- Touch column headings to change sorting.
- Refine conflict display behavior for `Application ID` vs `Physical ID` cases.
- Allow defining acceptable conflicts (for example, multiple values per device such as receivers or servos).
