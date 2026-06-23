# Session Notes 2026-06-23 - Codex Review Fallback

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `workflow`
- Store this file under:
  - `deslopification/memory/notes/session/repo/`

## What changed

- Investigated PR #98 `@codex review` comments. GitHub API showed the first
  Codex review ran against commit `ac232f887c`, while later comments did not
  receive visible reactions or new review output after the PR advanced to
  `864d6efc`.
- Added a repository-owned GitHub Actions fallback for exact `@codex review`
  issue comments on pull requests and manual workflow dispatch by PR number.
- Added a dedicated Codex review prompt and contributor docs describing the
  fallback and required `OPENAI_API_KEY` repository secret.

## Validation

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`

## Follow-up items

- Ensure the GitHub repository has an `OPENAI_API_KEY` Actions secret before
  relying on the fallback workflow.
- Keep native Codex Cloud code review settings enabled; the fallback is a
  repository-owned safety net, not a replacement for the connector.
