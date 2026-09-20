# Contributing

## Repository Policy

Read [AGENTS.md](AGENTS.md) for repository goals, operational policy, and
change-specific validation. This guide covers contributor entry points and
links to that policy rather than maintaining another copy.

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
6. Run the checks for touched files in the
   [canonical validation matrix](AGENTS.md#validation-matrix), including
   packaging or simulator checks when applicable.

7. Open a PR into `main` using the repository PR template and include linked-closing issue keywords (for example, `Closes #29`).
8. Follow the [PR merge strategy](AGENTS.md#pr-merge-strategy).

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

- Follow the [validation matrix](AGENTS.md#validation-matrix); documentation
  changes are complete when the relevant checks pass. Include the results in
  the PR `Verification` checklist.
- Follow [release guardrails](AGENTS.md#release-and-versioning-guardrails) for
  published download links; do not advertise unreleased script artifacts.
- Put script-owned tests under `scripts/{ProjectName}/tests/`; keep generic
  repository tooling and documentation contracts under root `tests/`.
- Follow [documentation and memory sync](AGENTS.md#documentation-and-memory-sync)
  for durable findings or handoff notes.

## Bug Report Quality

Follow [bug issue hygiene](AGENTS.md#bug-issue-hygiene) for templates, labels,
runtime details, and stable screenshot links.

## Commit Guidance

- Use concise, imperative commit messages.
- Include context in the body when behavior changes.
- Avoid mixing refactors and feature changes in the same commit when possible.

## Coding Notes

- Keep Ethos callbacks defensive (`paint`, `wakeup`, etc.) to avoid runtime crashes from unexpected callback invocation contexts.
- Prefer incremental performance changes; simulator can hit instruction limits quickly.
- Keep widget behavior deterministic and readable first, then optimize.
