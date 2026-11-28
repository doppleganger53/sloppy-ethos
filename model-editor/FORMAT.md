# X20 Pro AW model format evidence

The public fixtures were created from a fresh **X20 Pro AW FCC, ETHOS 1.6.8** radio profile. They contain synthetic settings. The original files, the desktop/CLI tests, and a firmware read-back can be used independently. The checked-in [fixture manifest](fixtures/manifest.json) records file hashes, requested and accepted values, diagnostics and byte differences. Byte ranges use zero-based offsets with the end excluded.

## Reproduce the corpus

The reproduction commands and fixture links work from the source `model-editor` directory or the extracted Windows ZIP. The two throttle examples are also available under `examples/` in the ZIP.

Download and extract FrSky's [1.6.8 X20 Pro AW FCC WebSimulator](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/1.6.8/X20PROAW-FCC-WebSimulator.zip). The [26.1.2 WebSimulator](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/26.1.2/X20PROAW-FCC-WebSimulator.zip) checks the saved telemetry through JSON. The [26.2.0 WebSimulator](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/26.2.0/X20PROAW-FCC-WebSimulator.zip) also exposes mixes, functions, screens, and Lua registrations. With Node.js 20 or newer, run the generator from `model-editor` and supply a new output directory.

```powershell
node fixtures/generate.cjs C:\sim\X20PROAW-FCC-1.6.8 C:\temp\ethos-fixtures C:\sim\X20PROAW-FCC-26.1.2 C:\sim\X20PROAW-FCC-26.2.0
node --test simulator/runtime.test.cjs
```

Append `--case servo-addresses` to reproduce one case. Repeat `--case` to select several. The generator includes their comparison baselines and seed models automatically. Compare a full run with `node fixtures/verify-replay.cjs fixtures C:\temp\ethos-fixtures`. For a selected subset, append `--partial`.

The optional third and fourth arguments enable the independent read-back checks. The checked-in binaries and manifest can be regenerated with the legacy runtime alone. [Experiment definitions](fixtures/cases.cjs) use the matching firmware's published Lua API and native menus. The Lua registration step runs before the simulator reads the model. The generator opens the experiment tool after preflight. Every experiment starts with a new in-memory filesystem and waits for a model write on shutdown. The [runtime adapter](simulator/runtime.cjs) verifies the extracted JavaScript and WASM separately.

| ETHOS | Reported revision | Model format written | ZIP SHA-256 |
| --- | --- | --- | --- |
| 1.6.8 | `a6b30b5e` | `0.1.37` | `efe70e0e2c529951772108d8fe4535059273464276e740640963ba8d6cd0ab12` |
| 26.1.2 | `ca464ac3` | `0.2.22` | `4e4c3588fce7a5ff8407a34cd41471cdf2016ea3ca7e48faca7353399f2fa82d` |
| 26.2.0 | `0a6b337c` | `0.2.29` | `5df9c74f9267bccb121a7c145109d59ed89ae977a2deca5645c3be0a5528cbf9` |

The final four payload bytes are the model runtime counter. Their unsigned little-endian value agreed with the newer firmware's `runTime` field, including 111 before normalization and 0 after it. The generator sets that counter to zero on public fixtures and refreshes the header CRC to make repeated experiments comparable. Two independent default model saves reproduced the original 450-byte default after this normalization.

### Model categories

All five 1.6.8 wizard categories use the same X20 Pro AW board word and `0.1.37` version. Their prefixes and mixes differ. The stock models below were created in independent disposable radio profiles. Each before/after sensor pair starts from the same saved stock file. [Category experiments](fixtures/categories.cjs)

| Legacy category | Firmware-created configurations checked |
| --- | --- |
| 0 · Airplane | Synthetic fixtures and two local pilot backups. |
| 1 · Glider | Stock three-mix model, named four-sensor repair pair, and four-flap model with a fourth mix. |
| 2 · Helicopter | Stock Flybarless model, named four-sensor repair pair, and a three-servo 135° swash model. |
| 3 · Multi | Stock five-mix model, named four-sensor repair pair, and Roll mapped to channel 6. |
| 4 · Other | Empty stock model and named four-sensor repair pair. The 26.2.0 JSON oracle represents this category as 99. The editor retains the original legacy byte. |

On all four additional categories the native Windows CLI repaired the selected `03/6803` sensors to `03/6802`. Each output matched the separate firmware-generated target exactly, with only four ID bytes and the checksum changed. The 1.6.8 firmware loaded and resaved each exported copy.

**Fresh Multi wizard results.** Independent wizard initializations produced `01`, `87` and `DF` in the byte containing the throttle-cut sticky flag. Bit 0 remained set. Ordinary 1.6.8 reopen/resave and actual standalone repairs preserved the complete source byte. Snapshot SHA-256 values identify the captured files. Across fresh builds, the checker reports the observed values and offsets and allows only the high bits at that public fixture's measured offset and its CRC to differ. Both sides of a Multi sensor pair must retain the entire byte from that run's stock file. Other fixtures require byte-identical replay. [Replay checker](fixtures/verify-replay.cjs)

## Fields and experiment coverage

| File structure | Verified with | Observations |
| --- | --- | --- |
| Header | All fixtures and independent CRC vector | `FRSK`, marker 1, version bytes 00 01 25, little-endian payload length, board word `0x1381`, payload CRC-16/XMODEM. ASCII `123456789` has CRC `0x31C3`. |
| Model label | `default`, `name-short`, `name-long` | Absolute byte 18 gives the stored byte count. Text follows and shifts subsequent fields. Empty stored name displays as “New model”. Firmware truncates inputs at 15 bytes, including UTF-8 sequences spanning that boundary. |
| Model bitmap | `bitmap-path`, `bitmap-max`, `bitmap-clear` | A 16-byte slot stores up to 15 UTF-8 filename bytes followed by NUL. Native clearing replaced the first byte and preserved the old trailing bytes. The model contains a reference to the separate image. |
| Output channel | `channel` | Output index 1 named “Flap left”, center +3, limits −72/+83, inverted. Matching Lua read-back and converted JSON agree. |
| Curve | `curve` | A native “none” curve named “Bench rate” changes the counted data before timers. Other types require individual evidence. |
| Timer | `timer`, `layout-references` | Timer 1 has down direction, start 173, alarm 37. Timer 0 in the second fixture refers to telemetry slot 1. Lua and converted JSON confirm the slot. |
| S.Port identity | `throttle-before`, `throttle-after`, `layout-references` | Four slots retain band 1, module 0, Physical ID `03` and sub-IDs 0, 1, 2, 4. Independently building them with Application ID `6802` instead of `6803` changes exactly bytes 366, 402, 447, 487 and checksum bytes 14, 15. Duplicate names occur on different module/band/sub-ID combinations. |
| S.Port variants | `sensor-variants`, `servo-addresses` | The first experiment creates 47 keys for receiver voltage, signal quality, RSSI and servo measurements. The second independently exercises 19 further servo address/sub-ID pairs. Different identities can select different trailing fields. |
| Order and key conflicts | `throttle-reordered`, `id-conflict` | Independent firmware saves swap slots, add/remove measurements, and create two “Twin” rows with matching module, band, sub-ID and Physical ID. Changing either twin to its peer's Application ID would collide. |
| Other sensor kinds | `diy`, `calculated-consumption`, `calculated-custom`, `calculated-operation` | The DIY key carries tag 2. Native calculated Consumption and Custom use tag 1 with subtypes 0 and 6. Default sources, a persistent flag and an operation count have different shapes. |
| Timer actions and limits | `timer-audio-one`, `timer-audio-three`, `timer-limits` | Ordered countdown value, countdown beep and named sound file, individual haptic flags, minimum and maximum signed 32-bit times. |
| Flight modes, mixes, logic switches, functions and variables | `flight-mode`, `free-mix`, `logic-switch`, `special-function`, `variable` | Independent default insertions identify each byte span. Firmware JSON confirms the second mode, free mix and 100% weight action, comparison switch, disabled Reset and variable bounds. Other variants need matching evidence. |
| Physical and extra trims | `trim-range-*`, `extra-trim-*` | Physical and extra trim ranges run from 0% to 100%. Extra trims add a name and separate Up/Down source references to the shared trim settings. Captures include the four menu modes, FS1 triggers and a second trim following a variable-length name. [Trim and trainer experiments](fixtures/trim-trainer.cjs) |
| Trainer | `trainer`, `trainer-master-*`, `trainer-slave-*`, `trainer-trim-sensors-*` | The eight Master/Slave and cable/Bluetooth/external SBUS/external CPPM combinations have native save/load evidence. Master stores 16 channel mappings with condition, operation, signed weight and destination. Slave stores start and count. The external-module settings survived ordinary 1.6.8 reloads. Converted 26.2 JSON can omit them when the module is disabled. |
| Radio modules | `rf-*` | Coverage includes internal ACCESS/ACCST, TD and TW plus tested XJT Lite, R9M Lite, Twin Lite Pro, PPM and SBUS variants. The plane's enabled external PPM (4–16 channels) and SBUS (16 channels) profiles support verified span edits when receiver slots are empty. Receiver names and the XJT D16 failsafe menu are browsable. Physical binding, transmission, and failsafe behavior require a radio. [RF experiments](fixtures/radio.cjs) |
| Checklist | `checklist`, `checklist-sa-down`, `checklist-sa-none`, `checklist-failsafe` | The low two switch bits encode all four SA choices. The failsafe reminder has its own byte. Firmware-generated variants verify that edits preserve other switch bits. |
| Screen and widget data | `screen`, `widget-title`, `widget-timer-source`, `timer-widget`, `chart-widget`, `gps-widget`, `lipo-widget`, `channels-widget`, `text-widget` | Two layouts, counted widget slots, indices, per-widget fields and timer source references. A Value widget supports a zero-based `Timer N`, `Telemetry N` or `None` reference. The codec checks the selected target exists. |
| Checklist and chart limits | `checklist-throttle-*`, `chart-period-*`, `chart-auto`, `chart-manual`, `chart-inverted`, `chart-min-*`, `chart-max-*` | The checklist accepts a throttle threshold of −100 to +100. Chart intervals use 10 ms units (5–100), with a separate inversion flag. Auto range omits the two signed manual limits. For an unassigned source, manual bounds are −1024 to +1024 and may be equal. Source-dependent bound edits need a matching source experiment. |
| External Lua references | `lua-references`, `widget-reference` | The synthetic references are `FxTask`, `FxSrc` and `FxView`. The codec preserves length-delimited configuration bytes. The Lua implementation and assets reside outside the model. |
| Telemetry flags | `telemetry-competition`, `telemetry-bluetooth` | Independent toggle experiments identify the first two bytes after sensors. Competition remains read-only because the firmware clears the telemetry collection when it is enabled. |

A firmware experiment establishes the values it exercised. The [editor's support matrix](README.md) lists the fields available through the desktop and CLI. Sensor names, bitmap text and other opaque bytes can contain misleading byte patterns. A matching text fragment supplies no evidence that an offset belongs to a sensor entry.

## Telemetry collection

The default file's 16-bit little-endian slot count begins at offset 353. A five-byte model label moves it to 358. Adding a timer name and assigning its telemetry source also moves it. Each record has an eight-byte key, a one-byte name length, its text and a type-dependent body. The source slot is its index in the original ordered collection.

| Offset within the S.Port key | Encoding |
| --- | --- |
| 0 | Variant 0 for S.Port |
| 1 | Module index, verified with internal 0 and external 1 |
| 2 | Band, verified with 0 and 1 |
| 3 | Raw receiver routing byte. Bit meanings need a controlled receiver-index experiment. |
| 4 | Sub-ID |
| 5 | Physical ID |
| 6..8 | Little-endian Application ID |

After the text, the first 11 bytes hold a display unit, decimal count, flags and two 32-bit limits. The reset source follows. A missing source is one zero byte. The observed telemetry and switch-position references store a one-byte category, one option byte and a two-byte little-endian member. A timer referencing telemetry slot 1 stored `16 00 01 00`. Variable and arithmetic operands need additional framing.

The following suffix sizes start after the reset source and include the two-byte lost-sensor delay. They apply to the sampled keys. Equal lengths can encode different fields. In a controlled read-back, substituting `0100` for `F010` changed a 10,000 ms warning delay to 12,839 ms. ID repair requires a verified compatible sensor family.

| Application ID and sub-ID | Observed suffix size |
| --- | --- |
| `6800`–`6803`, subs 0, 4, 5, 6 | 2 |
| `6800`–`6803`, sub 1 | 10 |
| `6800`–`6803`, sub 2 | 6 |
| `6804`, `6805`, `6808`, `6809`, subs 0 / 1 / 2 / 4 | 2 / 10 / 6 / 2 |
| `6807`, subs 0 / 1 / 2 | 2 / 10 / 6 |
| `F108` (subs 0–2), `0DE0` (subs 0–1), `F103` (sub 0) | 10 |
| `F010` and `F011`, sub 0 | 3 |
| `F101`, sub 0 / sub 1 | 18 / 2 |
| `0D80`, `0DA0`, `0DB0`, subs 0–1 | 2 |
| `F107`, `0E20`, `0DC0`, `0DF0` (sub 0), `0D90` and `0DD0` (sub 1) | 2 |
| `0100`, sub 0 | 3 |

## Delivery and support by milestone

### Sensor collection changes

The desktop app appends complete records to the counted telemetry collection. New S.Port entries use the display fields and protocol payload of a matching firmware-discovered Application ID and sub-ID from the public corpus. Paste copies the complete encoded record. Export checks S.Port keys against the original model and preserves the source file.

The dependency graph uses stored source operands and curve selectors encountered by the sequential decoder. Stored categories `13`, `17`, `21`, `22`, `23` and `24` resolve to logic switches, variables, timers, sensors, sensor-alive status and flight modes respectively. Cascade can remove verified counted records and clear widget slots. A retained reference keeps its source category and options while its member follows the target's new slot. The writer checks the resulting counts and reference stream after reopening. Fixed settings, opaque Lua and incomplete formats can block deletion.

### Curves, logical switches and variables

| Native experiment | On-disk observation and read-back |
| --- | --- |
| Custom curves | After the packed offset and point count, the first byte is Easy mode and the next is Smooth. Easy mode stores Y only. Explicit X uses whole percentages and Y uses tenths. The native UI supports 2–21 points and endpoint X values of −100 and +100. Equal neighbouring X values make a step. ETHOS clamped requested Y values of ±125.5% to ±100%. [Curve experiments](fixtures/curves.cjs), [FrSky curve controls](https://ethos-doc.frsky-rc.com/model-setup/curves/) |
| Curve selectors and dependencies | A free mix used its default curve and an FS1 conditional curve. Native movement from slot 0 to slots 1 and 4 changed both stored selectors from 2 to 3 and 6. Removing the referenced curve cleared both selectors to zero while keeping the condition and the other curves. Inline Expo uses selector 1 followed by a packed operand. The existing curve also retained its mix reference when resized 3 → 5 → 2 points. |
| Expo and function curves | Expo offset uses tenths, exponent and weight use whole units. The corpus exercises signed ±125 for exponent/weight and ±125.5% for offset. Seven function menu selections are recorded. Calling the 1.6.8 Lua Expo offset setter on another curve type changes it to Expo. |
| Logical switches | Native controls verify equality, greater/less-than, Boolean AND/OR/XOR and timer generator. The Boolean functions use 2–9 source operands. SA, SB and FS1 have independent category/member references. The analog threshold menu gave −100% and +100%. The timer duration menu gave 0.1–175.0 seconds, stored in tenths. Inversion and the two delay fields have separate controlled captures. [Logical switch experiments](fixtures/logic.cjs) |
| Variables | Each conditional rate stores its condition and value. The first eight action operators share the same framing. Repurpose adds an explicit trim source and step. The native step menu gave −1000 and +1000 raw units. Adding and removing the first of two rows preserved the remaining row. A variable can store up to 9 conditional rates and 10 actions on this firmware. [Variable experiments](fixtures/variables.cjs) |

| Issue | Current evidence and scope |
| --- | --- |
| #124 · Fixtures | The manifest inventories the public 1.6.8 captures from all five wizard categories and records each file's digest, payload length, CRC and board. Independent runtime read-backs confirm the requested settings. |
| #125 · Codec and CLI | The offline reader opens the X20 Pro AW `0.1.37` configurations listed above and exposes typed paths, exact byte ranges and opaque data. A supported unchanged model serializes byte-for-byte. |
| #126 · Comparison and repair | The four Application ID differences are reproducible in the public and local acceptance pairs. Selected repairs preserve independent bytes. The writer rejects new key conflicts and Application ID changes outside the demonstrated servo family `6800`–`6803`. |
| #127 · Desktop | Windows x64 desktop and CLI share inspection, comparison, staging, validation and exclusive copy export. The package includes an example backup/current pair. |
| #128 · Schema coverage | The experiments above establish category prefixes, the sampled aircraft mixes, trim and trainer layouts, and specific RF profiles. Further mix, logic, special-function and telemetry variants need independent fixtures. External Lua configuration depends on the owning script's schema. |
| #129 · Same-format editing | Editable fields include names and bitmap references, output and trim settings, selected compatible RF spans, trainer links and channels, free-mix settings, supported sensor IDs, ordered timer actions, eligible curves, logical switches, variable rates/actions, checklist fields and chart/widget settings. The codec exposes write capability per field. Unverified source forms and opaque dependency-bearing data block affected edits. |
| #130 · Optional validation | Real firmware loaded and resaved disposable copies in 1.6.8. Explicit tests of 26.1.2 and 26.2.0 detect conversions. Saved exports remain separate from disposable read-backs. |
| #131 · Distribution | The plane milestone at `c20b9da` passed the hosted portable codec, Windows build and clean-Windows package jobs. The smoke test opens the examples, compares, stages and exports through the GUI and packaged CLI. Public release publication and physical-radio trials are pending. [Hosted run](https://github.com/doppleganger53/sloppy-ethos/actions/runs/46085294357) |

## Optional firmware read-back

```powershell
node simulator/validate.cjs C:\sim\X20PROAW-FCC-1.6.8 fixtures/throttle-after.bin
node simulator/validate.cjs C:\sim\X20PROAW-FCC-1.6.8 fixtures/throttle-after.bin --readback C:\temp\new-readback.bin
$env:ETHOS_EDITOR_SIM_168 = 'C:\sim\X20PROAW-FCC-1.6.8'
$env:ETHOS_EDITOR_SIM_2612 = 'C:\sim\X20PROAW-FCC-26.1.2'
$env:ETHOS_EDITOR_SIM_2620 = 'C:\sim\X20PROAW-FCC-26.2.0'
node --test simulator/runtime.test.cjs
```

The standalone adapter emits a JSON report with the firmware revision, read/write diagnostics, warnings, source and resulting formats, and changed byte ranges. A successful `ok` or `converted` report requires fresh firmware read and write diagnostics plus validation of the resulting model header and checksum. The 26.1.2 and 26.2.0 adapters report conversion to `0.2.22` and `0.2.29`, respectively. A requested read-back file must have a fresh name. Normalization can change runtime counters and firmware-managed settings.

The CLI and desktop add decoded differences for a `0.1.37` read-back. Conversion reports include byte ranges. Their decoded difference list is empty because the standalone codec currently supports only the legacy target.

Firmware evidence comes from FrSky's published [1.6.8 Lua reference](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/1.6.8/lua-doc.zip) and [WebSimulator API](https://github.com/FrSkyRC/ethos-tools/blob/f0a24ec3a2d9f3949d8fc521fd21d4001c8ee912/simulation/wasm_simulator.md). All experiment code and public fixtures were produced for this editor. Physical-radio validation is pending.
