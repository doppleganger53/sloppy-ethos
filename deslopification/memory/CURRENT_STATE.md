# Current State (2026-02-27)

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
- `CATALOG.md` must be regenerated with:
  - `python tools/update_memory_catalog.py`.
- `CATALOG.md` now includes both:
  - coarse `Category` classification
  - selective `Focus` classification for faster topical filtering.
- New memory notes are stored under:
  - `deslopification/memory/notes/{category}/{focus}/`.
- Issue-linked sessions must run `tools/session_preflight.py --mode issue ...`
  and may not mutate files on `main`.
- Non-issue sessions may run on `main`, but require user confirmation before
  file mutations.

## Active Tooling Decisions

- Canonical build/deploy workflow remains `tools/build.py`.
- `build.py` to `doit` migration decision: retain `build.py` (Issue #22
  evaluation complete).
- Prompt templates live in `deslopification/prompts/templates/`.

## How To Use With CATALOG

1. Start here.
2. Open `CATALOG.md` and read `Recent High-Signal Notes (Auto-generated)`.
3. If needed, continue with the full entries table filtered by `Focus` and date.
4. Read only notes required for the current task scope.
