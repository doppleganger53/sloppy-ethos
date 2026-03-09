# Session Notes 2026-03-07 - Issue #58 SensorList Value Polling

## Note Placement

- Artifact: `session`
- Scope: `sensorlist`
- Concern: `implementation`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Added periodic SensorList value refresh during `wakeup()` when `Display Value` is enabled, targeting a 0.2s interval without introducing a separate sensor-read path.
- Reused the existing refresh pipeline so value changes continue to flow through signature detection, sort preservation, conflict grouping, and redraw invalidation.
- Added Lua unit coverage for the 5 Hz polling path and for the disabled path when `Display Value` is off.
- Updated SensorList behavior docs to reflect that value polling now occurs while the value column is enabled.
- Files touched:
  - `scripts/SensorList/main.lua`
  - `tests/lua/test_sensorlist.lua`
  - `scripts/SensorList/README.md`
  - `docs/SensorList/SENSORLIST_ARCHITECTURE.md`

## Why

- Root cause or objective:
  - Issue #58 reported that SensorList value cells were stale because the widget only refreshed on `create()` and manual long-press, even when the value column was visible.
- Scope guardrails:
  - Kept staged sensor discovery and manual full-rescan behavior intact instead of replacing them with continuous deep scans.

## Validation run(s)

- `luac -p scripts/SensorList/main.lua`
  - result: pass
- `python -m pytest tests/test_sensorlist_widget.py -q`
  - result: pass (`6 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`91 passed, 14 skipped`)

## Follow-up items

- Validate the 5 Hz value updates on the physical X20RS runtime, since simulator and radio timing behavior can differ.

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: script behavior changed, but no durable repository workflow/policy baseline changed
