# Session Notes 2026-02-24 - scripts Root Refactor

## What changed

- Moved Lua project folders from `src/scripts/` to root `scripts/`:
  - `src/scripts/SensorList` -> `scripts/SensorList`
  - `src/scripts/ethos_events` -> `scripts/ethos_events`
- Updated build tooling to use new source path:
  - `tools/build.py` now resolves projects from `scripts/{ProjectName}`
- Updated tests for new paths:
  - `tests/test_build_py.py`
  - `tests/lua/test_sensorlist.lua`
  - `tests/test_docs_commands.py`
- Updated repository docs and contributor instructions to `scripts/` paths:
  - `README.md`
  - `docs/DEVELOPMENT.md`
  - `docs/REPOSITORY_LAYOUT.md`
  - `CONTRIBUTING.md`
  - `AGENTS.md`
  - `scripts/ethos_events/README.md`
- Updated local tooling/config to point at root scripts:
  - `.vscode/tasks.json`
  - `.luarc.json`
  - `stylua.toml`
- Updated prompt/reference artifacts that still pointed to `src/scripts`:
  - `deslopification/prompts/*.md` and template references
  - `tools/create_todo_issues.py` backlog text

## Validation run(s)

- `luac -p scripts/SensorList/main.lua`
  - result: passed
- `luac -p scripts/ethos_events/main.lua`
  - result: passed
- `luac -p scripts/ethos_events/ethos_events.lua`
  - result: passed
- `python -m pytest tests/test_sensorlist_widget.py -q`
  - result: `6 passed`
- `python -m pytest tests/test_build_py.py -q`
  - result: `37 passed`
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `57 passed, 9 skipped`
- `python -m pytest -q`
  - result: `100 passed, 9 skipped`

## Follow-up

- Update `CHANGELOG.md` path references in a release-focused pass if you want historical entries to use current repo layout terms.
