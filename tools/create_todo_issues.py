#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys


TODO_REF = "Source backlog: TODO.md (TODO-03..TODO-09)."


CHILD_ISSUES = [
    {
        "title": "[Enhancement] Generate roadmap prompt files for SensorList roadmap items",
        "labels": ["enhancement", "docs"],
        "body": """## Problem Statement
Roadmap items exist in README but prompt artifacts are incomplete.

## Desired Outcome
Prompt files exist in deslopification/prompts for each roadmap item using the widget template structure.

## Proposed Approach
Create prompt files for sort headers, conflict display refinement, and acceptable conflicts model. Include non-goals, callback constraints, validation checklist, and acceptance criteria.

## Acceptance Criteria
1. Three roadmap prompt files are added.
2. Prompt content maps directly to README roadmap bullets.
3. Each prompt includes explicit non-goals and regression checks.

## Risks / Edge Cases
- Scope creep into implementation details.
- Prompt ambiguity for acceptable conflicts model.

"""
        + TODO_REF,
    },
    {
        "title": "[Enhancement] Add unit tests for SensorList Lua behavior",
        "labels": ["enhancement", "tests"],
        "body": """## Problem Statement
SensorList behavior is validated manually but lacks repeatable unit tests.

## Desired Outcome
Pytest-managed Lua tests validate core SensorList helper behavior and edge cases.

## Proposed Approach
Add a guarded `_test` export in `main.lua`, Lua test script with mocked Ethos APIs, and pytest wrapper to run the Lua test.

## Acceptance Criteria
1. Sensor ID parsing/formatting and normalization are covered.
2. Grouping/signature behavior is covered.
3. Scroll clamp and missing API fallback behaviors are covered.

## Risks / Edge Cases
- Lua runtime differences across developer machines.
- Mocked APIs drifting from simulator behavior.

"""
        + TODO_REF,
    },
    {
        "title": "[Enhancement] Add unit tests for tools/build.py",
        "labels": ["enhancement", "tests"],
        "body": """## Problem Statement
Build tooling has no automated regression tests for version/config/deploy behavior.

## Desired Outcome
Pytest unit tests cover core `tools/build.py` logic with mocked filesystem/process interactions.

## Proposed Approach
Add tests for version normalization, config parsing, simulator path resolution, deploy error formatting, and no-action guard path in main.

## Acceptance Criteria
1. Core pure helpers are covered with positive and negative cases.
2. Main no-op path exits with clear message.
3. Tests run without requiring real simulator paths.

## Risks / Edge Cases
- Over-mocking may hide integration issues.

"""
        + TODO_REF,
    },
    {
        "title": "[Enhancement] Add tests for documentation command examples",
        "labels": ["enhancement", "tests", "docs"],
        "body": """## Problem Statement
Documentation command snippets can drift from actual working workflows.

## Desired Outcome
Automated tests parse documented commands and validate syntax/path references, with explicit manual/environment-dependent skips.

## Proposed Approach
Add pytest docs command parser for README, DEVELOPMENT, SensorList README, and CONTRIBUTING.

## Acceptance Criteria
1. Command snippets are discovered from docs.
2. Referenced scripts/files exist.
3. Runnable commands execute or are explicitly skipped as manual.

## Risks / Edge Cases
- Some command snippets are intentionally partial in prose context.

"""
        + TODO_REF,
    },
    {
        "title": "[Refactor] Streamline AGENTS.md and move SensorList-specific notes to memory",
        "labels": ["refactor", "docs"],
        "body": """## Current Pain / Complexity
AGENTS mixes repo-wide and script-specific operational detail, causing repeated context noise.

## Refactor Boundaries
- In scope: Keep AGENTS concise and repo-level; move SensorList specifics to memory.
- Out of scope: Widget behavior changes.

## Safety Constraints
No workflow regressions; command references remain consistent with documented Python-first build flow.

## Validation Checklist
1. AGENTS startup workflow remains clear.
2. SensorList script notes are captured in `deslopification/memory/SensorList.md`.
3. No conflicting command guidance remains.

## Rollback Plan
Restore prior AGENTS content from git history and remove script-specific memory note if needed.

"""
        + TODO_REF,
    },
    {
        "title": "[Enhancement] Refresh SensorList README with current behavior and Python-first workflow",
        "labels": ["enhancement", "docs"],
        "body": """## Problem Statement
`scripts/SensorList/README.md` is stale and uses PowerShell-first packaging steps.

## Desired Outcome
README reflects current widget behavior and uses `python tools/build.py` as primary workflow, with PowerShell as fallback.

## Proposed Approach
Update behavior section (scroll/conflict grouping/discovery strategy) and replace install/build command examples.

## Acceptance Criteria
1. Python-first build/deploy commands are documented.
2. Scroll and conflict-group behavior are documented.
3. Wording aligns with repository README and DEVELOPMENT docs.

## Risks / Edge Cases
- Inconsistency with other docs if not updated together.

"""
        + TODO_REF,
    },
]


def run_gh(args: list[str]) -> str:
    result = subprocess.run(args, capture_output=True, text=True, check=False)
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip() or "unknown error"
        raise RuntimeError(message)
    return result.stdout.strip()


def ensure_gh_available():
    if shutil.which("gh") is None:
        sys.exit("GitHub CLI not found on PATH.")
    try:
        run_gh(["gh", "auth", "status"])
    except RuntimeError as exc:
        sys.exit(f"GitHub CLI is not authenticated. Run 'gh auth login' first. Details: {exc}")


def new_issue(repo: str, title: str, labels: list[str], body: str) -> str:
    command = ["gh", "issue", "create", "--repo", repo, "--title", title, "--body", body]
    for label in labels:
        command.extend(["--label", label])
    url = run_gh(command)
    if not url:
        raise RuntimeError(f"Failed to create issue: {title}")
    return url


def parse_args():
    parser = argparse.ArgumentParser(description="Create TODO-tracking GitHub issues for sloppy-ethos.")
    parser.add_argument("--repo", default="doppleganger53/sloppy-ethos", help="GitHub repository in owner/name format.")
    parser.add_argument("--dry-run", action="store_true", help="Print issue titles without creating them.")
    return parser.parse_args()


def main():
    args = parse_args()
    if args.dry_run:
        for issue in CHILD_ISSUES:
            print(issue["title"])
        print("[Enhancement] Track TODO backlog execution (tests, prompts, docs, AGENTS refactor)")
        return

    ensure_gh_available()

    child_urls: list[str] = []
    for issue in CHILD_ISSUES:
        child_urls.append(new_issue(args.repo, issue["title"], issue["labels"], issue["body"]))

    child_list = "\n".join(f"- {url}" for url in child_urls)
    parent_body = f"""Tracks execution of TODO backlog items that remain after issue template creation.

## Child Issues
{child_list}

## Notes
- Linked from TODO.md item TODO-03.
- Child issues correspond to TODO-04 through TODO-09.
"""
    parent_title = "[Enhancement] Track TODO backlog execution (tests, prompts, docs, AGENTS refactor)"
    parent_url = new_issue(args.repo, parent_title, ["enhancement"], parent_body)
    print(f"Created parent issue: {parent_url}")


if __name__ == "__main__":
    main()
