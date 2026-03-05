# Session Notes 2026-03-02 - Issue #48 SensorList Staged Scan Budget Fix

## Note Placement

- Artifact: `session`
- Scope: `ethos-platform`
- Concern: `implementation`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Reworked SensorList `system.getSource()` discovery into a staged scan model so category probing stays cheap and large known categories expand over later `wakeup()` ticks.
- Adjusted manual long-press refresh to queue a fresh staged scan instead of forcing a full inline rescan in the touch callback.
- Updated SensorList docs to describe background expansion and the queued long-press refresh behavior.
- Files touched:
  - `scripts/SensorList/main.lua`
  - `scripts/SensorList/README.md`
  - `docs/SensorList/SENSORLIST_ARCHITECTURE.md`

## Why

- Root cause or objective:
  - Issue #48 passed feature verification, but large sensor lists and long-press refreshes were still tripping the Ethos callback instruction budget on device.
- Scope guardrails:
  - Preserved the final SubID display/sort/conflict behavior while changing only the internal discovery and refresh scheduling strategy.

## Validation run(s)

- `luac -p scripts/SensorList/main.lua`
  - result: pass
- `python -m pytest tests/test_sensorlist_widget.py -q`
  - result: pass (`6 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass

## Follow-up items

- Manual verification on the physical radio confirmed normal rendering; monitor whether staged refresh remains responsive with very large telemetry configurations.

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: this is an issue-specific widget runtime behavior fix and does not alter repository-wide workflow policy.

## Related Notes

- Companion note: `notes/session/sensorlist/SESSION_NOTES_2026-03-02_ISSUE_48_SENSORLIST_STAGED_SCAN_BUDGET_FIX_EXTRACT_SENSORLIST_REFRESH_FLOW.md`
