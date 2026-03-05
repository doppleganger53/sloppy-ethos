from __future__ import annotations

import importlib.util
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
    assert "git checkout -b feature/16-memory-optimization" in output


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
            "--project",
            "SensorList",
            "--script-gate-issue",
            "32",
        ]
    )
    assert args.release_kind == "script"
    assert args.project == "SensorList"
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
            "--project",
            "SensorList",
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
            "SensorList",
        ],
    ],
)
def test_parse_args_rejects_invalid_release_scope_combinations(argv):
    with pytest.raises(SystemExit):
        preflight.parse_args(argv)


def test_issue_mode_script_release_blocks_when_gate_issue_open(monkeypatch, capsys):
    monkeypatch.setattr(preflight, "get_current_branch", lambda: "fix/39-repo-release-workflow-failures")
    monkeypatch.setattr(preflight, "is_worktree_dirty", lambda: False)
    monkeypatch.setattr(
        preflight,
        "get_issue_metadata",
        lambda _issue_number: ("OPEN", "Manual testing of SensorList-v1.0.0 changes", "https://github.com/example/32"),
    )
    args = Namespace(
        mode="issue",
        issue_number="39",
        issue_kind="bug",
        slug="repo-release-workflow-failures",
        release_kind="script",
        project="SensorList",
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
        lambda _issue_number: ("CLOSED", "Manual testing complete", "https://github.com/example/32"),
    )
    args = Namespace(
        mode="issue",
        issue_number="39",
        issue_kind="bug",
        slug="repo-release-workflow-failures",
        release_kind="script",
        project="SensorList",
        script_gate_issue=["32"],
    )
    result = preflight.run_preflight(args)
    output = capsys.readouterr().out
    assert result == 0
    assert "Result: PASS" in output
    assert "Release kind: script" in output
    assert "Script gate issues: #32" in output


