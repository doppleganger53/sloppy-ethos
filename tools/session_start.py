#!/usr/bin/env python3
from __future__ import annotations

import argparse
import io
import importlib.util
import json
import re
import subprocess
import sys
from contextlib import redirect_stdout
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

try:
    import session_preflight
except ModuleNotFoundError:
    _SESSION_PREFLIGHT_PATH = Path(__file__).resolve().parent / "session_preflight.py"
    _SPEC = importlib.util.spec_from_file_location("session_preflight", _SESSION_PREFLIGHT_PATH)
    if not _SPEC or not _SPEC.loader:
        raise RuntimeError("Failed to load tools/session_preflight.py")
    session_preflight = importlib.util.module_from_spec(_SPEC)
    sys.modules[_SPEC.name] = session_preflight
    _SPEC.loader.exec_module(session_preflight)


ISSUE_KIND_LABEL_MAP = {
    "bug": "bug",
    "docs": "docs",
    "documentation": "docs",
    "enhancement": "enhancement",
    "chore": "chore",
    "workflow": "chore",
    "refactor": "chore",
    "tests": "chore",
}

ISSUE_KIND_TITLE_PREFIX = {
    "bug": "bug",
    "docs": "docs",
    "documentation": "docs",
    "enhancement": "enhancement",
    "chore": "chore",
    "refactor": "chore",
}


@dataclass
class IssueContext:
    number: str
    title: str
    url: str
    state: str
    issue_kind: str
    slug: str
    recommended_branch: str


def run_cmd(argv: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, text=True, capture_output=True, check=False)


def run_gh_json(args: list[str]) -> dict:
    result = run_cmd(["gh", *args])
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "unknown gh error"
        raise RuntimeError(detail)
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Invalid JSON from gh {' '.join(args)}: {exc}") from exc


def normalize_issue_kind_label(label: str) -> Optional[str]:
    return ISSUE_KIND_LABEL_MAP.get(label.strip().lower())


def infer_issue_kind(labels: list[str], title: str) -> Optional[str]:
    for label in labels:
        mapped = normalize_issue_kind_label(label)
        if mapped:
            return mapped

    title_match = re.match(r"^\[(?P<prefix>[^\]]+)\]\s*", title.strip())
    if title_match:
        prefix = title_match.group("prefix").strip().lower()
        mapped = ISSUE_KIND_TITLE_PREFIX.get(prefix)
        if mapped:
            return mapped

    return None


def slugify_issue_title(title: str) -> str:
    cleaned = re.sub(r"^\[[^\]]+\]\s*", "", title.strip())
    cleaned = re.sub(r"[^a-z0-9]+", "-", cleaned.lower()).strip("-")
    cleaned = re.sub(r"-{2,}", "-", cleaned)
    if not cleaned:
        return "update-work-item"
    return cleaned


def collect_issue_context(
    issue_number: str,
    *,
    issue_kind_override: Optional[str],
    slug_override: Optional[str],
) -> IssueContext:
    payload = run_gh_json(["issue", "view", issue_number, "--json", "number,title,url,state,labels"])
    label_names = [str(item.get("name", "")).strip() for item in payload.get("labels", []) if item.get("name")]
    title = str(payload.get("title", "")).strip() or f"Issue {issue_number}"
    inferred_kind = infer_issue_kind(label_names, title)
    issue_kind = issue_kind_override or inferred_kind
    if not issue_kind:
        raise RuntimeError(
            "Unable to infer issue kind from labels/title. Use --issue-kind {bug|docs|enhancement|chore}."
        )

    slug = slug_override or slugify_issue_title(title)
    recommended_branch = session_preflight.build_recommended_branch(issue_kind, issue_number, slug)
    state = str(payload.get("state", "")).upper() or "UNKNOWN"
    if state not in {"OPEN", "CLOSED"}:
        state = "UNKNOWN"

    return IssueContext(
        number=str(payload.get("number", issue_number)),
        title=title,
        url=str(payload.get("url", "")).strip() or f"(issue #{issue_number})",
        state=state,
        issue_kind=issue_kind,
        slug=slug,
        recommended_branch=recommended_branch,
    )


def run_preflight_json(args: argparse.Namespace) -> tuple[int, dict]:
    preflight_args = argparse.Namespace(
        mode="issue",
        issue_number=args.issue_number,
        issue_kind=args.issue_kind,
        slug=args.slug,
        strict_branch_match=args.strict_branch_match,
        json=True,
        release_kind=args.release_kind,
        project=args.project,
        script_gate_issue=list(args.script_gate_issue or []),
    )

    buffer = io.StringIO()
    with redirect_stdout(buffer):
        exit_code = session_preflight.run_preflight(preflight_args)
    output = buffer.getvalue().strip()
    if not output:
        raise RuntimeError("session_preflight returned no output")
    try:
        payload = json.loads(output)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Failed to parse session_preflight JSON output: {exc}\nRaw output:\n{output}") from exc
    return exit_code, payload


def local_branch_exists(branch: str) -> bool:
    result = run_cmd(["git", "show-ref", "--verify", "--quiet", f"refs/heads/{branch}"])
    return result.returncode == 0


def try_checkout_issue_branch(issue_number: str, branch_name: str) -> tuple[bool, str]:
    gh_create = run_cmd(["gh", "issue", "develop", issue_number, "--name", branch_name, "--checkout"])
    if gh_create.returncode == 0:
        return True, "Created/switched branch with gh issue develop."

    if local_branch_exists(branch_name):
        git_switch = run_cmd(["git", "switch", branch_name])
        if git_switch.returncode == 0:
            return True, "Switched to existing branch with git switch."
        git_checkout = run_cmd(["git", "checkout", branch_name])
        if git_checkout.returncode == 0:
            return True, "Switched to existing branch with git checkout."
    else:
        git_create = run_cmd(["git", "switch", "-c", branch_name])
        if git_create.returncode == 0:
            return True, "Created/switched branch with git switch -c."
        git_checkout_create = run_cmd(["git", "checkout", "-b", branch_name])
        if git_checkout_create.returncode == 0:
            return True, "Created/switched branch with git checkout -b."

    detail = gh_create.stderr.strip() or gh_create.stdout.strip() or "branch checkout failed"
    return False, detail


def should_attempt_checkout(preflight_payload: dict) -> bool:
    status = str(preflight_payload.get("status", "")).upper()
    if status == "PASS_WITH_WARNING":
        return True
    if status != "BLOCKED":
        return False

    message = str(preflight_payload.get("message", ""))
    return "Current branch differs from recommended branch" in message or "cannot mutate on 'main'" in message


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Guided issue session startup. Infers preflight fields from GitHub issue metadata."
    )
    subparsers = parser.add_subparsers(dest="mode", required=True)

    issue_parser = subparsers.add_parser("issue", help="Start an issue-linked session.")
    issue_parser.add_argument("issue_number", help="Issue number (digits only).")
    issue_parser.add_argument("--issue-kind", choices=sorted(session_preflight.ISSUE_KIND_TO_PREFIX))
    issue_parser.add_argument("--slug")
    issue_parser.add_argument("--checkout", action="store_true", help="Create/switch to the recommended branch.")
    issue_parser.add_argument(
        "--strict-branch-match",
        action="store_true",
        help="Block when branch differs from recommendation.",
    )
    issue_parser.add_argument("--release-kind", choices=session_preflight.RELEASE_KINDS)
    issue_parser.add_argument("--project")
    issue_parser.add_argument("--script-gate-issue", action="append", default=[])
    issue_parser.add_argument("--json", action="store_true", help="Emit structured JSON output.")

    non_issue_parser = subparsers.add_parser("non-issue", help="Run non-issue preflight.")
    non_issue_parser.add_argument("--json", action="store_true", help="Emit structured JSON output.")

    args = parser.parse_args(argv)
    if args.mode == "issue" and not re.fullmatch(r"[0-9]+", args.issue_number):
        parser.error("issue_number must contain digits only.")
    return args


def _run_non_issue(args: argparse.Namespace) -> int:
    preflight_args = argparse.Namespace(
        mode="non-issue",
        issue_number=None,
        issue_kind=None,
        slug=None,
        strict_branch_match=False,
        json=getattr(args, "json", False),
        release_kind=None,
        project=None,
        script_gate_issue=[],
    )
    return session_preflight.run_preflight(preflight_args)


def _run_issue(args: argparse.Namespace) -> int:
    issue_ctx = collect_issue_context(
        args.issue_number,
        issue_kind_override=args.issue_kind,
        slug_override=args.slug,
    )
    args.issue_kind = issue_ctx.issue_kind
    args.slug = issue_ctx.slug

    first_code, first_payload = run_preflight_json(args)
    checkout_result: Optional[str] = None
    second_payload: Optional[dict] = None
    final_code = first_code

    if args.checkout and should_attempt_checkout(first_payload):
        checked_out, checkout_result = try_checkout_issue_branch(args.issue_number, issue_ctx.recommended_branch)
        if checked_out:
            final_code, second_payload = run_preflight_json(args)
        else:
            final_code = 2

    effective_payload = second_payload or first_payload

    if args.json:
        response = {
            "issue": {
                "number": issue_ctx.number,
                "title": issue_ctx.title,
                "url": issue_ctx.url,
                "state": issue_ctx.state,
                "issue_kind": issue_ctx.issue_kind,
                "slug": issue_ctx.slug,
                "recommended_branch": issue_ctx.recommended_branch,
            },
            "preflight": effective_payload,
            "checkout_attempted": bool(args.checkout),
            "checkout_result": checkout_result,
            "exit_code": final_code,
        }
        print(json.dumps(response, indent=2, sort_keys=True))
        return final_code

    print("Guided session startup")
    print(f"- Issue: #{issue_ctx.number} {issue_ctx.title}")
    print(f"- URL: {issue_ctx.url}")
    print(f"- State: {issue_ctx.state}")
    print(f"- Inferred issue kind: {issue_ctx.issue_kind}")
    print(f"- Inferred slug: {issue_ctx.slug}")
    print(f"- Recommended branch: {issue_ctx.recommended_branch}")

    if checkout_result:
        print(f"- Branch checkout: {checkout_result}")

    print(f"- Preflight result: {effective_payload.get('status')}")
    message = effective_payload.get("message")
    if message:
        print(f"- Detail: {message}")
    next_steps = effective_payload.get("next_steps") or []
    if next_steps:
        print("- Next steps:")
        for step in next_steps:
            print(f"  - {step}")
    reminder = effective_payload.get("reminder")
    if reminder:
        print(f"- Reminder: {reminder}")

    return final_code


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(argv)
    if args.mode == "non-issue":
        return _run_non_issue(args)
    try:
        return _run_issue(args)
    except RuntimeError as exc:
        if getattr(args, "json", False):
            print(json.dumps({"status": "ERROR", "error": str(exc)}, indent=2, sort_keys=True))
        else:
            print(f"Error: {exc}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
