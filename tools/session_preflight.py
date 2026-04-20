#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass, field
import json
import re
import subprocess
import sys
from typing import Optional


ISSUE_KIND_TO_PREFIX = {
    "enhancement": "feature",
    "bug": "fix",
    "docs": "docs",
    "chore": "chore",
}
RELEASE_KINDS = ("repo", "script")


@dataclass
class PreflightOutcome:
    mode: str
    status: str
    exit_code: int
    current_branch: Optional[str] = None
    worktree_dirty: Optional[bool] = None
    recommended_branch: Optional[str] = None
    release_kind: Optional[str] = None
    release_project: Optional[str] = None
    script_gate_issues: list[str] = field(default_factory=list)
    open_script_gate_issues: list[dict[str, str]] = field(default_factory=list)
    message: Optional[str] = None
    next_steps: list[str] = field(default_factory=list)
    reminder: Optional[str] = None
    error: Optional[str] = None


def run_git(args: list[str]) -> str:
    result = subprocess.run(
        ["git", *args],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        stderr = result.stderr.strip()
        detail = f": {stderr}" if stderr else ""
        raise RuntimeError(f"git {' '.join(args)} failed{detail}")
    return result.stdout.strip()


def run_gh(args: list[str]) -> str:
    result = subprocess.run(
        ["gh", *args],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        stderr = result.stderr.strip()
        detail = f": {stderr}" if stderr else ""
        raise RuntimeError(f"gh {' '.join(args)} failed{detail}")
    return result.stdout.strip()


def get_current_branch() -> str:
    return run_git(["branch", "--show-current"])


def is_worktree_dirty() -> bool:
    return bool(run_git(["status", "--porcelain"]))


def build_recommended_branch(issue_kind: str, issue_number: str, slug: str) -> str:
    return f"{ISSUE_KIND_TO_PREFIX[issue_kind]}/{issue_number}-{slug}"


def get_issue_metadata(issue_number: str) -> tuple[str, str, str]:
    payload = run_gh(["issue", "view", issue_number, "--json", "state,title,url"])
    data = json.loads(payload)
    state = str(data.get("state", "")).upper()
    if state not in {"OPEN", "CLOSED"}:
        raise RuntimeError(f"gh issue view {issue_number} returned unexpected state: {state!r}")
    title = str(data.get("title", "")).strip() or "(untitled)"
    url = str(data.get("url", "")).strip() or "(url unavailable)"
    return state, title, url


def validate_issue_mode_args(parser: argparse.ArgumentParser, args: argparse.Namespace) -> None:
    missing = []
    if not args.issue_number:
        missing.append("--issue-number")
    if not args.issue_kind:
        missing.append("--issue-kind")
    if not args.slug:
        missing.append("--slug")
    if missing:
        parser.error(f"--mode issue requires {' '.join(missing)}")

    if not re.fullmatch(r"[0-9]+", args.issue_number):
        parser.error("--issue-number must contain digits only.")
    if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", args.slug):
        parser.error("--slug must be lowercase kebab-case (a-z, 0-9, hyphen).")

    validate_release_scope_args(parser, args)


def validate_release_scope_args(parser: argparse.ArgumentParser, args: argparse.Namespace) -> None:
    if args.release_kind == "repo":
        if args.project:
            parser.error("--release-kind repo does not accept --project.")
        if args.script_gate_issue:
            parser.error("--release-kind repo does not accept --script-gate-issue.")
    elif args.release_kind == "script":
        if not args.project:
            parser.error("--release-kind script requires --project.")
        if not re.fullmatch(r"[A-Za-z0-9_-]+", args.project):
            parser.error("--project must contain letters, digits, underscore, or hyphen.")
        if not args.script_gate_issue:
            parser.error("--release-kind script requires at least one --script-gate-issue.")
        for issue_number in args.script_gate_issue:
            if not re.fullmatch(r"[0-9]+", issue_number):
                parser.error("--script-gate-issue must contain digits only.")
    else:
        if args.project or args.script_gate_issue:
            parser.error("--project and --script-gate-issue require --release-kind.")


def find_open_script_gate_issues(gate_issues: list[str]) -> list[tuple[str, str, str]]:
    open_issues: list[tuple[str, str, str]] = []
    for issue_number in gate_issues:
        state, title, url = get_issue_metadata(issue_number)
        if state != "CLOSED":
            open_issues.append((issue_number, title, url))
    return open_issues


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Session branch-policy preflight checks.")
    parser.add_argument("--mode", choices=["issue", "non-issue"], required=True)
    parser.add_argument("--issue-number")
    parser.add_argument("--issue-kind", choices=sorted(ISSUE_KIND_TO_PREFIX))
    parser.add_argument("--slug")
    parser.add_argument(
        "--strict-branch-match",
        action="store_true",
        help="Block when current branch differs from the recommended issue branch.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit structured JSON output for automation.",
    )
    parser.add_argument("--release-kind", choices=RELEASE_KINDS)
    parser.add_argument("--project")
    parser.add_argument("--script-gate-issue", action="append", default=[])
    args = parser.parse_args(argv)

    if args.mode == "issue":
        validate_issue_mode_args(parser, args)
    else:
        if args.issue_number or args.issue_kind or args.slug:
            parser.error("--mode non-issue does not accept issue-specific arguments.")
        if args.release_kind or args.project or args.script_gate_issue:
            parser.error("--mode non-issue does not accept release-specific arguments.")

    return args


def _emit_outcome(args: argparse.Namespace, outcome: PreflightOutcome) -> int:
    if getattr(args, "json", False):
        print(json.dumps(asdict(outcome), indent=2, sort_keys=True))
        return outcome.exit_code

    lines: list[str] = [f"Mode: {outcome.mode}"]
    if outcome.current_branch is not None:
        lines.append(f"Current branch: {outcome.current_branch}")
    if outcome.worktree_dirty is not None:
        lines.append(f"Worktree dirty: {'yes' if outcome.worktree_dirty else 'no'}")
    if outcome.recommended_branch:
        lines.append(f"Recommended issue branch: {outcome.recommended_branch}")
    if outcome.release_kind:
        lines.append(f"Release kind: {outcome.release_kind}")
    if outcome.release_project:
        lines.append(f"Release project: {outcome.release_project}")
    if outcome.script_gate_issues:
        issues = ", ".join(f"#{item}" for item in outcome.script_gate_issues)
        lines.append(f"Script gate issues: {issues}")

    lines.append(f"Result: {outcome.status}")

    if outcome.error:
        lines.append(outcome.error)
    if outcome.message:
        lines.append(outcome.message)
    if outcome.open_script_gate_issues:
        for item in outcome.open_script_gate_issues:
            lines.append(f"- #{item['number']}: {item['title']} ({item['url']})")
    if outcome.next_steps:
        lines.append("Next steps:")
        lines.extend(f"- {step}" for step in outcome.next_steps)
    if outcome.reminder:
        lines.append(f"Reminder: {outcome.reminder}")

    print("\n".join(lines))
    return outcome.exit_code


def run_preflight(args: argparse.Namespace) -> int:
    release_kind = getattr(args, "release_kind", None)
    project = getattr(args, "project", None)
    script_gate_issues = list(getattr(args, "script_gate_issue", []) or [])
    strict_branch_match = bool(getattr(args, "strict_branch_match", False))

    try:
        branch = get_current_branch()
        dirty = is_worktree_dirty()
    except RuntimeError as exc:
        outcome = PreflightOutcome(
            mode=args.mode,
            status="ERROR",
            exit_code=1,
            error=str(exc),
        )
        return _emit_outcome(args, outcome)

    if args.mode == "issue":
        recommended = build_recommended_branch(args.issue_kind, args.issue_number, args.slug)
        outcome = PreflightOutcome(
            mode="issue",
            status="PASS",
            exit_code=0,
            current_branch=branch,
            worktree_dirty=dirty,
            recommended_branch=recommended,
            release_kind=release_kind,
            release_project=project,
            script_gate_issues=script_gate_issues,
        )

        if branch == "main":
            outcome.status = "BLOCKED"
            outcome.exit_code = 2
            outcome.message = "Issue-linked work cannot mutate on 'main'."
            outcome.next_steps = [
                f"Create a branch (recommended): git switch -c {recommended}",
                f"Create a branch (fallback): git checkout -b {recommended}",
                f"If the branch already exists: git switch {recommended}",
                f"Fallback for existing branch: git checkout {recommended}",
            ]
            return _emit_outcome(args, outcome)

        if release_kind == "script":
            try:
                open_issues = find_open_script_gate_issues(script_gate_issues)
            except RuntimeError as exc:
                outcome.status = "ERROR"
                outcome.exit_code = 1
                outcome.error = str(exc)
                return _emit_outcome(args, outcome)
            if open_issues:
                outcome.status = "BLOCKED"
                outcome.exit_code = 2
                outcome.message = "Script release requires all script gate issues to be CLOSED."
                outcome.open_script_gate_issues = [
                    {"number": issue_number, "title": title, "url": url}
                    for issue_number, title, url in open_issues
                ]
                return _emit_outcome(args, outcome)

        if branch != recommended:
            if strict_branch_match:
                outcome.status = "BLOCKED"
                outcome.exit_code = 2
                outcome.message = f"Current branch differs from recommended branch '{recommended}'."
                outcome.next_steps = [
                    f"Switch to recommended branch: git switch {recommended}",
                    f"Fallback switch command: git checkout {recommended}",
                ]
                return _emit_outcome(args, outcome)

            outcome.status = "PASS_WITH_WARNING"
            outcome.message = f"Current branch differs from recommended branch '{recommended}'."
            return _emit_outcome(args, outcome)

        return _emit_outcome(args, outcome)

    outcome = PreflightOutcome(
        mode="non-issue",
        status="PASS",
        exit_code=0,
        current_branch=branch,
        worktree_dirty=dirty,
    )
    if branch == "main":
        outcome.reminder = "Non-issue work on 'main' is allowed, but ask the user before mutating files."
    return _emit_outcome(args, outcome)


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(argv)
    return run_preflight(args)


if __name__ == "__main__":
    sys.exit(main())
