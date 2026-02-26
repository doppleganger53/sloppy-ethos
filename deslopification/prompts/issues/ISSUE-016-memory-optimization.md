# Prompt: Implement Issue #16 (Memory Optimization)

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/16`
- Title: `[Enhancement] - Memory optimization`
- Labels: `enhancement`
- Snapshot state: open on `2026-02-26`

## Mission

Optimize repository memory/process artifacts for faster, lower-churn agentic
sessions while preserving critical historical traceability.

## Required Context

- `AGENTS.md`
- `README.md`
- `docs/DEVELOPMENT.md`
- `CONTRIBUTING.md`
- Entire `deslopification/memory/` tree
- Current prompt assets in `deslopification/prompts/`
- Recent history: `git log --oneline -n 60`

## Scope

- In scope:
  - Analyze memory-note sprawl and propose compaction strategy.
  - Implement selected high-value, low-risk improvements.
  - Improve structure for cold-start agent sessions.
  - Update guidance docs if process changes are introduced.
- Out of scope:
  - Deleting historical records without replacement summaries.
  - Changing source-code behavior unrelated to workflow/memory.
  - Broad refactors outside memory/prompt/process artifacts.

## Technical Constraints

- Keep root-cause-first policy intact.
- Preserve auditability: what changed, why, and validation evidence.
- If introducing compatibility bridge text, include removal condition.

## Suggested Execution Plan

1. Inventory current memory artifacts and classify by ongoing value.
2. Draft grouped recommendations with dependency order and impact.
3. Implement approved baseline group directly in repo artifacts.
4. Add/update reusable prompt/template guidance where justified.
5. Record a fresh session note capturing rationale and follow-ups.

## Required Deliverables

1. Actionable memory optimization summary.
2. Concrete file changes (not only suggestions).
3. Clear "implemented now" vs "recommended next" split.
4. Follow-up backlog list grouped by dependency/benefit.

## Validation (Required)

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- If tooling/tests are touched: run matching matrix command(s) from `AGENTS.md`
- If changes are broad: `python -m pytest -q`

## Done Criteria

1. Memory/process improvements are concrete and measurable.
2. Repo guidance remains internally consistent.
3. Required validation passes.
4. Session note documents changes, validations, and follow-up items.
