# Prompt Template: Agentic Repo/Script Release

Use this template for repository-level or script-level releases.
Replace all `{...}` placeholders before execution.

## Release Target

- Release kind (`repo` or `script`): `{RELEASE_KIND}`
- Project (`script` only; use `N/A` for repo): `{PROJECT_NAME}`
- Blocking gate issues for this release scope: `{BLOCKING_GATE_ISSUES}`
- Out-of-scope gate issues for this release scope: `{OUT_OF_SCOPE_GATES}`
- Repository version source: `VERSION`
- Script artifact version source (`script` only): `scripts/{PROJECT_NAME}/VERSION`
- Expected repository version: `{REPO_VERSION}`
- Expected artifact version (`script` only): `{SCRIPT_VERSION}`
- Base branch: `main`
- Release branch (recommended `release/v{REPO_VERSION}`): `{RELEASE_BRANCH}`
- Release tag: `{RELEASE_TAG}`
- Release title: `{RELEASE_TITLE}`
- Release notes file: `{RELEASE_NOTES_FILE}`
- Prerelease flag for `gh release create` (use `--prerelease` for `-rc.N`, else leave empty): `{PRERELEASE_FLAG}`
- Snapshot date: `{YYYY-MM-DD}`

## Mission

Package and publish `{RELEASE_KIND}` release `{RELEASE_TAG}` with strict
scope clarity, explicit gate handling, and auditable validation output.

## Mandatory Startup Workflow

1. Read latest relevant note in `deslopification/memory/`.
2. Check branch/worktree state:
   - `git branch --show-current`
   - `git status --short --branch`
3. Run issue preflight with release scope:
   - `python tools/session_preflight.py --mode issue --issue-number {ISSUE_NUMBER} --issue-kind {ISSUE_KIND} --slug {ISSUE_SLUG} --release-kind {RELEASE_KIND}`
   - For `script` releases also provide:
     - `--project {PROJECT_NAME}`
     - one or more `--script-gate-issue {N}` flags for scope-specific manual gates
4. Confirm version sources:
   - `VERSION`
   - `scripts/{PROJECT_NAME}/VERSION` (`script` only)
5. Cross-check workflow commands in:
   - `README.md`
   - `docs/DEVELOPMENT.md`

## Branch/Worktree Gate (Required Before Release Actions)

1. Ensure current branch is `{RELEASE_BRANCH}` (recommended format: `release/v{REPO_VERSION}`).
2. If worktree is dirty, stop and confirm commit/stash strategy with the user.
3. Sync refs before release:
   - `git fetch origin`
4. Confirm release branch is based on latest `main`:
   - `git rev-list --left-right --count origin/main...{RELEASE_BRANCH}`
5. If `{RELEASE_BRANCH}` already exists on origin, sync and confirm no ahead/behind drift:
   - `git pull --ff-only origin {RELEASE_BRANCH}`
   - `git rev-list --left-right --count origin/{RELEASE_BRANCH}...{RELEASE_BRANCH}`

## Release Scope Gate (Required)

1. Confirm `{RELEASE_KIND}` matches the intended scope and release notes.
2. Confirm all `{BLOCKING_GATE_ISSUES}` are closed for this scope.
3. Confirm `{OUT_OF_SCOPE_GATES}` remain explicitly out of scope and are not used as blockers.
4. If `{RELEASE_KIND}` is `repo`, do not treat script-only manual gates as blockers.
5. If `{RELEASE_KIND}` is `script`, require script gate issue closure before tagging.

## Release Preflight Checks

1. Confirm `VERSION` equals `{REPO_VERSION}`.
2. If `{RELEASE_KIND}` is `script`, confirm `scripts/{PROJECT_NAME}/VERSION` equals `{SCRIPT_VERSION}`.
3. Confirm `CHANGELOG.md` includes release notes for `{REPO_VERSION}`.
4. Confirm release tag does not already exist:
   - `git ls-remote --tags origin {RELEASE_TAG}`
5. Confirm release name/tag is not already published:
   - `gh release list --limit 100`
6. Confirm GitHub auth availability:
   - `gh auth status`

## Packaging And Validation

Run the minimum required checks for this release action:

1. If `{RELEASE_KIND}` is `repo`:
   - run validation based on touched files (see `AGENTS.md` matrix)
   - if scope is broad/cross-cutting, run `python -m pytest -q`
2. If `{RELEASE_KIND}` is `script`:
   - `python tools/build.py --project {PROJECT_NAME} --dist`
   - confirm expected artifact exists:
     - `dist/{PROJECT_NAME}-{SCRIPT_VERSION}.zip`
   - run validation based on touched files (see `AGENTS.md` matrix)
3. Prefer `--notes-file` when publishing release notes to avoid truncation risk.
4. Generate `{RELEASE_NOTES_FILE}` from `CHANGELOG.md` before publishing:
   - For `repo` releases:
     - `python tools/write_release_notes.py --version {REPO_VERSION} --output {RELEASE_NOTES_FILE}`
   - For `script` releases:
     - `python tools/write_release_notes.py --version {SCRIPT_VERSION} --project {PROJECT_NAME} --output {RELEASE_NOTES_FILE}`

## Publish Steps

1. Commit release-branch version/documentation updates.
2. Push release branch to origin.
3. Open and merge PR from `{RELEASE_BRANCH}` into `main`.
4. Sync `main` after merge:
   - `git checkout main`
   - `git pull --ff-only origin main`
5. Tag merged `main` commit and push tag:
   - `git tag {RELEASE_TAG}`
   - `git push origin {RELEASE_TAG}`
6. Generate `{RELEASE_NOTES_FILE}` from the matching `CHANGELOG.md` entry with the helper command for this scope.
7. Create release:
   - For `repo` releases:
     - `gh release create {RELEASE_TAG} --title "{RELEASE_TITLE}" --notes-file {RELEASE_NOTES_FILE} {PRERELEASE_FLAG}`
   - For `script` releases:
     - `gh release create {RELEASE_TAG} dist/{PROJECT_NAME}-{SCRIPT_VERSION}.zip --title "{RELEASE_TITLE}" --notes-file {RELEASE_NOTES_FILE} {PRERELEASE_FLAG}`
8. Verify release metadata:
   - `gh release view {RELEASE_TAG} --json tagName,name,url,isDraft,isPrerelease,publishedAt`
9. Delete release branch after publication:
   - `git push origin --delete {RELEASE_BRANCH}`
   - `git branch -d {RELEASE_BRANCH}`

## Delivery Contract

Return:

1. Exact commit(s) included.
2. Release kind and explicit gate issue outcomes.
3. Exact artifact path produced (`script` only).
4. Release URL and tag.
5. Validation commands with pass/fail results.
6. Remaining risks/follow-up (if any).

## Session Memory Sync

Record release work in `deslopification/memory/notes/{category}/{focus}/` with:

- release kind and scope boundaries
- blocking/out-of-scope gate issues
- validation run(s)
- release URL/tag
- follow-up items
