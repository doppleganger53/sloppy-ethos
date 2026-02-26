# Session Notes 2026-02-26 - Main-Only Branching And Release Conventions

## What changed

- Replaced milestone-integration branching guidance with a main-only strategy in `docs/DEVELOPMENT.md`:
  - `main` is the only long-lived branch.
  - short-lived branch naming conventions documented (`feature/`, `fix/`, `docs/`, `chore/`).
  - release-prep branch convention documented (`release/v{VERSION}`).
  - root/script version bump timing clarified for main-only flow.
  - optional prerelease `-rc.N` usage aligned to release branches.
  - release workflow updated to PR into `main`, tag on `main`, publish, then delete release branch.
- Updated contributor-facing docs to match:
  - `README.md` release policy reference now points to the main-branch policy section.
  - `CONTRIBUTING.md` workflow now requires branch-from-main with naming conventions and main-targeted PR flow.
- Added repo-level Codex branch policy in `AGENTS.md` under `## Branching Conventions`.
- Aligned release prompt template with main-only release flow:
  - `deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md`
- Aligned issue prompt defaults to `main`:
  - `deslopification/prompts/templates/ISSUE_RESOLUTION_TEMPLATE.md`
  - `deslopification/prompts/issues/ISSUE-014-sensor-values-column.md`
  - `deslopification/prompts/issues/ISSUE-022-doit-migration-evaluation.md`
  - `deslopification/prompts/issues/ISSUE-026-ethos-events-ui-output.md`
  - `deslopification/prompts/issues/ISSUE-029-milestone-versioning-policy.md`
  - `deslopification/prompts/issues/ISSUE-030-x20s-simulator-bug.md`

## Validation run(s)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: `83 passed, 14 skipped`

## Follow-up items

- Archived prompt files under `deslopification/prompts/issues/done/` still reference historical `enhancements` defaults intentionally and were not rewritten.
