# ETHOS Model Editor

Open **ethos-model-desktop.exe** on Windows x64. The editor reads model backups locally and exports repairs to a new `.bin` file.

To try it with synthetic data, open `examples/throttle-after.bin` as the current model and compare `examples/throttle-before.bin` as the backup. The four throttle sensors keep Physical ID `03` and change Application ID from `6803` to `6802`. In the source checkout these files are under `fixtures/`.

## Open, compare and repair

1. Choose **Open current…** and select the model you want to repair. Choose **Compare backup…** to add an earlier model. You can also launch the editor with one current file, or with two paths in **backup, current** order.
2. Select **Model settings** or one of its sections. **Search** filters names, stored values and paths. Select a field and choose **Inspect bytes** to see its full byte range. **Differences** displays the backup and current values together.
3. Select **Telemetry IDs**. Search for a sensor label, Physical ID or Application ID. Select the sensor, enter its intended hexadecimal values and choose **Stage change**. You can also stage an editable field from **Model settings**.
4. Choose **Preview edits** to see the current and proposed values. Resolve any validation messages, then choose **Save copy…** and supply a new filename. The editor verifies the saved file by reopening it. The current file and backup stay available for comparison.

Settings and differences follow the [FrSky Model Setup menu order](https://ethos-doc.frsky-rc.com/model-setup/overview/). File metadata and opaque regions appear after the radio's settings.

To edit a custom response curve, select its point list or a coordinate under **Curves**, then choose **Edit points…**. Enter one `X, Y` pair per line, or enter one Y per line to space X evenly. **Stage points** checks the coordinates. **Preview edits** shows the revised list before you save a copy.

For ordered fields, the proposed value starts with the current list. For example, setting `curves.order` to `[1,0]` swaps the first two curves in a two-curve model. The preview shows the resulting reference changes. A timer audio action list can contain an entry such as `[{"type":"countdown_value","start":10,"step":30,"haptic":true}]`. The `fixtures/` folder in the ZIP contains the public source models and their firmware-verified values.

In **Variables**, a conditional rate can be entered as `[{"condition":"SA Up","value":3}]` and an action as `[{"condition":"FS1","function":"Add","source":3}]`. The tested radio displays these literal values as 0.3. Keep the rows in the order in which the radio should evaluate them.

In a **Plane** model, **RF system** shows external module settings. For a PPM or SBUS module, select `rf.modules[1].channel_range` and enter a span such as `CH1-CH8` for PPM or `CH49-CH64` for SBUS. Choose 4–16 consecutive channels for PPM or exactly 16 for SBUS, within channels 1–64. The region, binding, receiver and failsafe fields are available for inspection.

Under **Edit model**, `model.bitmap` stores the image name under the radio's `bitmaps/models` directory. The verified field accepts up to 15 UTF-8 bytes, including a relative subdirectory and extension. A blank value selects the default image. Supported logical-switch on/off delays accept 0–60 seconds in tenths.

In **Trims**, select an existing extra trim to inspect or edit its verified name, triggers, range and step. In an active **Trainer** profile, choose a supported link from Cable, Bluetooth, External SBUS or External CPPM. A Master profile exposes verified activation and channel mode, destination and weight fields. A Slave profile exposes its output channel span, such as `CH3-CH11`. The saved role appears as a read-only field.

Under **Checklist**, the throttle startup value is a whole percentage from −100 to 100. For a verified line chart under **Screens**, its log interval uses 10 ms units. An interval of `5` represents 50 ms. The chart's automatic range control is available for the verified unassigned-source layout.

The ID shown by the backup is a comparison value. Choose repair values from your actual sensor configuration.

## Manage telemetry sensors

- In **Telemetry IDs**, choose **Add sensor…** to start with blank S.Port fields. The editor uses firmware-captured defaults for the entered Application ID and Sub-ID. **Preview edits** lists the required fields for any incomplete row. For an example, use module `0`, band `1`, routing `0`, sub-ID `0`, Physical ID `05` and Application ID `6803` in `fixtures/layout-references.bin`.
- Select a row, choose **Copy**, then **Paste**. The new entry retains the source settings. Change a pasted S.Port entry's key before exporting if it duplicates another key. In a Plane model, **Edit sensor…** can rename the entry.
- Right-click a row in **Model settings** or **Telemetry IDs** and choose **Dependencies**. Expand **References** or **Referenced by** to trace stored references. Circular paths end at a row marked **cycle**.
- Use Ctrl-click or Shift-click to select sensor rows and choose **Delete selected…**. With **Cascade delete** off, the dialog lists references that block deletion. Turn it on to preview direct and indirect dependents. **Stage deletion** applies the displayed list. The editor clears affected widget slots and keeps the screen layout.
- **Save copy…** uses a new filename. Reopen the copy to review retained sensors and references. In the example fixture, deleting `telemetry.sensors[1]` cascades to `timers[0]` and its Value widget. The two other widgets continue to reference their original timers in the new slots.

The dependency tree covers decoded model sources and curve selectors. If a cascade would require deleting a fixed model setting, or the file contains opaque Lua configuration or an unverified layout, the dialog reports the blocker. Blank entry creation covers S.Port identities with captured defaults. The editor can copy and delete decoded calculated and DIY entries.

## Support

| Target | Supported workflow |
| --- | --- |
| Windows x64 | Native desktop browser, diff, staging and copy export |
| FrSky X20 Pro AW FCC plane models, format `0.1.37` | Offline operations on structures verified in the [format and fixture matrix](FORMAT.md) |
| X20 Pro AW FCC Glider, Helicopter, Multi and Other, format `0.1.37` | Browse and compare the verified layouts and repair existing S.Port IDs. Other fields are read-only in these categories. |
| Plane RF modules | Browse Twin Lite Pro, XJT Lite, the three verified R9M Lite variants, PPM and SBUS. Authoring includes the verified PPM/SBUS channel spans. |
| Existing S.Port telemetry | Physical IDs `00–1B`. Verified servo sensors accept same-family Application IDs `6800–6803`. The editor displays each field's edit capability. |
| Optional simulator read-back | ETHOS 1.6.8 preserves `0.1.37`. ETHOS 26.1.2 converts to `0.2.22` and 26.2.0 converts to `0.2.29` in disposable storage. |
| Other model boards or formats | The editor reports the unsupported target when opening it |
| Physical radio | Hardware verification is pending |

Opaque sections are available in the byte inspector. The available edit controls follow the schema for the opened model.

The bundled `examples/sensor-variants.bin` illustrates sensors with editable Physical IDs and read-only Application IDs.

The unpacked x64 build was exercised on Windows build 26300, with Windows system directories as the applications' only PATH entries. The optional Validate in ETHOS action reopened an exported copy in the 1.6.8 FCC simulator. At commit `c20b9da`, a fresh hosted Windows runner verified telemetry repairs, variable-length renaming, curve edits and reordering, logical switches, and variable rates and actions through the native controls. [Hosted Windows package test](https://github.com/doppleganger53/sloppy-ethos/actions/runs/46085294357/job/140794060119)

## Optional firmware read-back

After exporting a copy, choose **Validate in ETHOS…** and select an extracted, supported X20 Pro AW FCC WebSimulator directory. This optional action requires Node.js and the matching FrSky JavaScript and WASM assets. The report shows the input and read-back formats, firmware diagnostics and detected changes. See the [format evidence](FORMAT.md) for pinned runtimes and conversion cases.

## Command line

The ZIP also includes **ethos-model.exe**. These examples compare a backup to a current file and repair the Application ID of sensor slot 2. Slot numbers start at zero. `-` preserves the existing Physical ID.

```powershell
.\ethos-model.exe inspect current.bin
.\ethos-model.exe diff backup.bin current.bin --json
.\ethos-model.exe preview current.bin --set 2:-:6803
.\ethos-model.exe repair current.bin --output repaired.bin --set 2:-:6803
```

## Build and verify from source

Install a native Windows x64 Rust toolchain and run the following commands in `model-editor/`. The default build and tests use the checked-in fixtures. The desktop package includes both executables and the optional validation adapter.

```powershell
cargo fmt --all -- --check
cargo test --locked --workspace --all-targets
.\packaging\build.ps1
```

The package is created in Cargo's `target/package/` directory. To exercise the packaged Windows controls, extract the ZIP and run the following commands from `model-editor/`, replacing `C:\scratch\editor` with the extracted directory.

```powershell
python -m pip install pywinauto==0.6.9
python packaging/smoke.py C:\scratch\editor fixtures
```

The smoke test also checks unsupported-format reporting. Supply `--runtime C:\sim\X20PROAW-FCC-1.6.8` to test the optional validation button using a separately installed Node.js.

Desktop builds run in the dedicated **ETHOS model editor** workflow. A tag of the form `ethos-model-editor-v<VERSION>` prepares a draft desktop release after the package, portable codec and Windows package smoke jobs pass. A rerun can refresh an existing draft. If the release has already been published, its assets are preserved.
