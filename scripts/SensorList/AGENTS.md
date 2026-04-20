# AGENTS.md (SensorList)

This file augments root `AGENTS.md` for work under `scripts/SensorList/`.

## Scope

- Applies to `scripts/SensorList/**`.
- Keep SensorList behavior deterministic and readability-first.

## Implementation guidance

- Keep callback handling defensive (`create`, `paint`, `wakeup`, `configure`, `read`, `write`, and event paths).
- Prefer incremental performance changes; avoid expensive per-frame scans.
- Preserve input/navigation compatibility unless the task explicitly changes it.

## Validation additions

- Required Lua parse check:
  - `luac -p scripts/SensorList/main.lua`
- Required SensorList test target:
  - `python -m pytest tests/test_sensorlist_widget.py -q`
- For cross-cutting changes, defer to root matrix and run:
  - `python -m pytest -q`
