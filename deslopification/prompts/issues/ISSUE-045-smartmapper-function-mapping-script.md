# Draft Prompt: Implement Issue #45 (SmartMapper Function Mapping Script)

Use this prompt to implement GitHub issue `#45`:
`https://github.com/doppleganger53/sloppy-ethos/issues/45`

# Prompt

You are generating code for a FrSky Ethos Lua widget.

## Goal

Create a widget named `SmartMapper` that implements:

- Primary function: inspect existing model configuration and build a readable
  function-to-input mapping across mixes, logic switches, trims, special
  functions, and related controls
- Primary user value: let pilots quickly understand what is already assigned and
  easily identify unused switches or unassigned control surfaces

## Environment

- Radio target(s): `X20RS` (and other Ethos radios exposing comparable model
  data APIs)
- Ethos version target: `1.6.4+`
- Project layout:
  - Script entry point: `scripts/SmartMapper/main.lua`
  - Optional images: `scripts/SmartMapper/images/`
  - Optional i18n: `scripts/SmartMapper/i18n/`

## Ethos API Requirements (Critical)

- Use Ethos-native widget registration:
  - `init()` calling `system.registerWidget({...})`
  - `return { init = init }`
- Use Ethos callbacks only (as needed): `create`, `paint`, `wakeup`,
  `configure`, `read`, `write`
- Do not assume OpenTX/EdgeTX callback naming (`refresh`, `background`,
  etc.) unless explicitly requested.
- Callback safety:
  - Defensively handle unexpected `nil` callback args.
  - Never crash on startup or page navigation if context is incomplete.

## Functional Requirements

1. Discover relevant model assignments that are accessible from the Ethos Lua
   runtime, including mixes, logic switches, trims, special functions, and
   comparable mapped controls.
2. Normalize discovered assignments into a single in-memory model that groups
   entries by control/input and by function usage, even when source metadata is
   inconsistent.
3. Render a readable mapping view that shows each assigned control, what it is
   used for, and the source subsystem (mix, logic switch, trim, special
   function, etc.).
4. Empty-state behavior: show `No accessible model mappings found.` when no
   supported assignment data is available.
5. Data ordering/grouping rules: show assigned controls first grouped by control
   type, sorted case-insensitively by display label; then show unused switches
   in a separate trailing section.
6. Interaction model:
   - Default behavior when list/content exceeds visible area: manual scrolling
     only; do not auto-scroll
   - Input navigation method(s): wheel up/down, touch swipe/drag if supported by
     the current radio

## Data Access Requirements

- Primary data source(s): Ethos Lua model/configuration APIs that expose mixes,
  logic switches, trims, special functions, switch definitions, and related
  model state accessible from a widget-safe context
- Fallback data source(s): cached last-known-good normalized mapping plus
  defensive placeholders when a specific subsystem cannot be enumerated
- Handle simulator-vs-radio API differences safely.
- If data handles may be table or userdata, support both where practical.
- Avoid expensive rescans every frame; use cached discovery where possible.

## Performance & Stability Requirements

- Keep `wakeup` lightweight; avoid heavy loops each call.
- Poll data on a defined interval: `1.0` seconds.
- Use periodic deep-rescan only when needed: full model remap on startup,
  after manual refresh, and no more than once every `5` seconds during steady
  state unless the model context changes.
- UI should be stable and deterministic (no forced auto-scroll unless requested).
- Avoid instruction-budget issues (`max instructions count reached`).

## UI Requirements

- Layout target: fullscreen widget
- Visual style: compact, high-signal, model-diagnostics-first
- Text/layout constraints:
  - truncate long labels and descriptions before overlap; do not wrap rows
  - align control labels left and mapped targets/details in consistent columns
  - use a clear visual distinction between assigned entries and unused-switch
    entries without relying on noisy color treatment

## Technical Constraints

- Keep implementation intentionally simple and maintainable.
- Prefer root-cause solutions over compatibility shims or temporary workarounds.
- Avoid unnecessary global state.
- Add brief comments only where behavior is non-obvious.
- Do not add unrelated features unless requested (alarms, logs, advanced menus).

## Packaging & Delivery Requirements

- Provide complete `main.lua`.
- If requested, include/update packaging script so output ZIP installs through
  Ethos Suite Lua import.
- Expected ZIP layout: `scripts/SmartMapper/...`

## Validation Requirements

- Script compiles with `luac -p`.
- Widget appears in Ethos widget picker.
- Widget renders without runtime errors in simulator and/or target radio.
- Data display path and empty-state path both validated.
- Interaction behavior validated (for example: manual scroll input works).

## Output Format

Return:

1. Complete `main.lua` implementation.
2. Short explanation of architecture and data flow.
3. Explicit test checklist (simulator + radio when applicable).
4. Any assumptions made due to API/version uncertainty.

## Optional Enhancements (Only if explicitly requested)

- Compact mode for smaller zones.
- User-configurable options (`configure/read/write`).
- Debug overlay toggle for API/event diagnostics.
