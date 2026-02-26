# Prompt Template: Agentic Issue Resolution

Use this template to drive end-to-end implementation for one GitHub issue.
Replace all `{...}` placeholders before execution.

## Canonical Issue

- Issue number: `{ISSUE_NUMBER}`
- Title: `{ISSUE_TITLE}`
- URL: `{ISSUE_URL}`
- Labels: `{ISSUE_LABELS}`
- Snapshot date: `{YYYY-MM-DD}`

## Mission

Implement issue `{ISSUE_NUMBER}` with a root-cause-first approach, scoped to
the stated acceptance criteria and repository policies.

## Operating Mode

You are Codex working directly in this repository.

- Keep changes focused on this issue only.
- Preserve existing behavior unless this issue explicitly changes behavior.
- Prefer minimal, high-signal diffs over broad refactors.
- If uncertain, inspect code/tests/history before editing.

## Mandatory Startup Workflow

1. Read latest relevant note in `deslopification/memory/`.
2. Run `git status --short --branch`.
3. Confirm root version from `VERSION`.
4. Cross-check workflow commands in:
   - `README.md`
   - `docs/DEVELOPMENT.md`
5. Review issue body/comments and linked artifacts.

## Repo Context To Load

- Primary implementation files: `{TARGET_FILES}`
- Related tests: `{RELATED_TEST_FILES}`
- Related docs: `{RELATED_DOC_FILES}`
- Related memory notes: `{RELATED_MEMORY_FILES}`
- Recent commit context (`git log --oneline -n 30`) for regressions and intent.

## Scope

- In scope:
  - `{IN_SCOPE_1}`
  - `{IN_SCOPE_2}`
  - `{IN_SCOPE_3}`
- Out of scope:
  - `{OUT_SCOPE_1}`
  - `{OUT_SCOPE_2}`

## Execution Plan

1. Reproduce or define baseline behavior.
2. Implement the smallest coherent change-set.
3. Add/update tests for new behavior and regressions.
4. Update docs only when behavior/workflow changes.
5. Record session notes in `deslopification/memory/`.

## Implementation Constraints

- Root-cause fix preferred over compatibility shims.
- If a compatibility layer is unavoidable:
  - explain why it is needed now;
  - define a clear removal condition.
- No destructive git commands.
- No unrelated cleanup.
- Keep temporary analysis artifacts in `deslopification/memory/temp/`.

## Validation Matrix (Required)

Choose the minimum required checks based on touched files:

- Documentation-only changes:
  - `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- Lua widget/script behavior changes (`scripts/**/*.lua`):
  - `luac -p scripts/SensorList/main.lua`
  - `python -m pytest tests/test_sensorlist_widget.py -q`
- Build/tooling changes (`tools/`, Python harness):
  - `python -m pytest tests/test_build_py.py -q`
- Broad/cross-cutting changes:
  - `python -m pytest -q`

If a required command times out/hangs:

1. rerun once with higher timeout;
2. if still failing, report exact command and failure mode;
3. do not claim validation passed.

## Delivery Contract

Return:

1. Change summary mapped to acceptance criteria.
2. File list with key edits and rationale.
3. Validation commands + pass/fail results.
4. Risks, edge cases, and follow-up items.
5. Proposed PR title/body including `Closes #{ISSUE_NUMBER}`.

## Quality Gates

- Acceptance criteria satisfied.
- Required validation completed in this session.
- `.gitignore`/workspace reviewed for accidental sensitive artifacts.
- Security concerns (PII/PHI/secrets/unsafe config) explicitly called out.

## Best-Practice Anchors

Apply these external references when deciding workflow behavior:

- GitHub: linking PRs to issues with closing keywords  
  `https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue`
- GitHub: milestones for grouped delivery tracking  
  `https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/about-milestones`
- GitHub: release management practices  
  `https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository`
- Semantic Versioning 2.0.0  
  `https://semver.org/`
- Keep a Changelog  
  `https://keepachangelog.com/en/1.1.0/`
- Kubernetes PR flow (small, focused, linked work)  
  `https://www.kubernetes.dev/docs/guide/pull-requests/`
- NumPy development workflow (small coherent PR discipline)  
  `https://numpy.org/devdocs/dev/development_workflow.html`
