# AGENTS.md

Repository-level operating policy for coding assistants working in `sloppy-ethos`.

## Repository Goals And Policy Maintenance

- Build practical Ethos utilities with an approachable, repeatable Lua workflow.
- This file is the canonical operational policy and validation matrix.
- Workflow changes remain issue-linked and documented. Update this policy and
  affected links or command examples together; contributor docs and prompts
  reference it instead of repeating policy wording.

## Priority And Scope

- These rules are repository-wide and apply to every task in this checkout.
- A standalone checkout needs only this repository's policy. If starting in
  the optional `EthosLua` workspace, read its `AGENTS.md` for routing, then
  enter this checkout.
- For a pilot's idea, the pilot describes the behavior and decides whether it
  is useful. The assistant discovers available tools and version-matched APIs,
  implements it and provides something to try. If a capability is missing,
  hand off the smallest next step and tell the pilot what result to bring back.
- Keep changes scoped to the user request. Do not perform unrelated cleanup or broad refactors.
- Preserve existing behavior unless the task explicitly requests behavior changes.

## Parent Workspace And Reference Checkouts

- The parent `EthosLua` workspace is an orchestration shell; this repository is
  the primary implementation repo.
- Sibling directories such as `ETHOS-Feedback-Community/`,
  `ETHOS-Feedback-Community-prerelease/`, and `Ethos-GPS-AccuMapv1/` are
  reference checkouts, not part of this repo's working tree.
- Treat sibling reference projects as read-only evidence unless the user
  explicitly asks to update or modify them.
- Pull reference updates only with commands scoped to that checkout, for
  example `git -C ../ETHOS-Feedback-Community pull --ff-only`.
- Do not stage parent-workspace or sibling-reference files into `sloppy-ethos`.
- Before copying code, docs, or assets from a reference project, verify the
  source license and record attribution in the relevant repo docs or session
  notes.

## Root-Cause Strategy

- Prefer root-cause fixes over compatibility shims, legacy aliases, or temporary band-aids.
- Diagnose and clear local state issues first (for example: IDE/test discovery caches) when failures are caused by stale metadata rather than code defects.
- Only introduce compatibility layers when explicitly requested or when a real external compatibility requirement is confirmed.
- If a compatibility layer is approved, document why it is needed and define a removal condition in session notes.

## Required Startup Workflow

1. If launched from the parent `EthosLua` workspace, enter `sloppy-ethos/`
   before running repository commands. Check `git status --short --branch`.
2. Determine whether the task is issue-linked (a GitHub issue number/URL or a
   prompt under `deslopification/prompts/issues/`) or non-issue work. Follow
   the [session branch gate](#session-branch-gate).
3. For issue-linked work, run preflight before editing:
   - `python tools/session_preflight.py --mode issue --issue-number {N} --issue-kind {enhancement|bug|docs|chore} --slug {short-slug}`
4. Read linked guidance when its trigger applies. Consult the
   [current-state index](deslopification/memory/CURRENT_STATE.md) and search
   [CATALOG.md](deslopification/memory/CATALOG.md) only when prior decisions
   or unresolved work matter; open matching references or notes, not the
   complete history. Verify version-dependent facts against current evidence.
5. Check `VERSION` and affected script versions when the task affects build or
   release state. When changing workflow commands, check the relevant examples
   in [README.md](README.md) and [DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Validation Policy

- Validation is mandatory for any change.
- Use static or fixture checks for local logic and a supported
  [isolated WebSimulator](tools/sim/harness/README.md) for simulator behavior.
  Write to a live Suite installation only when the task calls for updating
  that named target. Claim real-radio behavior only from an observation on the
  named hardware and firmware.
- Run the minimum test/check commands based on the files touched:

### Validation Matrix

- Documentation or guidance changes (`AGENTS.md`, `README.md`, `docs/`,
  `CONTRIBUTING.md`, PR/issue templates):
  - `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- Lua widget behavior changes (`scripts/**/*.lua`):
  - `luac -p scripts/SensorList/main.lua`
  - `python -m pytest scripts/SensorList/tests -q`
  - For another script, run that script's local test folder, for example
    `python -m pytest scripts/BoundryMap/tests -q`
- Build/tooling changes (`tools/`, Python build/test harness):
  - `python -m pytest tests/test_build_py.py -q`
  - For simulator harness changes, also run `python -m pytest tests/test_sim_harness.py -q`
- Formatting changes:
  - `stylua --config-path tools/config/stylua.toml scripts`
- Packaging changes: build the affected package using
  [Core Commands](docs/DEVELOPMENT.md#core-commands) and inspect its contents.
- Broad or cross-cutting changes:
  - `python -m pytest -q`

## Timeout And Failure Handling

- If a required validation command times out or hangs:
  1. rerun once with a higher timeout;
  2. if it still fails, report the exact command and failure mode to the user;
  3. do not claim validation passed when it did not complete.

## Documentation And Memory Sync

- Update affected user-facing behavior and command documentation in the same
  session; link to this policy rather than maintaining parallel copies.
- Preserve existing notes. Add a project-memory note for a durable decision,
  reusable finding, or unresolved handoff; routine activity and command
  transcripts do not require a new historical note.
- When a note is warranted, record what changed, evidence/validation, and
  follow-up items using the [memory guide](deslopification/memory/README.md)
  and [note template](deslopification/memory/SESSION_NOTE_TEMPLATE.md).
- Place notes under `deslopification/memory/notes/{artifact}/{scope}/` with
  matching `Artifact`, `Scope`, and `Concern` metadata. Use `ethos-platform`
  for reusable runtime/API/simulator findings and script scopes for local facts.
- After adding notes, run `python tools/update_memory_catalog.py`. Update
  the current-state index only when a durable retrieval pointer changes.

## Bug Issue Hygiene

- When creating bug issues, use `.github/ISSUE_TEMPLATE/bug_report.md`.
- Ensure the issue has the `bug` label.
- Include runtime environment details for both Ethos and non-Ethos bug types.
- Use stable screenshot links (GitHub issue uploads or `raw.githubusercontent.com`), not `blob` image URLs.

## Branching Conventions

- `main` is the only long-lived branch.
- Create short-lived branches from `main` using:
  - `feature/{issue-number}-{short-slug}` for enhancements
  - `fix/{issue-number}-{short-slug}` for bugs
  - `docs/{issue-number}-{short-slug}` for docs/process changes
  - `chore/{issue-number}-{short-slug}` for tooling/maintenance changes
- Use `release/v{VERSION}` for repo release-prep work.
- Use `release/{ProjectName}-v{VERSION}` for script release-prep work.
- Delete release-prep branches after merge/release.
- Open PRs into `main` and include linked-closing keywords when applicable.

## PR Merge Strategy

- Default merge method for normal issue PRs (`feature/`, `fix/`, `docs/`, `chore/`) is `squash`.
- Use `merge commit` for release-prep branches (`release/v{VERSION}` or `release/{ProjectName}-v{VERSION}`) and for PRs where branch commit lineage must be preserved.
- `rebase` merge is not the default practice for this repository.

## Session Branch Gate

- Issue-linked work is any session tied to a GitHub issue number/URL or any
  session using `deslopification/prompts/issues/*.md`.
- Issue-linked work must not mutate repository files while on `main`.
- If issue preflight fails on `main`, create/switch to the recommended
  short-lived branch before editing.
- Non-issue work may be performed on `main`, but agents must ask the user for
  confirmation before mutating files on `main`.

## Release And Versioning Guardrails

- Root `VERSION` is the repository version source of truth.
- Script artifact versions are sourced from `scripts/{ProjectName}/VERSION`.
- Single-script dist ZIP naming remains `dist/{ProjectName}-{version}.zip`.
- Multi-script dist bundles are an explicit naming exception and are intentionally unversioned (`dist/sloppy-ethos_scripts.zip`).
- `README.md` `Download Latest Script Releases` is a published-release surface, not a preview of unreleased script directories.
- Keep release steps aligned with repository documentation.

## Script-Specific Notes

- Reusable Ethos operating guidance lives in:
  - `deslopification/memory/notes/reference/ethos-platform/EthosPlatform.md`
- SensorList-specific operating guidance lives in:
  - `deslopification/memory/notes/reference/sensorlist/SensorList.md`

## Definition Of Done (Before Commit/Push)

- Required validation commands for touched files completed in this session.
- Say what was observed, on which target, and what the pilot still needs to try.
- Parent workspace, repository status, and `.gitignore` reviewed for
  environment-specific files, sibling-reference changes, and sensitive data
  risks.
- Any potential security concern (PII, PHI, secrets, unsafe config, API keys, auth tokens) explicitly called out to the user.
