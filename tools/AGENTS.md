# AGENTS.md (tools)

This file augments root `AGENTS.md` for work under `tools/`.

## Scope

- Applies to `tools/**`.
- Prioritize contributor approachability and cross-platform behavior for all workflow tooling.

## Implementation guidance

- Keep CLI messaging explicit for non-expert contributors:
  - explain why a command blocked
  - provide copy/paste next steps
- Prefer backward-compatible CLI additions (`--json`, guided wrappers) over breaking flag changes.
- When a workflow command is documented, keep docs and tests synchronized in the same session.

## Validation additions

- Required tooling test target:
  - `python -m pytest tests/test_build_py.py -q`
- If preflight/session startup behavior changes:
  - `python -m pytest tests/test_session_preflight.py tests/test_session_start.py -q`
- Run broader suites per root matrix when changes cross docs/contracts/workflows.
