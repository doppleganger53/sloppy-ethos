# Development Notes

## Prerequisites

- PowerShell
- Lua tooling on PATH (`lua`, `luac`)
- Optional: `stylua` for formatting

## Core Commands

- Syntax check:
  `luac -p src/scripts/SensorList/main.lua`
- Build package:
  `powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-package.ps1 -ProjectName SensorList`

## Packaging Behavior

`tools/build-package.ps1`:

- validates Lua syntax using `luac -p`
- assembles `scripts/SensorList/...` package layout
- writes ZIP to `dist/` with timestamped name

## Simulator Tips

- If stale behavior appears after script updates, remove cached `main.luac` for the script path and reinstall.
- Ethos error entries persist in Info until reset; always reset after validating a fix.
