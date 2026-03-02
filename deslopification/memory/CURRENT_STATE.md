# Current State (2026-03-02)

This file is the high-signal memory entrypoint for cold-start sessions.
Historical detail remains in individual session notes referenced from
`CATALOG.md`.

## Repository Baseline

- Long-lived branch model: `main` only.
- Root version source of truth: `VERSION`.
- Script artifact versions: `scripts/{ProjectName}/VERSION`.
- Single-script ZIP naming: `dist/{ProjectName}-{version}.zip`.
- Multi-script ZIP naming exception: `dist/sloppy-ethos_scripts.zip`.
- Repo release asset baseline: include `dist/sloppy-ethos_scripts.zip` alongside
  applicable single-script ZIP artifacts.

## Workflow Baseline

- Root-cause-first policy is mandatory; avoid compatibility shims unless
  explicitly justified.
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
- `CATALOG.md` entries and distributions index note artifacts under
  `deslopification/memory/notes/**` (control files are listed separately).
- `CATALOG.md` snapshot byte totals are computed from normalized text line
  endings to remain stable across Windows (`CRLF`) and Linux (`LF`) checkouts.
- Focus distribution lines in `CATALOG.md` are sourced from per-focus `.desc`
  files stored in each `notes/{category}/{focus}/` folder.
- Session notes should use specific focus classifiers (for example,
  `build-tooling`, `docs-process`, `issue-lifecycle`, `repo`, `lua-ethos`) and
  avoid `general` by default.
- Ethos Lua script/widget/tool notes (SensorList, ethos_events, and future
  similar scripts) should use focus `lua-ethos`.
- Session-note compaction summaries now use weekly rollups stored under:
  - `deslopification/memory/notes/weekly-summary/{focus}/`
  - `SUMMARY_YYYY-MM-DD_to_YYYY-MM-DD.md`
- New memory notes are stored under:
  - `deslopification/memory/notes/{category}/{focus}/`.
- Issue-linked sessions must run `tools/session_preflight.py --mode issue ...`
  and may not mutate files on `main`.
- Release scope is explicit and required in release execution:
  - `repo` for repository-level release work.
  - `script` for installable script artifact releases.
- GitHub release bodies should be generated from `CHANGELOG.md` with:
  - `tools/write_release_notes.py`
  - then published with `gh release create --notes-file` to preserve markdown formatting.
- Script manual gate issues block only matching `script` releases and should be
  treated as out of scope for `repo` releases unless explicitly included.
- `tools/session_preflight.py` supports release-scope guards for issue sessions:
  - `--release-kind {repo|script}`
  - `--project {ProjectName}` (`script` only)
  - one or more `--script-gate-issue {N}` flags (`script` only)
- Hybrid PR merge strategy baseline:
  - `squash` for normal issue PRs (`feature/`, `fix/`, `docs/`, `chore/`).
  - `merge commit` for `release/v{VERSION}` PRs and lineage-sensitive cases.
  - `rebase` is not the default merge method.
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
