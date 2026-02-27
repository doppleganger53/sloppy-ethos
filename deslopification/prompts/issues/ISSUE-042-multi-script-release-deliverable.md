# Prompt: Implement Issue #42 (Multi-script release deliverable)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/42`
- Title: `[Enhancement] Include multi-script bundle in repo release deliverables`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-27`
- Target branch (default): `feature/42-multi-script-release-deliverable`

## Mission

Ensure the repository release explicitly includes `dist/sloppy-ethos_scripts.zip` as a deliverable by documenting the deliverable set, calling out the build command that produces the archive, and confirming the asset is published along with the single-script ZIPs.

## Required Context

- `AGENTS.md` (release scope and validation expectations)
- `README.md` (current release summary, install instructions, and roadmap)
- `docs/DEVELOPMENT.md` (release workflow and packaging reference)
- `tools/build.py` (multi-script bundle packaging logic)
- `tools/build_help.txt` (command reference for multi-script bundles)
- `CHANGELOG.md` (release notes for repo releases)
- `deslopification/prompts/todo/SENSORLIST_V100_RELEASE_PART2.md` (prior release workflow for script-only artifacts)
- `deslopification/memory/notes/session-note/release-versioning/SESSION_NOTES_2026-02-26_MULTISCRIPT_ZIP_RENAME.md` (context on the fixed `dist/sloppy-ethos_scripts.zip` naming)

## Branch/Worktree Gate (Required Before Editing)

1. Run issue preflight:
   - `python tools/session_preflight.py --mode issue --issue-number 42 --issue-kind enhancement --slug multi-script-release-deliverable`
2. Confirm target branch and current branch:
   - `git branch --show-current`
   - `git status --short --branch`
3. If preflight requires a branch other than `main`, switch to the recommended branch before editing.
4. If the worktree is dirty, pause to decide whether to stash, commit, or clean before continuing.
5. After switching branches, sync before editing:
   - `git pull --ff-only origin {target-branch}`

## Scope

- In scope:
  - Document that repo releases publish `dist/sloppy-ethos_scripts.zip` alongside single-script ZIPs and describe how to build it.
  - Ensure release notes/checklists mention the multi-script bundle as a deliverable and call out the `python tools/build.py --project SensorList --project ethos_events --dist` command.
  - Verify release guidance highlights the multi-script bundle asset entry in GitHub Releases.
- Out of scope:
  - Creating new multi-script packaging tooling (the bundle already exists via `tools/build.py`).
  - Script-specific release workflow changes (SensorList-only release tasks).
  - Versioning changes to the multi-script bundle (it intentionally remains unversioned).

## Technical Constraints

- The bundle filename must remain `dist/sloppy-ethos_scripts.zip` to stay consistent with tooling and existing release notes.
- Release documentation must keep instructions reproducible using the current `tools/build.py` behavior (no new dependencies or custom scripts).
- Repository release notes should not assert script gating requirements from SensorList-only releases.

## Suggested Execution Plan

1. Update the release documentation (README, docs/DEVELOPMENT) to list the multi-script archive as a repo-release artifact and describe how to build it.
2. Refresh CHANGELOG/roadmap entries to summarize the addition of the multi-script deliverable for repo releases.
3. Confirm the release checklist (e.g., release workflow docs or README) captures the `python tools/build.py --project SensorList --project ethos_events --dist` step.
4. Optionally note in the release prompt (or separate checklist) that the built `dist/sloppy-ethos_scripts.zip` should be attached to the GitHub release assets for repo releases.

## Manual Acceptance Scenarios (Required)

1. Run `python tools/build.py --project SensorList --project ethos_events --dist` and confirm `dist/sloppy-ethos_scripts.zip` is generated with both `SensorList` and `ethos_events` directories.
2. Confirm repo release docs (README and `docs/DEVELOPMENT.md`) explicitly describe the multi-script bundle as a deliverable and reference the build command as part of the release checklist.
3. Validate that release notes or checklist entries call out attaching `dist/sloppy-ethos_scripts.zip` to the GitHub release so the deliverable is published.

## Validation (Required)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`

## Done Criteria

1. Multi-script bundle is documented as part of repo release deliverables along with the build command.
2. Release notes/checklists mention attaching `dist/sloppy-ethos_scripts.zip` to the GitHub release assets.
3. Required validation passes.
4. Session notes or other memory artifacts capture the change context and follow-up actions.
