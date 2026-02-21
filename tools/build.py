#!/usr/bin/env python3
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path
from typing import Optional


def load_config(path: Path):
    if not path.exists():
        return {}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        sys.exit(f"Failed to parse config at {path}: {exc}")


def resolve_simulator_path(config_path: Path, override_env: Optional[str]):
    if override_env:
        return Path(override_env)
    config = load_config(config_path)
    path_value = config.get("ETHOS_SIM_PATH")
    if not path_value:
        return None
    return Path(path_value)


def ensure_luac_available():
    luac = shutil.which("luac")
    if not luac:
        sys.exit("Required command 'luac' not found on PATH.")
    return luac


def run_lua_check(project_dir: Path, luac_exec: str):
    target = project_dir / "main.lua"
    if not target.exists():
        sys.exit(f"Widget entrypoint not found: {target}")
    print(f"Checking Lua syntax: {target}")
    subprocess.run([luac_exec, "-p", str(target)], check=True)


def build_zip(project_dir: Path, project_name: str, dist_dir: Path, repo_root: Path):
    dist_dir.mkdir(parents=True, exist_ok=True)
    staging_root = repo_root / ".build-staging"
    if staging_root.exists():
        shutil.rmtree(staging_root, ignore_errors=True)

    try:
        timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        base_name = dist_dir / f"{project_name}-ethos-install-{timestamp}"
        staging = staging_root / "package"
        destination = staging / "scripts" / project_name
        shutil.copytree(project_dir, destination)
        archive_path = shutil.make_archive(str(base_name), "zip", staging)
    finally:
        shutil.rmtree(staging_root, ignore_errors=True)

    print(f"Packaged widget ZIP: {archive_path}")
    return Path(archive_path)


def deploy_to_simulator(project_dir: Path, project_name: str, sim_path: Path):
    if not sim_path.exists():
        sys.exit(f"Simulator path '{sim_path}' does not exist.")
    target = sim_path / "scripts" / project_name
    target.parent.mkdir(parents=True, exist_ok=True)
    try:
        shutil.copytree(project_dir, target, dirs_exist_ok=True)
    except PermissionError as exc:
        sys.exit(
            f"Permission denied while copying to '{target}'. "
            "Ensure the simulator is closed and you have write access.\nDetails: "
            f"{exc}"
        )
    print(f"Deployed {project_name} -> {target}")


def parse_args():
    parser = argparse.ArgumentParser(description="Build or deploy Ethos widget packages.")
    parser.add_argument("--project", "-p", default="SensorList", help="Name of the widget project.")
    parser.add_argument("--dist", action="store_true", help="Produce an Ethos install ZIP in dist/.")
    parser.add_argument("--deploy", action="store_true", help="Copy the widget folder into the simulator scripts directory.")
    parser.add_argument("--config", "-c", help="Path to JSON config containing ETHOS_SIM_PATH.")
    parser.add_argument("--no-zip", action="store_true", help="Skip ZIP even when --dist is provided.")
    return parser.parse_args()


def main():
    args = parse_args()
    repo_root = Path(__file__).resolve().parent.parent
    project_dir = repo_root / "src" / "scripts" / args.project
    luac_exec = ensure_luac_available()
    run_lua_check(project_dir, luac_exec)

    if not args.dist and not args.deploy:
        sys.exit("Nothing to do: specify --dist or --deploy.")

    if args.dist and not args.no_zip:
        build_zip(project_dir, args.project, repo_root / "dist", repo_root)

    if args.deploy:
        config_path = Path(args.config) if args.config else (repo_root / "tools" / "deploy.config.json")
        sim_path = resolve_simulator_path(config_path, os.environ.get("ETHOS_SIM_PATH"))
        if not sim_path:
            sys.exit("Simulator path not configured. Set ETHOS_SIM_PATH or create tools/deploy.config.json.")
        deploy_to_simulator(project_dir, args.project, sim_path)


if __name__ == "__main__":
    main()
