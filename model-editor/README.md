# ETHOS Model Editor

Open **ethos-model-desktop.exe** on Windows x64. The editor reads model backups locally and exports repairs to a new `.bin` file.

To try it with synthetic data, open `examples/throttle-after.bin` as the current model and compare `examples/throttle-before.bin` as the backup. The four throttle sensors keep Physical ID `03` and change Application ID from `6803` to `6802`. In the source checkout these files are under `fixtures/`.

## Open, compare and repair

1. Choose **Open current…** and select the model you want to repair. Choose **Compare backup…** to add an earlier model. You can also launch the editor with one current file, or with two paths in **backup, current** order.
2. Select **Model settings** or one of its sections. **Search** filters names, stored values and paths. Select a field and choose **Inspect bytes** to see its full byte range. **Differences** displays the backup and current values together.
3. Select **Telemetry IDs**. Search for a sensor label, Physical ID or Application ID. Select the sensor, enter its intended hexadecimal values and choose **Stage change**. You can also stage an editable field from **Model settings**.
4. Choose **Preview edits** to see the current and proposed values. Resolve any validation messages, then choose **Save copy…** and supply a new filename. The editor verifies the saved file by reopening it. The current file and backup stay available for comparison.

Settings and differences follow the [FrSky Model Setup menu order](https://ethos-doc.frsky-rc.com/model-setup/overview/). File metadata and opaque regions appear after the radio's settings.

The ID shown by the backup is a comparison value. Choose repair values from your actual sensor configuration.

## Support

| Target | Supported workflow |
| --- | --- |
| Windows x64 | Native desktop browser, diff, staging and copy export |
| FrSky X20 Pro AW FCC, model format `0.1.37` | Offline operations on structures verified in the [format and fixture matrix](FORMAT.md) |
| Existing S.Port telemetry | Physical IDs `00–1B`. Verified servo sensors accept same-family Application IDs `6800–6803`. The editor displays each field's edit capability. |
| Optional simulator read-back | ETHOS 1.6.8 preserves `0.1.37`. ETHOS 26.1.2 converts to `0.2.22` and 26.2.0 converts to `0.2.29` in disposable storage. |
| Other model boards or formats | The editor reports the unsupported target when opening it |
| Physical radio | Hardware verification is pending |

Opaque sections are available in the byte inspector. The available edit controls follow the schema for the opened model.

The bundled `examples/sensor-variants.bin` illustrates sensors with editable Physical IDs and read-only Application IDs.

The unpacked x64 build was exercised on Windows build 26300, with Windows system directories as the applications' only PATH entries. The native Open, Compare and Save dialogs produced the same bytes as the CLI for a servo ID repair, a Physical-only repair and a variable-length model rename. The optional Validate in ETHOS action reopened an exported copy in the 1.6.8 FCC simulator. A separate hosted Windows job repeats the portable test for each proposed release.

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

Desktop builds run in the dedicated **ETHOS model editor** workflow. A tag of the form `ethos-model-editor-v<VERSION>` prepares a draft desktop release after the package, portable codec and Windows package smoke jobs pass.
