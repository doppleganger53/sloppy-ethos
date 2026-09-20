# Prompt Template: Agentic Issue Resolution

Use this template to drive end-to-end implementation for one GitHub issue.
Replace all `{...}` placeholders before execution.

## Canonical Issue

- Issue number: `{ISSUE_NUMBER}`
- Title: `{ISSUE_TITLE}`
- URL: `{ISSUE_URL}`
- Labels: `{ISSUE_LABELS}`
- Issue kind: `{ISSUE_KIND}` (`enhancement` | `bug` | `docs` | `chore`)
- Short slug: `{SHORT_SLUG}`
- Snapshot date: `{YYYY-MM-DD}`
- Target branch (default by issue kind, never `main`): `{TARGET_BRANCH}`

## Mission

Implement issue `{ISSUE_NUMBER}` with a root-cause-first approach, scoped to
the stated acceptance criteria and repository policies.

## Operating Policy And Startup

Follow [AGENTS.md](../../../AGENTS.md) and its
[startup workflow](../../../AGENTS.md#required-startup-workflow).
Read the issue body/comments and linked artifacts. Consult relevant project
memory only when prior decisions or unresolved work affect this issue.
The task-specific context and acceptance criteria below supplement that policy.

## Branch And Worktree Gate (Required Before Editing)

1. Run issue preflight before editing:
   - `python tools/session_preflight.py --mode issue --issue-number {ISSUE_NUMBER} --issue-kind {ISSUE_KIND} --slug {SHORT_SLUG}`
2. Confirm target branch from prompt: `{TARGET_BRANCH}`.
3. Run:
   - `git branch --show-current`
   - `git status --porcelain`
4. If preflight blocks due to `main`, create/switch to the recommended issue branch before editing.
5. If branch differs from target, stop and confirm how to handle local changes before switching.
6. If worktree is dirty, stop and confirm whether to stash, commit, or continue as-is.
7. After any branch switch, sync and re-check:
   - `git pull --ff-only origin {TARGET_BRANCH}`
   - `git status --short --branch`

## Repo Context To Load

- Primary implementation files: `{TARGET_FILES}`
- Related tests: `{RELATED_TEST_FILES}`
- Related docs: `{RELATED_DOC_FILES}`
- Related memory notes: `{RELATED_MEMORY_FILES}`
- Recent commit context (`git log --oneline -n 30`) for regressions and intent.
- Runtime evidence/artifacts (if applicable): `{RUNTIME_ARTIFACTS}`

## Interaction Contract (For Input/Event Issues)

Document the runtime interaction contract before editing:

- Runtime target(s): `{RUNTIME_TARGETS}`
- Observed category/value mappings: `{EVENT_MAPPINGS}`
- Event consume/pass-through policy: `{EVENT_CONSUMPTION_POLICY}`
- Navigation guardrails (keys/gestures that must remain unchanged): `{NAVIGATION_GUARDRAILS}`

## UI Output Contract (For UI-Facing Issues)

- Visible line-prefix/tag policy: `{UI_PREFIX_POLICY}`
- Readability expectations (font density, truncation, line budget): `{UI_READABILITY_POLICY}`
- Empty/error/status state text requirements: `{UI_STATE_COPY}`

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
5. Apply [documentation and memory sync](../../../AGENTS.md#documentation-and-memory-sync)
   when the result is durable or needed for handoff.

## Implementation Constraints

- Root-cause fix preferred over compatibility shims.
- If a compatibility layer is unavoidable:
  - explain why it is needed now;
  - define a clear removal condition.
- If helper fallback loaders exist, preserve functional parity for:
  - accepted options
  - returned values
  - output shape used by callers
- No destructive git commands.
- No unrelated cleanup.
- Keep temporary analysis artifacts in `deslopification/memory/temp/`.
- Treat sibling reference checkouts as read-only evidence unless the issue or
  user request explicitly asks to update them. Before copying reference code,
  docs, or assets, verify license and attribution requirements.

## Validation (Required)

Run the checks for touched files in the
[canonical validation matrix](../../../AGENTS.md#validation-matrix).
Apply its [failure handling](../../../AGENTS.md#timeout-and-failure-handling)
and report commands with actual pass/fail results.

## Manual Acceptance Scenarios (Required For UI/Input Changes)

Define and execute at least one scenario for each bucket:

- Positive behavior: `{MANUAL_POSITIVE_1}`
- Negative/guardrail behavior: `{MANUAL_GUARDRAIL_1}`
- Regression prevention behavior: `{MANUAL_REGRESSION_1}`
- Navigation/system interoperability behavior: `{MANUAL_NAVIGATION_1}`

Report results in the delivery summary; retain a note when the
[documentation and memory policy](../../../AGENTS.md#documentation-and-memory-sync) applies.

## Delivery Contract

Return:

1. Change summary mapped to acceptance criteria.
2. File list with key edits and rationale.
3. Validation commands + pass/fail results.
4. Manual acceptance scenario results (if applicable).
5. Risks, edge cases, and follow-up items.
6. Proposed PR title/body including `Closes #{ISSUE_NUMBER}`.
7. Issue closure sequencing:
   - do not manually close the issue before PR merge;
   - rely on merged PR closing keywords for automatic issue closure.

## Quality Gates

- Acceptance criteria satisfied.
- [Definition of done](../../../AGENTS.md#definition-of-done-before-commitpush)
  satisfied, including validation and sensitive-artifact review.
- Durable findings or unresolved handoffs recorded under the
  [memory policy](../../../AGENTS.md#documentation-and-memory-sync), with the
  catalog regenerated if a note was added.

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
