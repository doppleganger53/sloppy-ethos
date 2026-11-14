# Development Notes

## Repository Policy

[AGENTS.md](../AGENTS.md) is the canonical operational policy. Use its
[validation matrix](../AGENTS.md#validation-matrix) for the files changed;
this document describes tooling and release procedures.

## Prerequisites

- Python 3.9+
- Lua tooling on PATH (`lua`, `luac`)
- Optional: `stylua` for formatting

## Parent Workspace Boundary

- `sloppy-ethos/` is the primary implementation repository inside the parent
  `EthosLua` workspace.
- A standalone checkout needs no parent. If starting from the parent workspace, read its `AGENTS.md`,
  then enter `sloppy-ethos/` before running repository commands or applying
  this repo's policy.
- Keep sibling reference checkouts outside this repo's Git scope. Use scoped
  update commands such as `git -C ../ETHOS-Feedback-Community pull --ff-only`.
- Treat references as read-only evidence unless a user request explicitly asks
  to update them, and verify license/attribution before copying anything into
  this repository.

## Core Commands

- Parse each changed Lua file. For example, to check the SensorList entry point, run
  `luac -p scripts/SensorList/main.lua`
- Package locally:
  `python tools/build.py --project SensorList --dist`
  `python tools/build.py --project BoundryMap --dist`
  `python tools/build.py --project SensorList --project ethos_events --dist`
  `python tools/build.py --help`
- For an isolated headless or GUI run, use the examples in the
  [WebSimulator harness guide](../tools/sim/harness/README.md).

## Configuring Simulator Path

- Only for a requested change to a named live Suite profile: verify its
  configured path and select the radio, for example
  `python tools/build.py --project SensorList --deploy --sim-radio X20RS`.
  `--clean` removes installed files **and** build outputs. Neither command is
  needed for the WebSimulator.
- Copy `tools/deploy.config.example.json` to `tools/deploy.config.json` and populate `ETHOS_SIM_PATHS`.
- `ETHOS_SIM_PATHS` must be an array of entries with `radio` and `path`.
- Mark exactly one entry as `"default": true` for deploy/clean when `--sim-radio` is not provided.
- `tools/deploy.config.json` is ignored via `.gitignore`, so each contributor can keep their private path local.

## Packaging Behavior

`tools/build.py`:

- validates Lua syntax for all project Lua files using `luac -p`.
- reads single-script package version from `scripts/{ProjectName}/VERSION` (or `--version` override).
- produces single-script ZIPs as `dist/{ProjectName}-{version}.zip`.
- supports repeatable `--project` for multi-script dist bundles and produces the unversioned bundle ZIP `dist/sloppy-ethos_scripts.zip`.
- supports an optional project-local build manifest with `radioFiles` entries for explicit installable files that belong outside `/scripts` or under the owning `scripts/{ProjectName}` folder.
- supports generic `assets` entries that scan a project-local source directory with manifest-declared glob patterns and install matches under manifest-declared radio or script-local destinations.
- packages manifest-declared `radioFiles` and discovered `assets` into the ZIP root at their radio-relative destinations.
- optionally copies `scripts/{ProjectName}` plus manifest-declared/discovered radio files into `${ETHOS_SIM_PATHS[default_or_selected_radio]}` when `--deploy` is specified (no ZIP).
- supports `--clean` to remove `scripts/{ProjectName}`, manifest-declared/discovered radio files, and clear `dist/` (or `--out-dir`) artifacts.
- supports `--sim-radio` to resolve model-specific simulator paths via `ETHOS_SIM_PATHS`.
- supports `--help` to print `tools/build_help.txt` command reference.
- supports custom ZIP destination via `--out-dir`.
- Write operations fail fast with clear errors when paths are missing or unwritable.
- BoundryMap map assets are local-only and ignored because flying-site maps can reveal private locations. [scripts/BoundryMap/build.json](../scripts/BoundryMap/build.json) defines the source folder, include globs, script-local destinations, flattening behavior, and optional-source behavior for private maps.

## Automated WebSimulator Harness

`tools/sim/harness/run.py`:

- resolves `latest-26.1` to the newest Ethos `26.1` release from `FrSkyRC/ETHOS-Feedback-Community` when no matching cached `26.1` runtime is already present.
- keeps the `download` command as the explicit path for refreshing a latest-alias release lookup.
- downloads the requested radio WebSimulator package, such as `X20RS-FCC-WebSimulator.zip`, into `tools/sim/radios/{Radio}-{Region}/{EthosVersion}/{PackageName}/`.
- validates the GitHub asset SHA-256 digest when release metadata provides one.
- stages scripts in a disposable persist directory using the existing `tools/build.py` install-spec helpers.
- accepts repeated `--project` values so GUI and headless sessions can stage one project or a project set into the same simulator persist tree.
- supports JSON smoke suites, starting with `tools/sim/harness/suites/SensorList-X20RS-FCC.json`.
- runs headless smoke checks through Node.js and reports structured statuses for success, missing runtime, download failure, startup failure, script failure, and timeout.
- can prepare and serve a browser-based GUI view for manual confirmation without hand-copying scripts.
- serves GUI runtime JS/WASM from the cached package under `tools/sim/radios/` instead of copying runtime files into each run directory.
- keeps downloaded runtimes, extracted JavaScript/WASM, logs, and generated GUI files out of git.
- does not call `_writeDefaultSettingsAndModel()` by default in headless or GUI startup because some X20RS-FCC runtimes can block or abort in that export; use `--write-default-model` only when validating a runtime known not to block there.

## Tooling Decision Records

- Issue #22 build tooling evaluation: [Issue #22 build.py to doit migration decision](decisions/ISSUE-022-doit-migration-evaluation.md).
- Current decision state (2026-02-26): retain `tools/build.py` as the canonical build/deploy workflow.
- Ethos `26.1` reference baseline and compatibility backlog:
  [ETHOS_26_1_COMPATIBILITY.md](ETHOS_26_1_COMPATIBILITY.md).

## Session Memory Navigation

Use [documentation and memory sync](../AGENTS.md#documentation-and-memory-sync)
to decide when a note is useful. When prior decisions or unresolved work are
relevant, consult the [current-state index](../deslopification/memory/CURRENT_STATE.md)
and search [CATALOG.md](../deslopification/memory/CATALOG.md) by topic.
The [memory guide](../deslopification/memory/README.md) describes note placement
and catalog maintenance; historical notes need not be loaded for routine work.

## Simulator Tips

- If stale behavior appears after script updates, remove cached `main.luac` and reinstall the widget.
- Ethos Info errors persist until you hit the `Reset` button; clear them between test runs.

## Running the Python tests

- Select a Python 3.9+ interpreter in VS Code via `Python: Select Interpreter`, or use a shell where `python` resolves to your desired interpreter.
- Install the pytest harness listed in `requirements/dev.txt`: `python -m pip install -r requirements/dev.txt` (includes `pytest-cov` for coverage in the VS Code Test Explorer).
- Run all repo-level and script-local tests: `python -m pytest -q`.
- Run one script's tests directly, for example: `python -m pytest scripts/SensorList/tests -q`.
- Place script-owned pytest wrappers and Lua harnesses under `scripts/{ProjectName}/tests/`; keep generic tooling and documentation contracts under the root `tests/` folder.
- Select required checks from the [validation matrix](../AGENTS.md#validation-matrix).
- Use the VS Code Testing view to run/discover tests and trigger coverage once dependencies are installed for the selected interpreter.

## Lua Coverage In VS Code

- Install the recommended workspace extension: `Coverage Gutters`.
- Install Lua coverage tools with LuaRocks:
  - `luarocks install luacov`
  - `luarocks install luacov-reporter-lcov`
- The workspace includes `.luacov` config that targets `scripts/SensorList/main.lua` and writes reports under `coverage/lua/`.
- Generate a Lua coverage report manually with `lua -lluacov scripts/SensorList/tests/lua/test_sensorlist.lua` followed by `luacov -r lcov`.
- Coverage Gutters is configured in `.vscode/settings.json` to pick up `luacov.report.out` from the workspace coverage output folder.

## Main Branch Release And Versioning Policy

This repository uses `main` as the only long-lived integration branch and aligns release/version behavior with [Semantic Versioning](https://semver.org/) and [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

### Release Scope Types

- `repo` release:
  - Covers repository-level delivery (workflow/docs/tooling/policy/release metadata).
  - Root version source of truth is `VERSION`.
  - Script-specific manual gate issues are not implicit blockers unless the release scope explicitly includes that script.
- `script` release:
  - Covers an installable script artifact release for a specific project (`SensorList`, `ethos_events`, or future scripts).
  - Script version source of truth is `scripts/{ProjectName}/VERSION`.
  - Script-specific gate issues (for example manual test gates) must be closed before tag/publish.

### Tag Naming Clarity

- Repository release tags: `v{VERSION}`.
- Script release tags should include project prefix to avoid ambiguity:
  - `sensorlist-v{VERSION}`
  - `ethos_events-v{VERSION}`
- Legacy tags without project prefix remain valid historical artifacts; use the scoped style for new script releases.

### Branch Roles And Naming

Follow the [branching conventions](../AGENTS.md#branching-conventions),
[session branch gate](../AGENTS.md#session-branch-gate), and
[PR merge strategy](../AGENTS.md#pr-merge-strategy). Delete merged short-lived
branches after PR merge.

### Version Bump Timing

- Bump `scripts/{ProjectName}/VERSION` in the same branch/PR that changes installable behavior/assets for that script.
- Do not bump script versions for docs-only or workflow-only changes.
- Bump root `VERSION` on `release/v{VERSION}` only, after release scope is finalized.
- Root `VERSION` must match the final release tag value without the leading `v`.
- Apply SemVer to root/script versions:
  - patch for backward-compatible fixes.
  - minor for backward-compatible features.
  - major for breaking changes.

### Optional Prerelease (`-rc.N`) Usage

- `-rc.N` is optional and used only on release-prep branches during stabilization (`release/v{VERSION}` for repo releases or `release/{ProjectName}-v{VERSION}` for script releases).
- If used, apply `-rc.N` consistently to root `VERSION` and each script `VERSION` included in that candidate build.
- Increment `N` for each candidate (`-rc.1`, `-rc.2`, ...).
- Use prerelease tags for candidates (`v{VERSION}-rc.N`) and publish with `--prerelease`.
- Remove the `-rc.N` suffix before final tag/release publication.

## Release Workflow

### Prepare a public ZIP

With Git, Python and `luac` on PATH, choose a committed revision, its expected
release version, and an output directory that does **not** exist. Local prep
does not need GitHub access. Examples (replace the ref and version for your release):

- Repo bundle (SensorList and ethos_events):
  `python tools/prepare_public_zip.py --ref HEAD --bundle --expect-version 1.0.3 --out-dir dist/review-bundle`
- One script (BoundryMap shown):
  `python tools/prepare_public_zip.py --ref HEAD --project BoundryMap --expect-version 0.2.0 --out-dir dist/review-script`

The helper reads the selected projects, version and builder directly from the
resolved commit's Git blobs, preserving tracked bytes independently of local
attributes and line-ending settings. It runs that builder in temporary storage, opens the new ZIP,
checks its install paths, project versions and bundle manifest, and rejects map
directories and boundary sidecars. Current public projects install under their
own script directories. Review any new radio-root destination before permitting
it in a public ZIP. Keep using the ordinary build command for personal packages
with local maps.

Review the entry list, versions, source commit, archive name and SHA-256 in the
JSON receipt beside the ZIP. Existing archives are never reused or replaced.
Stop here for a dry run. Only for a separately authorized release, prepare the
final artifact from the exact commit to be tagged, upload that ZIP, then download
the published asset to a new directory and compare its SHA-256 to this receipt.

### Shared Steps (All Release Kinds)

1. Use the [release branch and worktree gate](../deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md#branchworktree-gate-required-before-release-actions)
   to select a clean worktree for `release/v{VERSION}` (repo) or
   `release/{ProjectName}-v{VERSION}` (script).
2. Run issue preflight with scope:
   - run `tools/session_preflight.py` with:
     - `--mode issue --issue-number {N} --issue-kind {enhancement|bug|docs|chore} --slug {slug} --release-kind {repo|script} --target-branch {RELEASE_BRANCH}`
   - For `script` releases also include:
     - `--project {ProjectName}`
     - one or more `--script-gate-issue {N}` flags
3. Sync an existing branch only with its verified upstream. For a new branch,
   prepare locally from the verified base. Set the upstream on the first push
   when sharing the release-prep PR.
4. Validate according to the [validation matrix](../AGENTS.md#validation-matrix).
5. Commit release metadata, [prepare a public ZIP](#prepare-a-public-zip) and generate the standalone release body from the matching `CHANGELOG.md` entry with `tools/write_release_notes.py` (use `--version`, plus `--project` for script releases, and write to a temporary `--output` file).
6. During a separately authorized release, open and merge a release-prep PR from `{RELEASE_BRANCH}` into `main`.
7. Sync `main`, prepare the final ZIP in a **new** directory from the merged commit, inspect the receipt, and tag that same commit:
   - `git checkout main`
   - `git pull --ff-only origin main`
   - `git tag {RELEASE_TAG} {AUDITED_COMMIT}`
   - `git push origin {RELEASE_TAG}`
8. Recheck the ZIP's SHA-256 against the receipt. Publish the audited ZIP with the generated notes file (`gh release create --verify-tag` and `--notes-file`), download the asset into a new directory, and verify its SHA-256 matches the receipt.
9. Delete release-prep branch after successful verification:
   - `git push origin --delete {RELEASE_BRANCH}`
   - `git branch -d {RELEASE_BRANCH}`

### Repository Release (`release-kind=repo`)

1. Finalize release metadata on `release/v{VERSION}`:
   - root `VERSION`
   - `CHANGELOG.md`
   - any repo-level workflow/docs/process files
2. Use the [public bundle recipe](#prepare-a-public-zip) with the expected root version. Attach only the audited bundle at publication.
3. Do not treat script-only manual gates as implicit blockers unless that script is in scope.

### Script Release (`release-kind=script`)

1. Finalize release metadata on `release/{ProjectName}-v{VERSION}`:
   - root `VERSION` (if the repository release is part of this scope)
   - touched `scripts/{ProjectName}/VERSION` file(s)
   - `CHANGELOG.md`
2. Require all script gate issues passed through `--script-gate-issue` to be closed before tag/publish.
3. Use the [public single-script recipe](#prepare-a-public-zip) with the target project and expected script version.
4. Generate the script release notes file from `CHANGELOG.md` with `tools/write_release_notes.py` (use `--version`, `--project`, and `--output`) so the published GitHub release keeps the changelog markdown formatting.
5. Attach only the audited script ZIP. After the published download is verified, update `README.md` `Download Latest Script Releases` so it reflects the newly published asset.
   - Do not add unreleased script ZIP names to that README section before the GitHub release exists.
