# Development Notes

## Prerequisites

- Python 3.9+
- Lua tooling on PATH (`lua`, `luac`)
- Optional: `stylua` for formatting

## Core Commands

- Syntax check:
  `luac -p scripts/SensorList/main.lua`
- Package and deploy:
  `python tools/build.py --project SensorList --dist`
  `python tools/build.py --project SensorList --project ethos_events --dist`
  `python tools/build.py --project SensorList --deploy`
  `python tools/build.py --project ethos_events --deploy`
  `python tools/build.py --project SensorList --clean --sim-radio X20RS`
  `python tools/build.py --help`

## Configuring Simulator Path

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
- optionally copies `scripts/{ProjectName}` into `${ETHOS_SIM_PATHS[default_or_selected_radio]}/scripts/{ProjectName}` when `--deploy` is specified (no ZIP).
- supports `--clean` to remove `scripts/{ProjectName}` from simulator deploy path and clear `dist/` (or `--out-dir`) artifacts.
- supports `--sim-radio` to resolve model-specific simulator paths via `ETHOS_SIM_PATHS`.
- supports `--help` to print `tools/build_help.txt` command reference.
- supports custom ZIP destination via `--out-dir`.
- Write operations fail fast with clear errors when paths are missing or unwritable.

## Tooling Decision Records

- Issue #22 build tooling evaluation: [Issue #22 build.py to doit migration decision](decisions/ISSUE-022-doit-migration-evaluation.md).
- Current decision state (2026-02-26): retain `tools/build.py` as the canonical build/deploy workflow.

## Session Memory Navigation

- Memory entrypoint: `deslopification/memory/README.md`.
- Current high-signal context: `deslopification/memory/CURRENT_STATE.md`.
- Full historical index: `deslopification/memory/CATALOG.md`.
- Weekly rollup example for compact history review:
  `deslopification/memory/notes/weekly-summary/memory-ops/SUMMARY_2026-02-21_to_2026-02-27.md`.
- When adding notes, follow `deslopification/memory/SESSION_NOTE_TEMPLATE.md`.
- New notes are stored under:
  `deslopification/memory/notes/{category}/{focus}/`.
- Prefer specific focus classifiers for session notes and avoid `general` by
  default.
- Use focus `lua-ethos` for Ethos Lua script/widget/tool notes (SensorList,
  ethos_events, and similar future scripts).

## Simulator Tips

- If stale behavior appears after script updates, remove cached `main.luac` and reinstall the widget.
- Ethos Info errors persist until you hit the `Reset` button; clear them between test runs.

## Running the Python tests

- Select a Python 3.9+ interpreter in VS Code via `Python: Select Interpreter`, or use a shell where `python` resolves to your desired interpreter.
- Install the pytest harness listed in `requirements/dev.txt`: `python -m pip install -r requirements/dev.txt` (includes `pytest-cov` for coverage in the VS Code Test Explorer).
- Run the Lua-driven sensor list test file through pytest: `python -m pytest tests/test_sensorlist_widget.py`.
- For any documentation updates, run docs contract checks: `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`.
- Use the VS Code Testing view to run/discover tests and trigger coverage once dependencies are installed for the selected interpreter.

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

- `main` is the only long-lived branch.
- All issue work uses short-lived branches created from the latest `main`.
- Branch naming conventions:
  - `feature/{issue-number}-{short-slug}` for enhancements.
  - `fix/{issue-number}-{short-slug}` for defects.
  - `docs/{issue-number}-{short-slug}` for docs/process-only changes.
  - `chore/{issue-number}-{short-slug}` for maintenance/tooling work.
- Release-prep work uses short-lived branches named `release/v{VERSION}` created from `main`.
- Every PR targets `main` and should include linked-closing keywords when applicable (for example, `Closes #29`).
- Delete merged short-lived branches after PR merge.

### PR Merge Strategy

- Default merge method for normal issue PRs (`feature/`, `fix/`, `docs/`, `chore/`) is `squash`.
- Use `merge commit` for release-prep PRs (`release/v{VERSION}`) and lineage-sensitive PRs where preserving branch commit structure is intentional.
- `rebase` merge is not the repository default.

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

- `-rc.N` is optional and used only on `release/v{VERSION}` during stabilization.
- If used, apply `-rc.N` consistently to root `VERSION` and each script `VERSION` included in that candidate build.
- Increment `N` for each candidate (`-rc.1`, `-rc.2`, ...).
- Use prerelease tags for candidates (`v{VERSION}-rc.N`) and publish with `--prerelease`.
- Remove the `-rc.N` suffix before final tag/release publication.

## Release Workflow

### Shared Steps (All Release Kinds)

1. Sync `main` and create release-prep branch:
   - `git checkout main`
   - `git pull --ff-only origin main`
   - `git checkout -b release/v{VERSION}`
2. Run issue preflight with scope:
   - run `tools/session_preflight.py` with:
     - `--mode issue --issue-number {N} --issue-kind {enhancement|bug|docs|chore} --slug {slug} --release-kind {repo|script}`
   - For `script` releases also include:
     - `--project {ProjectName}`
     - one or more `--script-gate-issue {N}` flags
3. Sync release branch and confirm drift:
   - `git fetch origin`
   - `git push -u origin release/v{VERSION}`
   - `git pull --ff-only origin release/v{VERSION}`
   - `git rev-list --left-right --count origin/release/v{VERSION}...release/v{VERSION}`
4. Validate according to touched files (see `AGENTS.md` validation matrix).
5. Generate the standalone release body from the matching `CHANGELOG.md` entry with `tools/write_release_notes.py` (use `--version`, plus `--project` for script releases, and write to a temporary `--output` file), then reuse that file with `gh release create --notes-file`.
6. Open and merge a release-prep PR from `release/v{VERSION}` into `main`.
7. Sync `main`, tag, and push:
   - `git checkout main`
   - `git pull --ff-only origin main`
   - `git tag v{VERSION}`
   - `git push origin v{VERSION}`
8. Publish GitHub release notes from the generated notes file sourced from `CHANGELOG.md`.
9. Delete release-prep branch after publication:
   - `git push origin --delete release/v{VERSION}`
   - `git branch -d release/v{VERSION}`

### Repository Release (`release-kind=repo`)

1. Finalize release metadata on `release/v{VERSION}`:
   - root `VERSION`
   - `CHANGELOG.md`
   - any repo-level workflow/docs/process files
2. Build repo release install artifacts:
   - `python tools/build.py --project SensorList --project ethos_events --dist`
3. Ensure GitHub release assets include:
   - `dist/sloppy-ethos_scripts.zip`
   - included single-script ZIP artifacts (for example `dist/SensorList-{scripts/SensorList/VERSION}.zip` and `dist/ethos_events-{scripts/ethos_events/VERSION}.zip`)
4. Do not treat script-only manual gates as implicit blockers unless that script is in scope.
5. Publish release notes and install assets with `gh release create v{VERSION} dist/sloppy-ethos_scripts.zip dist/SensorList-{scripts/SensorList/VERSION}.zip dist/ethos_events-{scripts/ethos_events/VERSION}.zip --title "{TITLE}" --notes-file {RELEASE_NOTES_FILE}` after generating `{RELEASE_NOTES_FILE}` with `tools/write_release_notes.py`.

### Script Release (`release-kind=script`)

1. Finalize release metadata on `release/v{VERSION}`:
   - root `VERSION` (if the repository release is part of this scope)
   - touched `scripts/{ProjectName}/VERSION` file(s)
   - `CHANGELOG.md`
2. Require all script gate issues passed through `--script-gate-issue` to be closed before tag/publish.
3. Build release artifact(s), for example:
   - `python tools/build.py --project SensorList --dist`
4. Generate the script release notes file from `CHANGELOG.md` with `tools/write_release_notes.py` (use `--version`, `--project`, and `--output`) so the published GitHub release keeps the changelog markdown formatting.
5. Attach script artifact(s) to the release, for example:
   - `dist/SensorList-{scripts/SensorList/VERSION}.zip`
6. Publish the script release with `gh release create ... --notes-file {RELEASE_NOTES_FILE}` instead of inline `--notes`.
