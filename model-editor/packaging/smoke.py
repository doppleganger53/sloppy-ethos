"""Exercise the unpacked Windows GUI and compare its saved bytes with the packaged CLI."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
from time import monotonic, sleep
from urllib.parse import unquote, urlsplit

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


def close_editor(window):
    window.wrapper_object().post_message(win32defines.WM_CLOSE)
    window.wait_not("exists", timeout=10)


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
    for window in app.windows(visible_only=True, enabled_only=False):
        controls = [(c.class_name(), c.control_id(), c.is_enabled(), c.window_text()[:240])
                    for c in window.descendants()
                    if c.is_visible() and c.class_name() in {"Static", "Edit", "Button"}]
        details.append((window.window_text(), controls))
    return f"Visible windows and controls (class, id, enabled, text): {details!r}"


def verify_bundled_evidence(package):
    for guide in (package / "README.md", package / "FORMAT.md"):
        for href in re.findall(r"\[[^\]\n]+\]\(([^)]+)\)", guide.read_text(encoding="utf-8")):
            link = urlsplit(href)
            if link.scheme or not link.path:
                continue
            target = (guide.parent / unquote(link.path)).resolve()
            assert target.is_relative_to(package) and target.is_file(), f"Broken link in {guide.name}: {href}"
    evidence = package / "fixtures"
    manifest = json.loads((evidence / "manifest.json").read_text(encoding="utf-8"))
    expected = {item["file"]: item["sha256"] for item in manifest["fixtures"]}
    actual = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in evidence.glob("*.bin")}
    assert actual == expected, "The portable fixture corpus differs from its provenance manifest"
    for script in [*evidence.glob("*.cjs"), *(package / "simulator").glob("*.cjs")]:
        for relative in re.findall(r"""\brequire\(\s*["'](\.{1,2}/[^"']+)["']\s*\)""", script.read_text(encoding="utf-8")):
            imported = (script.parent / relative).resolve()
            assert imported.is_relative_to(package) and imported.is_file(), f"Missing module for {script.name}: {relative}"


def press_button(window, title):
    button = window.child_window(title=title, class_name="Button")
    button.wait("ready", timeout=10)
    # Execute the native push-button action on its UI thread. Separate posted
    # mouse-down/up messages can lose capture even after the button is enabled.
    button.wrapper_object().send_message_timeout(win32defines.BM_CLICK, timeout=10)


def curve_window_state(app, table):
    if not app.is_process_running():
        return "The editor process exited."
    try:
        windows = [(w.window_text(), w.is_visible(), w.is_enabled())
                   for w in app.windows(visible_only=False, enabled_only=False)]
        count = table.item_count()
        selected = table.send_message(win32defines.LVM_GETNEXTITEM, -1, win32defines.LVNI_SELECTED)
        field = table.get_item(selected).text() if 0 <= selected < count else None
        return (f"Rows={count}, selected field={field!r}; windows (title, visible, enabled)={windows!r}. "
                f"{window_state(app)}")
    except Exception as error:
        return f"Window inspection failed: {error!r}"


def exercise_curve(launch, cli, package, scratch, env):
    original = package / "fixtures" / "curve-referenced.bin"
    app, window = launch(original)
    window.wait("visible", timeout=20)
    table = window.child_window(class_name="SysListView32").wrapper_object()
    search = min((e for e in window.descendants(class_name="Edit") if e.is_visible()), key=lambda e: e.rectangle().top)
    path = "curves[0].points[1].y"
    dialog = app.window(title="Curve points", visible_only=False)
    try:
        search.set_edit_text(path)
        wait_for(lambda: rows(table), lambda rs: len(rs) == 1 and rs[0][0] == path, "curve coordinate")
        item = table.get_item(0)
        item.select()
        wait_for(item.is_selected, bool, "selected curve coordinate")
        press_button(window, "Edit points…")
        dialog.wait("visible enabled", timeout=10)
        assert not window.is_enabled(), "The curve editor did not disable its owner."
        coordinates = dialog.child_window(class_name="Edit")
        point_rows = [tuple(float(v) for v in line.split(",")) for line in coordinates.window_text().splitlines()]
        assert point_rows == [(-100, -100), (22, 32), (100, 100)], f"Unexpected initial curve: {point_rows!r}"
    except Exception as error:
        raise AssertionError(f"Could not open the point list for {path}: {error!r}. "
                             f"{curve_window_state(app, table)}") from error
    coordinates.set_edit_text("-100, -90\r\n-20, 101\r\n100, 80")
    press_button(dialog, "Stage points")
    wait_for(lambda: [c.window_text() for c in dialog.descendants(class_name="Static")],
             lambda messages: any("curves[0].points[1].y" in m for m in messages),"invalid curve coordinate")
    coordinates.set_edit_text("-100, -90\r\n-20, 12.5\r\n100, 80")
    press_button(dialog, "Stage points")
    dialog.wait_not("visible",timeout=10)
    press_button(window, "Preview edits")
    wait_for(lambda: rows(table),lambda rs: any(r[0]=="curves[0].points" and "12.5" in r[2] for r in rs),
             "curve preview")
    output=scratch/"curve-gui.bin"
    choose_file(app,window,"Save copy…","Save repaired model as a new file",output)
    wait_for(output.exists,bool,"curve export")
    canonical='[{"x":-100,"y":-90},{"x":-20,"y":12.5},{"x":100,"y":80}]'
    cli_output=scratch/"curve-cli.bin"
    subprocess.run([str(cli),"repair",str(original),"--output",str(cli_output),"--field",f"curves[0].points={canonical}"],
                   check=True,cwd=scratch,env=env,stdout=subprocess.DEVNULL)
    assert output.read_bytes()==cli_output.read_bytes(), "GUI curve edit disagrees with the CLI"
    decoded = json.loads(subprocess.check_output([str(cli),"inspect",str(output),"--json"],cwd=scratch,env=env))
    by_path = {f["path"]: f["value"] for f in decoded["fields"]}
    assert json.loads(by_path["curves[0].points"]) == json.loads(canonical)
    assert any(k.startswith("mixes[0].") and "Curve slot 0" == v for k,v in by_path.items())
    close_editor(window)


def exercise_mixed_diff(launch, cli, package, env):
    before = package / "fixtures/mixed-telemetry-before.bin"
    after = package / "fixtures/mixed-telemetry-after.bin"
    expected = {
        ("Bench DIY [slot 0].order", "0", "1", "moved"),
        ("Bench DIY [slot 0].display_decimals", "1", "2", "changed"),
        ("Bench current [slot 1].order", "1", "0", "moved"),
    }
    report = json.loads(subprocess.check_output([str(cli), "diff", str(before), str(after), "--json"], env=env))
    assert {(d["path"], d["before"], d["after"], d["kind"]) for d in report["differences"]
            if d["section"] == "Telemetry"} == expected
    _, window = launch(before, after)
    window.wait("visible", timeout=20)
    table = window.child_window(class_name="SysListView32").wrapper_object()
    search = min((e for e in window.descendants(class_name="Edit") if e.is_visible()),
                 key=lambda e: e.rectangle().top)
    search.set_edit_text("Bench")
    displayed = wait_for(lambda: rows(table), lambda result: len(result) == 3, "mixed telemetry diff")
    assert {(path, old, new, kind.lower()) for path, old, new, kind in displayed} == expected
    close_editor(window)


def exercise_field_edit(launch, package, scratch, source, expected, edits, read_only=()):
    corpus = package / "fixtures"
    app, window = launch(corpus / f"{source}.bin")
    window.wait("visible", timeout=20)
    table = window.child_window(class_name="SysListView32").wrapper_object()
    inputs = sorted((e for e in window.descendants(class_name="Edit") if e.is_visible()),
                    key=lambda e: (e.rectangle().top, e.rectangle().left))
    search, proposed = inputs[:2]
    for path in read_only:
        assert_read_only(window, table, search, proposed, path)
    for path, value, hint in edits:
        select_setting(table, search, path)
        wait_for(lambda: [c.window_text() for c in window.descendants(class_name="Static")],
                 lambda messages: any(hint in m for m in messages), "field input hint")
        proposed.set_edit_text(value)
        window.child_window(title="Stage change", class_name="Button").click()
    window.child_window(title="Preview edits", class_name="Button").click()
    output = scratch / f"edited-{expected}.bin"
    choose_file(app, window, "Save copy…", "Save repaired model as a new file", output)
    wait_for(output.exists, bool, f"{expected} export")
    assert output.read_bytes() == (corpus / f"{expected}.bin").read_bytes(), (
        f"Desktop output differs from independent firmware fixture {expected}"
    )
    close_editor(window)


def select_setting(table, search, path):
    search.set_edit_text(path)
    displayed = wait_for(lambda: rows(table), lambda rs: any(r[0] == path for r in rs), path)
    index = next(i for i, row in enumerate(displayed) if row[0] == path)
    table.get_item(index).select()
    return displayed[index]


def assert_read_only(window, table, search, proposed, path):
    row = select_setting(table, search, path)
    assert row[3] == "Read only", row
    assert not proposed.is_enabled(), path
    assert not window.child_window(title="Stage change", class_name="Button").is_enabled(), path
    return row[1]


def exercise_wireless_inspection(launch, package):
    _, window = launch(package / "fixtures/rf-xjt-failsafe-hold.bin")
    window.wait("visible", timeout=20)
    table = window.child_window(class_name="SysListView32").wrapper_object()
    editors = sorted((e for e in window.descendants(class_name="Edit") if e.is_visible()),
                     key=lambda e: (e.rectangle().top, e.rectangle().left))
    for path in ("rf.modules[1].protocol", "rf.modules[1].failsafe_mode"):
        saved = assert_read_only(window, table, editors[0], editors[1], path)
        if path.endswith(".failsafe_mode"):
            assert saved == "Hold"
    close_editor(window)


def exercise_multirotor(launch, cli, package, scratch, env):
    corpus = package / "fixtures"
    source = corpus / "category-multi-before.bin"
    app, window = launch(source)
    window.wait("visible", timeout=20)
    table = window.child_window(class_name="SysListView32").wrapper_object()
    tree = window.child_window(class_name="SysTreeView32").wrapper_object()
    controls = sorted((e for e in window.descendants(class_name="Edit") if e.is_visible()),
                      key=lambda e: (e.rectangle().top, e.rectangle().left))
    search, physical, application = controls
    wait_for(lambda: [c.window_text() for c in window.descendants(class_name="Static")],
             lambda labels: any("category-multi-before.bin (Multirotor)" in text for text in labels),
             "opened airframe category")
    assert_read_only(window, table, search, physical, "model.name")
    select_section(tree, "Telemetry IDs")
    sensor_rows = wait_for(lambda: rows(table), lambda rs: len(rs) == 4, "multirotor telemetry")
    assert all(r[1:3] == ["03", "6803"] for r in sensor_rows)
    for slot in range(4):
        table.get_item(slot).select()
        wait_for(application.window_text, lambda value: value == "6803", "multirotor application ID")
        application.set_edit_text("6802")
        window.child_window(title="Stage change", class_name="Button").click()
        if slot in (0, 3):
            window.child_window(title="Preview edits", class_name="Button").click()
            wait_for(lambda: rows(table), lambda rs: len(rs) == slot + 1, "selected ID changes")
            output = scratch / f"multirotor-{slot + 1}.bin"
            choose_file(app, window, "Save copy…", "Save repaired model as a new file", output)
            wait_for(output.exists, bool, "multirotor export")
            if slot == 0:
                cli_copy = scratch / "multirotor-cli-one.bin"
                subprocess.run([str(cli), "repair", str(source), "--output", str(cli_copy), "--set", "0:-:6802"],
                               check=True, cwd=scratch, env=env, stdout=subprocess.DEVNULL)
                assert output.read_bytes() == cli_copy.read_bytes()
                decoded = json.loads(subprocess.check_output([str(cli), "inspect", str(output), "--json"], env=env))
                assert [s["application_id"] for s in decoded["sensors"]] == [0x6802, 0x6803, 0x6803, 0x6803]
                select_section(tree, "Telemetry IDs")
            else:
                assert output.read_bytes() == (corpus / "category-multi-after.bin").read_bytes()
    close_editor(window)


def open_dependencies(app, window, table, index):
    point = table.get_item(index).rectangle(area="text").mid_point()
    coords = point.x | (point.y << 16)
    table.post_message(win32defines.WM_RBUTTONDOWN, 2, coords)
    table.post_message(win32defines.WM_RBUTTONUP, 0, coords)
    wait_for(lambda: [w.class_name() for w in app.windows()],
             lambda classes: "#32768" in classes, "Dependencies context menu")
    window.wrapper_object().post_message(win32defines.WM_KEYDOWN, ord("D"), 0)
    dialog = app.window(title="Dependencies", visible_only=False)
    dialog.wait("visible", timeout=10)
    return dialog


def exercise_sensor_management(launch, cli, package, scratch, env, runtime):
    source = package / "fixtures" / "layout-references.bin"
    original = source.read_bytes()
    app, window = launch(source)
    window.wait("visible", timeout=20)
    table = window.child_window(class_name="SysListView32").wrapper_object()
    search, proposed = sorted((e for e in window.descendants(class_name="Edit") if e.is_visible()),
                              key=lambda e: (e.rectangle().top, e.rectangle().left))[:2]
    for slot, name in [(1, "Keep timer 1"), (2, "Keep timer 2")]:
        select_setting(table, search, f"timers[{slot}].name")
        proposed.set_edit_text(name)
        press_button(window, "Stage change")
    select_setting(table, search, "timers[0].start_condition")
    dialog = open_dependencies(app, window, table, 0)
    graph = dialog.child_window(class_name="SysTreeView32").wrapper_object()
    root = ["Ref timer (timers[0])"]
    assert [node.text() for node in graph.get_item(root).children()] == ["Repeated (telemetry.sensors[1])"]
    press_button(dialog, "Referenced by")
    assert [node.text() for node in graph.get_item(root).children()] == ["screens[0].widgets[1]"]
    press_button(dialog, "Close")

    tree = window.child_window(class_name="SysTreeView32").wrapper_object()
    table = window.child_window(class_name="SysListView32").wrapper_object()
    select_section(tree, "Telemetry IDs")
    assert table.item_count() == 2
    dialog = open_dependencies(app, window, table, 1)
    graph = dialog.child_window(class_name="SysTreeView32").wrapper_object()
    assert graph.get_item(["Repeated (telemetry.sensors[1])"]).children() == []
    press_button(dialog, "Referenced by")
    timer = graph.get_item(["Repeated (telemetry.sensors[1])", "Ref timer (timers[0])"])
    timer.expand()
    wait_for(lambda: [node.text() for node in timer.children()],
             lambda children: "screens[0].widgets[1]" in children, "indirect dependency")
    press_button(dialog, "Close")

    press_button(window, "Add sensor…")
    form = app.window(title="Edit new S.Port sensor", visible_only=False)
    form.wait("visible", timeout=10)
    entries = sorted(form.descendants(class_name="Edit"), key=lambda entry: entry.rectangle().top)
    assert len(entries) == 7 and all(not entry.window_text() for entry in entries)
    for entry, value in zip(entries, ["Bench new", "0", "1", "0", "0", "05", "6803"]):
        entry.set_edit_text(value)
    press_button(form, "Stage sensor")
    form.wait_not("visible", timeout=10)
    assert table.item_count() == 3

    table.get_item(2).select()
    press_button(window, "Copy")
    press_button(window, "Paste")
    assert table.item_count() == 4
    table.get_item(3).select()
    editors = sorted((e for e in window.descendants(class_name="Edit") if e.is_visible()),
                     key=lambda e: (e.rectangle().top, e.rectangle().left))
    editors[1].set_edit_text("06")
    press_button(window, "Stage change")
    table.get_item(3).select()
    press_button(window, "Edit sensor…")
    rename = app.window(title="Rename sensor", visible_only=False)
    rename.wait("visible", timeout=10)
    next(e for e in rename.descendants(class_name="Edit") if e.is_visible()).set_edit_text("Bench pasted")
    press_button(rename, "Stage sensor")
    rename.wait_not("visible", timeout=10)
    sensor_rows = [(row[0].removesuffix(" • staged"), row[1]) for row in rows(table)[2:]]
    assert sensor_rows == [("Bench new [2]", "05"), ("Bench pasted [3]", "06")], sensor_rows

    press_button(window, "Add sensor…")
    form.wait("visible", timeout=10)
    press_button(form, "Cancel")
    press_button(window, "Preview edits")
    assert any("Missing: Name" in row[2] and "Physical ID" in row[2] for row in rows(table))
    select_section(tree, "Telemetry IDs")
    for index in [1, 3, 4]:
        table.get_item(index).select()
    press_button(window, "Delete selected…")
    deletion = app.window(title="Delete telemetry sensors", visible_only=False)
    deletion.wait("visible", timeout=10)
    preview = deletion.child_window(class_name="Edit")
    apply = deletion.child_window(title="Stage deletion", class_name="Button")
    assert "timers[0] references telemetry.sensors[1]" in preview.window_text()
    assert not apply.is_enabled()
    press_button(deletion, "Cascade delete")
    wait_for(preview.window_text, lambda text: all(item in text for item in (
        "Will remove 5 object(s)", "screens[0].widgets[1]", "timers[0]", "Bench pasted", "draft"
    )), "cascade preview")
    press_button(deletion, "Stage deletion")
    deletion.wait_not("visible", timeout=10)
    press_button(window, "Preview edits")
    output = scratch / "sensor-management.bin"
    choose_file(app, window, "Save copy…", "Save repaired model as a new file", output)
    wait_for(output.exists, bool, "sensor management export")
    reopened = json.loads(subprocess.check_output(
        [str(cli), "inspect", str(output), "--json"], cwd=scratch, env=env))
    assert [(s["name"], s["physical_id"]) for s in reopened["sensors"]] == [("Repeated", 3), ("Bench new", 5)]
    by_path = {row["path"]: row["value"] for row in reopened["fields"]}
    assert by_path["screens[0].widgets[1].key"] == "None"
    assert by_path["screens[0].widgets[2].source"] == "Timer 0"
    assert by_path["screens[0].widgets[3].source"] == "Timer 1"
    assert by_path["timers.count"] == "2"
    assert [by_path[f"timers[{slot}].name"] for slot in range(2)] == ["Keep timer 1", "Keep timer 2"]
    assert source.read_bytes() == original
    if runtime:
        readback = scratch / "sensor-management-readback.bin"
        report = json.loads(subprocess.check_output([
            "node", str(package / "simulator/validate.cjs"), str(runtime), str(output),
            "--readback", str(readback),
        ]))
        assert report["status"] == "ok" and report["resultFormat"] == "0.1.37", report
        native = json.loads(subprocess.check_output([str(cli), "inspect", str(readback), "--json"], env=env))
        assert native["sensors"] == reopened["sensors"], report
        native_fields = {row["path"]: row["value"] for row in native["fields"]}
        for field in ("timers.count", "timers[0].name", "timers[1].name", "screens[0].widgets[1].key",
                      "screens[0].widgets[2].source", "screens[0].widgets[3].source"):
            assert native_fields[field] == by_path[field], report
    close_editor(window)


def run_smoke(package, fixtures, runtime=None):
    verify_bundled_evidence(package)
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
            close_editor(window)

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
            close_editor(mixed_window)

            exercise_sensor_management(launch, cli, package, scratch, env, runtime)
            exercise_curve(launch,cli,package,scratch,env)
            exercise_mixed_diff(launch, cli, package, env)
            exercise_multirotor(launch, cli, package, scratch, env)
            exercise_wireless_inspection(launch, package)
            exercise_field_edit(launch, package, scratch, "rf-ppm-min", "rf-ppm-last-four",
                                [("rf.modules[1].channel_range", "CH61-CH64", "consecutive channels")])
            exercise_field_edit(launch, package, scratch, "rf-external-sbus", "rf-sbus-last",
                                [("rf.modules[1].channel_range", "CH49-CH64", "consecutive channels")])
            exercise_field_edit(launch, package, scratch, "default", "bitmap-path",
                                [("model.bitmap", "b/c image.png", "bitmaps/models")])
            exercise_field_edit(launch, package, scratch, "logic-and", "logic-or",
                                [("logic_switches[0].key", "OR", "AND, OR or XOR")])
            exercise_field_edit(launch, package, scratch, "logic-gt", "logic-delay-on", [
                ("logic_switches[0].values[1]", "0", "Whole number"),
                ("logic_switches[0].delay_on", "0.2", "0 to 60 seconds"),
            ])
            exercise_field_edit(launch, package, scratch, "curve-library", "curve-library-moved",
                                [("curves.order", "[1,2,3,4,0]", "curve slots")])
            exercise_field_edit(launch, package, scratch, "variable", "var-combined", [
                ("variables[0].rates", '[{"condition":"SA Up","value":3}]', "condition"),
                ("variables[0].actions", '[{"condition":"FS1","function":"Add","source":3}]', "function"),
            ])
            exercise_field_edit(launch, package, scratch, "var-action-two", "var-action-empty", [
                ("variables[0].actions", '[{"condition":"None","function":"Add","source":0}]', "function"),
            ])
            exercise_field_edit(launch, package, scratch, "extra-trim", "extra-trim-name",
                                [("extra_trims[0].name", "AZ", "UTF-8")])
            exercise_field_edit(launch, package, scratch, "trainer-slave-cable", "trainer-slave-span",
                                [("trainer.channel_range", "CH3-CH11", "consecutive channels")],
                                read_only=("trainer.mode",))
            exercise_field_edit(launch, package, scratch, "default", "checklist-throttle-max",
                                [("checklist.throttle_value", "100", "percentage")])
            exercise_field_edit(launch, package, scratch, "chart-widget", "chart-auto",
                                [("screens[1].widgets[0].auto_range", "On", "Choose On or Off")])

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
            print("Packaged Windows UI passed native dialogs, mixed diff, plane and multirotor ID repairs, "
                  "sensor management and cascade deletion, bitmap and RF channel edits, curves, logic, variables, extra trims and trainer, "
                  "read-only controls, CLI/firmware byte agreement and unsupported-format reporting.")
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
