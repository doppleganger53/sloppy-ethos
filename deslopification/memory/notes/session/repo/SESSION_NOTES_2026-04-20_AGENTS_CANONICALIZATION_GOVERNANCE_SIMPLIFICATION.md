# Session Notes 2026-04-20 - AGENTS Canonicalization + Governance Simplification

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `workflow`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Canonicalized agent policy to root+nested `AGENTS.md` contracts:
  - root `AGENTS.md` rewritten as the repository source of truth
  - added nested policy files for `scripts/SensorList/`, `scripts/ethos_events/`, and `tools/`.
- Implemented newcomer-friendly issue startup flow:
  - added `tools/session_start.py` for guided issue startup (issue kind/slug inference, optional branch checkout)
  - extended `tools/session_preflight.py` with `--strict-branch-match` and `--json`.
- Migrated prompt workflow to template-first:
  - moved open issue snapshot prompts to `deslopification/prompts/issues/archive/`
  - updated prompt index and guardrail tests to treat templates as active contracts.
- Aligned issue intake with preflight kind taxonomy:
  - updated `.github/ISSUE_TEMPLATE/refactor.md` to chore-aligned labeling
  - added `.github/ISSUE_TEMPLATE/config.yml` to disable blank issues for non-maintainers.
- Added governance verification assets:
  - `docs/GOVERNANCE.md` maintainer runbook
  - `tools/audit_repo_governance.py` governance audit helper.
- Applied GitHub-side main-branch governance baseline:
  - enabled `main` branch protection with required status check `test`
  - retained no ruleset/branch-name restriction requirement.
- Files touched:
  - `AGENTS.md`
  - `scripts/SensorList/AGENTS.md`
  - `scripts/ethos_events/AGENTS.md`
  - `tools/AGENTS.md`
  - `tools/session_preflight.py`
  - `tools/session_start.py`
  - `tools/audit_repo_governance.py`
  - `.github/ISSUE_TEMPLATE/refactor.md`
  - `.github/ISSUE_TEMPLATE/config.yml`
  - `README.md`
  - `CONTRIBUTING.md`
  - `docs/DEVELOPMENT.md`
  - `docs/REPOSITORY_LAYOUT.md`
  - `docs/GOVERNANCE.md`
  - `deslopification/prompts/issues/README.md`
  - `deslopification/prompts/templates/ISSUE_RESOLUTION_TEMPLATE.md`
  - `deslopification/memory/CURRENT_STATE.md`
  - `tests/test_session_preflight.py`
  - `tests/test_session_start.py`
  - `tests/test_prompt_guardrails.py`
  - `tests/test_docs_contracts.py`

## Why

- Root cause or objective:
  - Preflight and branch-gate controls were robust but high-friction for newcomers due manual issue kind/slug entry and snapshot-driven prompt drift.
  - Governance enforcement was mostly documentation-based; `main` lacked active GitHub protection.
- Scope guardrails:
  - No Lua runtime behavior changes were introduced.
  - Changes were constrained to agent workflow scaffolding, governance, docs, and tests.

## Validation run(s)

- `python -m pytest tests/test_session_preflight.py tests/test_session_start.py -q`
  - result: pass (`30 passed`)
- `python -m pytest tests/test_prompt_guardrails.py tests/test_docs_commands.py tests/test_docs_contracts.py tests/test_memory_catalog_sync.py -q`
  - result: pass (`139 passed, 14 skipped`)
- `python -m pytest -q`
  - result: pass (`239 passed, 14 skipped`)
- `python tools/audit_repo_governance.py --check`
  - result: pass (main protection + required `test` check verified)

## Follow-up items

- Consider adding CI coverage for `tools/audit_repo_governance.py` with mocked `gh` output if governance-audit regression protection is needed in pull-request checks.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
- If `yes`, summary:
  - Added durable workflow baseline entries for template-first prompts, guided startup (`session_start`), structured/strict preflight controls, and GitHub-side main-branch governance.
