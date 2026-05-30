# Contributing

## Repository Goals And Workflow Mutability

- Repository goals:
  - Build practical Ethos utilities and scripts.
  - Keep Lua contribution workflow approachable for non-experts.
  - Prioritize repeatable, low-maintenance workflows that reduce process overhead.
- Workflow/process guidance is intentionally mutable:
  - Propose workflow changes via issue-linked updates.
  - Keep policy wording synchronized across `AGENTS.md`, `README.md`, `docs/DEVELOPMENT.md`, and this file when process expectations change.

## Workflow

1. If you start from the parent `EthosLua` workspace, enter `sloppy-ethos/`
   before running repository commands. The parent `AGENTS.MD` defines workspace
   boundaries; this repo's `AGENTS.md` defines the contribution and agent
   workflow once inside the checkout.
2. Create a short-lived branch from the latest `main` using naming conventions:
   - `feature/{issue-number}-{short-slug}` for enhancements
   - `fix/{issue-number}-{short-slug}` for bugs
   - `docs/{issue-number}-{short-slug}` for docs/process changes
   - `chore/{issue-number}-{short-slug}` for maintenance/tooling changes
3. Keep changes focused and scoped to one concern.
4. If installable script behavior/assets changed, bump that script `scripts/{ProjectName}/VERSION` in the same PR. Do not bump version files for docs-only or workflow-only changes.
   If the script needs installable files outside `/scripts` or optional local assets under its own script folder, declare them in a project-local build manifest so `tools/build.py` can package, deploy, and clean them consistently.
   Keep user-specific location assets local. For BoundryMap, place private flying-site maps under the ignored `scripts/BoundryMap/assets/maps/` folder and let the generic `assets` manifest entries package them.
5. Do not bump root `VERSION` on issue branches; root release versioning is finalized on `release/v{VERSION}`.
6. Run local checks before opening a PR:
   - `luac -p scripts/SensorList/main.lua`
   - package build:
     `python tools/build.py --project SensorList --dist`
     or multi-script bundle build:
     `python tools/build.py --project SensorList --project ethos_events --dist`
   - docs validation (required for any documentation changes):
     `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
   - full repo and script-local tests when touching test layout, shared tooling, or cross-script behavior:
     `python -m pytest -q`
   - one script's local tests when touching that script's behavior:
     `python -m pytest scripts/{ProjectName}/tests -q`
   - `stylua --config-path tools/config/stylua.toml scripts` (if formatting changed)
7. Open a PR into `main` using the repository PR template and include linked-closing issue keywords (for example, `Closes #29`).
8. Use the repository merge strategy policy:
   - `squash` merge for normal issue PRs (`feature/`, `fix/`, `docs/`, `chore/`).
   - `merge commit` for release-prep PRs (`release/v{VERSION}`) or lineage-sensitive PRs.
   - avoid `rebase` merge as default.
9. Do not manually close linked issues before merge; let GitHub close them from the merged PR closing keywords.
10. Follow release branch/tag flow in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#main-branch-release-and-versioning-policy) and [Release Workflow](docs/DEVELOPMENT.md#release-workflow).

## Reference Checkouts

- Sibling directories in the parent `EthosLua` workspace are references, not
  part of this repo.
- Update reference repos only with commands scoped to that checkout, for
  example `git -C ../ETHOS-Feedback-Community pull --ff-only`.
- Treat reference projects as read-only evidence unless an issue or user request
  explicitly calls for updating them.
- Verify license and attribution before copying reference code, docs, or assets
  into `sloppy-ethos`.

## Documentation Changes

- If a change touches `README.md`, `docs/`, or other contributor-facing docs, run:
  `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- Docs updates are not complete until docs contract checks pass.
- Keep `README.md` 'Download Latest Script Releases' links aligned only with currently published script release ZIP assets.
- Do not add unreleased scripts to that README section just because `scripts/{ProjectName}/VERSION` exists on a branch.
- Include the docs validation command output in the PR `Verification` checklist.
- Put script-owned tests under `scripts/{ProjectName}/tests/`; keep generic repository tooling and documentation contracts under root `tests/`.
- If workflow/behavior changed, add a session note under
  `deslopification/memory/notes/{artifact}/{scope}/`, regenerate
  `deslopification/memory/CATALOG.md`, use explicit `Artifact`, `Scope`, and
  `Concern` metadata, prefer `ethos-platform` for reusable Ethos knowledge, and
  update `deslopification/memory/CURRENT_STATE.md` when the change is durable.

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
