# Session Notes 2026-03-02 - SensorList Staged Scan Budget Fix (Refresh Flow Extract)

## Note Placement

- Artifact: `session`
- Scope: `sensorlist`
- Concern: `implementation`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Captured the SensorList-specific behavior change from the staged scan work: long-press refresh now queues a new staged scan instead of forcing an inline rescan in the touch callback.
- The reusable Ethos callback-budget pattern remains documented under `ethos-platform`.

## Why

- Root cause or objective:
  - Preserve the widget UX change separately from the broader Ethos instruction-budget guidance.
- Scope guardrails:
  - No additional code change beyond the original issue work.

## Validation run(s)

- `luac -p scripts/SensorList/main.lua`
  - result: pass (historical)
- `python -m pytest tests/test_sensorlist_widget.py -q`
  - result: pass (historical)

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: companion extract added for retrieval precision only.

## Related Notes

- Reusable Ethos companion note: `notes/session/ethos-platform/SESSION_NOTES_2026-03-02_ISSUE_48_SENSORLIST_STAGED_SCAN_BUDGET_FIX.md`
