# SensorList Operating Notes

Script-specific guidance for `src/scripts/SensorList`.

## Behavior Constraints

- Do not filter out duplicate sensor names.
- Keep list rendering deterministic and readable.
- Maintain low-frequency polling plus periodic deep scan to avoid instruction budget overrun.
- Keep touch scrolling stable across simulator/radio event differences.

## Validation Checklist

From repo root:

- `luac -p src/scripts/SensorList/main.lua`
- `python tools/build.py --project SensorList --dist`
- `python tools/build.py --project SensorList --deploy` (when simulator path is configured)

## Notes

- Deploy path resolves from `tools/deploy.config.json` or `ETHOS_SIM_PATH`.
- `tools/deploy.config.json` is local-only and gitignored.
