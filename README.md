# sloppy-ethos

Ethos Lua widget workspace. Current active project: `SensorList`.

## Current Status

- Widget loads in Ethos simulator and appears in fullscreen widget picker.
- Sensor discovery works in simulator for the tested model.
- Packaging flow produces Ethos-installable ZIP archives.
- Manual list scrolling is implemented via wheel/button events.

## Quick Start

1. Build package from repo root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-package.ps1 -ProjectName SensorList
```

2. Install in Ethos Suite using Lua install/import.
3. Sync/transfer to radio or simulator SD.

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
