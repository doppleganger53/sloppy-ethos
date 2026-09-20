# Memory Operating Guide

This folder preserves project decisions and delivery history. The canonical
[memory policy](../../AGENTS.md#documentation-and-memory-sync) defines when to
read or add notes; routine tasks do not require a memory preload or new note.

## Retrieve Relevant Context

1. If prior decisions or unresolved work matter, use [CURRENT_STATE.md](CURRENT_STATE.md)
   for a topic pointer, or search [CATALOG.md](CATALOG.md) by topic, Scope, and Concern.
2. Read only matching current references or the latest relevant note. Check
   changing facts against the source/version/runtime being used.
3. For Ethos runtime behavior, consult the concise
   [platform reference](notes/reference/ethos-platform/EthosPlatform.md);
   read a [script reference](notes/reference/sensorlist/SensorList.md) only
   when that script is in scope.
4. Read weekly summaries or archived snapshots only when historical context is
   needed. They preserve prior evidence, not current policy.

## File Roles

- `CURRENT_STATE.md`:
  short topic index pointing to current sources and relevant historical evidence.
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
- Update `CURRENT_STATE.md` only when a durable retrieval pointer changes.

## Maintenance Checklist

When adding a new memory note:

1. Add the note using `SESSION_NOTE_TEMPLATE.md`.
   - Place it under `notes/{artifact}/{scope}/`.
   - Choose the most reusable scope that fits the note.
   - Use `ethos-platform` for reusable Ethos runtime/API/simulator knowledge,
     even when it was discovered during work on one script.
   - Use script scopes such as `boundrymap`, `sensorlist`, or `ethos-events`
     only for script-local behavior, architecture, or release history.
   - If a session yields both reusable Ethos knowledge and script-local detail,
     prefer two short notes rather than one mixed note.
   - If creating a new scope folder, add a non-empty `.desc` file in that
     folder.
2. Regenerate `CATALOG.md`:
   - `python tools/update_memory_catalog.py`
3. If the change needs a different retrieval pointer, update `CURRENT_STATE.md`.
4. If the change affects the compact weekly process narrative, update the
   relevant `notes/summary/memory/SUMMARY_YYYY-MM-DD_to_YYYY-MM-DD.md`
   rollup.
