# Session Notes 2026-09-20 — Guidance consolidation

- Artifact: `session`
- Scope: `memory`
- Concern: `workflow`

## Durable decision

AGENTS.md owns operational policy and the validation matrix. Contributor docs
and reusable prompts link to it. Retrieve project history only when relevant;
new notes capture durable findings or unresolved handoffs, not routine activity.
CURRENT_STATE.md is a retrieval index; the prior July 17 snapshot is preserved
in notes/reference/memory/CURRENT_STATE_2026-07-17_ARCHIVE.md.

Active SensorList issue prompts now reference script-local tests. Branch gates,
preflight behavior, release invariants, domain constraints, and completed prompt
history are unchanged. User authorized this guidance change on main.

## Validation

- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py tests/test_memory_catalog_sync.py tests/test_session_preflight.py -q`
  - 273 passed, 27 skipped (manual/environment-dependent documentation commands).
- `python -m pytest tests/test_build_py.py -q`
  - 70 passed.
- Used an existing Python environment with pytest; no dependency installation
  or runtime/package/release mutation was required.
