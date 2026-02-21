# Contributing

## Workflow

1. Create a branch from `main`.
2. Keep changes focused and scoped to one concern.
3. Run local checks before opening a PR:
   - `luac -p src/scripts/SensorList/main.lua`
   - `stylua src` (if formatting changed)
   - package build:
     `powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-package.ps1 -ProjectName SensorList`
4. Open a PR using the repository PR template.

## Commit Guidance

- Use concise, imperative commit messages.
- Include context in the body when behavior changes.
- Avoid mixing refactors and feature changes in the same commit when possible.

## Coding Notes

- Keep Ethos callbacks defensive (`paint`, `wakeup`, etc.) to avoid runtime crashes from unexpected callback invocation contexts.
- Prefer incremental performance changes; simulator can hit instruction limits quickly.
- Keep widget behavior deterministic and readable first, then optimize.
