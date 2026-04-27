# AGENTS.md

Repository-level operating policy for agent sessions in `sloppy-ethos`.

## Scope and precedence

- This file applies repository-wide unless a closer `AGENTS.md` exists.
- Nested contracts are active in:
  - `scripts/SensorList/AGENTS.md`
  - `scripts/ethos_events/AGENTS.md`
  - `tools/AGENTS.md`
- Keep changes scoped to the user request. Do not perform unrelated cleanup or broad refactors.
- Preserve existing behavior unless the task explicitly requests behavior changes.

## Repository goals and workflow model

- Repository goals:
  - Build practical Ethos utilities and scripts.
  - Keep Lua contribution workflow approachable for non-experts.
  - Prioritize repeatable, low-maintenance workflows that help contributors spend more time flying and less time on process overhead.
- Workflow model is intentionally mutable:
  - Workflow/process metadata can evolve when repository goals or contributor needs change.
  - Workflow mutations must be issue-linked, documented, and kept consistent across `AGENTS.md`, `CONTRIBUTING.md`, `README.md`, and `docs/DEVELOPMENT.md` in the same session.
- Stability rule:
  - Treat this file as the operational policy source of truth for agent execution guardrails.

## Root-cause strategy

- Prefer root-cause fixes over compatibility shims, legacy aliases, or temporary band-aids.
- Diagnose and clear local state issues first (for example stale IDE/test metadata) when failures are environmental.
- Introduce compatibility layers only when explicitly requested or when a real external compatibility requirement is confirmed.
- If a compatibility layer is approved, document why it is needed and define a removal condition in session notes.

## Required startup workflow

1. Read memory entrypoint files:
   - `deslopification/memory/README.md`
   - `deslopification/memory/CURRENT_STATE.md`
2. Use `deslopification/memory/CATALOG.md` to open only task-relevant notes.
3. Check branch/worktree state:
   - `git status --short --branch`
4. Confirm active repository version from `VERSION`.
5. Before introducing or changing workflow commands, cross-check:
   - `README.md`
   - `docs/DEVELOPMENT.md`
6. Determine session type:
   - issue-linked work: tied to a GitHub issue URL/number or explicitly run through `tools/session_start.py issue {N}`
   - non-issue work: all other sessions

## Session branch gate and preflight

- Issue-linked work must not mutate repository files while on `main`.
- Recommended issue startup:
  - `python tools/session_start.py issue {N}`
- Explicit issue preflight contract (automation/power-user path):
  - `python tools/session_preflight.py --mode issue --issue-number {N} --issue-kind {enhancement|bug|docs|chore} --slug {short-slug}`
- If issue preflight blocks on `main`, create/switch to the recommended short-lived issue branch before editing.
- Non-issue work may run on `main`, but agents must ask the user before mutating files on `main`.
- Optional strict branch enforcement for issue sessions:
  - `python tools/session_preflight.py ... --strict-branch-match`

## Branching and merge strategy

- `main` is the only long-lived branch.
- Use short-lived branches from `main`:
  - `feature/{issue-number}-{short-slug}` for enhancements
  - `fix/{issue-number}-{short-slug}` for bugs
  - `docs/{issue-number}-{short-slug}` for docs/process changes
  - `chore/{issue-number}-{short-slug}` for maintenance/tooling changes
- Release-prep branches:
  - `release/v{VERSION}` for repo release work
  - `release/{ProjectName}-v{VERSION}` for script release work
- Open PRs into `main` and include issue-closing keywords when applicable.
- Default merge strategy:
  - `squash` for normal issue PRs (`feature/`, `fix/`, `docs/`, `chore/`)
  - `merge commit` for release-prep branches and lineage-sensitive PRs
  - `rebase` is not the repository default

## Validation policy

- Validation is mandatory for every change.
- Debugging-session work on simulator-visible Lua behavior must deploy the touched script to the configured Ethos simulator before closeout.
- For SensorList debugging, minimum deploy command:
  - `python tools/build.py --project SensorList --deploy`

### Validation matrix

- Documentation changes (`README.md`, `docs/`, `CONTRIBUTING.md`, PR/issue templates):
  - `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- Lua widget behavior changes (`scripts/**/*.lua`):
  - `luac -p scripts/SensorList/main.lua`
  - `python -m pytest tests/test_sensorlist_widget.py -q`
- Build/tooling changes (`tools/`, Python build/test harness):
  - `python -m pytest tests/test_build_py.py -q`
- Broad or cross-cutting changes:
  - `python -m pytest -q`

### Timeout/failure handling

If a required validation command times out or hangs:

1. rerun once with a higher timeout;
2. if it still fails, report the exact command and failure mode;
3. do not claim validation passed when validation did not complete.

## Prompt and issue-intake policy

- Template-first prompting is the active model:
  - `deslopification/prompts/templates/ISSUE_RESOLUTION_TEMPLATE.md`
  - `deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md`
- Snapshot issue prompts under `deslopification/prompts/issues/archive/` and `deslopification/prompts/issues/done/` are historical references only.
- Issue intake should map to preflight kinds:
  - `bug`, `docs`, `enhancement`, `chore`
- Maintenance/refactor intake is governed under `chore` branch-prefix policy.

## Documentation and memory sync

- When workflow, behavior, or contributor process changes, update docs in the same session.
- Record meaningful work in `deslopification/memory/` with:
  - what changed
  - validation run(s)
  - follow-up items
- Store new notes under:
  - `deslopification/memory/notes/{artifact}/{scope}/`
- Every note must declare explicit `Artifact`, `Scope`, and `Concern` metadata.
- Prefer reusable scopes:
  - use `ethos-platform` for reusable Ethos runtime/API/simulator knowledge
  - use script scopes (for example `sensorlist`, `ethos-events`) only for script-local behavior/history
- Keep memory indexes synchronized when adding notes:
  - `python tools/update_memory_catalog.py`
  - update `deslopification/memory/CURRENT_STATE.md` when durable workflow/behavior decisions change

## Bug issue hygiene

- Use `.github/ISSUE_TEMPLATE/bug_report.md` for bug reports.
- Ensure bug issues carry the `bug` label.
- Include runtime environment details for both Ethos and non-Ethos bug types.
- Use stable screenshot links (GitHub issue uploads or `raw.githubusercontent.com`), not `blob` image URLs.

## Release and versioning guardrails

- Root `VERSION` is the repository version source of truth.
- Script artifact versions are sourced from `scripts/{ProjectName}/VERSION`.
- Single-script dist ZIP naming remains `dist/{ProjectName}-{version}.zip`.
- Multi-script dist bundles are an explicit naming exception and are intentionally unversioned (`dist/sloppy-ethos_scripts.zip`).
- `README.md` `Download Latest Script Releases` is a published-release surface, not a preview of unreleased script directories.
- Keep release steps aligned with repository docs and release templates.

## Definition of done (before commit/push)

- Required validation commands for touched files completed in this session.
- If this was a simulator debugging session for Lua behavior, updated script deployed in this session.
- Workspace and `.gitignore` reviewed for environment-specific files and sensitive data risks.
- Any potential security concern (PII, PHI, secrets, unsafe config, API keys, auth tokens) explicitly called out.
