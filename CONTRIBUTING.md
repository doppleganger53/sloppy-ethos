# Contributing

## Workflow

1. Create a branch from `main`.
2. Keep changes focused and scoped to one concern.
3. Run local checks before opening a PR:
   - `luac -p scripts/SensorList/main.lua`
   - package build:
     `python tools/build.py --project SensorList --dist`
   - docs validation (required for any documentation changes):
     `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
   - `stylua --config-path tools/config/stylua.toml scripts` (if formatting changed)
4. Open a PR using the repository PR template.

## Documentation Changes

- If a change touches `README.md`, `docs/`, or other contributor-facing docs, run:
  `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- Docs updates are not complete until docs contract checks pass.
- Include the docs validation command output in the PR `Verification` checklist.

## Good First Issues

- New contributors can start with items labeled
  [good first issue](https://github.com/doppleganger53/sloppy-ethos/labels/good%20first%20issue).
- If onboarding support is needed, look for
  [help wanted](https://github.com/doppleganger53/sloppy-ethos/labels/help%20wanted)
  issues.

## Commit Guidance

- Use concise, imperative commit messages.
- Include context in the body when behavior changes.
- Avoid mixing refactors and feature changes in the same commit when possible.

## Coding Notes

- Keep Ethos callbacks defensive (`paint`, `wakeup`, etc.) to avoid runtime crashes from unexpected callback invocation contexts.
- Prefer incremental performance changes; simulator can hit instruction limits quickly.
- Keep widget behavior deterministic and readable first, then optimize.
