# Prompt Template: Agentic Script Release

Use this template for packaging and publishing a script release asset.
Replace all `{...}` placeholders before execution.

## Release Target

- Project: `{PROJECT_NAME}`
- Repository version source: `VERSION`
- Artifact version source: `scripts/{PROJECT_NAME}/VERSION`
- Expected repository version: `{REPO_VERSION}`
- Expected artifact version: `{SCRIPT_VERSION}`
- Base branch: `main`
- Release branch (recommended `release/v{REPO_VERSION}`): `{RELEASE_BRANCH}`
- Release tag: `{RELEASE_TAG}`
- Release title: `{RELEASE_TITLE}`
- Prerelease flag for `gh release create` (use `--prerelease` for `-rc.N`, else leave empty): `{PRERELEASE_FLAG}`
- Snapshot date: `{YYYY-MM-DD}`

## Mission

Package and publish release `{RELEASE_TAG}` with root-cause-first handling,
strict version/branch guardrails, and auditable validation output.

## Mandatory Startup Workflow

1. Read latest relevant note in `deslopification/memory/`.
2. Check branch/worktree state:
   - `git branch --show-current`
   - `git status --short --branch`
3. Confirm version sources:
   - `VERSION`
   - `scripts/{PROJECT_NAME}/VERSION`
4. Cross-check workflow commands in:
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

## Release Preflight Checks

1. Confirm `VERSION` equals `{REPO_VERSION}` and `scripts/{PROJECT_NAME}/VERSION` equals `{SCRIPT_VERSION}`.
2. Confirm `CHANGELOG.md` includes release notes for `{REPO_VERSION}`.
3. Confirm release tag does not already exist:
   - `git ls-remote --tags origin {RELEASE_TAG}`
4. Confirm release name/tag is not already published:
   - `gh release list --limit 100`
5. Confirm GitHub auth availability:
   - `gh auth status`

## Packaging And Validation

Run the minimum required checks for this release action:

1. `python tools/build.py --project {PROJECT_NAME} --dist`
2. Confirm expected artifact exists:
   - `dist/{PROJECT_NAME}-{SCRIPT_VERSION}.zip`
3. If documentation/templates were changed in session:
   - `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
4. If Lua behavior changed in session:
   - `luac -p scripts/SensorList/main.lua`
   - `python -m pytest tests/test_sensorlist_widget.py -q`

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
6. Create release with tag and artifact attachment:
   - `gh release create {RELEASE_TAG} dist/{PROJECT_NAME}-{SCRIPT_VERSION}.zip --title "{RELEASE_TITLE}" --notes "{RELEASE_NOTES}" {PRERELEASE_FLAG}`
7. Verify release metadata:
   - `gh release view {RELEASE_TAG} --json tagName,name,url,isDraft,isPrerelease,publishedAt`
8. Delete release branch after publication:
   - `git push origin --delete {RELEASE_BRANCH}`
   - `git branch -d {RELEASE_BRANCH}`

## Delivery Contract

Return:

1. Exact commit(s) included.
2. Exact artifact path produced.
3. Release URL and tag.
4. Validation commands with pass/fail results.
5. Remaining risks/follow-up (if any).

## Session Memory Sync

Record release work in `deslopification/memory/notes/{category}/{focus}/` with:

- what changed
- validation run(s)
- release URL/tag
- follow-up items
