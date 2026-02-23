# Development Notes

## Prerequisites

- Python 3.9+
- Lua tooling on PATH (`lua`, `luac`)
- Optional: `stylua` for formatting

## Core Commands

- Syntax check:
  `luac -p src/scripts/SensorList/main.lua`
- Package and deploy:
  `python tools/build.py --project SensorList --dist`
  `python tools/build.py --project SensorList --deploy`

## Configuring Simulator Path

- Copy `tools/deploy.config.example.json` to `tools/deploy.config.json` and populate `ETHOS_SIM_PATH` with your simulator persist directory.
- Alternatively override via `ETHOS_SIM_PATH` environment variable (Python script honors the override).
- `tools/deploy.config.json` is ignored via `.gitignore`, so each contributor can keep their private path local.

## Packaging Behavior

`tools/build.py`:

- validates Lua syntax using `luac -p`.
- reads package version from `VERSION` (or `--version` override).
- produces `dist/{ProjectName}-{version}.zip`.
- optionally copies `src/scripts/{ProjectName}` into `${ETHOS_SIM_PATH}/scripts/{ProjectName}` when `--deploy` is specified (no ZIP).
- Write operations fail fast with clear errors when paths are missing or unwritable.
- PowerShell helpers (`tools/build-package.ps1`, `tools/deploy-ethos-sim.ps1`) remain available as documented fallbacks for Windows-only workflows.

## Simulator Tips

- If stale behavior appears after script updates, remove cached `main.luac` and reinstall the widget.
- Ethos Info errors persist until you hit the `Reset` button; clear them between test runs.

## Running the Python tests

- Select a Python 3.9+ interpreter in VS Code via `Python: Select Interpreter`, or use a shell where `python` resolves to your desired interpreter.
- Install the pytest harness listed in `requirements-dev.txt`: `python -m pip install -r requirements-dev.txt` (includes `pytest-cov` for coverage in the VS Code Test Explorer).
- Run the Lua-driven sensor list test file through pytest: `python -m pytest tests/test_sensorlist_widget.py`.
- For any documentation updates, run docs contract checks: `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`.
- Use the VS Code Testing view to run/discover tests and trigger coverage once dependencies are installed for the selected interpreter.
