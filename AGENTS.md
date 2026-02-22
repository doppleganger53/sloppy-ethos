# AGENTS.md

Repository-level operating notes for Codex sessions in `sloppy-ethos`.

## Startup Workflow

1. Read the latest relevant memory note in `deslopification/memory/`.
2. Check branch/worktree state:
   - `git status --short --branch`
3. Confirm active package version:
   - `VERSION`
4. Use the command index in `README.md` and `docs/DEVELOPMENT.md` before adding new workflow commands.

## Scope Discipline

- Keep changes focused on the requested task.
- Avoid unrelated cleanup or broad refactors.
- Preserve existing behavior unless the task explicitly requests behavior change.

## Documentation and Memory Sync

- Update user/developer docs in the same session when workflow or behavior changes.
- Record meaningful changes in `deslopification/memory/` with:
  - what changed
  - validation run
  - follow-up items

## Release and Versioning Guardrails

- `VERSION` is the package version source of truth.
- Dist ZIP naming must remain `dist/{ProjectName}-{version}.zip`.
- Keep release procedure consistent with repository documentation.

## Script-Specific Notes

- SensorList-specific operating guidance lives in:
  - `deslopification/memory/SensorList.md`
