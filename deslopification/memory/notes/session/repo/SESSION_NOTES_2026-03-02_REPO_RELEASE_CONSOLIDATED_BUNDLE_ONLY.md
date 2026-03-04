# Session Notes 2026-03-02 - Repo Releases Consolidated Bundle Only

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `release`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Updated repo release workflow docs so future repository releases attach only `dist/sloppy-ethos_scripts.zip`.
- Removed repo release instructions that previously attached individual script ZIP assets alongside the consolidated bundle.
- Synced the release prompt template and current-state memory to the new repo release asset baseline.
- Files touched:
  - `README.md`
  - `docs/DEVELOPMENT.md`
  - `deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md`
  - `deslopification/memory/CURRENT_STATE.md`
  - `CHANGELOG.md`
  - `deslopification/memory/notes/session-note/release-versioning/SESSION_NOTES_2026-03-02_REPO_RELEASE_CONSOLIDATED_BUNDLE_ONLY.md`
  - `deslopification/memory/CATALOG.md`

## Why

- Root cause or objective:
  - Align the documented repo release contract with the simpler distribution model: repository releases ship the consolidated bundle only, while script-specific releases remain responsible for individual script ZIP assets.
- Scope guardrails:
  - No build behavior changed in `tools/build.py`.
  - No previously published releases were modified in this session.

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `90 passed, 14 skipped`

## Follow-up items

- Future repo releases should omit individual script ZIPs from `gh release create` and attach only the consolidated bundle.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
