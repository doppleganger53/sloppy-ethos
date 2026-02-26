# Decision Record: Issue #22 - Evaluate `build.py` -> `doit` Migration

- Date: 2026-02-26
- Issue: [#22](https://github.com/doppleganger53/sloppy-ethos/issues/22)
- Status: Accepted
- Decision: No-go for migration now; keep `tools/build.py` as the canonical workflow command surface.

## Context

Issue #22 requested a decision-quality evaluation of migrating from the current
`tools/build.py` orchestration to `doit`, with implementation only if evidence
supported the change.

Current build workflow state at evaluation time:

- `tools/build.py` provides `--dist`, `--deploy`, `--clean`, `--sim-radio`,
  `--project` repeatability, and `--help`.
- Command behavior is documented in `README.md`, `docs/DEVELOPMENT.md`, and
  `tools/build_help.txt`.
- Tooling tests are mature (`tests/test_build_py.py`) and recently expanded to
  full line coverage for `tools/build.py`.

## Responsibility Matrix

| Responsibility | Current `build.py` status | `doit` migration delta | Risk assessment |
| --- | --- | --- | --- |
| Command surface for contributors | Stable and documented | Requires new command entrypoint and retraining | Medium |
| Lua parse preflight (`luac`) | Implemented and tested | Must be re-wired into task graph | Low |
| Packaging rules and ZIP naming policy | Implemented and tested | Must preserve all naming/version rules | Medium |
| Multi-project bundle behavior | Implemented and tested | Task dependency graph adds complexity | Medium |
| Simulator deploy/clean with model-path resolution | Implemented and tested | Requires careful stateful task behavior | Medium |
| Help text and docs contract alignment | Implemented and tested | Requires dual docs during transition | Medium |
| Dependency footprint | Python stdlib only | Adds external dependency (`doit`) | Medium |

## Options Considered

1. Keep `tools/build.py` as-is and continue incremental hardening.
2. Migrate now to `doit` with one-release compatibility shim in `build.py`.

## Cost / Benefit Summary

| Criterion | Keep `build.py` | Migrate to `doit` now |
| --- | --- | --- |
| Delivery risk | Low | Medium-High |
| Contributor disruption | Low | Medium |
| Short-term engineering effort | Low | Medium-High |
| Near-term functional gain | Low-Moderate | Moderate |
| Long-term extensibility | Moderate | High |

The current repository scale and command surface do not justify migration cost
or transitional complexity right now. Existing behavior is already reliable,
well-tested, and documented.

## Decision

Do not migrate to `doit` in issue #22. Keep `tools/build.py` as the single
command orchestration mechanism and continue targeted improvements there.

## Rationale

- Root-cause analysis did not identify an active defect caused by the current
  task runner approach.
- Current tooling already satisfies current build/deploy/clean requirements.
- A migration now would create avoidable dual-system overlap (or command churn)
  with limited near-term value.

## Compatibility / Deprecation

- No compatibility shim introduced because migration is deferred.
- No command deprecations introduced.
- Existing documented commands remain unchanged.

## Revisit Conditions

Re-open migration evaluation if one or more conditions become true:

1. Build workflow expands into several independent task families where graph
   execution/caching materially reduces CI or local runtime.
2. `tools/build.py` accumulates repeated orchestration logic that cannot be
   contained with small, testable refactors.
3. Contributor friction appears around composing reusable workflow steps that a
   declarative task graph would materially simplify.
