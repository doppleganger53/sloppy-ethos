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
- Monthly rollup example for compact history review:
  `deslopification/memory/SUMMARY_2026-02.md`.
- When adding notes, follow `deslopification/memory/SESSION_NOTE_TEMPLATE.md`.

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

1. Sync `main` and create release-prep branch:
   - `git checkout main`
   - `git pull --ff-only origin main`
   - `git checkout -b release/v{VERSION}`
2. Finalize release metadata on `release/v{VERSION}`:
   - root `VERSION`
   - touched `scripts/{ProjectName}/VERSION` files
   - `CHANGELOG.md`
3. Sync release branch and confirm drift:
   - `git fetch origin`
   - `git push -u origin release/v{VERSION}`
   - `git pull --ff-only origin release/v{VERSION}`
   - `git rev-list --left-right --count origin/release/v{VERSION}...release/v{VERSION}`
4. Build release artifact(s):
   `python tools/build.py --project SensorList --dist`
5. Validate according to touched files (see `AGENTS.md` validation matrix).
6. Open and merge a release-prep PR from `release/v{VERSION}` into `main`.
7. Sync `main`, tag, and push:
   - `git checkout main`
   - `git pull --ff-only origin main`
   - `git tag v{VERSION}`
   - `git push origin v{VERSION}`
8. Publish GitHub release notes from `CHANGELOG.md` and attach release ZIP assets (for example, `dist/SensorList-{scripts/SensorList/VERSION}.zip`).
9. Delete release-prep branch after publication:
   - `git push origin --delete release/v{VERSION}`
   - `git branch -d release/v{VERSION}`
