# ETHOS model editor implementation reference

Build a standalone Rust desktop editor for ETHOS model `.bin` files. The first
release should let a pilot compare two backups, see which telemetry IDs changed,
and repair selected existing sensors in their original model format. The full
editor should browse every stored structure, enumerate its keys and known enum
values, and edit every field supported by the verified schema.

Keep the codec in a reusable Rust library. The CLI and desktop app should use
the same parser, diff, validation, and writer. A future Rust SloppyLogExplorer
can reuse that library or launch the editor with two file paths.

## First release scope

- **Radio target** is X20 Pro AW with the FCC simulator and model format
  `0.1.37`.
- **Pilot workflow** starts with opening one or two model files. Browse
  decoded data, inspect a
  settings diff, stage Physical ID and Application ID edits for existing S.Port
  entries, preview the changes, and save a separate copy.
- **Offline operation** covers open, diff, edit, and export with
  the simulator absent. FrSky simulators generate development fixtures and can
  later provide an optional cross-check. Keep original and output files at the
  same model format within this scope.
- **Compatibility expansion** treats each additional radio or model format as a
  target with its own fixtures and support claims. A radio model file and its
  external Lua scripts, widget assets, and radio-wide settings are separate
  inputs.

## Evidence from the prototypes

Two private X20 Pro AW exports in format `0.1.37` had four throttle telemetry
entries for current, voltage, temperature, and position. All four kept Physical
ID `03`. Their Application ID changed from `6803` in the older export to `6802`
in the later one. The sub-IDs were `0`, `1`, `2`, and `4`, respectively.
This comparison establishes the stored change. The correct target ID for a
repair comes from the actual sensor configuration.

The desktop prototype used offsets verified against one file layout. The
browser prototype used firmware-decoded keys to locate the corresponding
records in the original file. It exported a same-format ID repair, checked that
only the requested ID bytes and checksum changed, and reopened the result in
the simulator. A separate browser test edited telemetry and a timer and
reopened the firmware-generated `0.2.29` copy. Those tests exercised the
prototype and its simulator workflow. Physical-radio behavior remains
unverified. A Rust format implementation still needs to establish record
boundaries and work across models with different layouts.

### Model file envelope

The following 16-byte header was verified on the prototype's X20 Pro AW
fixtures. Offsets are zero-based.

| Offset | Encoding | Observed meaning |
| --- | --- | --- |
| `0..4` | ASCII `FRSK` | File signature |
| `4` | `01` | Observed model-header marker |
| `5..8` | Three bytes, displayed in decimal | Model format, for example `00 01 25` = `0.1.37` |
| `8..12` | Little-endian `u32` | Payload length, equal to the file size minus 16 |
| `12..14` | Little-endian `u16` | `0x1381` on the X20 Pro AW samples |
| `14..16` | Little-endian `u16` | CRC-16/XMODEM over the complete payload starting at byte 16 |

The CRC uses polynomial `0x1021`, an initial value and final XOR of zero, and
no reflection. As a standalone check, ASCII `123456789` produces `0x31C3`.
Validate signature, supported header and format, exact payload length, and
checksum before interpreting typed data. The marker and board discriminator
belong to the observed compatibility entry until verified on other targets.

### Legacy S.Port key and sensor label

In the observed `0.1.37` S.Port records, six bytes immediately precede the
one-byte name length and name bytes. Relative offsets below start at the first
byte of that prefix.

| Relative offset | Observed data |
| --- | --- |
| `0` | Band |
| `1` | Raw RF routing byte. Establish its bit meanings with controlled module and receiver cases. |
| `2` | Sub-ID |
| `3` | Physical ID |
| `4..6` | Little-endian Application ID |
| `6` | Name length in bytes |
| `7..` | Name bytes |

For example, `01 00 00 03 02 68 08` followed by eight name bytes represents
an observed S.Port entry on band 1 with sub-ID 0, Physical ID `03` and
Application ID `6802`. Within that verified entry, changing the Application ID
to `6803` changes `02 68` to `03 68`. The writer also updates the file CRC.
FrSky displays S.Port Physical IDs in the hex range `00` through `1B`.
Application IDs occupy a 16-bit field and display as four hex digits. Check
firmware-specific rules for sensor identity and allowed combinations separately
from the numeric encoding. [FrSky telemetry manual](https://ethos-doc.frsky-rc.com/model-setup/telemetry/#smart-port-telemetry)

The collection start, count, record boundaries, variants, and optional fields
still need verification. Model JSON from the simulator exposes indexed sources
such as `{"category":"telemetry_sensor","member":2}`. That helps design
experiments for dependencies. Confirm each reference's encoding in the original
format and test the effect of renaming, moving, or deleting the target.

## Fixture generator and schema research

Use FrSky's official X20 Pro AW FCC WebSimulator assets pinned to the runtime
under test. These are the prior prototype observations.

| ETHOS release | Observed firmware revision | Model format evidence |
| --- | --- | --- |
| [1.6.8](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/1.6.8/X20PROAW-FCC-WebSimulator.zip) | `a6b30b5e` | Loaded and resaved `0.1.37`. Use programmatic UI input or manual control of the isolated simulator for legacy fixture generation. The inspected build had no model JSON export. |
| [26.1.2](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/26.1.2/X20PROAW-FCC-WebSimulator.zip) | `ca464ac3` | Loaded `0.1.37`, exposed model JSON, and saved converted `0.2.22` files. The inspected build did not export `applyModelJson`. |
| [26.2.0](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/26.2.0/X20PROAW-FCC-WebSimulator.zip) | `0a6b337c` | Exposed model JSON read/apply and saved `0.2.29`. One legacy export loaded directly. Another needed conversion through 26.1.2 before 26.2.0 could load it. |

Each release link names `X20PROAW-FCC-WebSimulator.zip`, containing
`X20PROAW_FCC.js` and `X20PROAW_FCC.wasm`. The inspected 26.1.2 and 26.2.0
WASM files retain model load diagnostics and field-name strings. They lacked
the inspected function-name, DWARF, and source-map sections. Start with
generated fixtures and use the diagnostics or disassembly for questions the
fixtures leave open.

1. **Pin the target and isolate storage.** Reuse runtime discovery and caching
   from the [repo simulator harness](../tools/sim/harness/README.md) where it
   fits. Use disposable persistence and a scratch copy of each input.
   Record the asset name, release tag, reported revision, SHA-256, and model
   header. Keep user backups, account paths, simulator binaries, and generated
   traces in local storage. Check in synthetic fixtures with their provenance.
2. **Generate a repeatable baseline.** Start with a default model and resave
   it without edits. Repeat runs to expose background writes or changing
   values. Record the action, before/after files, observed values, and firmware
   diagnostics. Wait for the write and validate the persisted file before
   collecting the result. FrSky documents the model and filesystem hooks in
   its [WebSimulator API reference pinned to the prototype revision](https://github.com/FrSkyRC/ethos-tools/blob/f0a24ec3a2d9f3949d8fc521fd21d4001c8ee912/simulation/wasm_simulator.md).
3. **Vary one field at a time.** Use one-field JSON edits plus rereads where
   the matching build supports them. On legacy builds, capture UI input
   sequences and verify the actual saved parameter. Use S.Port injection where
   exported, and record which RF module and band are enabled. If a matching
   build cannot generate a case, collect it using that firmware on a test
   radio and record the source. Keep converted-file evidence marked with its
   resulting format.
4. **Find the record grammar.** Repeat a change on different records and at
   different positions. Rename with different lengths, insert, remove, and
   reorder entries. Include duplicate names, repeated keys, empty collections,
   different sensor types, and different bands or receivers. Record offsets,
   widths, endianness, string encodings, variant tags, optional fields,
   counts, and length rules. Require structural parsing before authorizing
   edits to an entry.
5. **Map enums and references.** Exercise each available choice on the
   matching firmware. Record raw values with their display labels and
   accepted ranges. Create telemetry references in a calculated sensor, logic
   switch, and screen. Change IDs, names, and order separately to establish
   which operation changes or invalidates each reference. Compare with
   firmware read-back to catch clamping, defaults, and normalization.

Keep a coverage matrix by board, format, and structure. For each structure,
record fixture IDs, keys, known enums, decode support, byte preservation,
supported edits, and dependency checks. Keep an observed raw value available
when the display label is still unknown.

## Rust codec and UI contract

- **One typed model tree.** Expose the file's board and format, ordered
  collections, and per-field paths, stored values, decoded types, enum labels,
  and edit capability. Keep file provenance and byte ranges so callers can
  connect a displayed value to the parsed record. Represent absent, null,
  default, and unidentified values distinctly.
- **Typed comparison.** Group differences in the same model sections used
  by the editor. Compare stored values, additions, removals, and meaningful
  order changes. Pair named telemetry records across ID changes and use
  verified routing or sub-ID fields to disambiguate duplicates. Report
  ambiguous matches and changes in opaque regions. [FrSky Model Setup menu](https://ethos-doc.frsky-rc.com/model-setup/overview/)
- **Edits target a parsed entry.** For ID repair, stage edits against the
  existing record and preserve its slot, sub-ID, routing, and every
  unrelated byte. Validate final IDs as a batch and detect newly conflicting
  sensor keys. Use a separate, exclusive output file, recompute the checksum,
  reopen it through the Rust codec, and verify the requested values before
  reporting success.
- **Unknown structures remain inspectable.** Expose bounded opaque bytes with
  their offset and length. Retain those bytes during unrelated edits. If a
  requested write would rebuild an unverified region or affect unresolved
  references, report the unsupported edit with its field path. Enable edit
  support per field only when its encoding and dependencies are verified.
- **Separate entry points.** Start with library operations for inspection,
  comparison, edit validation, and serialization. Give the CLI readable and
  structured output. Build the desktop UI over the same operations and keep
  FrSky runtime integration in an optional validation adapter. Leave the
  desktop toolkit choice to the UI issue.

## Implementation stages and acceptance

### Stage 1 — Generate fixtures and document the format

Establish the version-matched generator, record schema evidence beside fixture
manifests, and exercise the legacy telemetry cases above. A second contributor
can reproduce the test cases from synthetic models and identify the input,
requested change, accepted value, output bytes, and model format.

### Stage 2 — Build the Rust codec and inspection CLI

Decode the envelope and enumerate legacy telemetry records and keyed values
across independently built fixtures with the simulator absent. Verify
byte-for-byte decode and re-encode through the serializer for untouched
fixtures. Exercise wrong versions, truncation, bad sizes and CRCs, malformed
records, and unsupported variants. Return actionable errors on invalid input.

### Stage 3 — Add decoded diff and original-format telemetry repair

Report the four throttle Application ID changes from `6803` to `6802` with
Physical ID `03` in the local acceptance pair. Reproduce the same result with
public synthetic fixtures. Repair selected rows offline and verify that only
the selected ID bytes and checksum differ. Test ambiguity, newly conflicting
keys, failed writes, source protection, and reopening the repaired file.

### Stage 4 — Deliver the desktop editor and diff

Open one or two files, search the decoded tree, and browse differences by
section. Filter telemetry by name or ID, show old and proposed hex values,
and export edits to a separate copy. Exercise the same workflow with the
simulator absent and verify that the GUI and CLI agree. This is the first
desktop milestone.

### Stage 5 — Decode the remaining model structures and enums

Expand the typed schema by structure group. Start with model settings, timers,
channels, and trims. Cover mixes, flight modes, curves, variables, logic
switches, and special functions. Include RF and receiver data, checklist,
trainer, Lua references, and screen and widget settings. Give each group
separate fixtures and track unresolved bytes or reference forms in the
coverage matrix. Split the implementation issues where the confirmed
structure boundaries support independent reviews.

### Stage 6 — Enable full same-format editing

Support verified scalar and enum edits plus variable-length strings and
collection changes. Preserve ordered records and remap references when the
format requires it. Test edits that change size, remove references, and use
minimum and maximum values. A saved copy must reopen with the intended
changes and every untouched structure preserved. Gate full-edit support for
each target on its documented schema coverage.

### Stage 7 — Add optional simulator validation

Provide a **Validate in ETHOS** action using a separately cached or
user-selected, version-matched simulator and a disposable model copy. Report
the original and resulting format, load status, warnings, and changed or
normalized values. Distinguish conversion from same-format validation.
Exercise both the standalone workflow and the optional adapter under test.

### Stage 8 — Package the desktop editor separately

Add Rust CI when the crate lands, with format checks, build and fixture
tests. The default tests run without FrSky assets. Put opt-in simulator
checks in a separate job or manual workflow, with pinned runtime downloads.
Build the desktop artifact through its own release path and verify it on a
clean Windows machine with supported and unsupported files. Keep the
existing radio script ZIP workflow and its validation active. Publish a
board/format support matrix with the desktop download and state which checks
were observed in the simulator and on a physical radio. Packaging can ship
the telemetry desktop milestone from stage 4, then expand as later stages land.
