# Contributing

## Workflow

1. Create a short-lived branch from the latest `main` using naming conventions:
   - `feature/{issue-number}-{short-slug}` for enhancements
   - `fix/{issue-number}-{short-slug}` for bugs
   - `docs/{issue-number}-{short-slug}` for docs/process changes
   - `chore/{issue-number}-{short-slug}` for maintenance/tooling changes
2. Keep changes focused and scoped to one concern.
3. If installable script behavior/assets changed, bump that script `scripts/{ProjectName}/VERSION` in the same PR. Do not bump version files for docs-only or workflow-only changes.
4. Do not bump root `VERSION` on issue branches; root release versioning is finalized on `release/v{VERSION}`.
5. Run local checks before opening a PR:
   - `luac -p scripts/SensorList/main.lua`
   - package build:
     `python tools/build.py --project SensorList --dist`
     or multi-script bundle build:
     `python tools/build.py --project SensorList --project ethos_events --dist`
   - docs validation (required for any documentation changes):
     `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
   - `stylua --config-path tools/config/stylua.toml scripts` (if formatting changed)
6. Open a PR into `main` using the repository PR template and include linked-closing issue keywords (for example, `Closes #29`).
7. Use the repository merge strategy policy:
   - `squash` merge for normal issue PRs (`feature/`, `fix/`, `docs/`, `chore/`).
   - `merge commit` for release-prep PRs (`release/v{VERSION}`) or lineage-sensitive PRs.
   - avoid `rebase` merge as default.
8. Do not manually close linked issues before merge; let GitHub close them from the merged PR closing keywords.
9. Follow release branch/tag flow in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#main-branch-release-and-versioning-policy) and [Release Workflow](docs/DEVELOPMENT.md#release-workflow).

## Documentation Changes

- If a change touches `README.md`, `docs/`, or other contributor-facing docs, run:
  `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- Docs updates are not complete until docs contract checks pass.
- Include the docs validation command output in the PR `Verification` checklist.
- If workflow/behavior changed, add a session note under
  `deslopification/memory/notes/{category}/{focus}/`, append it to
  `deslopification/memory/CATALOG.md`, prefer a specific focus classifier over
  `general` (use `lua-ethos` for Ethos Lua script/widget/tool notes), and update
  `deslopification/memory/CURRENT_STATE.md` when the change is durable.

## Good First Issues

- New contributors can start with items labeled
  [good first issue](https://github.com/doppleganger53/sloppy-ethos/labels/good%20first%20issue).
- If onboarding support is needed, look for
  [help wanted](https://github.com/doppleganger53/sloppy-ethos/labels/help%20wanted)
  issues.

## Bug Report Quality

- Use `.github/ISSUE_TEMPLATE/bug_report.md` when opening a bug.
- Confirm the issue has the `bug` label.
- Fill the environment section for both Ethos and non-Ethos bug types.
- For screenshots, prefer GitHub issue-upload links or `raw.githubusercontent.com` links and avoid `blob` image URLs.

## Commit Guidance

- Use concise, imperative commit messages.
- Include context in the body when behavior changes.
- Avoid mixing refactors and feature changes in the same commit when possible.

## Coding Notes

- Keep Ethos callbacks defensive (`paint`, `wakeup`, etc.) to avoid runtime crashes from unexpected callback invocation contexts.
- Prefer incremental performance changes; simulator can hit instruction limits quickly.
- Keep widget behavior deterministic and readable first, then optimize.
