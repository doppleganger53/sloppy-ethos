# Development Notes

## Prerequisites

- Python 3.9+
- Lua tooling on PATH (`lua`, `luac`)
- Optional: `stylua` for formatting

## Core Commands

- Syntax check:
  `luac -p scripts/SensorList/main.lua`
- Package and deploy:
  `python tools/build.py --project SensorList --dist`
  `python tools/build.py --project SensorList --deploy`
  `python tools/build.py --project ethos_events --deploy`
  `python tools/build.py --project SensorList --clean --sim-radio X20RS`
  `python tools/build.py --help`

## Configuring Simulator Path

- Copy `tools/deploy.config.example.json` to `tools/deploy.config.json` and populate `ETHOS_SIM_PATHS`.
- `ETHOS_SIM_PATHS` must be an array of entries with `radio` and `path`.
- Mark exactly one entry as `"default": true` for deploy/clean when `--sim-radio` is not provided.
- `tools/deploy.config.json` is ignored via `.gitignore`, so each contributor can keep their private path local.

## Packaging Behavior

`tools/build.py`:

- validates Lua syntax for all project Lua files using `luac -p`.
- reads package version from `VERSION` (or `--version` override).
- produces `dist/{ProjectName}-{version}.zip`.
- optionally copies `scripts/{ProjectName}` into `${ETHOS_SIM_PATHS[default_or_selected_radio]}/scripts/{ProjectName}` when `--deploy` is specified (no ZIP).
- supports `--clean` to remove `scripts/{ProjectName}` from simulator deploy path.
- supports `--sim-radio` to resolve model-specific simulator paths via `ETHOS_SIM_PATHS`.
- supports `--help` to print `tools/build_help.txt` command reference.
- supports custom ZIP destination via `--out-dir`.
- Write operations fail fast with clear errors when paths are missing or unwritable.

## Simulator Tips

- If stale behavior appears after script updates, remove cached `main.luac` and reinstall the widget.
- Ethos Info errors persist until you hit the `Reset` button; clear them between test runs.

## Running the Python tests

- Select a Python 3.9+ interpreter in VS Code via `Python: Select Interpreter`, or use a shell where `python` resolves to your desired interpreter.
- Install the pytest harness listed in `requirements/dev.txt`: `python -m pip install -r requirements/dev.txt` (includes `pytest-cov` for coverage in the VS Code Test Explorer).
- Run the Lua-driven sensor list test file through pytest: `python -m pytest tests/test_sensorlist_widget.py`.
- For any documentation updates, run docs contract checks: `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`.
- Use the VS Code Testing view to run/discover tests and trigger coverage once dependencies are installed for the selected interpreter.

## Release Workflow

1. Confirm `VERSION` contains the release version and update `CHANGELOG.md`.
2. Build release artifact:
   `python tools/build.py --project SensorList --dist`
3. Validate the release branch according to touched files (see `AGENTS.md` validation matrix).
4. Tag and push:
   `git tag v{VERSION}`
   `git push origin v{VERSION}`
5. Publish GitHub release with notes from `CHANGELOG.md` and attach `dist/SensorList-{VERSION}.zip`.
