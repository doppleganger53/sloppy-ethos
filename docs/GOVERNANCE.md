# Repository Governance Runbook

This runbook covers repository governance settings that complement local
preflight and branch-gate tooling.

## Governance model

- Local governance (agent session workflow):
  - `tools/session_start.py` for guided issue startup
  - `tools/session_preflight.py` for explicit branch/worktree policy checks
- GitHub-side governance (repository enforcement):
  - protect `main` so merges flow through PRs with passing CI
  - keep branch-name restrictions off for contributor approachability

## Main-branch baseline

Target baseline for `main`:

- branch protection enabled
- required status check includes CI job `test`
- repository merge settings:
  - squash merge enabled
  - merge commits enabled
  - rebase merge disabled

## Configure via `gh api` (maintainer step)

1. Set branch protection on `main`:

```bash
gh api -X PUT repos/{owner}/{repo}/branches/main/protection \
  -H "Accept: application/vnd.github+json" \
  -f required_status_checks[strict]=false \
  -f required_status_checks[contexts][]=test \
  -F enforce_admins=false \
  -F required_pull_request_reviews[dismiss_stale_reviews]=false \
  -F required_pull_request_reviews[require_code_owner_reviews]=false \
  -f required_pull_request_reviews[required_approving_review_count]=1 \
  -f restrictions=
```

2. Verify settings with the audit helper:

```bash
python tools/audit_repo_governance.py --check
```

3. Optional machine-readable output:

```bash
python tools/audit_repo_governance.py --json
```

## Notes

- If repository permissions block API updates, perform equivalent changes in
  GitHub repository settings and re-run the audit helper.
- `rulesets` are intentionally optional for this repository baseline; branch
  name restrictions are not required by default.
