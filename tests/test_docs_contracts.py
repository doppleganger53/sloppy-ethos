from __future__ import annotations

import json
import re
import shlex
from pathlib import Path
from urllib.parse import urlparse

import pytest

from tests.helpers import REPO_ROOT
from tests.test_docs_commands import DOC_FILES, discover_commands, is_manual_command


README_PATH = REPO_ROOT / "README.md"
DEVELOPMENT_PATH = REPO_ROOT / "docs" / "DEVELOPMENT.md"
REPOSITORY_LAYOUT_PATH = REPO_ROOT / "docs" / "REPOSITORY_LAYOUT.md"
COMPATIBILITY_BASELINE_PATH = REPO_ROOT / "docs" / "ETHOS_26_1_COMPATIBILITY.md"
CONTRIBUTING_PATH = REPO_ROOT / "CONTRIBUTING.md"
CODEOWNERS_PATH = REPO_ROOT / ".github" / "CODEOWNERS"
ARCHITECTURE_PATH = REPO_ROOT / "docs" / "SensorList" / "SENSORLIST_ARCHITECTURE.md"
REQUIRED_EXTERNAL_LINKS = (
    ("Visual Studio Code", "https://code.visualstudio.com/"),
    ("OpenAI Codex", "https://openai.com/codex/"),
)
AGENTS_PATH = REPO_ROOT / "AGENTS.md"
PROMPTS_ROOT = REPO_ROOT / "deslopification" / "prompts"
LIVE_GUIDANCE_FILES = [
    AGENTS_PATH,
    REPO_ROOT / "deslopification" / "memory" / "README.md",
    REPO_ROOT / "deslopification" / "memory" / "CURRENT_STATE.md",
    *sorted((PROMPTS_ROOT / "templates").glob("*.md")),
    *sorted((PROMPTS_ROOT / "issues").glob("*.md")),
]


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _extract_backtick_tokens(text: str) -> list[str]:
    return [token.strip() for token in re.findall(r"`([^`\n]+)`", text) if token.strip()]


def _extract_markdown_links(text: str) -> list[tuple[str, str]]:
    return [(label.strip(), target.strip()) for label, target in re.findall(r"\[([^\]]+)\]\(([^)]+)\)", text)]


def _extract_section(text: str, heading: str) -> str:
    lines = text.splitlines()
    capture = False
    collected: list[str] = []
    for line in lines:
        if line.strip() == heading:
            capture = True
            continue
        if capture and line.startswith("## "):
            break
        if capture:
            collected.append(line)
    return "\n".join(collected).strip()


def _looks_like_local_file_reference(token: str) -> bool:
    if "://" in token or " " in token:
        return False
    if token == "VERSION":
        return True
    if token.startswith(("dist/", "dist\\", "${", "<")):
        return False
    return token.endswith((".md", ".py", ".ps1", ".lua", ".json", ".txt", ".toml"))


def _iter_documented_file_refs() -> list[str]:
    refs: list[str] = []
    for path in DOC_FILES:
        for token in _extract_backtick_tokens(_read(path)):
            if _looks_like_local_file_reference(token):
                refs.append(token)
    return sorted(set(refs))


def _has_companion_example(ref: str) -> bool:
    path = Path(ref)
    example_name = f"{path.stem}.example{path.suffix}"
    return (REPO_ROOT / path.with_name(example_name)).exists()


def _iter_markdown_links() -> list[tuple[Path, str, str]]:
    links: list[tuple[Path, str, str]] = []
    for path in DOC_FILES:
        for label, target in _extract_markdown_links(_read(path)):
            links.append((path, label, target))
    return links


@pytest.mark.parametrize(
    ("path", "sections"),
    [
        (README_PATH, ("## Quick Start", "## Development", "### Running the Python tests")),
        (DEVELOPMENT_PATH, ("## Prerequisites", "## Core Commands", "## Running the Python tests")),
    ],
)
def test_docs_include_required_sections(path: Path, sections: tuple[str, ...]):
    text = _read(path)
    for section in sections:
        assert section in text, f"Missing required section '{section}' in {path}."


@pytest.mark.parametrize("ref", _iter_documented_file_refs())
def test_documented_local_file_references_exist(ref: str):
    path = REPO_ROOT / ref
    if path.exists():
        return

    # Ethos compatibility docs cite upstream reference-checkout examples under lua/.
    if ref.startswith("lua/") or ref.startswith("lua\\"):
        return

    # Allow local-only config files when docs also point to a committed template.
    if ref.endswith(".json") and _has_companion_example(ref):
        return

    assert False, f"Missing documented path: {ref}"


def test_python_version_guidance_consistent_across_docs():
    assert "Python 3.9+" in _read(README_PATH)
    assert "Python 3.9+" in _read(DEVELOPMENT_PATH)


def test_test_dependency_guidance_consistent_across_docs():
    readme_text = _read(README_PATH)
    development_text = _read(DEVELOPMENT_PATH)
    assert "requirements/dev.txt" in readme_text
    assert "requirements/dev.txt" in development_text
    assert "pytest-cov" in readme_text
    assert "pytest-cov" in development_text


def test_zip_naming_policy_documented():
    assert "dist/{ProjectName}-{version}.zip" in _read(README_PATH)
    assert "dist/{ProjectName}-{version}.zip" in _read(DEVELOPMENT_PATH)


def test_multi_script_zip_naming_exception_documented():
    assert "dist/sloppy-ethos_scripts.zip" in _read(README_PATH)
    assert "dist/sloppy-ethos_scripts.zip" in _read(DEVELOPMENT_PATH)


def test_root_version_file_exists_and_nonempty():
    version_text = (REPO_ROOT / "VERSION").read_text(encoding="utf-8").strip()
    assert version_text, "VERSION file must contain a non-empty version string."


def test_script_version_files_exist_and_nonempty():
    scripts_root = REPO_ROOT / "scripts"
    for project_dir in scripts_root.iterdir():
        if not project_dir.is_dir():
            continue
        version_file = project_dir / "VERSION"
        assert version_file.exists(), f"Missing script version file: {version_file}"
        assert version_file.read_text(encoding="utf-8").strip(), f"Empty script version file: {version_file}"


def test_codeowners_exists():
    assert CODEOWNERS_PATH.exists(), "Expected .github/CODEOWNERS to exist."


def test_readme_has_visual_overview_section():
    assert "## Visual Overview" in _read(README_PATH)


def test_readme_links_sensorlist_architecture_doc():
    assert "(docs/SensorList/SENSORLIST_ARCHITECTURE.md)" in _read(README_PATH)
    assert ARCHITECTURE_PATH.exists()


def test_core_docs_link_ethos_26_1_compatibility_baseline():
    assert "(docs/ETHOS_26_1_COMPATIBILITY.md)" in _read(README_PATH)
    assert "(ETHOS_26_1_COMPATIBILITY.md)" in _read(DEVELOPMENT_PATH)
    assert "(ETHOS_26_1_COMPATIBILITY.md)" in _read(REPOSITORY_LAYOUT_PATH)
    assert COMPATIBILITY_BASELINE_PATH.exists()


def test_ethos_26_1_compatibility_baseline_captures_reference_targets_and_follow_ups():
    text = _read(COMPATIBILITY_BASELINE_PATH)
    for heading in ("## Reference Checkout", "## Validation Targets", "## Compatibility Matrix", "### API Surface Matrix"):
        assert heading in text
    assert "ETHOS-Feedback-Community" in text
    assert "`26.1`" in text
    assert re.search(r"Commit:\s*`?[0-9a-f]{8}`?", text)
    assert "git clone" in text
    assert re.search(r"git\s+(-C\s+\S+\s+)?fetch\b", text)
    assert "X20RS" in text
    assert "X20S" in text
    for target in ("../tools/sim/harness/README.md",
                   "../tools/sim/harness/suites/SensorList-X20RS-FCC.json"):
        assert f"]({target})" in text
        assert (COMPATIBILITY_BASELINE_PATH.parent / target).is_file()
    for area in ("SensorList", "BoundryMap", "ethos_events", "SmartMapper", "Arduino FBus"):
        assert area in text


def test_ethos_26_1_api_surface_matrix_lists_shared_surfaces_and_smoke_targets():
    text = _read(COMPATIBILITY_BASELINE_PATH)
    for token in (
        "system.registerWidget",
        "system.registerSystemTool",
        "system.registerTask",
        "system.getMemoryUsage",
        "system.getSource",
        "system.getSensors",
        "model.getSensors",
        "form.addSensorField",
        "form.addSourceField",
        "form.addChoiceField",
        "form.addNumberField",
        "form.addTextButton",
        "form.addBitmapField",
        "form.addStaticText",
        "form.addBooleanField",
        "EVT_TOUCH",
        "16640",
        "16641",
        "16642",
    ):
        assert token in text
    for token in (
        "SensorList: yes",
        "BoundryMap: yes",
        "ethos_events: yes",
        "SmartMapper: no",
        "FBus: no",
        "python -m pytest scripts/SensorList/tests -q",
        "tools/sim/harness/suites/SensorList-X20RS-FCC.json",
        "python -m pytest scripts/BoundryMap/tests -q",
        "luac -p scripts/ethos_events/main.lua",
    ):
        assert token in text


def test_ethos_26_1_compatibility_baseline_links_follow_up_issues():
    text = _read(COMPATIBILITY_BASELINE_PATH)
    links = set(re.findall(r"https://github\.com/doppleganger53/sloppy-ethos/issues/(\d+)", text))
    assert len(links) >= 5


def test_contributing_has_workflow_section():
    assert "## Workflow" in _read(CONTRIBUTING_PATH)


@pytest.mark.parametrize(
    "path",
    [
        README_PATH,
        DEVELOPMENT_PATH,
        CONTRIBUTING_PATH,
        PROMPTS_ROOT / "templates" / "ISSUE_RESOLUTION_TEMPLATE.md",
        PROMPTS_ROOT / "templates" / "RELEASE_RESOLUTION_TEMPLATE.md",
    ],
)
def test_workflow_entrypoints_link_to_canonical_validation(path: Path):
    targets = [target for _, target in _extract_markdown_links(_read(path))]
    canonical_links = [
        target
        for target in targets
        if (path.parent / target.split("#", 1)[0]).resolve() == AGENTS_PATH.resolve()
    ]
    assert canonical_links, f"Missing canonical policy link in {path}"
    assert any(target.endswith("#validation-matrix") for target in canonical_links), (
        f"Missing canonical validation link in {path}"
    )


@pytest.mark.parametrize("path", LIVE_GUIDANCE_FILES)
def test_live_guidance_local_links_and_sections_exist(path: Path):
    for label, target in _extract_markdown_links(_read(path)):
        if urlparse(target).scheme:
            continue
        local_path, _, anchor = target.partition("#")
        destination = (path.parent / local_path).resolve() if local_path else path
        assert destination.is_file(), f"Broken link in {path}: [{label}]({target})"
        if anchor:
            headings = re.findall(r"^#{1,6}\s+(.+)$", _read(destination), re.MULTILINE)
            anchors = {
                re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-")
                for heading in headings
            }
            assert anchor in anchors, f"Missing section in {path}: [{label}]({target})"


@pytest.mark.parametrize("path", LIVE_GUIDANCE_FILES)
def test_live_guidance_concrete_source_and_test_paths_exist(path: Path):
    # Inspect references only: never execute deployment/release commands or
    # treat completed issue prompts as current instructions. Placeholder paths
    # deliberately do not match; concrete examples must resolve in this checkout.
    refs = re.findall(r"\b(?:scripts|tests|tools)/[\w./-]+\.(?:py|lua|json)\b", _read(path))
    for ref in refs:
        assert (REPO_ROOT / ref).is_file(), f"Missing live guidance path in {path}: {ref}"


def _check_validation_targets(root: Path, command: str):
    # Resolve the concrete arguments even for commands we never execute in docs CI.
    args = shlex.split(command)
    if args[:3] == ["python", "-m", "pytest"]:
        for arg in args[3:]:
            if arg.startswith("-") or "{" in arg:
                continue
            path = root / arg.split("::", 1)[0]
            assert path.is_file() or (path.is_dir() and any(path.rglob("test_*.py"))), (
                f"Missing pytest target: {arg}"
            )
    if args[:2] == ["python", "tools/sim/harness/run.py"]:
        for index, arg in enumerate(args):
            if arg == "--suite" or arg.startswith("--suite="):
                if arg == "--suite":
                    assert index + 1 < len(args), f"Missing suite path: {command}"
                    suite = args[index + 1]
                else:
                    suite = arg.partition("=")[2]
                if "{" not in suite:
                    assert (root / suite).is_file(), f"Missing suite: {suite}"


@pytest.mark.parametrize("path", sorted(set(DOC_FILES + LIVE_GUIDANCE_FILES)))
def test_documented_validation_targets_exist(path: Path):
    for command in discover_commands([path]):
        _check_validation_targets(REPO_ROOT, command)


@pytest.mark.parametrize("command", [
    "python -m pytest existing/tests missing/tests -q",
    "python tools/sim/harness/run.py headless --suite missing.json",
])
def test_missing_validation_target_is_reported_offline(tmp_path: Path, command: str):
    existing = tmp_path / "existing" / "tests"
    existing.mkdir(parents=True)
    (existing / "test_example.py").touch()
    with pytest.raises(AssertionError, match="missing"):
        _check_validation_targets(tmp_path, command)


def test_vscode_pytest_enabled_in_repo_settings():
    settings = json.loads((REPO_ROOT / ".vscode" / "settings.json").read_text(encoding="utf-8"))
    assert settings.get("python.testing.pytestEnabled") is True
    assert settings.get("python.testing.pytestArgs") == ["tests", "scripts"]


def test_script_local_tests_are_discovered_from_repo_root():
    pytest_config = (REPO_ROOT / "pytest.ini").read_text(encoding="utf-8")
    assert "testpaths =" in pytest_config
    assert "tests" in pytest_config
    assert "scripts" in pytest_config
    assert (REPO_ROOT / "scripts" / "SensorList" / "tests" / "test_sensorlist_widget.py").exists()
    assert (REPO_ROOT / "scripts" / "BoundryMap" / "tests" / "test_boundrymap_widget.py").exists()
    assert not (REPO_ROOT / "tests" / "lua").exists()




def test_readme_download_links_identify_released_script_versions():
    section = _extract_section(_read(README_PATH), "## Download Latest Script Releases")
    links = _extract_markdown_links(section)
    seen_projects: set[str] = set()
    for label, target in links:
        if not urlparse(target).path.endswith(".zip"):
            continue
        match = re.fullmatch(r"Download (.+) \(v(\d+\.\d+\.\d+)\)", label)
        assert match, f"Expected a versioned script download label: {label}"
        project, released = match.groups()
        version_file = REPO_ROOT / "scripts" / project / "VERSION"
        current = _read(version_file).strip().split("-")[0]
        assert tuple(map(int, released.split("."))) <= tuple(map(int, current.split(".")))
        assert target == (
            f"https://github.com/doppleganger53/sloppy-ethos/releases/download/"
            f"{project}-v{released}/{project}-{released}.zip"
        )
        assert project not in seen_projects, f"Duplicate published link for {project}"
        seen_projects.add(project)
    assert seen_projects, "At least one published single-script release must be linked."


def test_required_recommended_links_present_in_readme():
    readme_text = _read(README_PATH)
    links = set(_extract_markdown_links(readme_text))
    for required in REQUIRED_EXTERNAL_LINKS:
        assert required in links, f"Missing required README link: {required[0]} -> {required[1]}"


@pytest.mark.parametrize(("path", "label", "target"), _iter_markdown_links())
def test_markdown_link_targets_are_valid(path: Path, label: str, target: str):
    if target.startswith(("http://", "https://")):
        parsed = urlparse(target)
        assert parsed.scheme == "https", f"External docs link must use HTTPS: {target}"
        assert parsed.netloc, f"External docs link missing host: {target}"
        return

    if target.startswith(("#", "mailto:")):
        return

    local_target = target.split("#", 1)[0]
    assert local_target, f"Invalid markdown link target in {path}: [{label}]({target})"
    assert (path.parent / local_target).exists(), f"Broken local markdown link in {path}: [{label}]({target})"


def test_environment_dependent_doc_commands_are_manual():
    def should_be_manual(command: str) -> bool:
        return (
            "--dist" in command
            or "--deploy" in command
            or "--clean" in command
            or command.startswith("python tools/sim/harness/run.py")
            or command.startswith("python -m pytest")
            or command.startswith("powershell ")
            or command == "python -m pip install -r requirements/dev.txt"
        )

    for command in discover_commands():
        if should_be_manual(command):
            assert is_manual_command(command), f"Environment-dependent command not classified manual: {command}"
