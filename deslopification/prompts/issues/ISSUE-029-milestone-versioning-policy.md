# Prompt: Implement Issue #29 (Milestone Branch + Versioning Policy)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/29`
- Title: `[Enhancement] Define milestone branch and versioning workflow policy`
- Labels: `enhancement`, `workflow`
- Snapshot state: open on `2026-02-26`
- Target branch (default): `enhancements` (or as user-directed for current workflow)

## Mission

Create and integrate a lightweight, explicit policy for milestone branching,
version increments, prerelease usage, and release-prep flow.

## Required Context

- `AGENTS.md`
- `README.md`
- `docs/DEVELOPMENT.md`
- `CONTRIBUTING.md`
- `CHANGELOG.md`
- `VERSION`
- `scripts/SensorList/VERSION`
- `scripts/ethos_events/VERSION`
- `deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md`

## Branch/Worktree Gate (Required Before Editing)

1. Confirm target branch and current branch:
   - `git branch --show-current`
   - `git status --short --branch`
2. If branch mismatch or dirty worktree is present, stop and confirm stash/commit/switch strategy.
3. After switching branches, sync before editing:
   - `git pull --ff-only origin {target-branch}`

## Scope

- In scope:
  - Document issue-branch vs milestone-integration-branch rules.
  - Define exactly when to bump root `VERSION` and script `VERSION` files.
  - Define optional prerelease (`-rc.N`) usage boundaries.
  - Define release-prep branch/PR/tag flow.
  - Cross-link policy in contributor-facing docs.
- Out of scope:
  - Immediate automation overhaul.
  - Changing current packaging artifact naming guardrails.
  - Backfilling historical tags/version files.

## External Best-Practice Anchors

- GitHub milestones:
  `https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/about-milestones`
- GitHub linked-closing workflow:
  `https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue`
- GitHub release management:
  `https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository`
- Semantic Versioning:
  `https://semver.org/`
- Keep a Changelog:
  `https://keepachangelog.com/en/1.1.0/`

## Suggested Execution Plan

1. Draft policy text in `docs/DEVELOPMENT.md` with concise rules + exceptions.
2. Add a short summary/reference in `README.md`.
3. Verify policy alignment with existing `AGENTS.md` release/version guardrails.
4. Align release-preflight/tag/release checklist wording with:
   - `deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md`
5. Update any affected docs command references if command wording changes.

## Validation (Required)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`

## Done Criteria

1. Policy is explicit, lightweight, and reproducible.
2. Version bump timing is unambiguous for root and script versions.
3. Milestone branch usage rules and prerelease rules are clear.
4. Release-preflight/tag/release checks are explicitly defined and consistent with the release template.
5. Required validation passes and session notes are recorded.
