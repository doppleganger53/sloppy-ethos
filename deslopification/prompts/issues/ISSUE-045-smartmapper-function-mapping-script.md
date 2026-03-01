# Draft Prompt: Implement Issue #45 (SmartMapper Function Mapping Script)

Use this prompt to implement GitHub issue `#45`:
`https://github.com/doppleganger53/sloppy-ethos/issues/45`

## Canonical Issue

- URL: `https://github.com/doppleganger53/sloppy-ethos/issues/45`
- Title: `[Enhancement] SmartMapper function mapping script`
- Labels: `enhancement`
- Snapshot state: open on `2026-03-01`
- Target branch (default): `feature/45-smartmapper-function-mapping-script` (or as user-directed for current workflow)

## Branch/Worktree Gate (Required Before Editing)

1. Run issue preflight:
   - `python tools/session_preflight.py --mode issue --issue-number 45 --issue-kind enhancement --slug smartmapper-function-mapping-script`
2. Confirm target branch and current branch:
   - `git branch --show-current`
   - `git status --short --branch`
3. If preflight blocks due to `main`, create/switch to the recommended branch before editing.
4. If branch mismatch or dirty worktree is present, stop and confirm stash/commit/switch strategy.
5. After switching branches, sync before editing:
   - `git pull --ff-only origin {target-branch}`

## Current Status

Implementation is deferred pending Ethos `1.7.x` final runtime validation.

Reason:

- Ethos `1.6.4` widget runtime does not expose the model-enumeration APIs needed
  to inspect mixes, special functions, logical switches, trims, or switch
  inventory.
- The accessible `model.getChannel()` API exposes output-oriented channel data,
  not the routing metadata required to reconstruct function-to-input mappings.

Do not treat Ethos `1.6.4` widget context as a viable target for the full
feature. Before resuming implementation, first validate the relevant APIs on the
final Ethos `1.7.x` runtime.

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
- Ethos version target: `1.7.x final+`
- Project layout:
  - Script entry point: `scripts/SmartMapper/main.lua`
  - Optional images: `scripts/SmartMapper/images/`
  - Optional i18n: `scripts/SmartMapper/i18n/`

## References
- Open source projects for reference or code re-use (attribute appropriately)
  - https://github.com/lthole/Ethos-switch-maps

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
7. Function naming discovery hirearchy (if a definition is found traversing this list top to bottom, function is named, do not evaluate further):
   - If a special function active condition for a given switch position has a play audio action - use the first filename of the audio sequence as the switch position label
   - If a special function active condition for a given switch position has a play text action - use the text attribute as the switch position label.
   - If a mix defines a given switch position as its input, use the mix name as the switch position label


## Data Access Requirements

- Primary data source(s): Ethos Lua model/configuration APIs that expose mixes,
  logic switches, trims, special functions, switch definitions, and related
  model state accessible from a widget-safe context
- First implementation gate: verify that Ethos `1.7.x` final actually exposes
  these APIs in the intended script context before writing feature logic
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
