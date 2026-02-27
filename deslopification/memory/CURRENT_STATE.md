# Current State (2026-02-26)

This file is the high-signal memory entrypoint for cold-start sessions.
Historical detail remains in individual session notes referenced from
`CATALOG.md`.

## Repository Baseline

- Long-lived branch model: `main` only.
- Root version source of truth: `VERSION`.
- Script artifact versions: `scripts/{ProjectName}/VERSION`.
- Single-script ZIP naming: `dist/{ProjectName}-{version}.zip`.
- Multi-script ZIP naming exception: `dist/sloppy-ethos_scripts.zip`.

## Workflow Baseline

- Root-cause-first policy is mandatory; avoid compatibility shims unless
  explicitly justified and time-bounded.
- Validation is mandatory for all changes per `AGENTS.md` matrix.
- Documentation/process changes require docs contract validation.
- Session memory updates are required for meaningful workflow/behavior changes.
- Memory cold starts now use:
  - `README.md` -> `CURRENT_STATE.md` -> task-filtered lookup in `CATALOG.md`.

## Active Tooling Decisions

- Canonical build/deploy workflow remains `tools/build.py`.
- `build.py` to `doit` migration decision: retain `build.py` (Issue #22
  evaluation complete).
- Prompt templates live in `deslopification/prompts/templates/`.

## High-Value Recent Notes

- Main-only branching and release conventions:
  `SESSION_NOTES_2026-02-26_MAIN_ONLY_BRANCHING_CONVENTIONS.md`
- Prompt/template hardening and relocation:
  `SESSION_NOTES_2026-02-26_PROMPT_TEMPLATE_HARDENING.md`
  `SESSION_NOTES_2026-02-26_PROMPT_TEMPLATE_RELOCATION.md`
- Memory optimization baseline and index compaction:
  `SESSION_NOTES_2026-02-26_ISSUE_16_MEMORY_OPTIMIZATION.md`
- Build tooling evaluation (do not migrate now):
  `SESSION_NOTES_2026-02-26_ISSUE_22_DOIT_MIGRATION_EVALUATION.md`
- Latest repo metadata update:
  `SESSION_NOTES_2026-02-26_REPO_METADATA_DESCRIPTION_TOPICS.md`

## How To Use With CATALOG

1. Start here.
2. Open `CATALOG.md` and filter by date+topic.
3. Read only notes required for the current task scope.
