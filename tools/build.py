#!/usr/bin/env python3
import argparse
import json
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

HELP_FILE = Path(__file__).resolve().parent / "build_help.txt"
BUNDLE_ZIP_BASENAME = "sloppy-ethos_scripts"
PROJECT_BUILD_FILE = "build.json"
SCRIPT_LOCAL_TEST_DIR = "tests"


@dataclass(frozen=True)
class RadioFile:
    source: Path
    source_relative: Path
    destination: Path
    exclude_from_script: bool = True


@dataclass(frozen=True)
class ProjectInstallSpec:
    project_dir: Path
    project_name: str
    radio_files: tuple[RadioFile, ...]
    source_exclusions: tuple[Path, ...] = tuple()
    manifest_relative: Optional[Path] = None

    @property
    def script_destination(self) -> Path:
        return Path("scripts") / self.project_name

    @property
    def script_exclusions(self) -> tuple[Path, ...]:
        exclusions: list[Path] = [Path(SCRIPT_LOCAL_TEST_DIR)]
        if self.manifest_relative is not None:
            exclusions.append(self.manifest_relative)
        exclusions.extend(self.source_exclusions)
        for radio_file in self.radio_files:
            if radio_file.exclude_from_script and not any(
                _is_relative_to(radio_file.source_relative, excluded) for excluded in self.source_exclusions
            ):
                exclusions.append(radio_file.source_relative)
        return tuple(exclusions)


def load_config(path: Path):
    if not path.exists():
        return {}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        sys.exit(f"Failed to parse config at {path}: {exc}")


def load_json_object(path: Path, description: str):
    if not path.exists():
        return None

    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        sys.exit(f"Failed to parse {description} at {path}: {exc}")

    if not isinstance(payload, dict):
        sys.exit(f"{description.capitalize()} at {path} must contain a JSON object.")
    return payload


def _normalize_model_paths(raw_model_paths, config_path: Path):
    if raw_model_paths is None:
        sys.exit(f"Simulator path list 'ETHOS_SIM_PATHS' not found in config: {config_path}")
    if not isinstance(raw_model_paths, list):
        sys.exit(f"ETHOS_SIM_PATHS must be a JSON array in config: {config_path}")

    normalized = {}
    default_model: Optional[str] = None
    for entry in raw_model_paths:
        if not isinstance(entry, dict):
            sys.exit(
                f"Each ETHOS_SIM_PATHS entry must be an object with 'radio' and 'path' keys in {config_path}"
            )
        model = entry.get("radio")
        path_value = entry.get("path")
        if not model or not path_value:
            sys.exit(
                f"Each ETHOS_SIM_PATHS entry must include non-empty 'radio' and 'path' in {config_path}"
            )
        model_key = str(model)
        if model_key in normalized:
            sys.exit(f"Duplicate simulator radio '{model_key}' in ETHOS_SIM_PATHS at {config_path}")
        normalized[model_key] = str(path_value)

        if "default" in entry and not isinstance(entry.get("default"), bool):
            sys.exit(f"ETHOS_SIM_PATHS entry 'default' must be true/false in {config_path}")
        if entry.get("default") is True:
            if default_model is not None:
                sys.exit(f"Only one ETHOS_SIM_PATHS entry may set default=true in {config_path}")
            default_model = model_key

    return normalized, default_model


def resolve_simulator_path(config_path: Path, sim_radio: Optional[str] = None):
    config = load_config(config_path)
    model_paths, default_model = _normalize_model_paths(config.get("ETHOS_SIM_PATHS"), config_path)
    if sim_radio:
        model_path = model_paths.get(sim_radio)
        if not model_path:
            available = ", ".join(sorted(str(key) for key in model_paths.keys())) or "(none configured)"
            sys.exit(
                f"Simulator path for radio model '{sim_radio}' is not configured in {config_path}. "
                f"Configured models: {available}"
            )
        return Path(str(model_path))

    if not default_model:
        sys.exit(
            f"No default simulator path configured in ETHOS_SIM_PATHS at {config_path}. "
            "Set one entry with \"default\": true."
        )
    return Path(str(model_paths[default_model]))


def ensure_luac_available():
    luac = shutil.which("luac")
    if not luac:
        sys.exit("Required command 'luac' not found on PATH.")
    return luac


def run_lua_checks(project_dir: Path, luac_exec: str):
    if not project_dir.exists():
        sys.exit(f"Project directory not found: {project_dir}")

    target = project_dir / "main.lua"
    if not target.exists():
        sys.exit(f"Widget entrypoint not found: {target}")

    lua_files = sorted(project_dir.rglob("*.lua"))
    if not lua_files:
        sys.exit(f"No Lua files found under project directory: {project_dir}")

    for lua_file in lua_files:
        print(f"Checking Lua syntax: {lua_file}")
        subprocess.run([luac_exec, "-p", str(lua_file)], check=True)


def normalize_version(raw_value: str):
    version = raw_value.strip()
    if not version:
        sys.exit("Version cannot be empty.")
    if not re.match(r"^[0-9A-Za-z][0-9A-Za-z._-]*$", version):
        sys.exit(
            f"Invalid version '{version}'. Use only letters, numbers, '.', '-', '_' and no spaces."
        )
    return version


def resolve_version(repo_root: Path, explicit_version: Optional[str]):
    if explicit_version:
        return normalize_version(explicit_version)

    version_file = repo_root / "VERSION"
    if not version_file.exists():
        sys.exit(f"Version file not found: {version_file}")
    return normalize_version(version_file.read_text(encoding="utf-8").splitlines()[0])


def resolve_project_version(project_dir: Path, explicit_version: Optional[str]):
    if explicit_version:
        return normalize_version(explicit_version)

    version_file = project_dir / "VERSION"
    if not version_file.exists():
        sys.exit(f"Project version file not found: {version_file}")
    return normalize_version(version_file.read_text(encoding="utf-8").splitlines()[0])


def normalize_project_names(raw_projects: Optional[list[str]]):
    if not raw_projects:
        return ["SensorList"]

    normalized: list[str] = []
    seen: set[str] = set()
    for raw in raw_projects:
        project = str(raw).strip()
        if not project:
            sys.exit("Project name cannot be empty.")
        if project in seen:
            sys.exit(f"Duplicate project name '{project}' supplied via --project.")
        normalized.append(project)
        seen.add(project)
    return normalized


def _is_relative_to(path: Path, candidate_parent: Path):
    try:
        path.relative_to(candidate_parent)
    except ValueError:
        return False
    return True


def _normalize_relative_manifest_path(
    raw_value: object, field_name: str, manifest_path: Path, index: int, entry_name: str = "radioFiles"
):
    if not isinstance(raw_value, str) or not raw_value.strip():
        sys.exit(f"{manifest_path} {entry_name}[{index}] must include a non-empty '{field_name}' string.")

    normalized = raw_value.strip().replace("\\", "/")
    if re.match(r"^[A-Za-z]:", normalized) or normalized.startswith("/"):
        sys.exit(f"{manifest_path} {entry_name}[{index}] '{field_name}' must be a relative path.")

    parts = [part for part in normalized.split("/") if part and part != "."]
    if not parts or any(part == ".." for part in parts):
        sys.exit(
            f"{manifest_path} {entry_name}[{index}] '{field_name}' must stay within the project directory."
        )
    return Path(*parts)


def _normalize_asset_include(raw_value: object, manifest_path: Path, index: int):
    if isinstance(raw_value, str):
        includes = [raw_value]
    elif isinstance(raw_value, list):
        includes = raw_value
    else:
        sys.exit(f"{manifest_path} assets[{index}] must include a non-empty 'include' string or array.")

    normalized: list[str] = []
    for include_index, pattern in enumerate(includes, start=1):
        if not isinstance(pattern, str) or not pattern.strip():
            sys.exit(f"{manifest_path} assets[{index}] include[{include_index}] must be a non-empty string.")
        clean_pattern = pattern.strip().replace("\\", "/")
        if re.match(r"^[A-Za-z]:", clean_pattern) or clean_pattern.startswith("/"):
            sys.exit(f"{manifest_path} assets[{index}] include[{include_index}] must be a relative glob.")
        if any(part == ".." for part in clean_pattern.split("/")):
            sys.exit(f"{manifest_path} assets[{index}] include[{include_index}] must stay within the asset source.")
        normalized.append(clean_pattern)

    if not normalized:
        sys.exit(f"{manifest_path} assets[{index}] must include at least one glob pattern.")
    return tuple(normalized)


def _normalize_manifest_bool(
    raw_value: object, default: bool, field_name: str, manifest_path: Path, index: int, entry_name: str = "assets"
):
    if raw_value is None:
        return default
    if not isinstance(raw_value, bool):
        sys.exit(f"{manifest_path} {entry_name}[{index}] '{field_name}' must be true or false.")
    return raw_value


def _normalize_install_destination(
    raw_value: object, field_name: str, manifest_path: Path, index: int, entry_name: str, project_name: str
):
    destination = _normalize_relative_manifest_path(raw_value, field_name, manifest_path, index, entry_name)
    if destination.parts[0].lower() == "scripts" and (
        len(destination.parts) < 2 or destination.parts[1].lower() != project_name.lower()
    ):
        sys.exit(
            f"{manifest_path} {entry_name}[{index}] '{field_name}' destination "
            f"'{destination.as_posix()}' must stay outside scripts/ or under scripts/{project_name}/."
        )
    return destination


def _add_radio_file(
    radio_files: list[RadioFile],
    seen_destinations: set[str],
    source: Path,
    source_relative: Path,
    destination: Path,
    manifest_path: Path,
    context: str,
    exclude_from_script: bool = True,
):
    destination_key = destination.as_posix().lower()
    if destination_key in seen_destinations:
        sys.exit(f"{manifest_path} contains duplicate radio destination '{destination.as_posix()}' from {context}.")
    seen_destinations.add(destination_key)
    radio_files.append(
        RadioFile(
            source=source,
            source_relative=source_relative,
            destination=destination,
            exclude_from_script=exclude_from_script,
        )
    )


def _resolve_manifest_radio_files(
    manifest_path: Path,
    project_dir: Path,
    project_name: str,
    raw_radio_files: object,
    radio_files: list[RadioFile],
    seen_destinations: set[str],
):
    if not isinstance(raw_radio_files, list):
        sys.exit(f"'{manifest_path}' key 'radioFiles' must be a JSON array when present.")

    for index, entry in enumerate(raw_radio_files, start=1):
        if not isinstance(entry, dict):
            sys.exit(f"{manifest_path} radioFiles[{index}] must be an object with 'source' and 'destination'.")

        source_relative = _normalize_relative_manifest_path(entry.get("source"), "source", manifest_path, index)
        destination = _normalize_install_destination(
            entry.get("destination"), "destination", manifest_path, index, "radioFiles", project_name
        )

        source = project_dir / source_relative
        if not source.exists():
            sys.exit(f"{manifest_path} radioFiles[{index}] source file not found: {source}")
        if not source.is_file():
            sys.exit(f"{manifest_path} radioFiles[{index}] source must be a file: {source}")

        _add_radio_file(
            radio_files,
            seen_destinations,
            source,
            source_relative,
            destination,
            manifest_path,
            f"radioFiles[{index}]",
        )


def _resolve_manifest_assets(
    manifest_path: Path,
    project_dir: Path,
    project_name: str,
    raw_assets: object,
    radio_files: list[RadioFile],
    seen_destinations: set[str],
):
    if not isinstance(raw_assets, list):
        sys.exit(f"'{manifest_path}' key 'assets' must be a JSON array when present.")

    source_exclusions: list[Path] = []
    source_exclusion_keys: set[str] = set()
    for index, entry in enumerate(raw_assets, start=1):
        if not isinstance(entry, dict):
            sys.exit(f"{manifest_path} assets[{index}] must be an object.")

        source_root_relative = _normalize_relative_manifest_path(
            entry.get("source"), "source", manifest_path, index, "assets"
        )
        destination_root = _normalize_install_destination(
            entry.get("destination"), "destination", manifest_path, index, "assets", project_name
        )
        include_patterns = _normalize_asset_include(entry.get("include"), manifest_path, index)
        required = _normalize_manifest_bool(entry.get("required"), True, "required", manifest_path, index)
        flatten = _normalize_manifest_bool(entry.get("flatten"), False, "flatten", manifest_path, index)
        exclude_source = _normalize_manifest_bool(
            entry.get("excludeSource"), True, "excludeSource", manifest_path, index
        )

        source_root = project_dir / source_root_relative
        if exclude_source:
            source_exclusion_key = source_root_relative.as_posix().lower()
            if source_exclusion_key not in source_exclusion_keys:
                source_exclusion_keys.add(source_exclusion_key)
                source_exclusions.append(source_root_relative)
        if not source_root.exists():
            if required:
                sys.exit(f"{manifest_path} assets[{index}] source not found: {source_root}")
            continue
        if not source_root.is_dir():
            sys.exit(f"{manifest_path} assets[{index}] source must be a directory: {source_root}")

        matched_sources: list[Path] = []
        seen_sources: set[str] = set()
        for pattern in include_patterns:
            for source in sorted(source_root.glob(pattern)):
                if not source.is_file():
                    continue
                source_relative = source.relative_to(project_dir)
                source_key = source_relative.as_posix().lower()
                if source_key in seen_sources:
                    continue
                seen_sources.add(source_key)
                matched_sources.append(source)

        for source in matched_sources:
            source_relative = source.relative_to(project_dir)
            if flatten:
                destination = destination_root / source.name
            else:
                destination = destination_root / source.relative_to(source_root)
            _add_radio_file(
                radio_files,
                seen_destinations,
                source,
                source_relative,
                destination,
                manifest_path,
                f"assets[{index}]",
                exclude_from_script=exclude_source,
            )

    return tuple(source_exclusions)


def resolve_project_install_spec(project_dir: Path, project_name: str):
    manifest_path = project_dir / PROJECT_BUILD_FILE
    manifest = load_json_object(manifest_path, "project build manifest")
    if manifest is None:
        return ProjectInstallSpec(project_dir=project_dir, project_name=project_name, radio_files=tuple())

    raw_radio_files = manifest.get("radioFiles", [])
    raw_assets = manifest.get("assets", [])

    radio_files: list[RadioFile] = []
    seen_destinations: set[str] = set()
    _resolve_manifest_radio_files(manifest_path, project_dir, project_name, raw_radio_files, radio_files, seen_destinations)
    source_exclusions = _resolve_manifest_assets(
        manifest_path, project_dir, project_name, raw_assets, radio_files, seen_destinations
    )

    return ProjectInstallSpec(
        project_dir=project_dir,
        project_name=project_name,
        radio_files=tuple(radio_files),
        source_exclusions=source_exclusions,
        manifest_relative=Path(PROJECT_BUILD_FILE),
    )


def ensure_packaged_readme(destination: Path, project_name: str):
    package_readme = destination / "README.md"
    if package_readme.exists():
        return
    package_readme.write_text(
        (
            f"# {project_name}\n\n"
            "Install with Ethos Suite:\n\n"
            "1. Open Ethos Suite.\n"
            "2. Use the Lua script install/import function.\n"
            "3. Select this ZIP file.\n"
            "4. Sync/transfer to the radio.\n\n"
            f"This package installs to: scripts/{project_name}\n"
        ),
        encoding="utf-8",
    )


def write_bundle_manifest(
    destination: Path, project_names: list[str], project_versions: dict[str, str], repo_version: str
):
    lines = [
        "# sloppy-ethos multi-script bundle",
        "",
        "Included script projects:",
        "",
    ]
    for project_name in project_names:
        lines.append(f"- scripts/{project_name} (version {project_versions[project_name]})")
    lines.extend(
        [
            "",
            f"Repository version: {repo_version}",
            "",
            "Bundle ZIP names are intentionally unversioned.",
            "Individual script versions are sourced from scripts/<project>/VERSION.",
        ]
    )
    (destination / "README.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def prune_script_exclusions(script_destination: Path, exclusions: tuple[Path, ...]):
    for relative_path in exclusions:
        target = script_destination / relative_path
        if target.is_dir():
            shutil.rmtree(target, ignore_errors=True)
        elif target.exists():
            target.unlink()

        current = target.parent
        while current != script_destination and current.exists():
            try:
                current.rmdir()
            except OSError:
                break
            current = current.parent


def copy_radio_files(radio_files: tuple[RadioFile, ...], destination_root: Path):
    for radio_file in radio_files:
        target = destination_root / radio_file.destination
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(radio_file.source, target)


def stage_project_files(install_spec: ProjectInstallSpec, destination_root: Path, dirs_exist_ok: bool = False):
    script_destination = destination_root / install_spec.script_destination
    shutil.copytree(install_spec.project_dir, script_destination, dirs_exist_ok=dirs_exist_ok)
    prune_script_exclusions(script_destination, install_spec.script_exclusions)
    ensure_packaged_readme(script_destination, install_spec.project_name)
    copy_radio_files(install_spec.radio_files, destination_root)


def ensure_unique_radio_destinations(project_name: str, radio_files: tuple[RadioFile, ...], seen: dict[str, str]):
    for radio_file in radio_files:
        key = radio_file.destination.as_posix().lower()
        previous_owner = seen.get(key)
        if previous_owner is not None:
            sys.exit(
                "Radio file destination conflict while bundling projects: "
                f"'{radio_file.destination.as_posix()}' declared by both '{previous_owner}' and '{project_name}'."
            )
        seen[key] = project_name


def build_zip(project_dir: Path, project_name: str, version: str, dist_dir: Path, repo_root: Path):
    dist_dir.mkdir(parents=True, exist_ok=True)
    staging_root = repo_root / ".build-staging"
    if staging_root.exists():
        shutil.rmtree(staging_root, ignore_errors=True)

    try:
        base_name = dist_dir / f"{project_name}-{version}"
        staging = staging_root / "package"
        install_spec = resolve_project_install_spec(project_dir, project_name)
        stage_project_files(install_spec, staging)
        archive_path = shutil.make_archive(str(base_name), "zip", staging)
    finally:
        shutil.rmtree(staging_root, ignore_errors=True)

    print(f"Packaged widget ZIP: {archive_path}")
    return Path(archive_path)


def build_multi_project_zip(project_names: list[str], dist_dir: Path, repo_root: Path):
    if not project_names:
        sys.exit("At least one project is required for bundle packaging.")

    dist_dir.mkdir(parents=True, exist_ok=True)
    staging_root = repo_root / ".build-staging"
    if staging_root.exists():
        shutil.rmtree(staging_root, ignore_errors=True)

    try:
        staging = staging_root / "package"
        project_versions: dict[str, str] = {}
        seen_radio_destinations: dict[str, str] = {}
        for project_name in project_names:
            project_dir = repo_root / "scripts" / project_name
            if not project_dir.exists():
                sys.exit(f"Project directory not found: {project_dir}")
            install_spec = resolve_project_install_spec(project_dir, project_name)
            ensure_unique_radio_destinations(project_name, install_spec.radio_files, seen_radio_destinations)
            stage_project_files(install_spec, staging)
            project_versions[project_name] = resolve_project_version(project_dir, None)

        repo_version = resolve_version(repo_root, None)
        write_bundle_manifest(staging, project_names, project_versions, repo_version)
        base_name = dist_dir / BUNDLE_ZIP_BASENAME
        archive_path = shutil.make_archive(str(base_name), "zip", staging)
    finally:
        shutil.rmtree(staging_root, ignore_errors=True)

    print(f"Packaged multi-script ZIP: {archive_path}")
    return Path(archive_path)


def format_deploy_error(target: Path, exc: OSError):
    hint_lines = [
        f"Failed to deploy '{target.name}' to simulator path '{target}'.",
    ]

    if isinstance(exc, PermissionError):
        hint_lines.extend(
            [
                "Permission denied while writing to the simulator folder.",
                "Possible causes:",
                "- Ethos Suite or simulator still has the folder open.",
                "- Current shell/session cannot write outside the workspace sandbox.",
                "- Windows ACL/antivirus is blocking writes for this process.",
            ]
        )
    else:
        hint_lines.append("Unexpected filesystem error while copying files.")

    hint_lines.extend(
        [
            "Suggested checks:",
            "- Close Ethos Suite and retry.",
            "- Retry from a shell/session with elevated filesystem access.",
            f"Details: {exc}",
        ]
    )
    return "\n".join(hint_lines)


def deploy_to_simulator(project_dir: Path, project_name: str, sim_path: Path):
    if not sim_path.exists():
        sys.exit(f"Simulator path '{sim_path}' does not exist.")
    install_spec = resolve_project_install_spec(project_dir, project_name)
    target = sim_path / install_spec.script_destination
    target.parent.mkdir(parents=True, exist_ok=True)
    try:
        stage_project_files(install_spec, sim_path, dirs_exist_ok=True)
    except OSError as exc:
        sys.exit(format_deploy_error(target, exc))
    print(f"Deployed {project_name} -> {target}")


def clean_from_simulator(project_dir: Path, project_name: str, sim_path: Path):
    if not sim_path.exists():
        sys.exit(f"Simulator path '{sim_path}' does not exist.")
    install_spec = resolve_project_install_spec(project_dir, project_name)

    targets: list[Path] = [sim_path / install_spec.script_destination]
    targets.extend(sim_path / radio_file.destination for radio_file in install_spec.radio_files)

    for target in targets:
        if not target.exists():
            print(f"Clean skip: simulator target not found: {target}")
            continue
        try:
            if target.is_dir():
                shutil.rmtree(target)
            else:
                target.unlink()
        except OSError as exc:
            sys.exit(f"Failed to clean simulator target '{target}': {exc}")
        print(f"Cleaned simulator target: {target}")


def clean_dist_dir(dist_dir: Path):
    if not dist_dir.exists():
        print(f"Clean skip: dist directory not found: {dist_dir}")
        return
    artifacts = list(dist_dir.iterdir())
    if not artifacts:
        print(f"Dist directory already empty: {dist_dir}")
        return
    for artifact in artifacts:
        try:
            if artifact.is_dir():
                shutil.rmtree(artifact)
            else:
                artifact.unlink()
        except OSError as exc:
            sys.exit(f"Failed to clean dist artifact '{artifact}': {exc}")
    print(f"Cleaned dist directory: {dist_dir}")


def print_help_text():
    if not HELP_FILE.exists():
        sys.exit(f"Help file not found: {HELP_FILE}")
    print(HELP_FILE.read_text(encoding="utf-8"))


def parse_args():
    parser = argparse.ArgumentParser(description="Build or deploy Ethos widget packages.", add_help=False)
    parser.add_argument("--help", action="store_true", help="Print build command reference.")
    parser.add_argument(
        "--project",
        "-p",
        action="append",
        default=None,
        help="Project folder under scripts/. Repeat for multi-project dist bundles.",
    )
    parser.add_argument("--dist", action="store_true", help="Produce an Ethos install ZIP in dist/.")
    parser.add_argument(
        "--deploy", action="store_true", help="Copy the widget folder into the simulator scripts directory."
    )
    parser.add_argument(
        "--clean", action="store_true", help="Remove deployed widget folder from the simulator scripts directory."
    )
    parser.add_argument("--sim-radio", help="Radio model key to select simulator path from ETHOS_SIM_PATHS.")
    parser.add_argument("--config", "-c", help="Path to JSON config containing ETHOS_SIM_PATHS.")
    parser.add_argument("--no-zip", action="store_true", help="Skip ZIP even when --dist is provided.")
    parser.add_argument(
        "--version",
        help="Override script package version (single-project --dist only).",
    )
    parser.add_argument(
        "--out-dir",
        help="Output directory for ZIP artifacts (default: dist/ at repo root).",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    if args.help:
        print_help_text()
        return

    projects = normalize_project_names(args.project)
    primary_project = projects[0]
    multi_project = len(projects) > 1

    if multi_project and (args.deploy or args.clean):
        sys.exit("Multiple --project values are supported with --dist only.")

    if multi_project and args.version:
        sys.exit("--version override is only supported for single-project --dist packaging.")

    repo_root = Path(__file__).resolve().parent.parent
    primary_project_dir = repo_root / "scripts" / primary_project
    dist_dir = Path(args.out_dir) if args.out_dir else (repo_root / "dist")

    if args.dist or args.deploy:
        luac_exec = ensure_luac_available()
        for project_name in projects:
            run_lua_checks(repo_root / "scripts" / project_name, luac_exec)

    if not args.dist and not args.deploy and not args.clean:
        sys.exit("Nothing to do: specify --dist, --deploy, or --clean.")

    if args.deploy or args.clean:
        config_path = Path(args.config) if args.config else (repo_root / "tools" / "deploy.config.json")
        sim_path = resolve_simulator_path(config_path, args.sim_radio)
        if not sim_path:
            sys.exit("Simulator path not configured. Configure ETHOS_SIM_PATHS in tools/deploy.config.json.")
        if args.sim_radio and not sim_path.exists():
            sys.exit(f"Simulator path for radio model '{args.sim_radio}' does not exist: {sim_path}")
        if args.clean:
            clean_from_simulator(primary_project_dir, primary_project, sim_path)
            clean_dist_dir(dist_dir)
    if args.deploy:
        deploy_to_simulator(primary_project_dir, primary_project, sim_path)

    if args.dist and not args.no_zip:
        if multi_project:
            build_multi_project_zip(projects, dist_dir, repo_root)
        else:
            version = resolve_project_version(primary_project_dir, args.version)
            build_zip(primary_project_dir, primary_project, version, dist_dir, repo_root)


if __name__ == "__main__":
    main()
