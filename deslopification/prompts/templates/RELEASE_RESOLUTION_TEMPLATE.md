# Prompt Template: Agentic Script Release

Use this template for packaging and publishing a script release asset.
Replace all `{...}` placeholders before execution.

## Release Target

- Project: `{PROJECT_NAME}`
- Artifact version source: `scripts/{PROJECT_NAME}/VERSION`
- Expected artifact version: `{SCRIPT_VERSION}`
- Release tag: `{RELEASE_TAG}`
- Release title: `{RELEASE_TITLE}`
- Target branch: `{TARGET_BRANCH}`
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

1. Ensure current branch is `{TARGET_BRANCH}`.
2. If worktree is dirty, stop and confirm commit/stash strategy with the user.
3. Sync branch before release:
   - `git fetch origin`
   - `git pull --ff-only origin {TARGET_BRANCH}`
4. Confirm no ahead/behind drift:
   - `git rev-list --left-right --count origin/{TARGET_BRANCH}...{TARGET_BRANCH}`

## Release Preflight Checks

1. Confirm script version file equals `{SCRIPT_VERSION}`.
2. Confirm release tag does not already exist:
   - `git ls-remote --tags origin {RELEASE_TAG}`
3. Confirm release name/tag is not already published:
   - `gh release list --limit 100`
4. Confirm GitHub auth availability:
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

1. Commit version/documentation updates needed for the release.
2. Push target branch to origin.
3. Create release with tag and artifact attachment:
   - `gh release create {RELEASE_TAG} dist/{PROJECT_NAME}-{SCRIPT_VERSION}.zip --title "{RELEASE_TITLE}" --notes "{RELEASE_NOTES}"`
4. Verify release metadata:
   - `gh release view {RELEASE_TAG} --json tagName,name,url,isDraft,isPrerelease,publishedAt`

## Delivery Contract

Return:

1. Exact commit(s) included.
2. Exact artifact path produced.
3. Release URL and tag.
4. Validation commands with pass/fail results.
5. Remaining risks/follow-up (if any).

## Session Memory Sync

Record release work in `deslopification/memory/` with:

- what changed
- validation run(s)
- release URL/tag
- follow-up items
