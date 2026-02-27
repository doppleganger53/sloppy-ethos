# Memory Operating Guide

This folder is append-only session memory for repository workflow and delivery
history. The goal is fast cold-start context without losing auditability.

## Cold-Start Path (Default)

1. Read [CURRENT_STATE.md](CURRENT_STATE.md).
2. Use [CATALOG.md](CATALOG.md) to select only task-relevant historical notes.
3. Read domain notes (for example, [SensorList.md](SensorList.md)) only when the
   task touches that area.
4. For release/history context by month, read summary rollups (for example,
   [SUMMARY_2026-02.md](SUMMARY_2026-02.md)).

## File Roles

- `CURRENT_STATE.md`:
  high-signal current decisions and active workflow state.
- `CATALOG.md`:
  full index of all notes/handoffs in this folder with date/type/title.
- `SUMMARY_YYYY-MM.md`:
  replacement summary rollup by month for historical traceability.
- `SESSION_NOTES_*.md`, `HANDOFF_*.md`, `SESSION_RESTART_*.md`:
  immutable detail records.
- `SESSION_NOTE_TEMPLATE.md`:
  canonical structure for new notes.

## Compaction Policy

- Do not delete historical notes without a replacement summary file.
- Prefer adding/maintaining summary+index artifacts instead of rewriting old notes.
- Keep new session notes concise and scoped to one coherent change-set.
- Update `CURRENT_STATE.md` only for durable behavior/workflow decisions.

## Maintenance Checklist

When adding a new memory note:

1. Add the note using `SESSION_NOTE_TEMPLATE.md`.
2. Add one row to `CATALOG.md`.
3. If the change alters durable workflow/behavior, update `CURRENT_STATE.md`.
4. If the change affects monthly release/process narrative, update the active
   `SUMMARY_YYYY-MM.md` rollup.
