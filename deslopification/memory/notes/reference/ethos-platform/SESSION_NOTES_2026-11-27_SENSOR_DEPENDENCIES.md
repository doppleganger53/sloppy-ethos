# Session Notes 2026-11-27 - ETHOS sensor dependencies and collection changes

## Note Placement

- Artifact: `reference`
- Scope: `ethos-platform`
- Concern: `implementation`

## Format and verification

Issue #133 adds telemetry collection changes to the standalone X20 Pro AW FCC
`0.1.37` editor. New S.Port records take their common and variant defaults
from a firmware-discovered sensor with the same Application ID and sub-ID.
The editor copies complete encoded records and checks for new S.Port key
conflicts on export.

A stored four-byte source operand carries a category, options byte and
little-endian slot. Controlled conversion of a timer source from the legacy
format to 26.2.0 JSON confirmed categories 13 (logic switch), 17 (variable),
21 (timer), 22 (sensor), 23 (sensor alive) and 24 (flight mode). Collect
references as the decoder consumes operands, including those within partially
labelled records. Compare the retained reference stream and collection counts
on codec read-back after shifting slots. Clear dependent widget slots while
preserving their screen geometry. Opaque Lua objects and fixed settings can
block deletion.

The public `layout-references.bin` fixture contains two distinct telemetry
keys with the same label, plus a timer that reads sensor slot 1 and a Value
widget that reads that timer. Its two remaining widgets let a cascade check
verify that surviving timer references shift to the correct slots. The
packaged Windows smoke test stages a mixed deletion, reopens the separate
export in the codec, and can check an isolated ETHOS 1.6.8 resave with
`--runtime`. Physical-radio verification remains pending.

See the [format evidence](../../../../../model-editor/FORMAT.md#sensor-collection-changes).
