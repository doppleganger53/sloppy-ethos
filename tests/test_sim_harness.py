from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import sys
import zipfile
from argparse import Namespace
from pathlib import Path

import pytest


def load_harness_module():
    repo_root = Path(__file__).resolve().parents[1]
    harness_path = repo_root / "tools" / "sim" / "harness" / "run.py"
    spec = importlib.util.spec_from_file_location("sim_harness", harness_path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


harness = load_harness_module()


def _make_runtime_package(tmp_path: Path) -> harness.RuntimePackage:
    runtime_dir = tmp_path / "runtime"
    runtime_dir.mkdir()
    runtime_js = runtime_dir / "X20RS_FCC.js"
    runtime_js.write_text("module.exports = async () => ({});\n", encoding="utf-8")
    (runtime_dir / "X20RS_FCC.wasm").write_bytes(b"wasm")
    return harness.RuntimePackage(
        target=harness.normalize_radio_target("X20RS-FCC", None),
        version="26.1.0-RC2",
        asset_name="X20RS-FCC-WebSimulator.zip",
        package_dir=tmp_path / "pkg",
        archive_path=tmp_path / "pkg" / "X20RS-FCC-WebSimulator.zip",
        runtime_dir=runtime_dir,
        runtime_js=runtime_js,
    )


def test_normalize_radio_target_defaults_to_fcc_region():
    target = harness.normalize_radio_target("X20RS", None)
    assert target.key == "X20RS-FCC"
    assert target.runtime_stem == "X20RS_FCC"
    assert target.websim_asset_name == "X20RS-FCC-WebSimulator.zip"


def test_normalize_radio_target_accepts_explicit_region():
    target = harness.normalize_radio_target("x20rs-flex", "FCC")
    assert target.key == "X20RS-FLEX"


def test_select_websim_asset_matches_exact_radio_package():
    release = {
        "tag_name": "26.1.0-RC2",
        "assets": [
            {"name": "X20RS-EU-WebSimulator.zip"},
            {"name": "X20RS-FCC-WebSimulator.zip", "browser_download_url": "https://example.invalid/x.zip"},
        ],
    }
    asset = harness.select_websim_asset(release, harness.normalize_radio_target("X20RS-FCC", None))
    assert asset["name"] == "X20RS-FCC-WebSimulator.zip"


def test_runtime_package_uses_radio_version_and_package_identity(monkeypatch, tmp_path: Path):
    monkeypatch.setattr(harness, "RADIOS_ROOT", tmp_path / "radios")
    target = harness.normalize_radio_target("X20RS-FCC", None)
    package = harness.runtime_package_from_asset(target, "26.1.0-RC2", {"name": target.websim_asset_name})
    assert package.package_dir == tmp_path / "radios" / "X20RS-FCC" / "26.1.0-RC2" / "X20RS-FCC-WebSimulator"
    assert package.runtime_js == package.runtime_dir / "X20RS_FCC.js"


def test_ensure_runtime_extracts_downloaded_package_and_validates_digest(monkeypatch, tmp_path: Path):
    monkeypatch.setattr(harness, "RADIOS_ROOT", tmp_path / "radios")
    target = harness.normalize_radio_target("X20RS-FCC", None)
    zip_source = tmp_path / "source.zip"
    with zipfile.ZipFile(zip_source, "w") as payload:
        payload.writestr("X20RS_FCC.js", "module.exports = async () => ({});\n")
        payload.writestr("X20RS_FCC.wasm", b"wasm")
    digest = hashlib.sha256(zip_source.read_bytes()).hexdigest()
    asset = {
        "name": target.websim_asset_name,
        "browser_download_url": "https://example.invalid/X20RS-FCC-WebSimulator.zip",
        "digest": f"sha256:{digest}",
    }

    monkeypatch.setattr(harness, "resolve_ethos_version", lambda _version: "26.1.0-RC2")
    monkeypatch.setattr(harness, "fetch_release", lambda _version: {"tag_name": "26.1.0-RC2", "assets": [asset]})
    def fake_download(_asset, destination: Path):
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(zip_source.read_bytes())

    monkeypatch.setattr(harness, "download_asset", fake_download)

    package = harness.ensure_runtime("X20RS-FCC", None, "26.1.0-RC2")

    assert package.runtime_js.exists()
    metadata = json.loads((package.package_dir / "metadata.json").read_text(encoding="utf-8"))
    assert metadata["radio"] == "X20RS-FCC"
    assert metadata["version"] == "26.1.0-RC2"


def test_ensure_runtime_returns_cached_runtime_without_fetching_release(monkeypatch, tmp_path: Path):
    monkeypatch.setattr(harness, "RADIOS_ROOT", tmp_path / "radios")
    target = harness.normalize_radio_target("X20RS-FCC", None)
    package = harness.runtime_package_from_asset(target, "26.1.0-RC2", {"name": target.websim_asset_name})
    package.runtime_dir.mkdir(parents=True)
    package.runtime_js.write_text("module.exports = async () => ({});\n", encoding="utf-8")
    (package.runtime_dir / f"{target.runtime_stem}.wasm").write_bytes(b"wasm")

    def fail_fetch_release(_version):
        raise AssertionError("fetch_release should not be called for cached runtimes")

    monkeypatch.setattr(harness, "resolve_ethos_version", lambda _version: "26.1.0-RC2")
    monkeypatch.setattr(harness, "fetch_release", fail_fetch_release)

    result = harness.ensure_runtime("X20RS-FCC", None, "26.1.0-RC2", no_download=True)

    assert result.runtime_js == package.runtime_js


def test_ensure_runtime_uses_cached_latest_alias_without_fetching_release(monkeypatch, tmp_path: Path):
    monkeypatch.setattr(harness, "RADIOS_ROOT", tmp_path / "radios")
    target = harness.normalize_radio_target("X20RS-FCC", None)
    package = harness.runtime_package_from_asset(target, "26.1.0-RC2", {"name": target.websim_asset_name})
    package.runtime_dir.mkdir(parents=True)
    package.runtime_js.write_text("module.exports = async () => ({});\n", encoding="utf-8")
    (package.package_dir / "metadata.json").write_text(
        json.dumps({"downloadedAt": "2026-05-16T00:00:00+00:00"}) + "\n",
        encoding="utf-8",
    )

    def fail_fetch_release(_version):
        raise AssertionError("fetch_release should not be called for cached latest aliases")

    def fail_resolve_ethos_version(_version):
        raise AssertionError("resolve_ethos_version should not call GitHub for cached latest aliases")

    monkeypatch.setattr(harness, "fetch_release", fail_fetch_release)
    monkeypatch.setattr(harness, "resolve_ethos_version", fail_resolve_ethos_version)

    result = harness.ensure_runtime("X20RS-FCC", None, "latest-26.1")

    assert result.runtime_js == package.runtime_js
    assert result.version == "26.1.0-RC2"


def test_ensure_runtime_no_download_latest_alias_reports_missing_without_fetch(monkeypatch, tmp_path: Path):
    monkeypatch.setattr(harness, "RADIOS_ROOT", tmp_path / "radios")

    def fail_resolve_ethos_version(_version):
        raise AssertionError("resolve_ethos_version should not call GitHub when --no-download has no cache")

    monkeypatch.setattr(harness, "resolve_ethos_version", fail_resolve_ethos_version)

    with pytest.raises(harness.HarnessError) as exc:
        harness.ensure_runtime("X20RS-FCC", None, "latest-26.1", no_download=True)

    assert exc.value.status == "missing_runtime"
    assert str(tmp_path / "radios" / "X20RS-FCC") in exc.value.message


def test_ensure_runtime_can_refresh_latest_alias_for_download(monkeypatch, tmp_path: Path):
    monkeypatch.setattr(harness, "RADIOS_ROOT", tmp_path / "radios")
    target = harness.normalize_radio_target("X20RS-FCC", None)
    cached = harness.runtime_package_from_asset(target, "26.1.0-RC1", {"name": target.websim_asset_name})
    cached.runtime_dir.mkdir(parents=True)
    cached.runtime_js.write_text("module.exports = async () => ({});\n", encoding="utf-8")

    zip_source = tmp_path / "source.zip"
    with zipfile.ZipFile(zip_source, "w") as payload:
        payload.writestr("X20RS_FCC.js", "module.exports = async () => ({});\n")
        payload.writestr("X20RS_FCC.wasm", b"wasm")
    asset = {
        "name": target.websim_asset_name,
        "browser_download_url": "https://example.invalid/X20RS-FCC-WebSimulator.zip",
    }

    monkeypatch.setattr(harness, "resolve_ethos_version", lambda _version: "26.1.0-RC2")
    monkeypatch.setattr(harness, "fetch_release", lambda _version: {"tag_name": "26.1.0-RC2", "assets": [asset]})

    def fake_download(_asset, destination: Path):
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(zip_source.read_bytes())

    monkeypatch.setattr(harness, "download_asset", fake_download)

    result = harness.ensure_runtime("X20RS-FCC", None, "latest-26.1", prefer_cached_alias=False)

    assert result.version == "26.1.0-RC2"
    assert result.runtime_js.exists()


def test_ensure_runtime_wraps_bad_zipfile_and_cleans_invalid_archive(monkeypatch, tmp_path: Path):
    monkeypatch.setattr(harness, "RADIOS_ROOT", tmp_path / "radios")
    target = harness.normalize_radio_target("X20RS-FCC", None)
    asset = {
        "name": target.websim_asset_name,
        "browser_download_url": "https://example.invalid/X20RS-FCC-WebSimulator.zip",
    }
    package = harness.runtime_package_from_asset(target, "26.1.0-RC2", asset)
    package.archive_path.parent.mkdir(parents=True, exist_ok=True)
    package.archive_path.write_bytes(b"not-a-zip")

    monkeypatch.setattr(harness, "resolve_ethos_version", lambda _version: "26.1.0-RC2")
    monkeypatch.setattr(harness, "fetch_release", lambda _version: {"tag_name": "26.1.0-RC2", "assets": [asset]})

    with pytest.raises(harness.HarnessError) as exc:
        harness.ensure_runtime("X20RS-FCC", None, "26.1.0-RC2")

    assert exc.value.status == "download_failure"
    assert "invalid or truncated" in exc.value.message
    assert not package.archive_path.exists()
    assert not package.runtime_dir.exists()


def assert_unsafe_zip_member_rejected(member_name: str, escaped_path: Path, tmp_path: Path):
    archive_path = tmp_path / "unsafe.zip"
    destination = tmp_path / "runtime"
    with zipfile.ZipFile(archive_path, "w") as payload:
        payload.writestr(member_name, "outside")

    with pytest.raises(harness.HarnessError) as exc:
        harness.safe_extract_zip(archive_path, destination)

    assert exc.value.status == "missing_runtime"
    assert "Unsafe path" in exc.value.message
    assert not escaped_path.exists()
    assert not any(destination.rglob("*"))


def test_safe_extract_zip_rejects_parent_traversal_member(tmp_path: Path):
    assert_unsafe_zip_member_rejected("../escape.txt", tmp_path / "escape.txt", tmp_path)


def test_safe_extract_zip_rejects_absolute_member_path(tmp_path: Path):
    escaped_path = tmp_path / "absolute-escape.txt"
    assert_unsafe_zip_member_rejected(str(escaped_path), escaped_path, tmp_path)


def test_stage_project_reuses_build_install_spec_and_excludes_tests(tmp_path: Path):
    persist = harness.stage_project("SensorList", tmp_path / "persist")
    assert (persist / "scripts" / "SensorList" / "main.lua").exists()
    assert not (persist / "scripts" / "SensorList" / "tests").exists()


def test_stage_projects_stages_multiple_projects_into_same_persist_tree(tmp_path: Path):
    persist = harness.stage_projects(["SensorList", "BoundryMap"], tmp_path / "persist")
    assert (persist / "scripts" / "SensorList" / "main.lua").exists()
    assert (persist / "scripts" / "BoundryMap" / "main.lua").exists()
    assert not (persist / "scripts" / "SensorList" / "tests").exists()
    assert not (persist / "scripts" / "BoundryMap" / "tests").exists()


def test_stage_projects_does_not_delete_existing_persist_files(tmp_path: Path):
    persist = tmp_path / "persist"
    existing = persist / "models" / "keep.bin"
    existing.parent.mkdir(parents=True)
    existing.write_bytes(b"model")

    harness.stage_projects(["SensorList"], persist)

    assert existing.exists()
    assert (persist / "scripts" / "SensorList" / "main.lua").exists()


def test_resolve_persist_root_defaults_to_ethos_suite_version_and_radio(monkeypatch, tmp_path: Path):
    package = _make_runtime_package(tmp_path)
    monkeypatch.setattr(harness, "ethos_suite_data_root", lambda: tmp_path / "Ethos Suite")

    persist = harness.resolve_persist_root(package)

    assert persist == tmp_path / "Ethos Suite" / ".simulator" / "26.1.0-RC2" / "persist" / "X20RS"


def test_resolve_persist_root_accepts_explicit_override(tmp_path: Path):
    package = _make_runtime_package(tmp_path)
    explicit = tmp_path / "custom-persist"

    assert harness.resolve_persist_root(package, str(explicit)) == explicit.resolve()


def test_gui_handler_serves_runtime_files_from_cached_package(tmp_path: Path):
    package = _make_runtime_package(tmp_path)
    gui_root = tmp_path / "gui"
    gui_root.mkdir()
    handler_class = harness.make_gui_handler(
        gui_root,
        package.runtime_dir,
        {package.runtime_js.name, f"{package.target.runtime_stem}.wasm"},
    )
    handler = handler_class.__new__(handler_class)

    runtime_path = type(handler).translate_path(handler, f"/runtime/{package.runtime_js.name}")
    missing_path = type(handler).translate_path(handler, "/runtime/not-cached.js")

    assert runtime_path == str(package.runtime_js)
    assert missing_path == str(gui_root / "__missing_runtime_file__")


def test_apply_suite_args_loads_sensorlist_smoke_suite():
    args = Namespace(
        suite="tools/sim/harness/suites/SensorList-X20RS-FCC.json",
        project=["Other"],
        radio="X20S",
        region="FCC",
        ethos_version="old",
        startup_ms=1,
        settle_ms=1,
        timeout_ms=1,
        write_default_model=False,
    )
    harness.apply_suite_args(args)
    assert args.project == ["SensorList"]
    assert args.radio == "X20RS-FCC"
    assert args.ethos_version == "latest-26.1"
    assert args.timeout_ms == 12000
    assert args.write_default_model is False


def test_apply_suite_args_preserves_explicit_default_model_opt_in():
    args = Namespace(
        suite="tools/sim/harness/suites/SensorList-X20RS-FCC.json",
        project=["Other"],
        radio="X20S",
        region="FCC",
        ethos_version="old",
        startup_ms=1,
        settle_ms=1,
        timeout_ms=1,
        write_default_model=True,
    )
    harness.apply_suite_args(args)
    assert args.write_default_model is True


def test_parse_args_gui_accepts_write_default_model_flag(monkeypatch):
    monkeypatch.setattr(sys, "argv", ["run.py", "gui", "--write-default-model"])
    args = harness.parse_args()
    assert args.command == "gui"
    assert args.write_default_model is True


def test_parse_runner_result_reads_last_json_line():
    result = harness.parse_runner_result(
        "noise\n{\"status\":\"success\",\"project\":\"SensorList\"}\n",
        {"status": "startup_failure"},
    )
    assert result == {"status": "success", "project": "SensorList"}


def test_run_headless_invokes_node_runner_and_logs_output(monkeypatch, tmp_path: Path):
    package = _make_runtime_package(tmp_path)
    monkeypatch.setattr(harness, "ensure_runtime", lambda *_args, **_kwargs: package)
    monkeypatch.setattr(harness, "resolve_persist_root", lambda _package, _explicit=None: tmp_path / "persist")
    monkeypatch.setattr(harness, "stage_projects", lambda _projects, persist_root: persist_root)

    calls: dict[str, object] = {}

    def fake_run(command, **kwargs):
        calls["command"] = command
        calls["kwargs"] = kwargs
        return subprocess.CompletedProcess(command, 0, '{"status":"success"}\n', "")

    monkeypatch.setattr(harness.subprocess, "run", fake_run)
    args = Namespace(
        radio="X20RS-FCC",
        region="FCC",
        ethos_version="26.1.0-RC2",
        no_download=False,
        project=["SensorList", "BoundryMap"],
        node="node",
        startup_ms=1,
        settle_ms=1,
        timeout_ms=1000,
        run_dir=str(tmp_path / "run"),
        persist_dir=None,
    )
    result = harness.run_headless(args)

    assert result["status"] == "success"
    assert result["projects"] == ["SensorList", "BoundryMap"]
    assert result["persistDir"] == str(tmp_path / "persist")
    assert "--project" in calls["command"]
    assert "SensorList+BoundryMap" in calls["command"]
    assert str(harness.RUNNER_JS) in calls["command"]
    assert (tmp_path / "run" / "logs" / "websim.stdout.txt").exists()


def test_run_headless_timeout_decodes_captured_bytes(monkeypatch, tmp_path: Path):
    package = _make_runtime_package(tmp_path)
    monkeypatch.setattr(harness, "ensure_runtime", lambda *_args, **_kwargs: package)
    monkeypatch.setattr(harness, "resolve_persist_root", lambda _package, _explicit=None: tmp_path / "persist")
    monkeypatch.setattr(harness, "stage_projects", lambda _projects, persist_root: persist_root)

    def fake_run(command, **_kwargs):
        raise subprocess.TimeoutExpired(command, timeout=1, output=b"booting\n", stderr=b"still running\n")

    monkeypatch.setattr(harness.subprocess, "run", fake_run)
    args = Namespace(
        radio="X20RS-FCC",
        region="FCC",
        ethos_version="26.1.0-RC2",
        no_download=False,
        project=["SensorList"],
        node="node",
        startup_ms=1,
        settle_ms=1,
        timeout_ms=1000,
        run_dir=str(tmp_path / "run"),
        persist_dir=None,
    )

    result = harness.run_headless(args)

    assert result["status"] == "timeout"
    assert (tmp_path / "run" / "logs" / "websim.stdout.txt").read_text(encoding="utf-8") == "booting\n"
    assert (tmp_path / "run" / "logs" / "websim.stderr.txt").read_text(encoding="utf-8") == "still running\n"


@pytest.mark.parametrize(
    ("write_default_model", "expected_value"),
    [
        (False, "const writeDefaultModel = false;"),
        (True, "const writeDefaultModel = true;"),
    ],
)
def test_run_gui_controls_default_model_writer(monkeypatch, tmp_path: Path, write_default_model: bool, expected_value: str):
    package = _make_runtime_package(tmp_path)
    monkeypatch.setattr(harness, "ensure_runtime", lambda *_args, **_kwargs: package)
    monkeypatch.setattr(harness, "resolve_persist_root", lambda _package, _explicit=None: tmp_path / "persist")

    def fake_stage_projects(projects, persist_root):
        script_root = persist_root / "scripts" / projects[0]
        script_root.mkdir(parents=True, exist_ok=True)
        (script_root / "main.lua").write_text("-- staged\n", encoding="utf-8")
        return persist_root

    monkeypatch.setattr(harness, "stage_projects", fake_stage_projects)
    args = Namespace(
        radio="X20RS-FCC",
        region="FCC",
        ethos_version="26.1.0-RC2",
        no_download=False,
        project=["SensorList"],
        port=8765,
        no_open=True,
        dry_run=True,
        run_dir=str(tmp_path / "run"),
        persist_dir=None,
        suite=None,
        write_default_model=write_default_model,
    )

    result = harness.run_gui(args)
    index_html = Path(result["runDir"]) / "gui" / "index.html"
    gui_runtime_dir = Path(result["runDir"]) / "gui" / "runtime"
    html = index_html.read_text(encoding="utf-8")

    assert result["status"] == "gui_ready"
    assert not gui_runtime_dir.exists()
    assert expected_value in html
    assert '<link rel="icon" href="data:,">' in html
    runtime_token = f"{package.version}-{harness._sha256(package.runtime_js)[:12]}"
    runtime_token = f"{runtime_token}-{harness._sha256(package.runtime_dir / f'{package.target.runtime_stem}.wasm')[:12]}"
    assert f'<script src="runtime/{package.runtime_js.name}?v={runtime_token}"></script>' in html
    assert f'locateFile: (path) => "runtime/" + path + "?v={runtime_token}"' in html
    assert 'if (writeDefaultModel && typeof module._writeDefaultSettingsAndModel === "function")' in html
    assert 'typeof module._reloadScripts === "function"' in html
    assert "const expandRgb565 = (source, width, height) => {" in html
    assert "const drawRuntimeCanvas = (width, height, pointer) => {" in html
    assert "runtimeModule.HEAPU16.buffer" in html
    assert "new Uint16Array(runtimeModule.HEAPU16.buffer, pointer, pixelCount)" in html
    assert "const targetRow = (height - y - 1) * width;" in html
    assert "new ImageData(pixels, width, height)" in html
    assert 'runtimeModule.ccall(name, "void", args.map(() => "number"), args);' in html
    assert 'callRuntime("onMouseDown", [position.x, position.y]);' in html
    assert 'callRuntime("onMouseUp", [position.x, position.y]);' in html
    assert 'callRuntime("onMouseMove", [position.x, position.y]);' in html
    assert 'callRuntime("onMouseLongPress");' in html
    assert 'canvas.addEventListener("touchstart"' in html
    assert "updateCanvas: drawRuntimeCanvas" in html
    assert "preRun: [() => {" in html
    assert "runtimeModule = module;" in html


def test_download_no_download_reports_missing_runtime(monkeypatch, tmp_path: Path):
    monkeypatch.setattr(harness, "RADIOS_ROOT", tmp_path / "radios")
    target = harness.normalize_radio_target("X20RS-FCC", None)
    asset = {"name": target.websim_asset_name, "browser_download_url": "https://example.invalid/x.zip"}
    monkeypatch.setattr(harness, "resolve_ethos_version", lambda _version: "26.1.0-RC2")
    monkeypatch.setattr(harness, "fetch_release", lambda _version: {"tag_name": "26.1.0-RC2", "assets": [asset]})

    with pytest.raises(harness.HarnessError) as exc:
        harness.ensure_runtime("X20RS-FCC", None, "26.1.0-RC2", no_download=True)

    assert exc.value.status == "missing_runtime"
