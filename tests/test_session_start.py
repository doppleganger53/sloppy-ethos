from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path


def load_session_start_module():
    repo_root = Path(__file__).resolve().parents[1]
    session_start_path = repo_root / "tools" / "session_start.py"
    spec = importlib.util.spec_from_file_location("session_start_module", session_start_path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


session_start = load_session_start_module()


def test_normalize_issue_kind_maps_refactor_to_chore():
    assert session_start.normalize_issue_kind_label("refactor") == "chore"
    assert session_start.normalize_issue_kind_label("workflow") == "chore"
    assert session_start.normalize_issue_kind_label("docs") == "docs"


def test_infer_issue_kind_prefers_labels():
    assert session_start.infer_issue_kind(["bug", "enhancement"], "[Enhancement] Example") == "bug"


def test_infer_issue_kind_falls_back_to_title_prefix():
    assert session_start.infer_issue_kind([], "[Refactor] Improve tests") == "chore"
    assert session_start.infer_issue_kind([], "[Docs] Clarify setup") == "docs"


def test_slugify_issue_title_removes_prefix_and_symbols():
    title = "[Enhancement] Add Better Mapping! (v2)"
    assert session_start.slugify_issue_title(title) == "add-better-mapping-v2"


def test_collect_issue_context_uses_inference(monkeypatch):
    monkeypatch.setattr(
        session_start,
        "run_gh_json",
        lambda _args: {
            "number": 60,
            "title": "Replace PowerShell-only coverage clean command",
            "url": "https://github.com/example/repo/issues/60",
            "state": "OPEN",
            "labels": [{"name": "workflow"}],
        },
    )
    monkeypatch.setattr(
        session_start.session_preflight,
        "build_recommended_branch",
        lambda issue_kind, issue_number, slug: f"{issue_kind}/{issue_number}-{slug}",
    )

    ctx = session_start.collect_issue_context("60", issue_kind_override=None, slug_override=None)
    assert ctx.issue_kind == "chore"
    assert ctx.slug == "replace-powershell-only-coverage-clean-command"
    assert ctx.recommended_branch == "chore/60-replace-powershell-only-coverage-clean-command"


def test_should_attempt_checkout_rules():
    assert session_start.should_attempt_checkout({"status": "PASS_WITH_WARNING"}) is True
    assert session_start.should_attempt_checkout(
        {"status": "BLOCKED", "message": "Issue-linked work cannot mutate on 'main'."}
    ) is True
    assert session_start.should_attempt_checkout(
        {"status": "BLOCKED", "message": "Script release requires all script gate issues to be CLOSED."}
    ) is False


def test_run_issue_json_output(monkeypatch, capsys):
    issue_ctx = session_start.IssueContext(
        number="45",
        title="[Enhancement] SmartMapper function mapping script",
        url="https://github.com/example/repo/issues/45",
        state="OPEN",
        issue_kind="enhancement",
        slug="smartmapper-function-mapping-script",
        recommended_branch="feature/45-smartmapper-function-mapping-script",
    )
    monkeypatch.setattr(session_start, "collect_issue_context", lambda *_args, **_kwargs: issue_ctx)
    monkeypatch.setattr(
        session_start,
        "run_preflight_json",
        lambda _args: (
            0,
            {
                "status": "PASS",
                "message": None,
                "next_steps": [],
                "reminder": None,
            },
        ),
    )

    args = argparse.Namespace(
        mode="issue",
        issue_number="45",
        issue_kind=None,
        slug=None,
        checkout=False,
        strict_branch_match=False,
        release_kind=None,
        project=None,
        script_gate_issue=[],
        json=True,
    )
    result = session_start._run_issue(args)
    output = capsys.readouterr().out
    payload = json.loads(output)

    assert result == 0
    assert payload["issue"]["issue_kind"] == "enhancement"
    assert payload["preflight"]["status"] == "PASS"
    assert payload["checkout_attempted"] is False
