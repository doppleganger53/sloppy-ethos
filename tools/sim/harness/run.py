#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
import webbrowser
import zipfile
from dataclasses import dataclass
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
HARNESS_ROOT = Path(__file__).resolve().parent
RADIOS_ROOT = REPO_ROOT / "tools" / "sim" / "radios"
RUNS_ROOT = REPO_ROOT / "tools" / "sim" / "runs"
GITHUB_REPO = "FrSkyRC/ETHOS-Feedback-Community"
DEFAULT_ETHOS_VERSION = "latest-26.1"
DEFAULT_REGION = "FCC"
RUNNER_JS = HARNESS_ROOT / "websim_runner.js"
GITHUB_API = "https://api.github.com"


class HarnessError(Exception):
    def __init__(self, status: str, message: str, exit_code: int = 2, **details: Any):
        super().__init__(message)
        self.status = status
        self.message = message
        self.exit_code = exit_code
        self.details = details


@dataclass(frozen=True)
class RadioTarget:
    radio: str
    region: str

    @property
    def key(self) -> str:
        return f"{self.radio}-{self.region}"

    @property
    def runtime_stem(self) -> str:
        return self.key.replace("-", "_")

    @property
    def websim_asset_name(self) -> str:
        return f"{self.key}-WebSimulator.zip"


@dataclass(frozen=True)
class RuntimePackage:
    target: RadioTarget
    version: str
    asset_name: str
    package_dir: Path
    archive_path: Path
    runtime_dir: Path
    runtime_js: Path


def _load_build_module():
    build_path = REPO_ROOT / "tools" / "build.py"
    spec = importlib.util.spec_from_file_location("sloppy_ethos_build", build_path)
    if spec is None or spec.loader is None:
        raise HarnessError("startup_failure", f"Unable to import build helper at {build_path}", 10)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _http_json(url: str) -> Any:
    request = urllib.request.Request(
        url,
        headers={
            "Accept": "application/vnd.github+json",
            "User-Agent": "sloppy-ethos-sim-harness",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        raise HarnessError("download_failure", f"GitHub API request failed: {exc.code} {url}", 11) from exc
    except urllib.error.URLError as exc:
        raise HarnessError("download_failure", f"GitHub API request failed: {exc.reason}", 11) from exc


def normalize_radio_target(radio: str, region: str | None) -> RadioTarget:
    raw = radio.strip().upper().replace("_", "-")
    if not raw:
        raise HarnessError("missing_runtime", "Radio target cannot be empty.", 2)
    parts = raw.split("-", 1)
    if len(parts) == 2:
        return RadioTarget(parts[0], parts[1])
    return RadioTarget(raw, (region or DEFAULT_REGION).strip().upper())


def resolve_ethos_version(raw_version: str) -> str:
    version = raw_version.strip()
    if not version:
        raise HarnessError("missing_runtime", "Ethos version cannot be empty.", 2)
    if version.lower() not in {"latest-26.1", "latest26.1", "latest"}:
        return version

    releases = _http_json(f"{GITHUB_API}/repos/{GITHUB_REPO}/releases?per_page=50")
    candidates = []
    for release in releases:
        tag = str(release.get("tag_name") or "")
        name = str(release.get("name") or "")
        if tag.startswith("26.1.") or tag.lower() == "nightly26" or "26.1" in name:
            candidates.append(release)
    if not candidates:
        raise HarnessError("download_failure", "No Ethos 26.1 release found in GitHub releases.", 11)

    candidates.sort(key=lambda item: str(item.get("published_at") or ""), reverse=True)
    tag = str(candidates[0].get("tag_name") or "")
    if not tag:
        raise HarnessError("download_failure", "Latest Ethos 26.1 release did not include a tag name.", 11)
    return tag


def fetch_release(version: str) -> dict[str, Any]:
    payload = _http_json(f"{GITHUB_API}/repos/{GITHUB_REPO}/releases/tags/{version}")
    if not isinstance(payload, dict):
        raise HarnessError("download_failure", f"Unexpected release metadata for {version}.", 11)
    return payload


def select_websim_asset(release: dict[str, Any], target: RadioTarget) -> dict[str, Any]:
    assets = release.get("assets")
    if not isinstance(assets, list):
        raise HarnessError("download_failure", "Release metadata did not include an asset list.", 11)

    expected = target.websim_asset_name.lower()
    for asset in assets:
        if str(asset.get("name") or "").lower() == expected:
            return asset

    available = sorted(str(asset.get("name") or "") for asset in assets if "WebSimulator" in str(asset.get("name") or ""))
    raise HarnessError(
        "missing_runtime",
        f"Release {release.get('tag_name')} does not contain {target.websim_asset_name}.",
        12,
        available_websim_assets=available,
    )


def runtime_package_from_asset(target: RadioTarget, version: str, asset: dict[str, Any]) -> RuntimePackage:
    asset_name = str(asset.get("name") or target.websim_asset_name)
    package_dir = RADIOS_ROOT / target.key / version / Path(asset_name).stem
    archive_path = package_dir / asset_name
    runtime_dir = package_dir / "runtime"
    runtime_js = runtime_dir / f"{target.runtime_stem}.js"
    return RuntimePackage(target, version, asset_name, package_dir, archive_path, runtime_dir, runtime_js)


def _sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def _expected_digest(asset: dict[str, Any]) -> str | None:
    digest = str(asset.get("digest") or "")
    if digest.startswith("sha256:"):
        return digest.split(":", 1)[1].lower()
    return None


def download_asset(asset: dict[str, Any], destination: Path) -> None:
    url = str(asset.get("browser_download_url") or asset.get("url") or "")
    if not url.startswith("https://"):
        raise HarnessError("download_failure", f"Asset {asset.get('name')} does not include a download URL.", 11)

    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(delete=False, dir=str(destination.parent), suffix=".download") as temp:
        temp_path = Path(temp.name)
    request = urllib.request.Request(url, headers={"User-Agent": "sloppy-ethos-sim-harness"})
    try:
        with urllib.request.urlopen(request, timeout=120) as response, temp_path.open("wb") as handle:
            shutil.copyfileobj(response, handle)
        temp_path.replace(destination)
    except Exception:
        temp_path.unlink(missing_ok=True)
        raise


def safe_extract_zip(archive_path: Path, destination: Path) -> None:
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    with zipfile.ZipFile(archive_path) as payload:
        for member in payload.infolist():
            member_path = destination / member.filename
            try:
                member_path.resolve().relative_to(destination.resolve())
            except ValueError as exc:
                raise HarnessError("missing_runtime", f"Unsafe path in simulator package: {member.filename}", 12) from exc
        payload.extractall(destination)


def ensure_runtime(radio: str, region: str | None, ethos_version: str, no_download: bool = False) -> RuntimePackage:
    target = normalize_radio_target(radio, region)
    version = resolve_ethos_version(ethos_version)
    release = fetch_release(version)
    asset = select_websim_asset(release, target)
    package = runtime_package_from_asset(target, version, asset)
    expected_digest = _expected_digest(asset)

    if package.runtime_js.exists():
        return package

    if no_download:
        raise HarnessError(
            "missing_runtime",
            f"Runtime is not cached at {package.package_dir}. Run the download command first.",
            12,
        )

    if not package.archive_path.exists():
        download_asset(asset, package.archive_path)

    if expected_digest:
        actual_digest = _sha256(package.archive_path)
        if actual_digest != expected_digest:
            package.archive_path.unlink(missing_ok=True)
            raise HarnessError(
                "download_failure",
                f"Downloaded {package.asset_name} checksum mismatch.",
                11,
                expected_sha256=expected_digest,
                actual_sha256=actual_digest,
            )

    safe_extract_zip(package.archive_path, package.runtime_dir)
    if not package.runtime_js.exists():
        raise HarnessError(
            "missing_runtime",
            f"Simulator package did not contain {package.runtime_js.name}.",
            12,
            extracted_files=[path.name for path in package.runtime_dir.iterdir()],
        )

    metadata = {
        "repo": GITHUB_REPO,
        "version": version,
        "radio": target.key,
        "asset": package.asset_name,
        "digest": asset.get("digest"),
        "downloadedAt": dt.datetime.now(dt.timezone.utc).isoformat(),
    }
    (package.package_dir / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
    return package


def normalize_projects(raw_projects: str | list[str] | tuple[str, ...] | None) -> list[str]:
    if raw_projects is None:
        candidates = ["SensorList"]
    elif isinstance(raw_projects, str):
        candidates = [raw_projects]
    else:
        candidates = [str(project) for project in raw_projects]

    projects: list[str] = []
    seen: set[str] = set()
    for raw_project in candidates:
        project = raw_project.strip()
        if not project:
            raise HarnessError("script_failure", "Project name cannot be empty.", 20)
        key = project.lower()
        if key in seen:
            raise HarnessError("script_failure", f"Duplicate project requested: {project}", 20)
        seen.add(key)
        projects.append(project)
    return projects


def project_label(projects: list[str]) -> str:
    return "+".join(projects)


def stage_projects(projects: list[str], run_root: Path) -> Path:
    build = _load_build_module()
    persist_root = run_root / "persist"
    if persist_root.exists():
        shutil.rmtree(persist_root)
    persist_root.mkdir(parents=True)

    seen_radio_destinations: dict[str, str] = {}
    for project in projects:
        project_dir = REPO_ROOT / "scripts" / project
        if not project_dir.exists():
            raise HarnessError("script_failure", f"Project directory not found: {project_dir}", 20)
        install_spec = build.resolve_project_install_spec(project_dir, project)
        build.ensure_unique_radio_destinations(project, install_spec.radio_files, seen_radio_destinations)
        build.stage_project_files(install_spec, persist_root, dirs_exist_ok=True)
    return persist_root


def stage_project(project: str, run_root: Path) -> Path:
    return stage_projects([project], run_root)


def apply_suite_args(args: argparse.Namespace) -> None:
    suite_path = getattr(args, "suite", None)
    if not suite_path:
        return
    path = Path(suite_path)
    if not path.is_absolute():
        path = REPO_ROOT / path
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise HarnessError("startup_failure", f"Suite file not found: {path}", 10) from exc
    except json.JSONDecodeError as exc:
        raise HarnessError("startup_failure", f"Suite file is not valid JSON: {path}: {exc}", 10) from exc
    if not isinstance(payload, dict):
        raise HarnessError("startup_failure", f"Suite file must contain a JSON object: {path}", 10)

    mapping = {
        "radio": "radio",
        "region": "region",
        "ethosVersion": "ethos_version",
        "startupMs": "startup_ms",
        "settleMs": "settle_ms",
        "timeoutMs": "timeout_ms",
        "writeDefaultModel": "write_default_model",
    }
    if "projects" in payload:
        raw_projects = payload["projects"]
        if not isinstance(raw_projects, list):
            raise HarnessError("startup_failure", f"Suite file 'projects' must be an array: {path}", 10)
        setattr(args, "project", [str(project) for project in raw_projects])
    elif "project" in payload:
        setattr(args, "project", str(payload["project"]))

    for suite_key, arg_name in mapping.items():
        if suite_key in payload:
            if arg_name == "write_default_model" and getattr(args, arg_name, False):
                continue
            setattr(args, arg_name, payload[suite_key])


def new_run_root(project: str, package: RuntimePackage, explicit: str | None = None) -> Path:
    if explicit:
        return Path(explicit).resolve()
    stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return RUNS_ROOT / f"{stamp}-{project}-{package.target.key}-{package.version}"


def parse_runner_result(stdout: str, fallback: dict[str, Any]) -> dict[str, Any]:
    for line in reversed(stdout.splitlines()):
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(payload, dict):
            return payload
    return fallback


def run_headless(args: argparse.Namespace) -> dict[str, Any]:
    apply_suite_args(args)
    projects = normalize_projects(args.project)
    label = project_label(projects)
    package = ensure_runtime(args.radio, args.region, args.ethos_version, args.no_download)
    run_root = new_run_root(label, package, args.run_dir)
    persist_root = stage_projects(projects, run_root)
    logs_dir = run_root / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)

    command = [
        args.node,
        str(RUNNER_JS),
        "--runtime-js",
        str(package.runtime_js),
        "--runtime-dir",
        str(package.runtime_dir),
        "--persist",
        str(persist_root),
        "--project",
        label,
        "--startup-ms",
        str(args.startup_ms),
        "--settle-ms",
        str(args.settle_ms),
    ]
    if getattr(args, "write_default_model", False):
        command.extend(["--write-default-model", "true"])
    try:
        completed = subprocess.run(
            command,
            cwd=str(REPO_ROOT),
            text=True,
            capture_output=True,
            check=False,
            timeout=(args.timeout_ms / 1000) + 5,
        )
    except FileNotFoundError as exc:
        raise HarnessError("startup_failure", f"Node executable not found: {args.node}", 10) from exc
    except subprocess.TimeoutExpired as exc:
        stdout = exc.stdout or ""
        stderr = exc.stderr or ""
        (logs_dir / "websim.stdout.txt").write_text(stdout, encoding="utf-8")
        (logs_dir / "websim.stderr.txt").write_text(stderr, encoding="utf-8")
        return {
            "status": "timeout",
            "project": label,
            "projects": projects,
            "radio": package.target.key,
            "ethosVersion": package.version,
            "runDir": str(run_root),
            "message": f"Headless simulator timed out after {args.timeout_ms} ms.",
        }

    (logs_dir / "websim.stdout.txt").write_text(completed.stdout, encoding="utf-8")
    (logs_dir / "websim.stderr.txt").write_text(completed.stderr, encoding="utf-8")
    fallback = {
        "status": "startup_failure" if completed.returncode else "success",
        "project": label,
        "projects": projects,
        "radio": package.target.key,
        "ethosVersion": package.version,
        "runDir": str(run_root),
        "exitCode": completed.returncode,
        "message": "Simulator runner did not emit structured JSON.",
    }
    result = parse_runner_result(completed.stdout, fallback)
    result.setdefault("project", label)
    result.setdefault("projects", projects)
    result.setdefault("radio", package.target.key)
    result.setdefault("ethosVersion", package.version)
    result.setdefault("runDir", str(run_root))
    result["stdoutLog"] = str(logs_dir / "websim.stdout.txt")
    result["stderrLog"] = str(logs_dir / "websim.stderr.txt")
    return result


def _persist_manifest(root: Path) -> dict[str, str]:
    manifest: dict[str, str] = {}
    for path in sorted(root.rglob("*")):
        if path.is_file():
            relative = path.relative_to(root).as_posix()
            manifest[relative] = base64.b64encode(path.read_bytes()).decode("ascii")
    return manifest


def write_gui_files(
    package: RuntimePackage,
    persist_root: Path,
    run_root: Path,
    project: str,
    write_default_model: bool = False,
) -> Path:
    gui_root = run_root / "gui"
    runtime_root = gui_root / "runtime"
    if gui_root.exists():
        shutil.rmtree(gui_root)
    runtime_root.mkdir(parents=True)
    shutil.copy2(package.runtime_js, runtime_root / package.runtime_js.name)
    wasm = package.runtime_dir / f"{package.target.runtime_stem}.wasm"
    if wasm.exists():
        shutil.copy2(wasm, runtime_root / wasm.name)
    (gui_root / "persist_manifest.json").write_text(
        json.dumps(_persist_manifest(persist_root), indent=2) + "\n",
        encoding="utf-8",
    )
    (gui_root / "index.html").write_text(
        GUI_HTML.replace("__RUNTIME_JS__", f"runtime/{package.runtime_js.name}")
        .replace("__RUNTIME_FACTORY__", package.target.runtime_stem)
        .replace("__PROJECT__", project)
        .replace("__WRITE_DEFAULT_MODEL__", json.dumps(write_default_model)),
        encoding="utf-8",
    )
    return gui_root


class CrossOriginHandler(SimpleHTTPRequestHandler):
    def end_headers(self) -> None:
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Resource-Policy", "same-origin")
        super().end_headers()


def run_gui(args: argparse.Namespace) -> dict[str, Any]:
    apply_suite_args(args)
    projects = normalize_projects(args.project)
    label = project_label(projects)
    package = ensure_runtime(args.radio, args.region, args.ethos_version, args.no_download)
    run_root = new_run_root(label, package, args.run_dir)
    persist_root = stage_projects(projects, run_root)
    gui_root = write_gui_files(
        package,
        persist_root,
        run_root,
        label,
        bool(getattr(args, "write_default_model", False)),
    )
    url = f"http://127.0.0.1:{args.port}/index.html"
    result = {
        "status": "gui_ready",
        "project": label,
        "projects": projects,
        "radio": package.target.key,
        "ethosVersion": package.version,
        "runDir": str(run_root),
        "url": url,
    }
    if args.dry_run:
        return result

    os.chdir(gui_root)
    server = ThreadingHTTPServer(("127.0.0.1", args.port), CrossOriginHandler)
    if not args.no_open:
        webbrowser.open(url)
    print(json.dumps(result, sort_keys=True), flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        return result


def emit(result: dict[str, Any]) -> None:
    print(json.dumps(result, sort_keys=True))


def add_runtime_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--radio", default="X20RS", help="Radio target, for example X20RS or X20RS-FCC.")
    parser.add_argument("--region", default=DEFAULT_REGION, help="Radio region suffix when --radio omits one.")
    parser.add_argument(
        "--ethos-version",
        default=DEFAULT_ETHOS_VERSION,
        help="Ethos release tag, or latest-26.1 to resolve the newest 26.1 release.",
    )
    parser.add_argument("--no-download", action="store_true", help="Use only already-cached simulator runtimes.")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Download and run Ethos WebSimulator smoke checks.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    download = subparsers.add_parser("download", help="Download or reuse a cached WebSimulator runtime.")
    add_runtime_args(download)

    headless = subparsers.add_parser("headless", help="Run a headless WebSimulator smoke test.")
    add_runtime_args(headless)
    headless.add_argument("--suite", help="JSON smoke-suite definition. Suite values populate project/radio/version/timing.")
    headless.add_argument(
        "--project",
        action="append",
        default=None,
        help="Project under scripts/ to stage and reload. Repeat to stage multiple projects into one simulator persist tree.",
    )
    headless.add_argument("--node", default="node", help="Node.js executable.")
    headless.add_argument("--startup-ms", type=int, default=1000, help="Delay after simulator start.")
    headless.add_argument("--settle-ms", type=int, default=1500, help="Delay after script reload.")
    headless.add_argument("--timeout-ms", type=int, default=15000, help="Overall headless runner timeout.")
    headless.add_argument(
        "--write-default-model",
        action="store_true",
        help="Call the runtime default model writer before start. Disabled by default because some WebSimulator builds block in this call under Node.",
    )
    headless.add_argument("--run-dir", help="Explicit run artifact directory.")

    gui = subparsers.add_parser("gui", help="Stage a project and launch a browser-based manual simulator view.")
    add_runtime_args(gui)
    gui.add_argument("--suite", help="JSON GUI-suite definition. Suite values populate project/radio/version/timing.")
    gui.add_argument(
        "--project",
        action="append",
        default=None,
        help="Project under scripts/ to stage and reload. Repeat to stage multiple projects into one simulator persist tree.",
    )
    gui.add_argument("--port", type=int, default=8765, help="Local HTTP port for the GUI view.")
    gui.add_argument("--no-open", action="store_true", help="Serve without opening the default browser.")
    gui.add_argument("--dry-run", action="store_true", help="Prepare GUI files and print the URL without serving.")
    gui.add_argument(
        "--write-default-model",
        action="store_true",
        help="Call the runtime default model writer before start.",
    )
    gui.add_argument("--run-dir", help="Explicit run artifact directory.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.command == "download":
            package = ensure_runtime(args.radio, args.region, args.ethos_version, args.no_download)
            emit(
                {
                    "status": "success",
                    "radio": package.target.key,
                    "ethosVersion": package.version,
                    "asset": package.asset_name,
                    "runtimeJs": str(package.runtime_js),
                    "packageDir": str(package.package_dir),
                }
            )
            return 0
        if args.command == "headless":
            result = run_headless(args)
            emit(result)
            return 0 if result.get("status") == "success" else 20
        if args.command == "gui":
            result = run_gui(args)
            emit(result)
            return 0
    except HarnessError as exc:
        payload = {"status": exc.status, "message": exc.message, **exc.details}
        emit(payload)
        return exc.exit_code
    return 2


GUI_HTML = """<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Ethos WebSimulator - __PROJECT__</title>
  <link rel="icon" href="data:,">
  <style>
    body { margin: 0; font-family: system-ui, sans-serif; background: #111; color: #eee; }
    main { display: grid; grid-template-columns: minmax(480px, 1fr) 420px; min-height: 100vh; }
    canvas { width: 100%; image-rendering: pixelated; background: #000; align-self: start; }
    pre { margin: 0; padding: 12px; overflow: auto; background: #1f1f1f; font-size: 12px; }
  </style>
</head>
<body>
<main>
  <canvas id="screen" width="800" height="480"></canvas>
  <pre id="log"></pre>
</main>
<script src="__RUNTIME_JS__"></script>
<script>
const log = (message) => {
  const node = document.getElementById("log");
  node.textContent += String(message) + "\\n";
  node.scrollTop = node.scrollHeight;
};
const canvas = document.getElementById("screen");
const context = canvas.getContext("2d");
const writeDefaultModel = __WRITE_DEFAULT_MODEL__;
let runtimeModule = null;
const ensureDir = (FS, path) => {
  const parts = path.split("/").filter(Boolean);
  let current = "";
  for (const part of parts) {
    current += "/" + part;
    try { FS.mkdir(current); } catch (_) {}
  }
};
const writeFile = (FS, relative, encoded) => {
  const path = "/" + relative.replaceAll("\\\\", "/");
  ensureDir(FS, path.split("/").slice(0, -1).join("/"));
  const binary = atob(encoded);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  FS.writeFile(path, bytes);
};
const drawRuntimeCanvas = (width, height, pointer) => {
  if (!runtimeModule || !runtimeModule.HEAP8 || !pointer || !width || !height) return;
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }
  const pixelCount = width * height;
  const rgbaLength = pixelCount * 4;
  const heap = new Uint8Array(runtimeModule.HEAP8.buffer);
  if (pointer + rgbaLength > heap.length) return;
  const pixels = new Uint8ClampedArray(heap.subarray(pointer, pointer + rgbaLength));
  context.putImageData(new ImageData(pixels, width, height), 0, 0);
};
fetch("persist_manifest.json")
  .then((response) => response.json())
  .then((manifest) => {
    const runtimeConfig = {
      locateFile: (path) => "runtime/" + path,
      print: log,
      printErr: (line) => log("ERR " + line),
      updateCanvas: drawRuntimeCanvas,
      setModelJson: () => log("model json callback"),
      preRun: [() => {
        for (const [relative, encoded] of Object.entries(manifest)) {
          writeFile(runtimeConfig.FS, relative, encoded);
        }
      }]
    };
    return __RUNTIME_FACTORY__(runtimeConfig);
  })
  .then((module) => {
    runtimeModule = module;
    if (writeDefaultModel && typeof module._writeDefaultSettingsAndModel === "function") {
      module._writeDefaultSettingsAndModel();
    }
    module._start();
    setTimeout(() => {
      if (typeof module._reloadScripts === "function") {
        module._reloadScripts();
        log("reloadScripts complete for __PROJECT__");
      } else {
        log("reloadScripts unavailable in this runtime; startup complete for __PROJECT__");
      }
    }, 1000);
  })
  .catch((error) => log(error.stack || error));
</script>
</body>
</html>
"""


if __name__ == "__main__":
    raise SystemExit(main())
