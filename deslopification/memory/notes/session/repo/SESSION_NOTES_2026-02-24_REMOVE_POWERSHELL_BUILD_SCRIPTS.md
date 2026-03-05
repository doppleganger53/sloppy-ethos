# Session Notes 2026-02-24 - Remove PowerShell Build Scripts

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `build`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`


## What changed

- Removed PowerShell build/deploy scripts:
  - `tools/build-package.ps1`
  - `tools/deploy-ethos-sim.ps1`
- Replaced PowerShell TODO issue bootstrap script with Python:
  - removed `tools/create-todo-issues.ps1`
  - added `tools/create_todo_issues.py` (supports `--repo` and `--dry-run`)
- Expanded `tools/build.py` to cover previous packaging capabilities:
  - syntax-checks all Lua files under `src/scripts/{project}` (not only `main.lua`)
  - creates a package `README.md` automatically when missing
  - added `--out-dir` option for custom ZIP output directory
- Updated tooling tests in `tests/test_build_py.py` for:
  - recursive Lua checks
  - package README helper behavior
  - `--out-dir` argument handling
- Updated documentation and command references to Python-only build/deploy flow:
  - `README.md`
  - `docs/DEVELOPMENT.md`
  - `docs/REPOSITORY_LAYOUT.md`
  - `src/scripts/SensorList/README.md`
  - `CONTRIBUTING.md`
  - `tests/test_docs_commands.py`

## Validation run(s)

- `python -m pytest tests/test_build_py.py -q`
  - result: `37 passed`
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `57 passed, 9 skipped`

## Follow-up

- If desired, add a short section to README documenting `tools/create_todo_issues.py --dry-run` for maintainers.