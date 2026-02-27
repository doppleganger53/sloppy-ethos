# Memory Operating Guide

This folder is append-only session memory for repository workflow and delivery
history. The goal is fast cold-start context without losing auditability.

## Cold-Start Path (Default)

1. Read [CURRENT_STATE.md](CURRENT_STATE.md).
2. Use [CATALOG.md](CATALOG.md) to select only task-relevant historical notes.
3. Read domain notes (for example,
   [notes/domain-note/lua-ethos/SensorList.md](notes/domain-note/lua-ethos/SensorList.md))
   only when the task touches that area.
4. For release/history context by month, read summary rollups (for example,
   [notes/monthly-summary/memory-ops/SUMMARY_2026-02.md](notes/monthly-summary/memory-ops/SUMMARY_2026-02.md)).

## File Roles

- `CURRENT_STATE.md`:
  high-signal current decisions and active workflow state.
- `CATALOG.md`:
  note-artifact index (`notes/**`) with `Date`/`Category`/`Focus`, plus an
  auto-generated recent high-signal shortlist and a static control-files block.
- `notes/{category}/{focus}/...`:
  canonical location for note artifacts, including
  `SESSION_NOTES_*.md`, `HANDOFF_*.md`, `SESSION_RESTART_*.md`,
  `SUMMARY_YYYY-MM.md`, and domain notes.
  each focus folder should include a `.desc` file with a brief focus summary
  used by `CATALOG.md` snapshot output.
- `temp/`:
  temporary analysis artifacts that are intentionally excluded from catalog
  indexing.
- root index/control files (`README.md`, `CURRENT_STATE.md`, `CATALOG.md`,
  `SESSION_NOTE_TEMPLATE.md`):
  stable entrypoint and generation controls.
- `notes/monthly-summary/memory-ops/SUMMARY_YYYY-MM.md`:
  compact monthly rollups used as replacement summaries.
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
   - Place it under `notes/{category}/{focus}/`.
   - Prefer a specific focus (for example, `build-tooling`, `docs-process`,
     `issue-lifecycle`, `repo`, `lua-ethos`) over `general`.
   - Use `lua-ethos` for Ethos Lua script/widget/tool notes (for example,
     SensorList, ethos_events, and similar future script projects).
   - If creating a new focus folder, add a non-empty `.desc` file in that
     folder.
2. Regenerate `CATALOG.md`:
   - `python tools/update_memory_catalog.py`
3. If the change alters durable workflow/behavior, update `CURRENT_STATE.md`.
4. If the change affects monthly release/process narrative, update the active
   `notes/monthly-summary/memory-ops/SUMMARY_YYYY-MM.md` rollup.
