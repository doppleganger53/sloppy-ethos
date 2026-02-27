#!/usr/bin/env python3
from __future__ import annotations

import argparse
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


def get_current_branch() -> str:
    return run_git(["branch", "--show-current"])


def is_worktree_dirty() -> bool:
    return bool(run_git(["status", "--porcelain"]))


def build_recommended_branch(issue_kind: str, issue_number: str, slug: str) -> str:
    return f"{ISSUE_KIND_TO_PREFIX[issue_kind]}/{issue_number}-{slug}"


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Session branch-policy preflight checks.")
    parser.add_argument("--mode", choices=["issue", "non-issue"], required=True)
    parser.add_argument("--issue-number")
    parser.add_argument("--issue-kind", choices=sorted(ISSUE_KIND_TO_PREFIX))
    parser.add_argument("--slug")
    args = parser.parse_args(argv)

    if args.mode == "issue":
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
    else:
        if args.issue_number or args.issue_kind or args.slug:
            parser.error("--mode non-issue does not accept issue-specific arguments.")

    return args


def run_preflight(args: argparse.Namespace) -> int:
    try:
        branch = get_current_branch()
        dirty = is_worktree_dirty()
    except RuntimeError as exc:
        print(f"Result: ERROR\n{exc}")
        return 1

    if args.mode == "issue":
        recommended = build_recommended_branch(args.issue_kind, args.issue_number, args.slug)
        print("Mode: issue")
        print(f"Current branch: {branch}")
        print(f"Worktree dirty: {'yes' if dirty else 'no'}")
        print(f"Recommended issue branch: {recommended}")

        if branch == "main":
            print("Result: BLOCKED")
            print("Issue-linked work cannot mutate on 'main'.")
            print(f"Create/switch branch: git checkout -b {recommended}")
            print(f"If branch exists: git checkout {recommended}")
            return 2

        if branch != recommended:
            print("Result: PASS_WITH_WARNING")
            print(f"Current branch differs from recommended branch '{recommended}'.")
            return 0

        print("Result: PASS")
        return 0

    print("Mode: non-issue")
    print(f"Current branch: {branch}")
    print(f"Worktree dirty: {'yes' if dirty else 'no'}")
    print("Result: PASS")
    if branch == "main":
        print("Reminder: Non-issue work on 'main' is allowed, but ask the user before mutating files.")
    return 0


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(argv)
    return run_preflight(args)


if __name__ == "__main__":
    sys.exit(main())
