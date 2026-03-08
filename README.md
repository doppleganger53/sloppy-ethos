# sloppy-ethos

## Goal

`<human>`

What started as a simple experiment to play around with agentic AI coding for a little utility widget has morphed itself into an idea to start a community repo with a few main goals:

- Make some handy Ethos utilities
- Help those unfamiliar with lua to create useful scripts for themselves and others
- Spend more time flying than coding

So, if you're here to just grab a widget, script, or be amused by the slop - by all means enjoy yourself!

If you want to play around with finding repeatable, low-maintenance ways to efficiently generate decent scripts / utilities to make our RC hobbist lives easier, PRs are welcome :)

`</human>`

Ethos Lua widget workspace. Current active project: `SensorList`.

## Current Status

- Widget loads in Ethos simulator and appears in fullscreen widget picker.
- Sensor discovery works in simulator for the tested model.
- Packaging flow produces Ethos-installable ZIP archives.
- Manual list scrolling is implemented via wheel/button events.

## Visual Overview

Visual onboarding media is intentionally placeholder-only for now. A screenshot
or GIF walkthrough will be added in a follow-up docs update.

## Quick Start

- Build or deploy from repo root:

```powershell
python tools/build.py --project SensorList --dist
python tools/build.py --project SensorList --project ethos_events --dist
python tools/build.py --project SensorList --deploy
python tools/build.py --project ethos_events --deploy
python tools/build.py --project SensorList --clean --sim-radio X20RS
python tools/build.py --help
```

- Running --clean now clears dist/ (or the configured --out-dir) outputs along with removing the simulator scripts.
- Install the dist ZIP inside Ethos Suite for radio deployment.
- Configure `tools/deploy.config.json` with `ETHOS_SIM_PATHS` entries before running `--deploy`/`--clean`.
- Mark exactly one `ETHOS_SIM_PATHS` entry as `"default": true` for deploy/clean without `--sim-radio`.
- Single-script package version is read from `scripts/{ProjectName}/VERSION`; ZIP name format is `dist/{ProjectName}-{version}.zip`.
- Multi-script dist bundles are an explicit naming exception and use the unversioned ZIP name: `dist/sloppy-ethos_scripts.zip`.
- Root `VERSION` remains the repository version source of truth.

### 30-Second First Run

1. Run:
   `python tools/build.py --project SensorList --dist`
2. Confirm output contains both:
   - `Checking Lua syntax:`
   - `Packaged widget ZIP:`
3. Install the generated archive from `dist/` via Ethos Suite.

To build a single install ZIP with multiple scripts:
`python tools/build.py --project SensorList --project ethos_events --dist`

## Project Layout

- `scripts/SensorList/main.lua`: widget implementation
- `scripts/SensorList/README.md`: widget-focused usage notes
- `scripts/ethos_events/main.lua`: system-tool event tracer entrypoint
- `scripts/ethos_events/README.md`: event tracer usage notes
- `tools/build.py`: syntax-check + packaging + simulator deploy script
- `tools/create_todo_issues.py`: GitHub issue bootstrap for TODO backlog tracking
- `deslopification/prompts/done/SensorList.md`: original implementation prompt
- `docs/`: development notes and handoff documents
- `docs/REPOSITORY_LAYOUT.md`: reference for `tools/`, `tests/`, `deslopification/`, and root artifacts
- [docs/SensorList/SENSORLIST_ARCHITECTURE.md](docs/SensorList/SENSORLIST_ARCHITECTURE.md): SensorList lifecycle/state flow reference

## Development

- Recommended development environment: [Visual Studio Code](https://code.visualstudio.com/).
- Recommended coding agent: [OpenAI Codex](https://openai.com/codex/).
- Format Lua: VS Code task `Format Lua (stylua)` or run `stylua --config-path tools/config/stylua.toml scripts`
- Build package: VS Code task `Build Ethos Install ZIP`
- Lua parse check:

```powershell
luac -p scripts/SensorList/main.lua
```

### Running the Python tests

- Select a Python 3.9+ interpreter in VS Code via `Python: Select Interpreter` (or ensure `python` resolves to your preferred interpreter in terminal).
- Install test dependencies once per interpreter: `python -m pip install -r requirements/dev.txt`.
- Execute the sensor-list test file: `python -m pytest tests/test_sensorlist_widget.py`.
- VS Code Test Explorer coverage runs require `pytest-cov`, which is included in `requirements/dev.txt`.

### Lua Coverage In VS Code

- Install the recommended VS Code extension: `Coverage Gutters`.
- Install Lua coverage tools with LuaRocks:
  - `luarocks install luacov`
  - `luarocks install luacov-reporter-lcov`
- Run the VS Code task `Lua Coverage Refresh (SensorList)`.
- Coverage output is written to `coverage/lua/luacov.report.out` and the workspace is configured to let Coverage Gutters display it.

## Releases

- Changelog source of truth: `CHANGELOG.md`.
- Repository version source of truth: `VERSION`.
- Script artifact versions: `scripts/{ProjectName}/VERSION`.
- Generate GitHub release note bodies from `CHANGELOG.md` with `tools/write_release_notes.py`, then publish with `gh release create --notes-file` so markdown spacing and headings render correctly.
- Release scope is explicit:
  - repo releases track repository-wide workflow/docs/versioning changes.
  - script releases track installable script artifacts and script-specific manual gates.
- Release branch naming is scope-specific:
  - repo releases use `release/v{VERSION}`
  - script releases use `release/{ProjectName}-v{VERSION}`
- Repo release install deliverables include:
  - `dist/sloppy-ethos_scripts.zip` (built via `python tools/build.py --project SensorList --project ethos_events --dist`)
- Release publishing checklist for repo releases:
  - run `python tools/build.py --project SensorList --project ethos_events --dist`
  - attach only `dist/sloppy-ethos_scripts.zip` in the GitHub release
- Main-branch workflow, branch naming conventions, version bump timing, optional `-rc.N`, and release-prep flow:
  [Development policy](docs/DEVELOPMENT.md#main-branch-release-and-versioning-policy).
- Published release notes and install assets: [GitHub Releases](https://github.com/doppleganger53/sloppy-ethos/releases).

## Collaboration

- See `CONTRIBUTING.md` for contribution flow.
- Use `.github/PULL_REQUEST_TEMPLATE.md` when opening PRs.
- See `CODE_OF_CONDUCT.md` for collaboration expectations.
- See `SECURITY.md` for responsible vulnerability reporting.
- Start session-memory context with `deslopification/memory/README.md`.
- Use `deslopification/memory/CATALOG.md` for full historical note lookup.

## Troubleshooting

- `Required command 'luac' not found on PATH.`:
  install Lua tooling and verify `luac` resolves in your shell.
- `Simulator path not configured.`:
  configure `ETHOS_SIM_PATHS` in `tools/deploy.config.json` and set one default entry.
- Deploy fails with permission denied:
  close Ethos Suite, then retry deployment from a shell with write access.
- Stale simulator behavior after script changes:
  remove cached `main.luac`, reinstall widget, then reset Ethos Info errors.

## Roadmap

### sloppy-ethos

- Script idea: smart switch map that pre-populates mapped switches and identifies unused switches
- optimize agentic and human workflows in the repo
- Completed [#42](https://github.com/doppleganger53/sloppy-ethos/issues/42): repo releases include the multi-script bundle deliverable `dist/sloppy-ethos_scripts.zip`.

### SensorList

- Allow defining acceptable conflicts (for example, multiple values per device such as receivers or servos).

Roadmap implementation work is currently tracked in the following enhancement issues:

- [#10](https://github.com/doppleganger53/sloppy-ethos/issues/10) Acceptable conflict definition model.
