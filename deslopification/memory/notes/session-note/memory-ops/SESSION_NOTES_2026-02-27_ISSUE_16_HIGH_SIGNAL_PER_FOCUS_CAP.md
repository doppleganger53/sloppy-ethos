# Session Notes 2026-02-27 - Issue #16 High-Signal Per-Focus Cap

## Note Placement

- Category: `session-note`
- Focus: `memory-ops`
- Focus guidance:
  - prefer a specific focus classifier and avoid `general` when possible
- Store this file under:
  - `deslopification/memory/notes/{category}/{focus}/`

## What changed

- Updated `tools/update_memory_catalog.py` recent high-signal selection logic:
  - added `RECENT_HIGH_SIGNAL_PER_FOCUS_LIMIT = 3`
  - selection now applies per-focus cap first, then global cap (`RECENT_HIGH_SIGNAL_LIMIT = 12`)
  - sort key now uses `(date, name, rel_path)` for deterministic tie behavior
- Updated recent high-signal selection prose in catalog rendering:
  - generated focus list from `HIGH_SIGNAL_FOCUS`
  - criteria text now includes both per-focus and global cap rules
- Expanded `tests/test_memory_catalog_sync.py` with regression coverage for:
  - per-focus cap enforcement
  - global cap enforcement after per-focus selection
  - deterministic ordering under tie conditions
  - filtering of non-matching category/date/focus rows
- Regenerated `deslopification/memory/CATALOG.md`.

## Why

- Root cause or objective:
  - prevent recency crowd-out where a single active focus dominates the recent high-signal section
  - remove drift risk between configured focus constants and human-readable criteria text

## Validation run(s)

- `python -m pytest tests/test_memory_catalog_sync.py -q`
  - result: pass (20 passed)
- `python -m pytest tests/test_build_py.py -q`
  - result: pass (56 passed)

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: no change to priorities, branch context, or open workflow decisions
