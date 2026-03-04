# SensorList Operating Notes

## Note Placement

- Artifact: `reference`
- Scope: `sensorlist`
- Concern: `implementation`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

Script-specific guidance for `scripts/SensorList`.

## Behavior Constraints

- Do not filter out duplicate sensor names.
- Keep list rendering deterministic and readable.
- Preserve SensorList-specific sort and conflict semantics, including visible `SubID` handling where applicable.
- Keep touch scrolling stable once the chosen SensorList event mapping is validated for the target layout.

## Validation Checklist

From repo root:

- `luac -p scripts/SensorList/main.lua`
- `python tools/build.py --project SensorList --dist`
- `python tools/build.py --project SensorList --deploy` (when simulator path is configured)

## Notes

- Deploy path resolves from `tools/deploy.config.json` or `ETHOS_SIM_PATH`.
- `tools/deploy.config.json` is local-only and gitignored.
- Reusable Ethos runtime and API guidance now lives in `notes/reference/ethos-platform/EthosPlatform.md`.
