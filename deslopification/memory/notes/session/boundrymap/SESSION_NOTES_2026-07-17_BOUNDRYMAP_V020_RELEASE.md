# Session Notes 2026-07-17 - BoundryMap-v0.2.0 Release

## Note Placement

- Artifact: `session`
- Scope: `boundrymap`
- Concern: `release`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Prepared and published the first public BoundryMap script release as `0.2.0`.
- Merged release PR #106 with merge commit
  `f237c4872074c8b18176a222f4d8950db7156d8c` and published:
  - tag and title: `BoundryMap-v0.2.0`
  - asset: `BoundryMap-0.2.0.zip`
  - release:
    `https://github.com/doppleganger53/sloppy-ethos/releases/tag/BoundryMap-v0.2.0`
- Added the script-scoped `BoundryMap v0.2.0` changelog entry and left the root
  repository version at `1.0.3`.
- Added the `boundrymap` memory scope so script-local behavior and release
  history no longer need to use the repository-wide scope.
- During release preparation, kept published-release links and status text
  unchanged until the tag and GitHub release existed.
- Updated the README download link and repository-layout status only after the
  published asset was downloaded and verified.
- Required the public ZIP to be built from tracked repository content, not the
  ordinary working tree, because local map assets and boundary sidecars are
  intentionally ignored and private.
- Files touched:
  - `CHANGELOG.md`
  - `README.md`
  - `docs/REPOSITORY_LAYOUT.md`
  - `scripts/BoundryMap/VERSION`
  - `tools/update_memory_catalog.py`
  - `tests/test_memory_catalog_sync.py`
  - `deslopification/memory/README.md`
  - `deslopification/memory/SESSION_NOTE_TEMPLATE.md`
  - `deslopification/memory/CURRENT_STATE.md`
  - `deslopification/memory/CATALOG.md`
  - `deslopification/memory/notes/session/boundrymap/.desc`
  - `deslopification/memory/notes/session/boundrymap/SESSION_NOTES_2026-07-17_BOUNDRYMAP_V020_RELEASE.md`

## Why

- Root cause or objective:
  - Promote the release-ready BoundryMap script from internal patch version
    `0.1.8` to its first public minor release, with reproducible validation and
    a privacy-audited install ZIP.
- Scope guardrails:
  - `SensorList` and `ethos_events` have no unreleased installable changes, and
    `SmartMapper` remains incomplete, so their versions are unchanged.
  - Ethos 26.1 compatibility Issues #78, #79, and #80 remain outside this
    Ethos 1.6.6 release claim.
  - `README.md` and `docs/REPOSITORY_LAYOUT.md` were updated only after the
    GitHub release was published and verified.

## Validation run(s)

- `python tools/session_preflight.py --mode issue --issue-number 102 --issue-kind chore --slug boundrymap-release --release-kind script --project BoundryMap --script-gate-issue 102`
  - result: local branch and clean-worktree checks passed; the command stopped
    only because the GitHub CLI is intentionally unauthenticated in this remote
    mobile session. Issue #102 had already been independently verified closed
    during release readiness review.
- `luac -p scripts/BoundryMap/main.lua`
  - result: passed.
- `python -m pytest scripts/BoundryMap/tests -q`
  - result: `3 passed`.
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py tests/test_memory_catalog_sync.py -q`
  - result: `212 passed, 26 skipped`.
- `python -m pytest -q`
  - result: `366 passed, 26 skipped`.
- `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/BoundryMap-X20RS-FCC-1.6.6.json --no-download --run-dir tools/sim/runs/release-boundrymap-v020-x20rs-166 --persist-dir tools/sim/runs/release-boundrymap-v020-x20rs-166-persist --timeout-ms 20000`
  - result: success; Ethos `1.6.6` `X20RS-FCC` started with no errors.
    The informational `reloadScripts_unavailable` message is expected for this
    runtime.
- `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/BoundryMap-X20PROAW-FCC-1.6.6.json --no-download --run-dir tools/sim/runs/release-boundrymap-v020-x20proaw-166 --persist-dir tools/sim/runs/release-boundrymap-v020-x20proaw-166-persist --timeout-ms 20000`
  - result: success; Ethos `1.6.6` `X20PROAW-FCC` started with no errors.
    The informational `reloadScripts_unavailable` message is expected for this
    runtime.
- `git archive 1cd0cdb2b92ae7eed3f50d71727435acc6395b71` followed by
  `python tools/build.py --project BoundryMap --dist` in the extracted archive:
  - result: passed; built the install ZIP from the exact committed tracked
    snapshot rather than from the working tree.
  - artifact:
    `dist/release/BoundryMap-v0.2.0/BoundryMap-0.2.0.zip`
- ZIP entry and embedded-version audit:
  - result: passed; 20 entries, embedded version `0.2.0`, all required files
    present, and zero forbidden entries.
  - exclusions verified: local map folders, `*.boundries.json`, tests,
    `build.json`, Python caches, `GuideField`, and `WJRC` content.
  - size: `133153` bytes.
  - SHA-256:
    `CCE6FA9248AD52C347EC2E08BE1315FFA31441C395BEE6ED12654BBA799A5843`.
- `python tools/write_release_notes.py --version 0.2.0 --project BoundryMap --output dist/release/BoundryMap-v0.2.0/release-notes.md`
  - result: passed; generated the release body from the dated changelog entry.
- PR #106 merge and tag verification:
  - result: passed; CI completed successfully, the PR was merged with merge
    commit `f237c4872074c8b18176a222f4d8950db7156d8c`, and tag
    `BoundryMap-v0.2.0` points to that merged `main` commit.
- `gh release view BoundryMap-v0.2.0 --json tagName,name,url,isDraft,isPrerelease,publishedAt,targetCommitish,assets`
  - result: passed; the release is published, is neither draft nor prerelease,
    targets `main`, and contains only `BoundryMap-0.2.0.zip`.
- Remote release-asset download and checksum verification:
  - result: passed; downloaded size `133153` bytes and SHA-256
    `CCE6FA9248AD52C347EC2E08BE1315FFA31441C395BEE6ED12654BBA799A5843`
    match the audited local artifact.
- `python tools/update_memory_catalog.py --check`
  - result: passed; the publication-complete note and catalog are synchronized.
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py tests/test_memory_catalog_sync.py -q`
  - result: `213 passed, 26 skipped`.
- `python -m pytest -q`
  - result: `367 passed, 26 skipped`.
- `git diff --check`
  - result: passed.

## Follow-up items

- Ethos 26.1 compatibility remains tracked in Issues #78, #79, and #80; no
  further BoundryMap `0.2.0` publication work remains.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
