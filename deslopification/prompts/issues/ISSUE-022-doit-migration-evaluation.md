# Prompt: Implement Issue #22 (Evaluate `build.py` -> `doit` Migration)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/22`
- Title: `[Enhancement] Evaluate migration from build.py to doit task runner`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`

## Mission

Deliver a decision-quality evaluation for migrating to `doit`, and only proceed
to implementation if the decision evidence supports it and acceptance criteria
remain tightly scoped.

## Required Context

- `AGENTS.md`
- `README.md`
- `docs/DEVELOPMENT.md`
- `tools/build.py`
- `tools/build_help.txt`
- `tests/test_build_py.py`
- `tests/test_docs_commands.py`
- `tests/test_docs_contracts.py`
- memory notes for issue #21 closure and build workflow updates

## Scope

- In scope:
  - Compare current `build.py` responsibilities to `doit` task model.
  - Produce a clear go/no-go decision record with cost/benefit and risks.
  - If go: implement minimal migration slice with explicit compatibility plan.
  - Keep command docs/tests synchronized with any command-surface change.
- Out of scope:
  - Full tooling rewrite without decision record.
  - Long-lived dual systems with undefined retirement date.
  - Breaking contributor workflow without deprecation messaging.

## Technical Constraints

- Root-cause-first: do not migrate for novelty.
- Compatibility shim allowed only if explicitly justified with removal criteria.
- Keep release/version guardrails aligned with existing docs and `AGENTS.md`.

## Suggested Execution Plan

1. Build a responsibility matrix (`build.py` feature parity vs `doit` tasks).
2. Write decision record under `docs/` (or existing decisions location).
3. If go-decision:
   - implement minimal viable task graph,
   - preserve current user-visible commands via adapter or mapped aliases,
   - define deprecation timeline.
4. Update docs and docs-contract tests in same session.

## Validation (Required)

- `python -m pytest tests/test_build_py.py -q`
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- If migration touches many components: `python -m pytest -q`

## Done Criteria

1. Decision record exists and is technically defensible.
2. If migration proceeded, parity + compatibility/deprecation are explicit.
3. Docs and tests reflect actual command behavior.
4. Required validation passes and session notes are recorded.
