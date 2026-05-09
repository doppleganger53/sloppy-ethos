# Session Notes 2026-05-09 - Ethos 26.1 Release Docs And getSources

## Note Placement

- Artifact: `session`
- Scope: `ethos-platform`
- Concern: `implementation`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Pulled the `26.1.0-RC1` `lua-doc.zip` release asset into the local ignored
  `referenceProjects` tree for API reference.
- Confirmed the ZIP hash matched the GitHub release asset digest
  `ea3699fc597ca39371c0a64fcc82d8a60e8d301da60704cdcdb4cd36adc8561a`.
- Updated SmartMapper's probe to use the documented
  `system.getSources(categoryNumber)` call shape.
- Added all documented source category constant names to the probe so runtime
  reports capture their exact numeric values.
- Files touched:
  - `scripts/SmartMapper/main.lua`
  - `scripts/SmartMapper/tests/lua/test_smartmapper.lua`
  - `scripts/SmartMapper/README.md`
  - `docs/ETHOS_26_1_COMPATIBILITY.md`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - The first SmartMapper runtime report showed `system.getSources(...)`
    existed but failed because the probe passed a table argument. The release
    documentation shows `system.getSources(CATEGORY_TELEMETRY_SENSOR)`, so the
    probe needed to call with a category number.
- Scope guardrails:
  - The downloaded release documentation is local reference material and remains
    ignored under `referenceProjects/`.

## Validation run(s)

- `luac -p scripts/SmartMapper/main.lua`
  - result: pass
- `python -m pytest scripts/SmartMapper/tests -q`
  - result: pass (`3 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`170 passed, 21 skipped`)
- `python tools/build.py --project SmartMapper --dist`
  - result: pass; packaged `dist/SmartMapper-0.1.0.zip`

## Follow-up items

- Rerun `SmartMapper Probe` in the Ethos `26.1` simulator or target radio and
  capture the updated category constants plus `system.getSources(category)`
  results.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
