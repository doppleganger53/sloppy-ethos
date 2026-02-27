# Session Notes 2026-02-24 - ethos_events Remove lib Duplicate

## What changed

- Removed duplicate helper copy `src/scripts/lib/ethos_events.lua`.
- Removed now-empty `src/scripts/lib` directory.
- Updated `src/scripts/ethos_events/UPSTREAM_README.md` to reflect repo packaging path:
  - install path now documents `/SCRIPTS/ethos_events/ethos_events.lua`
  - usage snippet now loads helper from the same folder via `loadScript(...)`
- Updated `src/scripts/ethos_events/ethos_events.lua` header comments to match the self-contained folder layout.
- Updated `src/scripts/ethos_events/README.md` wording to indicate local adapted usage notes.

## Validation run(s)

- `luac -p src/scripts/SensorList/main.lua`
  - result: passed
- `luac -p src/scripts/ethos_events/main.lua`
  - result: passed
- `luac -p src/scripts/ethos_events/ethos_events.lua`
  - result: passed
- `python -m pytest tests/test_sensorlist_widget.py -q`
  - result: `6 passed`
- `python -m pytest tests/test_build_py.py -q`
  - result: `33 passed`
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `59 passed, 10 skipped`

## Follow-up

- Manually verify on physical radio that `ethos_events` still loads helper and icon correctly from `scripts/ethos_events`.
