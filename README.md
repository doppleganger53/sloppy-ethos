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

**[Make your first widget](docs/FIRST_WIDGET.md)** · **[Download an existing script](#download-latest-script-releases)**

Ethos Lua script workspace. Current active script directories include
`SensorList`, `BoundryMap`, `SmartMapper`, and `ethos_events`.

## Script Inventory and 26.1 Compatibility Status

- `SensorList` (released): active and tracked in the published release list.
- `BoundryMap` (released): active and tracked in the published release list;
  release `0.2.0` is validated on Ethos 1.6.6.
- `SmartMapper` (in progress): active, implemented for issue [#45](https://github.com/doppleganger53/sloppy-ethos/issues/45),
  but not yet part of the published script release list.
- `ethos_events` (released): active utility script with a published release asset.
- `Arduino FBus` (deferred): feasibility remains unresolved under issue
  [#84](https://github.com/doppleganger53/sloppy-ethos/issues/84), with no checked-in script release target yet.

For contributor-facing compatibility context, see
[docs/ETHOS_26_1_COMPATIBILITY.md](docs/ETHOS_26_1_COMPATIBILITY.md) and
issue [#87](https://github.com/doppleganger53/sloppy-ethos/issues/87).

## Current Status

- Widget loads in Ethos simulator and appears in fullscreen widget picker.
- Sensor discovery works in simulator for the tested model.
- Packaging flow produces Ethos-installable ZIP archives.
- Manual list scrolling is implemented via wheel/button events.

## Download Latest Script Releases

This section tracks currently published single-script GitHub release assets.
Unreleased scripts may exist under `scripts/` without appearing here yet.

- [Download SensorList (v1.0.1)](https://github.com/doppleganger53/sloppy-ethos/releases/download/SensorList-v1.0.1/SensorList-1.0.1.zip)
- [Download BoundryMap (v0.2.0)](https://github.com/doppleganger53/sloppy-ethos/releases/download/BoundryMap-v0.2.0/BoundryMap-0.2.0.zip)
- [Download ethos_events (v0.1.0)](https://github.com/doppleganger53/sloppy-ethos/releases/download/ethos_events-v0.1.0/ethos_events-0.1.0.zip)

SmartMapper is intentionally absent from this section until its first release
artifact is published.


## Visual Overview

Visual onboarding media is intentionally placeholder-only for now. A screenshot
or GIF walkthrough will be added in a follow-up docs update.

## Quick Start

- Build an install ZIP from the repository folder:

```powershell
python tools/build.py --project SensorList --dist
python tools/build.py --project BoundryMap --dist
python tools/build.py --project SensorList --project ethos_events --dist
python tools/build.py --help
```

- Packaging leaves Suite and the radio alone. To deliberately update a named
  Suite installation later, see [Development](docs/DEVELOPMENT.md#configuring-simulator-path).
- Single-script package version is read from `scripts/{ProjectName}/VERSION`; ZIP name format is `dist/{ProjectName}-{version}.zip`.
- Multi-script dist bundles are an explicit naming exception and use the unversioned ZIP name: `dist/sloppy-ethos_scripts.zip`.
- Projects can optionally declare extra installable files or generic asset groups in a project-local build manifest; those files are packaged into the ZIP root and are also deployed/cleaned alongside `scripts/{ProjectName}`.
- BoundryMap map assets are local-only because flying-site maps can reveal private locations. Put private maps under `scripts/BoundryMap/assets/maps/`; [scripts/BoundryMap/build.json](scripts/BoundryMap/build.json) defines the asset globs and script-local destinations for map images and metadata.
- Root `VERSION` remains the repository version source of truth.

- Download and run the automated Ethos WebSimulator harness:

```powershell
python tools/sim/harness/run.py download --radio X20RS-FCC --ethos-version latest-26.1
$scratch = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SensorList-X20RS-FCC.json --persist-dir "$scratch"
```

- The [WebSimulator harness](tools/sim/harness/README.md) also opens an isolated GUI.
  Read `persistDir` in its result to see where the scripts were staged.
- Use headless harness runs as the repeatable simulator layer between script-local Lua/pytest tests and manual Ethos Suite or physical-radio checks.

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
- `scripts/{ProjectName}/tests/`: script-owned pytest wrappers and Lua harnesses
- `scripts/BoundryMap/build.json`: project build manifest for installable assets
- `scripts/BoundryMap/main.lua`: BoundryMap widget implementation
- `scripts/BoundryMap/README.md`: BoundryMap usage notes
- `scripts/BoundryMap/assets/icons/`: tracked widget icon assets
- `scripts/BoundryMap/assets/maps/`: ignored local-only map asset folder, created by contributors when packaging private maps
- `scripts/ethos_events/main.lua`: system-tool event tracer entrypoint
- `scripts/ethos_events/README.md`: event tracer usage notes
- `tools/build.py`: syntax-check + packaging + simulator deploy script
- `tools/sim/harness/run.py`: automated Ethos WebSimulator download, headless smoke, and GUI launch harness
- `tools/create_todo_issues.py`: GitHub issue bootstrap for TODO backlog tracking
- `deslopification/prompts/done/SensorList.md`: original implementation prompt
- `docs/`: development notes and handoff documents
- [docs/ETHOS_26_1_COMPATIBILITY.md](docs/ETHOS_26_1_COMPATIBILITY.md): Ethos
  `26.1` reference checkout, validation targets, and compatibility matrix
- `docs/REPOSITORY_LAYOUT.md`: reference for `tools/`, repo-level `tests/`, script-local tests, `deslopification/`, and root artifacts
- [docs/SensorList/SENSORLIST_ARCHITECTURE.md](docs/SensorList/SENSORLIST_ARCHITECTURE.md): SensorList lifecycle/state flow reference

## Development

- Recommended development environment: [Visual Studio Code](https://code.visualstudio.com/).
- Recommended coding agent: [OpenAI Codex](https://openai.com/codex/).
- Any coding assistant can start from a standalone copy using the
  [first-widget starter](docs/FIRST_WIDGET.md). If starting from an
  `EthosLua` workspace, its `AGENTS.md` guides repository selection;
  enter this repository before running its commands.
- Keep sibling reference checkouts such as `ETHOS-Feedback-Community/` and
  `Ethos-GPS-AccuMapv1/` separate from this repo. Pull them with scoped
  commands such as `git -C ../ETHOS-Feedback-Community pull --ff-only`, and
  treat them as read-only evidence unless you are explicitly updating them.
- Before copying reference code, docs, or assets into this repo, verify the
  upstream license and record attribution.
- Format Lua: VS Code task `Format Lua (stylua)` or run `stylua --config-path tools/config/stylua.toml scripts`
- Build package: VS Code task `Build Ethos Install ZIP`
- Lua parse check:

```powershell
luac -p scripts/SensorList/main.lua
```

### Running the Python tests

- Select a Python 3.9+ interpreter in VS Code via `Python: Select Interpreter` (or ensure `python` resolves to your preferred interpreter in terminal).
- Install test dependencies once per interpreter: `python -m pip install -r requirements/dev.txt`.
- Run all repo-level and script-local tests: `python -m pytest -q`.
- Run one script's tests directly, for example: `python -m pytest scripts/SensorList/tests -q`.
- Add script-owned Python wrappers and Lua harnesses under `scripts/{ProjectName}/tests/`; keep generic tooling contracts under the root `tests/` folder.
- VS Code Test Explorer coverage runs require `pytest-cov`, which is included in `requirements/dev.txt`.

### Lua Coverage In VS Code

- Install the recommended VS Code extension: `Coverage Gutters`.
- Install Lua coverage tools with LuaRocks:
  - `luarocks install luacov`
  - `luarocks install luacov-reporter-lcov`
- Coverage output can be generated with `lua -lluacov scripts/SensorList/tests/lua/test_sensorlist.lua` followed by `luacov -r lcov`.
- The workspace is configured to let Coverage Gutters display `coverage/lua/luacov.report.out` when that report exists.

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
- Follow [Prepare a public ZIP](docs/DEVELOPMENT.md#prepare-a-public-zip) for a repo bundle or an individual script. The repo release attaches only `sloppy-ethos_scripts.zip` (SensorList and ethos_events). Personal builds can include local maps; the public prep command uses a committed snapshot and records the ZIP's SHA-256.
- Main-branch workflow, branch naming conventions, version bump timing, optional `-rc.N`, and release-prep flow:
  [Development policy](docs/DEVELOPMENT.md#main-branch-release-and-versioning-policy).
- Published release notes and install assets: [GitHub Releases](https://github.com/doppleganger53/sloppy-ethos/releases).

## Collaboration

- [Suggest an idea, report a radio test, or share an implementation](CONTRIBUTING.md#ways-to-contribute).
- Use `.github/PULL_REQUEST_TEMPLATE.md` when opening PRs.
- See `CODE_OF_CONDUCT.md` for collaboration expectations.
- See `SECURITY.md` for responsible vulnerability reporting.
- Follow [AGENTS.md](AGENTS.md) for canonical operational policy and the
  [validation matrix](AGENTS.md#validation-matrix).
- Consult the [current-state index](deslopification/memory/CURRENT_STATE.md)
  only when prior decisions or unresolved work matter; search the linked
  catalog for relevant notes.
- Follow [documentation and memory sync](AGENTS.md#documentation-and-memory-sync)
  for durable decisions, reusable findings, and handoffs.
- Parent-workspace policy changes live outside this repo; repo behavior,
  validation, release, and memory work still happens inside `sloppy-ethos/`.

## Troubleshooting

- `Required command 'luac' not found on PATH.`:
  install Lua tooling and verify `luac` resolves in your shell.
- `Simulator path not configured.`:
  configure `ETHOS_SIM_PATHS` in `tools/deploy.config.json` and set one default entry.
- Deploy fails with permission denied:
  close Ethos Suite, then retry deployment from a shell with write access.
- Stale simulator behavior after script changes:
  remove cached `main.luac`, reinstall widget, then reset Ethos Info errors.
- WebSimulator harness reports `missing_runtime`:
  run `python tools/sim/harness/run.py download --radio X20RS-FCC --ethos-version latest-26.1`, then retry the headless command.

## Roadmap

### sloppy-ethos

- Script idea: smart switch map that pre-populates mapped switches and identifies unused switches
- optimize agentic and human workflows in the repo

### SensorList

- Allow defining acceptable conflicts (for example, multiple values per device such as receivers or servos).

Roadmap implementation work is currently tracked in the following enhancement issues:

- [#10](https://github.com/doppleganger53/sloppy-ethos/issues/10) Acceptable conflict definition model.
