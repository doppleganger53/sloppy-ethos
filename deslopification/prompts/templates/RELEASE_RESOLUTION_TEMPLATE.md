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
- Release destination: `{RELEASE_REPO}` (GitHub `OWNER/REPO` on the release issue) and `{RELEASE_REMOTE}` (its Git remote).
- Release branch source: `{HEAD_REPO}` (GitHub `OWNER/REPO`) and `{HEAD_REMOTE}` (its Git remote). These may point to a contributor's fork.
- Release branch (repo: `release/v{REPO_VERSION}`; script: `release/{PROJECT_NAME}-v{SCRIPT_VERSION}`): `{RELEASE_BRANCH}`
- Release tag: `{RELEASE_TAG}`
- Release title: `{RELEASE_TITLE}`
- Release notes file: `{RELEASE_NOTES_FILE}`
- Prep source: `{SOURCE_COMMIT}` (full local commit ID); new output directory: `{PREP_DIR}`.
- Prerelease flag for `gh release create` (use `--prerelease` for `-rc.N`, else leave empty): `{PRERELEASE_FLAG}`
- Snapshot date: `{YYYY-MM-DD}`

## Mission

For a **local packaging dry run**, read [AGENTS.md](../../../AGENTS.md), then
use only **Packaging And Validation** below. No GitHub access is required.
For an actual release, complete the gates below and obtain separate authorization
to publish. A successful prep does not authorize a push, merge, tag, or upload.

## Mandatory Startup Workflow

1. Follow [AGENTS.md](../../../AGENTS.md). Consult relevant project notes only
   when prior release decisions or unresolved gates affect this release.
2. Select the release worktree as described in the branch/worktree gate below.
3. Run issue preflight with release scope:
   - `python tools/session_preflight.py --mode issue --issue-number {ISSUE_NUMBER} --issue-kind {ISSUE_KIND} --slug {ISSUE_SLUG} --release-kind {RELEASE_KIND} --target-branch {RELEASE_BRANCH}`
   - For `script` releases also provide:
     - `--release-repo {RELEASE_REPO}` to read the gate issues in the release project
     - `--project {PROJECT_NAME}`
     - one or more `--script-gate-issue {N}` flags for scope-specific manual gates
4. Confirm version sources:
   - `VERSION`
   - `scripts/{PROJECT_NAME}/VERSION` (`script` only)
5. Cross-check workflow commands in:
   - `README.md`
   - `docs/DEVELOPMENT.md`

## Branch/Worktree Gate (Required Before Release Actions)

1. Use the release issue URL and the intended PR head to identify the two
   repositories. Verify both fetch and push URLs from `git remote -v` against
   those identities. All URLs for each selected remote must point to its
   intended repository. The name `origin` alone does not identify either one.
   Follow the [issue branch and worktree gate](ISSUE_RESOLUTION_TEMPLATE.md#branch-and-worktree-gate-required-before-editing)
   using the named release branch and the release-scoped preflight above.
2. Fetch the release project's base with `git fetch --no-tags {RELEASE_REMOTE} refs/heads/main`
   and require `git merge-base --is-ancestor FETCH_HEAD HEAD` to succeed.
3. For a release branch with a verified upstream, confirm both counts from
   `git rev-list --left-right --count "@{upstream}...HEAD"` are zero after syncing.

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
   - `git ls-remote --tags {RELEASE_REMOTE} refs/tags/{RELEASE_TAG}` must succeed with no output.
5. Check the exact release tag with `gh release view {RELEASE_TAG} --repo {RELEASE_REPO}`.
   Continue only on a confirmed missing release. Resolve authentication or connection failures first.
6. Confirm GitHub auth availability:
   - `gh auth status`

## Packaging And Validation

1. Use an existing committed candidate (including its intended version changes).
   Run checks for touched files from the [validation matrix](../../../AGENTS.md#validation-matrix).
2. Build exactly one scope into a directory that does not yet exist:
   - Repo bundle (SensorList + ethos_events): `python tools/prepare_public_zip.py --ref {SOURCE_COMMIT} --bundle --expect-version {REPO_VERSION} --out-dir "{PREP_DIR}"`
   - Single script: `python tools/prepare_public_zip.py --ref {SOURCE_COMMIT} --project {PROJECT_NAME} --expect-version {SCRIPT_VERSION} --out-dir "{PREP_DIR}"`
3. Review the JSON receipt beside the ZIP. Confirm the commit, project list,
   embedded versions and install entries; record the exact ZIP path and SHA-256.
   A dry run ends here. Never substitute an existing personal build.
4. For release readiness, generate `{RELEASE_NOTES_FILE}` from `CHANGELOG.md`:
   - For `repo` releases:
     - `python tools/write_release_notes.py --version {REPO_VERSION} --output {RELEASE_NOTES_FILE}`
   - For `script` releases:
     - `python tools/write_release_notes.py --version {SCRIPT_VERSION} --project {PROJECT_NAME} --output {RELEASE_NOTES_FILE}`

## Publish Steps (Separately Authorized Release Only)

1. Confirm the reviewed release metadata and candidate commit.
2. Push the reviewed branch to its head repository:
   `git push -u {HEAD_REMOTE} refs/heads/{RELEASE_BRANCH}:refs/heads/{RELEASE_BRANCH}`.
3. Open and merge the reviewed PR with `{RELEASE_REPO}:main` as the base
   and `{HEAD_REPO}:{RELEASE_BRANCH}` as the head. Record its PR number.
4. Query that PR in the release repository:
   `gh pr view {PR_NUMBER} --repo {RELEASE_REPO} --json state,baseRefName,headRepository,headRefName,headRefOid,mergeCommit,url`.
   Require `MERGED`, base `main`, and the reviewed head repository, branch,
   and commit. Record `headRefOid` as `{REVIEWED_HEAD_SHA}` and set
   `{SOURCE_COMMIT}` to the full `mergeCommit.oid`.
   Run `git fetch --no-tags {RELEASE_REMOTE} refs/heads/main` and require
   `git merge-base --is-ancestor {SOURCE_COMMIT} FETCH_HEAD` to succeed.
   If the PR has no merge commit or ancestry fails, stop before tagging.
5. Create a fresh worktree with `git worktree add --detach "{FINAL_WORKTREE}" {SOURCE_COMMIT}`.
   From that worktree repeat the prep recipe with the exact SHA in a new directory
   and generate the final release notes from its changelog. Review the receipt:
   its commit must equal the PR's merge SHA. Use its `{AUDITED_ZIP}`, `{ASSET_NAME}`
   and SHA-256 for publication. Tag the verified commit:
   - `git tag {RELEASE_TAG} {SOURCE_COMMIT}`
   - `git push {RELEASE_REMOTE} refs/tags/{RELEASE_TAG}:refs/tags/{RELEASE_TAG}`
6. Read back `git ls-remote --tags {RELEASE_REMOTE} refs/tags/{RELEASE_TAG}`.
   The lightweight tag created above must point to `{SOURCE_COMMIT}`.
7. Compare the local ZIP's SHA-256 with the final receipt (PowerShell:
   `Get-FileHash -Algorithm SHA256 -LiteralPath "{AUDITED_ZIP}"`). Stop on mismatch.
   Attach only this ZIP:
   - `gh release create {RELEASE_TAG} "{AUDITED_ZIP}" --repo {RELEASE_REPO} --verify-tag --title "{RELEASE_TITLE}" --notes-file "{RELEASE_NOTES_FILE}" {PRERELEASE_FLAG}`
8. Verify release metadata and download the published asset into a new directory:
   - `gh release view {RELEASE_TAG} --repo {RELEASE_REPO} --json tagName,name,url,isDraft,isPrerelease,publishedAt`
   - `gh release download {RELEASE_TAG} --repo {RELEASE_REPO} --pattern "{ASSET_NAME}" --dir "{NEW_DOWNLOAD_DIR}"`
   - `Get-FileHash -Algorithm SHA256 -LiteralPath "{NEW_DOWNLOAD_DIR}/{ASSET_NAME}"`
   - Require the downloaded SHA-256 to match the final receipt. Report a mismatch before updating links or cleaning up.
9. After verification, clean up the published release branch on `{HEAD_REMOTE}`
   if it still points to the reviewed PR head:
   `git push --force-with-lease=refs/heads/{RELEASE_BRANCH}:{REVIEWED_HEAD_SHA} {HEAD_REMOTE} :refs/heads/{RELEASE_BRANCH}`.
   Remove the local branch once its worktree is free and its work is merged.

## Delivery Contract

Return:

1. Exact commit(s) included.
2. Release kind and explicit gate issue outcomes.
3. Audited artifact path, project versions, source commit and SHA-256 for either scope.
4. For a published release: URL, tag and downloaded SHA-256 verification.
5. Validation commands with pass/fail results.
6. Remaining risks/follow-up (if any).

## Session Memory Sync

Follow [documentation and memory sync](../../../AGENTS.md#documentation-and-memory-sync).
Published releases and unresolved release gates are durable context.

For that record, include:

- release kind and scope boundaries
- blocking/out-of-scope gate issues
- validation run(s)
- release URL/tag
- follow-up items
