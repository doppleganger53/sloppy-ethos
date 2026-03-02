# Memory Operating Guide

This folder is append-only session memory for repository workflow and delivery
history. The goal is fast cold-start context without losing auditability.

## Cold-Start Path (Default)

1. Read [CURRENT_STATE.md](CURRENT_STATE.md).
2. Use [CATALOG.md](CATALOG.md) to select only task-relevant historical notes.
3. Read reusable Ethos references first when the task touches Ethos runtime
   behavior (for example,
   [notes/reference/ethos-platform/EthosPlatform.md](notes/reference/ethos-platform/EthosPlatform.md)).
4. Read script-specific references only when the task narrows to that script
   (for example,
   [notes/reference/sensorlist/SensorList.md](notes/reference/sensorlist/SensorList.md)).
5. For compact historical context by week, read summary rollups (for example,
   [notes/summary/memory/SUMMARY_2026-02-21_to_2026-02-27.md](notes/summary/memory/SUMMARY_2026-02-21_to_2026-02-27.md)).

## File Roles

- `CURRENT_STATE.md`:
  high-signal current decisions and active workflow state.
- `CATALOG.md`:
  note-artifact index (`notes/**`) with `Date`/`Artifact`/`Scope`/`Concern`,
  plus auto-generated recent high-signal and Ethos-platform shortlists and a
  static control-files block.
- `notes/{artifact}/{scope}/...`:
  canonical location for note artifacts, including
  `SESSION_NOTES_*.md`, `HANDOFF_*.md`, `SESSION_RESTART_*.md`,
  `SUMMARY_YYYY-MM-DD_to_YYYY-MM-DD.md`, and domain notes.
  each scope folder should include a `.desc` file with a brief scope summary
  used by `CATALOG.md` snapshot output.
- `temp/`:
  temporary analysis artifacts that are intentionally excluded from catalog
  indexing.
- root index/control files (`README.md`, `CURRENT_STATE.md`, `CATALOG.md`,
  `SESSION_NOTE_TEMPLATE.md`):
  stable entrypoint and generation controls.
- `notes/summary/memory/SUMMARY_YYYY-MM-DD_to_YYYY-MM-DD.md`:
  compact weekly rollups used as replacement summaries.
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
   - Place it under `notes/{artifact}/{scope}/`.
   - Choose the most reusable scope that fits the note.
   - Use `ethos-platform` for reusable Ethos runtime/API/simulator knowledge,
     even when it was discovered during work on one script.
   - Use script scopes such as `sensorlist` or `ethos-events` only for
     script-local behavior, architecture, or release history.
   - If a session yields both reusable Ethos knowledge and script-local detail,
     prefer two short notes rather than one mixed note.
   - If creating a new scope folder, add a non-empty `.desc` file in that
     folder.
2. Regenerate `CATALOG.md`:
   - `python tools/update_memory_catalog.py`
3. If the change alters durable workflow/behavior, update `CURRENT_STATE.md`.
4. If the change affects the compact weekly process narrative, update the
   relevant `notes/summary/memory/SUMMARY_YYYY-MM-DD_to_YYYY-MM-DD.md`
   rollup.
