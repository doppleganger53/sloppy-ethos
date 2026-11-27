"""Exercise the unpacked Windows GUI and compare its saved bytes with the packaged CLI."""

import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
from time import monotonic, sleep

from pywinauto import Application
from pywinauto import win32defines
from pywinauto.findwindows import ElementNotFoundError
from pywinauto.timings import TimeoutError as ControlTimeout


def wait_for(read, accept, message, seconds=20):
    deadline = monotonic() + seconds
    value = None
    while monotonic() < deadline:
        try:
            value = read()
            if accept(value):
                return value
        except (ElementNotFoundError, RuntimeError):
            pass
        sleep(0.1)
    raise AssertionError(f"{message}; last observation {value!r}")


def rows(table):
    return [
        [table.get_item(row, col).text() for col in range(4)]
        for row in range(table.item_count())
    ]

def select_section(tree, label):
    item = tree.get_item([label])
    # Send to the target HWND so the test never moves the system cursor.
    assert tree.send_message(win32defines.TVM_SELECTITEM, win32defines.TVGN_CARET, item.elem)


def choose_file(app, window, button, dialog_title, filename):
    window.child_window(title=button, class_name="Button").wrapper_object().post_message(0x00F5)  # BM_CLICK
    dialog = app.window(title=dialog_title, class_name="#32770")
    dialog.wait("visible", timeout=15)
    try:
        # Windows exposes the filename in either the classic cmb13 host or the
        # DirectUI filename combo. Address/search edits are separate shell controls.
        def filename_edits():
            controls = []
            for edit in dialog.descendants(class_name="Edit"):
                parent = edit.parent()
                if edit.control_id() == 1152 and parent.handle == dialog.handle and edit.is_visible():
                    controls.append(edit)  # edt1 in the folder picker
                    continue
                host = parent.parent()
                classic = (edit.control_id(),host.class_name(),host.control_id()) == (1148,"ComboBoxEx32",1148)
                modern = (edit.control_id(),host.class_name()) == (1001,"FloatNotifySink")
                if edit.is_visible() and parent.class_name()=="ComboBox" and (classic or modern):
                    controls.append(edit)
            return controls
        edit = wait_for(filename_edits,lambda matches: len(matches)==1,"filename control")[0]
        edit.set_edit_text(str(filename))
        wait_for(edit.window_text, lambda text: text == str(filename), "filename control")
        dialog.child_window(control_id=1,class_name="Button").wrapper_object().post_message(0x00F5)
        dialog.wait_not("visible",timeout=15)
    except (ControlTimeout, ElementNotFoundError, AssertionError) as exc:
        raise AssertionError(f"{dialog_title} could not select {filename}. {window_state(app)}") from exc


def window_state(app):
    details = []
    for window in app.windows(visible_only=True):
        controls = [(c.class_name(), c.control_id(), c.window_text()[:240])
                    for c in window.descendants()
                    if c.is_visible() and c.class_name() in {"Static", "Edit", "Button"}]
        details.append((window.window_text(), controls))
    return f"Visible windows and controls: {details!r}"


def run_smoke(package, fixtures, runtime=None):
    gui = package / "ethos-model-desktop.exe"
    cli = package / "ethos-model.exe"
    for program in (gui, cli):
        image = program.read_bytes().lower()
        # GNU import symbols can spell an absolute path with underscores.
        for home in (str(Path.home()), Path.home().as_posix(), re.sub(r"[^\w]", "_", str(Path.home()))):
            for encoding in ("utf-8", "utf-16-le"):
                assert home.lower().encode(encoding) not in image, f"{program.name} embeds the build account's home"
    before = fixtures / "throttle-before.bin"
    after = fixtures / "throttle-after.bin"
    originals = before.read_bytes(), after.read_bytes()
    system = os.environ["SystemRoot"]
    # The shipped executables have only standard Windows DLLs available. Node, Cargo
    # and the simulator are deliberately outside PATH for both desktop and CLI.
    env = {**os.environ, "PATH": os.pathsep.join((str(Path(system) / "System32"), system))}
    with tempfile.TemporaryDirectory(prefix="ethos-editor-smoke-") as temp:
        scratch = Path(temp)
        apps = []

        def launch(*paths, with_node=False):
            launch_env = env
            if with_node:
                node = shutil.which("node")
                if not node:
                    raise RuntimeError("The optional firmware check requires Node.js.")
                launch_env = {**env, "PATH": os.pathsep.join((str(Path(node).parent),env["PATH"]))}
            process = subprocess.Popen([str(gui), *map(str, paths)], cwd=scratch, env=launch_env)
            apps.append(process)
            app = Application(backend="win32").connect(process=process.pid, timeout=20)
            return app, app.window(title="ETHOS Model Editor", class_name_re="(?!#32770).*")

        try:
            # Check the same two unpacked files through the shared codec before UI
            # automation so a bad copy or runtime dependency fails at its source.
            for source in (before, after):
                subprocess.run([str(cli), "inspect", str(source)], check=True, cwd=scratch,
                               env=env, stdout=subprocess.DEVNULL)
            app, window = launch()
            window.wait("visible", timeout=25)
            table = window.child_window(class_name="SysListView32").wrapper_object()
            tree = window.child_window(class_name="SysTreeView32").wrapper_object()
            choose_file(app, window, "Open current…", "Choose the current model", after)
            try:
                wait_for(lambda: rows(table), bool, "current model browser")
            except AssertionError as exc:
                raise AssertionError(f"{exc}. {window_state(app)}") from exc
            groups = [item.text() for item in tree.get_item(["Model settings"]).children()]
            radio_groups = ["Edit model", "Flight modes", "Mixes", "Channels", "Timers", "Trims", "RF system", "Telemetry"]
            assert groups[:len(radio_groups)] == radio_groups, groups
            choose_file(app, window, "Compare backup…", "Choose the earlier backup to compare", before)
            differences = wait_for(lambda: rows(table), lambda rs: len(rs) > 0, "missing decoded diff")
            transitions = [r for r in differences if r[1].upper().removeprefix("0X") == "6803"
                           and r[2].upper().removeprefix("0X") == "6802"]
            assert len(transitions) == 4, differences
            select_section(tree, "Telemetry IDs")
            inputs = sorted(window.descendants(class_name="Edit"), key=lambda e: (e.rectangle().top, e.rectangle().left))
            search, physical, application = inputs
            search.set_edit_text("6802")
            sensor_rows = wait_for(lambda: rows(table), lambda rs: len(rs) == 4, "ID filter")
            row = sensor_rows[0]
            slot = int(re.search(r"\[(\d+)\]", row[0]).group(1))
            assert row[1:3] == ["03", "6802"], row
            table.get_item(0).select()
            wait_for(lambda: application.window_text(), lambda s: s == "6802", "select sensor")
            assert physical.window_text() == "03"
            application.set_edit_text("6803")
            window.child_window(title="Stage change", class_name="Button").click()
            window.child_window(title="Preview edits", class_name="Button").click()
            wait_for(lambda: rows(table), lambda rs: len(rs) == 1 and rs[0][1:3] == ["6802", "6803"], "staged preview")
            output = scratch / "desktop-repair.bin"
            choose_file(app, window, "Save copy…", "Save repaired model as a new file", output)
            wait_for(lambda: output.is_file() and output.stat().st_size, lambda n: n == len(originals[1]), "GUI export")
            other = scratch / "cli-repair.bin"
            subprocess.run([str(cli), "repair", str(after), "--output", str(other), "--set", f"{slot}:-:6803"],
                           check=True, cwd=scratch, env=env)
            assert output.read_bytes() == other.read_bytes(), "GUI and CLI repair outputs differ"
            assert (before.read_bytes(), after.read_bytes()) == originals
            subprocess.run([str(cli), "inspect", str(output)], check=True, cwd=scratch, env=env, stdout=subprocess.DEVNULL)

            select_section(tree, "Model settings")
            search.set_edit_text("model.name")
            wait_for(lambda: rows(table), lambda rs: len(rs) == 1 and rs[0][0] == "model.name", "model name field")
            table.get_item(0).select()
            physical.set_edit_text("Bench model")
            window.child_window(title="Stage change", class_name="Button").click()
            window.child_window(title="Preview edits", class_name="Button").click()
            wait_for(lambda: rows(table), lambda rs: any(r[2]=="Bench model" for r in rs), "name edit preview")
            named_output = scratch / "renamed-repair.bin"
            choose_file(app, window, "Save copy…", "Save repaired model as a new file", named_output)
            wait_for(named_output.exists, bool, "saved variable-length name")
            named_cli = scratch / "cli-renamed.bin"
            subprocess.run([str(cli), "repair", str(after), "--output", str(named_cli), "--set", f"{slot}:-:6803",
                            "--field", "model.name=Bench model"], check=True, cwd=scratch, env=env)
            assert named_output.read_bytes() == named_cli.read_bytes()

            variants = fixtures / "sensor-variants.bin"
            mixed_app, mixed_window = launch(variants)
            mixed_window.wait("visible", timeout=20)
            mixed_tree = mixed_window.child_window(class_name="SysTreeView32").wrapper_object()
            mixed_table = mixed_window.child_window(class_name="SysListView32").wrapper_object()
            select_section(mixed_tree, "Telemetry IDs")
            mixed_table.get_item(0).select()
            mixed_inputs = sorted(mixed_window.descendants(class_name="Edit"),
                                  key=lambda e: (e.rectangle().top, e.rectangle().left))
            _, editable_physical, locked_application = mixed_inputs
            assert editable_physical.is_enabled()
            assert not locked_application.is_enabled() and locked_application.window_text() == "F108"
            editable_physical.set_edit_text("04")
            mixed_window.child_window(title="Stage change", class_name="Button").click()
            mixed_window.child_window(title="Preview edits", class_name="Button").click()
            assert [r[1:3] for r in rows(mixed_table)] == [["03", "04"]]
            physical_copy = scratch / "physical-only.bin"
            choose_file(mixed_app,mixed_window,"Save copy…","Save repaired model as a new file",physical_copy)
            wait_for(physical_copy.exists,bool,"mixed-capability export")
            physical_cli = scratch / "physical-cli.bin"
            subprocess.run([str(cli), "repair", str(variants), "--output", str(physical_cli), "--set", "0:04:-"],
                           check=True, cwd=scratch, env=env)
            assert physical_copy.read_bytes() == physical_cli.read_bytes()

            if runtime:
                firmware_app, firmware_window = launch(before, after, with_node=True)
                firmware_window.wait("visible", timeout=20)
                firmware_tree = firmware_window.child_window(class_name="SysTreeView32").wrapper_object()
                firmware_table = firmware_window.child_window(class_name="SysListView32").wrapper_object()
                select_section(firmware_tree, "Telemetry IDs")
                firmware_table.get_item(0).select()
                firmware_edits = sorted(firmware_window.descendants(class_name="Edit"),
                                        key=lambda e: (e.rectangle().top, e.rectangle().left))
                firmware_edits[2].set_edit_text("6803")
                firmware_window.child_window(title="Stage change", class_name="Button").click()
                firmware_window.child_window(title="Preview edits", class_name="Button").click()
                saved = scratch / "firmware-copy.bin"
                choose_file(firmware_app,firmware_window,"Save copy…","Save repaired model as a new file",saved)
                wait_for(saved.exists,bool,"firmware input export")
                expected = saved.read_bytes()
                choose_file(firmware_app,firmware_window,"Validate in ETHOS…",
                            "Choose the extracted X20 Pro AW FCC simulator",runtime)
                report = firmware_app.window(title="ETHOS validation", class_name="#32770")
                report.wait("visible", timeout=60)
                report_text = " ".join(x.window_text() for x in report.descendants(class_name="Static"))
                assert "0.1.37" in report_text and ("ok" in report_text or "converted" in report_text), report_text
                assert saved.read_bytes() == expected
                print("Packaged GUI optional firmware report:",report_text.splitlines()[2:5])

            bad = bytearray(originals[1])
            bad[5:8] = bytes([99, 1, 37])
            unsupported = scratch / "unsupported.bin"
            unsupported.write_bytes(bad)
            bad_app, _ = launch(unsupported)
            error = bad_app.window(title="ETHOS Model Editor", class_name="#32770")
            error.wait("visible", timeout=15)
            error_text = " ".join(x.window_text() for x in error.descendants(class_name="Static")).lower()
            assert "format" in error_text or "version" in error_text, error_text
            print("Packaged Windows UI passed native Open/Compare/Save dialogs, decoded diff, search, telemetry and variable-length setting edits, CLI parity and unsupported-format reporting.")
        finally:
            for process in apps:
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=10)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", type=Path)
    parser.add_argument("fixtures", type=Path)
    parser.add_argument("--runtime", type=Path, help="Also validate an exported copy in an extracted, pinned FrSky runtime")
    args = parser.parse_args()
    run_smoke(args.package.resolve(), args.fixtures.resolve(), args.runtime.resolve() if args.runtime else None)
