# SensorList

Ethos widget that lists configured sensors in a sortable table with:

- Name
- Physical ID
- Application ID

Conflicting sensors that share the same Physical ID are color-grouped to make
potential conflicts easier to identify.

## Install

1. Build the install ZIP from the repository root:
   `powershell -ExecutionPolicy Bypass -File tools/build-package.ps1 -ProjectName SensorList`
2. In Ethos Suite, run the Lua install/import action and select the ZIP file
   from `dist/`.
3. Transfer/sync to the radio.

Installed path on radio: `scripts/SensorList`
