#!/usr/bin/env python3
import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Optional

HELP_FILE = Path(__file__).resolve().parent / "build_help.txt"


def load_config(path: Path):
    if not path.exists():
        return {}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        sys.exit(f"Failed to parse config at {path}: {exc}")


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


def build_zip(project_dir: Path, project_name: str, version: str, dist_dir: Path, repo_root: Path):
    dist_dir.mkdir(parents=True, exist_ok=True)
    staging_root = repo_root / ".build-staging"
    if staging_root.exists():
        shutil.rmtree(staging_root, ignore_errors=True)

    try:
        base_name = dist_dir / f"{project_name}-{version}"
        staging = staging_root / "package"
        destination = staging / "scripts" / project_name
        shutil.copytree(project_dir, destination)
        ensure_packaged_readme(destination, project_name)
        archive_path = shutil.make_archive(str(base_name), "zip", staging)
    finally:
        shutil.rmtree(staging_root, ignore_errors=True)

    print(f"Packaged widget ZIP: {archive_path}")
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
    target = sim_path / "scripts" / project_name
    target.parent.mkdir(parents=True, exist_ok=True)
    try:
        shutil.copytree(project_dir, target, dirs_exist_ok=True)
    except OSError as exc:
        sys.exit(format_deploy_error(target, exc))
    print(f"Deployed {project_name} -> {target}")


def clean_from_simulator(project_name: str, sim_path: Path):
    if not sim_path.exists():
        sys.exit(f"Simulator path '{sim_path}' does not exist.")
    target = sim_path / "scripts" / project_name
    if not target.exists():
        print(f"Clean skip: simulator target not found: {target}")
        return
    try:
        shutil.rmtree(target)
    except OSError as exc:
        sys.exit(f"Failed to clean simulator target '{target}': {exc}")
    print(f"Cleaned simulator target: {target}")


def print_help_text():
    if not HELP_FILE.exists():
        sys.exit(f"Help file not found: {HELP_FILE}")
    print(HELP_FILE.read_text(encoding="utf-8"))


def parse_args():
    parser = argparse.ArgumentParser(description="Build or deploy Ethos widget packages.", add_help=False)
    parser.add_argument("--help", action="store_true", help="Print build command reference.")
    parser.add_argument("--project", "-p", default="SensorList", help="Name of the widget project.")
    parser.add_argument("--dist", action="store_true", help="Produce an Ethos install ZIP in dist/.")
    parser.add_argument("--deploy", action="store_true", help="Copy the widget folder into the simulator scripts directory.")
    parser.add_argument("--clean", action="store_true", help="Remove deployed widget folder from the simulator scripts directory.")
    parser.add_argument("--sim-radio", help="Radio model key to select simulator path from ETHOS_SIM_PATHS.")
    parser.add_argument("--config", "-c", help="Path to JSON config containing ETHOS_SIM_PATHS.")
    parser.add_argument("--no-zip", action="store_true", help="Skip ZIP even when --dist is provided.")
    parser.add_argument("--version", help="Override package version (default: read from VERSION file at repo root).")
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

    repo_root = Path(__file__).resolve().parent.parent
    project_dir = repo_root / "scripts" / args.project
    if args.dist or args.deploy:
        luac_exec = ensure_luac_available()
        run_lua_checks(project_dir, luac_exec)

    if not args.dist and not args.deploy and not args.clean:
        sys.exit("Nothing to do: specify --dist, --deploy, or --clean.")

    if args.dist and not args.no_zip:
        version = resolve_version(repo_root, args.version)
        dist_dir = Path(args.out_dir) if args.out_dir else (repo_root / "dist")
        build_zip(project_dir, args.project, version, dist_dir, repo_root)

    if args.deploy or args.clean:
        config_path = Path(args.config) if args.config else (repo_root / "tools" / "deploy.config.json")
        sim_path = resolve_simulator_path(config_path, args.sim_radio)
        if not sim_path:
            sys.exit("Simulator path not configured. Configure ETHOS_SIM_PATHS in tools/deploy.config.json.")
        if args.sim_radio and not sim_path.exists():
            sys.exit(f"Simulator path for radio model '{args.sim_radio}' does not exist: {sim_path}")
        if args.clean:
            clean_from_simulator(args.project, sim_path)
    if args.deploy:
        deploy_to_simulator(project_dir, args.project, sim_path)


if __name__ == "__main__":
    main()
