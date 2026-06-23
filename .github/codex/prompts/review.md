# Codex pull request review

Review the current pull request diff against its base branch.

Focus on high-confidence issues that could affect correctness, safety, data loss,
release quality, or contributor workflow. Avoid style-only feedback unless it
blocks documented repository policy.

Repository-specific review expectations:

- Respect the closest `AGENTS.md` guidance for each changed file.
- Verify required validation is updated or documented when behavior, workflow,
  or tooling changes.
- For Lua script changes, check simulator/runtime assumptions and packaging
  paths against the repository docs before flagging issues.
- For workflow changes, check that contributor-facing docs stay consistent with
  `AGENTS.md`, `CONTRIBUTING.md`, `README.md`, and `docs/DEVELOPMENT.md`.

Return concise Markdown feedback. If there are no actionable findings, say so.
