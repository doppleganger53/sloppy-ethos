#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from typing import Any, Optional


@dataclass
class GovernanceAudit:
    repo: str
    default_branch: Optional[str]
    allow_squash_merge: Optional[bool]
    allow_merge_commit: Optional[bool]
    allow_rebase_merge: Optional[bool]
    delete_branch_on_merge: Optional[bool]
    branch_protection_enabled: bool
    required_status_checks: list[str]
    ruleset_count: int
    checks: dict[str, bool]


def run_cmd(argv: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, text=True, capture_output=True, check=False)


def run_gh_json(args: list[str]) -> dict[str, Any]:
    result = run_cmd(["gh", *args])
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "unknown gh error"
        raise RuntimeError(detail)
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Failed to parse JSON for gh {' '.join(args)}: {exc}") from exc


def discover_repo_slug(repo_override: Optional[str]) -> str:
    if repo_override:
        return repo_override
    payload = run_gh_json(["repo", "view", "--json", "nameWithOwner"])
    slug = str(payload.get("nameWithOwner", "")).strip()
    if not slug:
        raise RuntimeError("Unable to discover repository slug from gh repo view.")
    return slug


def fetch_branch_protection(repo: str) -> dict[str, Any] | None:
    result = run_cmd(["gh", "api", f"repos/{repo}/branches/main/protection"])
    if result.returncode == 0:
        return json.loads(result.stdout)

    stderr = result.stderr.strip() or result.stdout.strip()
    if "Branch not protected" in stderr or "404" in stderr:
        return None
    raise RuntimeError(stderr or "Failed to fetch branch protection")


def audit(repo_slug: str) -> GovernanceAudit:
    repo_payload = run_gh_json(["api", f"repos/{repo_slug}"])
    rulesets_payload = run_gh_json(["api", f"repos/{repo_slug}/rulesets"])
    protection_payload = fetch_branch_protection(repo_slug)

    required_checks: list[str] = []
    if protection_payload:
        checks = protection_payload.get("required_status_checks") or {}
        required_checks = sorted(str(item) for item in checks.get("contexts", []) if item)

    audit_result = GovernanceAudit(
        repo=repo_slug,
        default_branch=repo_payload.get("default_branch"),
        allow_squash_merge=repo_payload.get("allow_squash_merge"),
        allow_merge_commit=repo_payload.get("allow_merge_commit"),
        allow_rebase_merge=repo_payload.get("allow_rebase_merge"),
        delete_branch_on_merge=repo_payload.get("delete_branch_on_merge"),
        branch_protection_enabled=protection_payload is not None,
        required_status_checks=required_checks,
        ruleset_count=len(rulesets_payload) if isinstance(rulesets_payload, list) else 0,
        checks={},
    )

    audit_result.checks = {
        "default_branch_is_main": audit_result.default_branch == "main",
        "allow_squash_merge": audit_result.allow_squash_merge is True,
        "allow_merge_commit": audit_result.allow_merge_commit is True,
        "allow_rebase_merge_disabled": audit_result.allow_rebase_merge is False,
        "main_branch_protected": audit_result.branch_protection_enabled,
        "ci_status_check_configured": "test" in audit_result.required_status_checks,
    }
    return audit_result


def print_human(audit_result: GovernanceAudit) -> None:
    print(f"Repository: {audit_result.repo}")
    print(f"Default branch: {audit_result.default_branch}")
    print(
        "Merge settings: "
        f"squash={audit_result.allow_squash_merge}, "
        f"merge_commit={audit_result.allow_merge_commit}, "
        f"rebase={audit_result.allow_rebase_merge}"
    )
    print(f"Delete branch on merge: {audit_result.delete_branch_on_merge}")
    print(f"Main branch protected: {audit_result.branch_protection_enabled}")
    print(f"Required status checks on main: {', '.join(audit_result.required_status_checks) or '(none)'}")
    print(f"Ruleset count: {audit_result.ruleset_count}")
    print("Check results:")
    for key, value in audit_result.checks.items():
        print(f"- {key}: {'PASS' if value else 'FAIL'}")


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Audit repository governance settings via gh api.")
    parser.add_argument("--repo", help="owner/repo slug. Defaults to current gh repo.")
    parser.add_argument("--check", action="store_true", help="Return non-zero when governance checks fail.")
    parser.add_argument("--json", action="store_true", help="Emit machine-readable JSON output.")
    return parser.parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(argv)
    try:
        repo_slug = discover_repo_slug(args.repo)
        audit_result = audit(repo_slug)
    except RuntimeError as exc:
        if args.json:
            print(json.dumps({"status": "ERROR", "error": str(exc)}, indent=2, sort_keys=True))
        else:
            print(f"Error: {exc}")
        return 1

    if args.json:
        print(json.dumps(asdict(audit_result), indent=2, sort_keys=True))
    else:
        print_human(audit_result)

    if args.check and not all(audit_result.checks.values()):
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
