from __future__ import annotations

import importlib.util
import os
from argparse import Namespace
from pathlib import Path

import pytest


def load_preflight_module():
    repo_root = Path(__file__).resolve().parents[1]
    preflight_path = repo_root / "tools" / "session_preflight.py"
    spec = importlib.util.spec_from_file_location("session_preflight_module", preflight_path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


preflight = load_preflight_module()


def test_branch_name_mapping():
    assert preflight.build_recommended_branch("enhancement", "16", "memory-optimization") == (
        "feature/16-memory-optimization"
    )
    assert preflight.build_recommended_branch("bug", "30", "x20s-simulator-bug") == "fix/30-x20s-simulator-bug"
    assert preflight.build_recommended_branch("docs", "11", "docs-baseline") == "docs/11-docs-baseline"
    assert preflight.build_recommended_branch("chore", "22", "tooling-cleanup") == "chore/22-tooling-cleanup"


@pytest.fixture
def issue_checkout(tmp_path, monkeypatch):
    """A private upstream and a checkout independent of the developer's Git settings."""
    monkeypatch.setenv("GIT_CONFIG_GLOBAL", os.devnull)
    monkeypatch.setenv("GIT_CONFIG_NOSYSTEM", "1")
    repo = tmp_path / "working"
    repo.mkdir()
    monkeypatch.chdir(repo)
    git = preflight.run_git
    git(["init", "--bare", "-b", "main", str(tmp_path / "upstream.git")])
    git(["init", "-b", "main"])
    git(["config", "user.name", "Fixture"])
    git(["config", "user.email", "fixture@example.invalid"])
    git(["commit", "--allow-empty", "-m", "Verified base"])
    git(["remote", "add", "origin", str(tmp_path / "upstream.git")])
    git(["push", "-u", "origin", "main"])
    return repo


def test_exact_target_isolates_new_branch_then_syncs_existing_upstream(issue_checkout, monkeypatch, capsys):
    git = preflight.run_git
    target = "docs/16-pilot-guide"
    args = ["--mode", "issue", "--issue-number", "16", "--issue-kind", "docs",
            "--slug", "pilot-guide", "--target-branch", target]
    base = git(["rev-parse", "refs/remotes/origin/main"])
    git(["switch", "-c", "other-task"])
    sentinel = issue_checkout / "notes.txt"
    sentinel.write_text("edits in progress", encoding="utf-8")
    assert preflight.main(args) == 2
    assert "Expected branch" in capsys.readouterr().out

    isolated = issue_checkout.parent / "pilot-guide"
    git(["worktree", "add", "--no-track", "-b", target, str(isolated), base])
    monkeypatch.chdir(isolated)
    assert git(["rev-parse", "HEAD"]) == base
    assert preflight.main(args) == 0
    assert "No upstream configured. Skip pull." in capsys.readouterr().out

    git(["push", "-u", "origin", target])
    # Another author advances the fixture's published branch.
    author = issue_checkout.parent / "author"
    git(["clone", "--branch", target, str(issue_checkout.parent / "upstream.git"), str(author)])
    git(["-C", str(author), "-c", "user.name=Author", "-c", "user.email=author@example.invalid",
         "commit", "--allow-empty", "-m", "Remote update"])
    git(["-C", str(author), "push"])
    assert preflight.main(args) == 0
    assert f"Local tracking ref: refs/remotes/origin/{target}" in capsys.readouterr().out
    remote = git(["config", "--get", f"branch.{target}.remote"])
    merge_ref = git(["config", "--get", f"branch.{target}.merge"])
    assert (remote, merge_ref) == ("origin", f"refs/heads/{target}")
    assert git(["ls-remote", "--exit-code", "--heads", remote, merge_ref]).split()[0] == git(
        ["-C", str(author), "rev-parse", "HEAD"]
    )
    git(["pull", "--ff-only"])
    assert git(["rev-parse", "HEAD"]) == git(["-C", str(author), "rev-parse", "HEAD"])
    git(["update-ref", "-d", f"refs/remotes/origin/{target}"])
    assert preflight.main(args) == 2
    assert "Could not verify the tracking ref" in capsys.readouterr().out
    assert sentinel.read_text(encoding="utf-8") == "edits in progress"
    assert git(["-C", str(issue_checkout), "branch", "--show-current"]) == "other-task"


def test_release_target_and_detached_head(issue_checkout, capsys):
    release = "release/v1.0.0"
    args = ["--mode", "issue", "--issue-number", "16", "--issue-kind", "chore",
            "--slug", "prepare-release", "--release-kind", "repo"]
    git = preflight.run_git
    git(["switch", "-c", release])
    assert preflight.main(args) == 0
    assert "PASS_WITH_WARNING" in capsys.readouterr().out
    exact = [*args, "--target-branch", release]
    assert preflight.main(exact) == 0
    assert "Result: PASS\n" in capsys.readouterr().out
    draft = issue_checkout / "draft.txt"
    draft.write_text("previous task", encoding="utf-8")
    assert preflight.main(exact) == 2
    assert "contains changes" in capsys.readouterr().out
    git(["switch", "--detach"])
    assert preflight.main(exact) == 2
    assert "Detached HEAD" in capsys.readouterr().out
    assert draft.read_text(encoding="utf-8") == "previous task"


def test_issue_mode_blocks_on_main(monkeypatch, capsys):
    monkeypatch.setattr(preflight, "get_current_branch", lambda: "main")
    monkeypatch.setattr(preflight, "is_worktree_dirty", lambda: True)
    args = Namespace(
        mode="issue",
        issue_number="16",
        issue_kind="enhancement",
        slug="memory-optimization",
        release_kind=None,
        project=None,
        script_gate_issue=[],
    )
    result = preflight.run_preflight(args)
    output = capsys.readouterr().out
    assert result == 2
    assert "Result: BLOCKED" in output
    assert "Issue-linked work cannot mutate on 'main'." in output
    assert "Prepare the issue worktree from a verified base" in output


def test_issue_mode_passes_on_recommended_branch(monkeypatch, capsys):
    monkeypatch.setattr(preflight, "get_current_branch", lambda: "feature/16-memory-optimization")
    monkeypatch.setattr(preflight, "is_worktree_dirty", lambda: False)
    args = Namespace(
        mode="issue",
        issue_number="16",
        issue_kind="enhancement",
        slug="memory-optimization",
        release_kind=None,
        project=None,
        script_gate_issue=[],
    )
    result = preflight.run_preflight(args)
    output = capsys.readouterr().out
    assert result == 0
    assert "Result: PASS" in output
    assert "Current branch: feature/16-memory-optimization" in output


def test_issue_mode_warns_when_branch_differs_from_recommended(monkeypatch, capsys):
    monkeypatch.setattr(preflight, "get_current_branch", lambda: "fix/16-memory-optimization")
    monkeypatch.setattr(preflight, "is_worktree_dirty", lambda: False)
    args = Namespace(
        mode="issue",
        issue_number="16",
        issue_kind="enhancement",
        slug="memory-optimization",
        release_kind=None,
        project=None,
        script_gate_issue=[],
    )
    result = preflight.run_preflight(args)
    output = capsys.readouterr().out
    assert result == 0
    assert "Result: PASS_WITH_WARNING" in output
    assert "Current branch differs from recommended branch 'feature/16-memory-optimization'." in output


def test_non_issue_mode_allows_main_with_reminder(monkeypatch, capsys):
    monkeypatch.setattr(preflight, "get_current_branch", lambda: "main")
    monkeypatch.setattr(preflight, "is_worktree_dirty", lambda: False)
    args = Namespace(
        mode="non-issue",
        issue_number=None,
        issue_kind=None,
        slug=None,
        release_kind=None,
        project=None,
        script_gate_issue=[],
    )
    result = preflight.run_preflight(args)
    output = capsys.readouterr().out
    assert result == 0
    assert "Result: PASS" in output
    assert "ask the user before mutating files" in output


def test_non_issue_mode_on_non_main_has_no_mutation_reminder(monkeypatch, capsys):
    monkeypatch.setattr(preflight, "get_current_branch", lambda: "feature/some-work")
    monkeypatch.setattr(preflight, "is_worktree_dirty", lambda: True)
    args = Namespace(
        mode="non-issue",
        issue_number=None,
        issue_kind=None,
        slug=None,
        release_kind=None,
        project=None,
        script_gate_issue=[],
    )
    result = preflight.run_preflight(args)
    output = capsys.readouterr().out
    assert result == 0
    assert "Result: PASS" in output
    assert "Worktree dirty: yes" in output
    assert "ask the user before mutating files" not in output


def test_run_preflight_returns_error_when_git_calls_fail(monkeypatch, capsys):
    monkeypatch.setattr(preflight, "get_current_branch", lambda: (_ for _ in ()).throw(RuntimeError("git failed")))
    args = Namespace(
        mode="non-issue",
        issue_number=None,
        issue_kind=None,
        slug=None,
        release_kind=None,
        project=None,
        script_gate_issue=[],
    )
    result = preflight.run_preflight(args)
    output = capsys.readouterr().out
    assert result == 1
    assert "Result: ERROR" in output
    assert "git failed" in output


def test_parse_args_accepts_valid_issue_mode():
    args = preflight.parse_args(
        ["--mode", "issue", "--issue-number", "16", "--issue-kind", "enhancement", "--slug", "memory-optimization"]
    )
    assert args.mode == "issue"
    assert args.issue_number == "16"
    assert args.issue_kind == "enhancement"
    assert args.slug == "memory-optimization"
    assert args.release_kind is None
    assert args.project is None
    assert args.script_gate_issue == []


def test_parse_args_accepts_valid_script_release_scope():
    args = preflight.parse_args(
        [
            "--mode",
            "issue",
            "--issue-number",
            "39",
            "--issue-kind",
            "bug",
            "--slug",
            "repo-release-workflow-failures",
            "--release-kind",
            "script",
            "--release-repo",
            "upstream/widgets",
            "--project",
            "WidgetX",
            "--script-gate-issue",
            "32",
        ]
    )
    assert args.release_kind == "script"
    assert args.release_repo == "upstream/widgets"
    assert args.project == "WidgetX"
    assert args.script_gate_issue == ["32"]


def test_parse_args_accepts_valid_non_issue_mode():
    args = preflight.parse_args(["--mode", "non-issue"])
    assert args.mode == "non-issue"
    assert args.issue_number is None
    assert args.issue_kind is None
    assert args.slug is None
    assert args.release_kind is None
    assert args.project is None
    assert args.script_gate_issue == []


def test_parse_args_requires_issue_fields_for_issue_mode():
    with pytest.raises(SystemExit):
        preflight.parse_args(["--mode", "issue"])


@pytest.mark.parametrize(
    "argv",
    [
        ["--mode", "issue", "--issue-number", "abc", "--issue-kind", "enhancement", "--slug", "memory-optimization"],
        ["--mode", "issue", "--issue-number", "16", "--issue-kind", "enhancement", "--slug", "Memory-Optimization"],
        ["--mode", "issue", "--issue-number", "16", "--issue-kind", "enhancement", "--slug", "bad_slug"],
    ],
)
def test_parse_args_rejects_invalid_issue_field_formats(argv):
    with pytest.raises(SystemExit):
        preflight.parse_args(argv)


@pytest.mark.parametrize(
    "argv",
    [
        ["--mode", "non-issue", "--issue-number", "16"],
        ["--mode", "non-issue", "--issue-kind", "enhancement"],
        ["--mode", "non-issue", "--slug", "memory-optimization"],
        ["--mode", "non-issue", "--target-branch", "feature/16-memory-optimization"],
    ],
)
def test_parse_args_rejects_issue_fields_for_non_issue_mode(argv):
    with pytest.raises(SystemExit):
        preflight.parse_args(argv)


@pytest.mark.parametrize(
    "argv",
    [
        [
            "--mode",
            "issue",
            "--issue-number",
            "39",
            "--issue-kind",
            "bug",
            "--slug",
            "repo-release-workflow-failures",
            "--release-kind",
            "repo",
            "--script-gate-issue",
            "32",
        ],
        [
            "--mode",
            "issue",
            "--issue-number",
            "39",
            "--issue-kind",
            "bug",
            "--slug",
            "repo-release-workflow-failures",
            "--release-kind",
            "script",
            "--release-repo",
            "upstream/widgets",
            "--project",
            "WidgetX",
        ],
        [
            "--mode",
            "issue",
            "--issue-number",
            "39",
            "--issue-kind",
            "bug",
            "--slug",
            "repo-release-workflow-failures",
            "--release-kind",
            "script",
            "--release-repo",
            "upstream/widgets",
            "--script-gate-issue",
            "32",
        ],
        [
            "--mode",
            "issue",
            "--issue-number",
            "39",
            "--issue-kind",
            "bug",
            "--slug",
            "repo-release-workflow-failures",
            "--project",
            "WidgetX",
        ],
    ],
)
def test_parse_args_rejects_invalid_release_scope_combinations(argv):
    with pytest.raises(SystemExit):
        preflight.parse_args(argv)


@pytest.mark.parametrize("repo_args", [[], ["--release-repo", "widgets"], ["--release-repo", "owner/repo/extra"]])
def test_script_gate_requires_explicit_repository(repo_args):
    with pytest.raises(SystemExit):
        preflight.parse_args([
            "--mode", "issue", "--issue-number", "39", "--issue-kind", "chore", "--slug", "release",
            "--release-kind", "script", "--project", "WidgetX", "--script-gate-issue", "32", *repo_args,
        ])


def test_issue_mode_script_release_blocks_when_gate_issue_open(monkeypatch, capsys):
    monkeypatch.setattr(preflight, "get_current_branch", lambda: "fix/39-repo-release-workflow-failures")
    monkeypatch.setattr(preflight, "is_worktree_dirty", lambda: False)

    def gh_with_fork_default(args):
        if args == ["issue", "view", "32", "--repo", "upstream/widgets", "--json", "state,title,url"]:
            return '{"state":"OPEN","title":"Test on radio","url":"https://github.com/upstream/widgets/issues/32"}'
        raise AssertionError(f"Unexpected repository or query: {args}")

    monkeypatch.setenv("GH_REPO", "contributor/widgets")
    monkeypatch.setattr(preflight, "run_gh", gh_with_fork_default)
    args = Namespace(
        mode="issue",
        issue_number="39",
        issue_kind="bug",
        slug="repo-release-workflow-failures",
        release_kind="script",
        release_repo="upstream/widgets",
        project="WidgetX",
        script_gate_issue=["32"],
    )
    result = preflight.run_preflight(args)
    output = capsys.readouterr().out
    assert result == 2
    assert "Result: BLOCKED" in output
    assert "Script release requires all script gate issues to be CLOSED." in output
    assert "#32" in output


def test_issue_mode_script_release_passes_when_gate_issue_closed(monkeypatch, capsys):
    monkeypatch.setattr(preflight, "get_current_branch", lambda: "fix/39-repo-release-workflow-failures")
    monkeypatch.setattr(preflight, "is_worktree_dirty", lambda: False)
    monkeypatch.setattr(
        preflight,
        "get_issue_metadata",
        lambda _issue_number, _repo: ("CLOSED", "Manual testing complete", "https://github.com/upstream/widgets/issues/32"),
    )
    args = Namespace(
        mode="issue",
        issue_number="39",
        issue_kind="bug",
        slug="repo-release-workflow-failures",
        release_kind="script",
        release_repo="upstream/widgets",
        project="WidgetX",
        script_gate_issue=["32"],
    )
    result = preflight.run_preflight(args)
    output = capsys.readouterr().out
    assert result == 0
    assert "Result: PASS" in output
    assert "Release kind: script" in output
    assert "Script gate issues: #32" in output


