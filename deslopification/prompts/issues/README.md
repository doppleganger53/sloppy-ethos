# Issue Prompt Artifacts

## Template-first workflow

Active issue implementation uses reusable templates and live issue metadata:

- `../templates/ISSUE_RESOLUTION_TEMPLATE.md`
- `../templates/RELEASE_RESOLUTION_TEMPLATE.md`

Use `tools/session_start.py issue {N}` to infer issue kind/slug from GitHub
issue metadata, then run explicit `tools/session_preflight.py` when needed for
automation or strict overrides.

## Archived issue snapshots

Issue-specific prompt snapshots are historical artifacts only and are not the
source of truth for current issue state:

- `deslopification/prompts/issues/archive/` is the canonical archive location
- `archive/` contains point-in-time issue prompt captures
- `done/` contains prompts for completed issue work

If a snapshot is still useful for background context, copy only reusable
details into the current template-driven workflow.
