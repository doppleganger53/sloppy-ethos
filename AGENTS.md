# AGENTS.md

Repository-level operating policy for Codex sessions in `sloppy-ethos`.

## Priority And Scope

- These rules are repository-wide and apply to every task in this workspace.
- Keep changes scoped to the user request. Do not perform unrelated cleanup or broad refactors.
- Preserve existing behavior unless the task explicitly requests behavior changes.

## Required Startup Workflow

1. Read the latest relevant note in `deslopification/memory/`.
2. Check branch/worktree state: `git status --short --branch`.
3. Confirm active package version from `VERSION`.
4. Before introducing or changing workflow commands, cross-check command docs in:
   - `README.md`
   - `docs/DEVELOPMENT.md`

## Validation Policy

- Validation is mandatory for any change.
- Run the minimum test/check commands based on the files touched:

### Validation Matrix

- Documentation changes (`README.md`, `docs/`, `CONTRIBUTING.md`, PR/issue templates):
  - `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- Lua widget behavior changes (`scripts/**/*.lua`):
  - `luac -p scripts/SensorList/main.lua`
  - `python -m pytest tests/test_sensorlist_widget.py -q`
- Build/tooling changes (`tools/`, Python build/test harness):
  - `python -m pytest tests/test_build_py.py -q`
- Broad or cross-cutting changes:
  - `python -m pytest -q`

## Timeout And Failure Handling

- If a required validation command times out or hangs:
  1. rerun once with a higher timeout;
  2. if it still fails, report the exact command and failure mode to the user;
  3. do not claim validation passed when it did not complete.

## Documentation And Memory Sync

- When workflow, behavior, or contributor process changes, update docs in the same session.
- Record meaningful work in `deslopification/memory/` using:
  - what changed
  - validation run(s)
  - follow-up items

## Release And Versioning Guardrails

- `VERSION` is the package version source of truth.
- Dist ZIP naming must remain `dist/{ProjectName}-{version}.zip`.
- Keep release steps aligned with repository documentation.

## Script-Specific Notes

- SensorList-specific operating guidance lives in:
  - `deslopification/memory/SensorList.md`

## Definition Of Done (Before Commit/Push)

- Required validation commands for touched files completed in this session.
- Workspace and `.gitignore` reviewed for environment-specific files and sensitive data risks.
- Any potential security concern (PII, PHI, secrets, unsafe config) explicitly called out to the user.
