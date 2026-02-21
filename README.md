# sloppy-ethos

Ethos Lua widget workspace. Current active project: `SensorList`.

## Current Status

- Widget loads in Ethos simulator and appears in fullscreen widget picker.
- Sensor discovery works in simulator for the tested model.
- Packaging flow produces Ethos-installable ZIP archives.
- Manual list scrolling is implemented via wheel/button events.

## Quick Start

1. Build or deploy from repo root:

```powershell
python tools/build.py --project SensorList --dist
python tools/build.py --project SensorList --deploy
```

2. Install the dist ZIP inside Ethos Suite for radio deployment.
3. Use the PowerShell helpers (`tools/build-package.ps1`, `tools/deploy-ethos-sim.ps1`) as Windows-only fallbacks.
4. Configure `tools/deploy.config.json` (copy the example) or set `ETHOS_SIM_PATH` before running `--deploy`.

## Project Layout

- `src/scripts/SensorList/main.lua`: widget implementation
- `src/scripts/SensorList/README.md`: widget-focused usage notes
- `tools/build-package.ps1`: syntax-check + packaging script
- `tools/deploy-ethos-sim.ps1`: simulator deploy helper (placeholder)
- `prompts/SensorWidgetTemplate.md`: original implementation prompt
- `docs/`: development notes and handoff documents

## Development

- Format Lua: VS Code task `Format Lua (stylua)` or run `stylua src`
- Build package: VS Code task `Build Ethos Install ZIP`
- Lua parse check:

```powershell
luac -p src/scripts/SensorList/main.lua
```

## Collaboration

- See `CONTRIBUTING.md` for contribution flow.
- Use `.github/PULL_REQUEST_TEMPLATE.md` when opening PRs.
- See `docs/HANDOFF_2026-02-21.md` for the latest session notes.
