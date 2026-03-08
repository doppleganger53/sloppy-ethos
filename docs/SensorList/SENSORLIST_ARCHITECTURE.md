# SensorList Architecture

This document captures the runtime lifecycle, state model, and event/data flow
for `scripts/SensorList/main.lua`.

## Widget Lifecycle

The widget registers through `init()` and uses the standard Ethos callbacks:

- `create()`: allocates per-instance widget state and performs initial deep
  sensor refresh.
- `configure(widget)`: exposes the `Display Value` widget option.
- `paint(widget)`: draws table headers and visible sensor rows.
- `wakeup(widget, event)`: advances staged sensor discovery, rate-limits redraw
  invalidation, and refreshes displayed sensor values at roughly 5 Hz when the
  `Display Value` option is enabled.
- `event(widget, category, value, x, y)`: dispatches touch/long-press input for
  manual refresh and scrolling.
- `read(widget)` / `write(widget)`: persist the `Display Value` option through
  Ethos widget storage when available.
- Runtime faults in `create()`, `paint()`, and `event()` are trapped so the
  widget can log `SLERR ...` to serial and attempt to render a minimal error
  banner instead of failing silently.

## State Model

Each widget instance owns state in the `widget` table, including:

- sensor and display data:
  - `sensors`, `groups`, `colorCache`, `lastSignature`, `lastRawCount`,
    `lastDebug`, `showValue`
- source-discovery cache:
  - `sourceCategory`, `sourceMaxMember`
- scroll/redraw state:
  - `scrollOffset`, `needsInvalidate`, `lastInvalidate`
- touch interaction state:
  - `touchActive`, `touchLastY`, `touchAccumY`, `touchStartY`, `touchStartClock`,
    `touchHoldTriggered`
- debug counters:
  - `debugRefreshCount`, `debugDeepScanCount`, `debugCachedScanCount`,
    `debugDeferredScanCount`

## Data And Refresh Flow

`refreshSensors(widget, allowDeepScan)` is the central refresh pipeline:

1. Source discovery via `getSensorList()`:
   - prefer `system.getSensors()`
   - fallback to `model.getSensors()`
   - fallback to `system.getSource()` category/member scans
2. Normalize records (`normalizeSensors`) and sort deterministically by:
   physical ID, application ID, sub ID, then name.
   - candidate fields/methods are probed defensively so radio-only accessor
     differences (for example `source:subId()`) fail soft instead of crashing
     the widget.
   - best-effort value text is captured at refresh time from formatted/string
     or numeric members so paint-time rendering stays cheap.
3. Build signature (`buildSignature`) and only update widget tables when the
   signature changes, including value-text changes from manual refresh and
   periodic value polling.
4. Recompute conflict group mapping (`buildConflictGroups`) and reset color cache.
5. Clamp `scrollOffset` against current visible row count.

`allowDeepScan` gates expensive category scans:

- `true`: allows dynamic category discovery and full scan.
- `false`: avoids full scan and defers deeper work.
- Once a category is known, the widget expands that category in bounded chunks
  across `wakeup()` ticks so large sensor lists can finish loading without
  exhausting the Ethos callback instruction budget.

## Event And Input Flow

The `event()` callback handles user-driven interactions:

- Long press (`EVT_TOUCH_LONG` or equivalent category) triggers explicit full
  sensor refresh by resetting the staged scan window, then resumes loading over
  later `wakeup()` ticks with best-effort completion feedback (`playHaptic`
  fallback to `playTone`).
- Touch phases are resolved through `resolveTouchPhase()` to normalize Ethos
  event code variants across runtime versions.
- Header taps only sort the `Name`, `PhysID`, and `AppID` columns; `SubID`
  and optional `Value` remain display-only.
- `handleTouchScroll()` accumulates movement and applies row-based scrolling.
- Scroll processing is protected by per-event caps to avoid extreme jumps.
- Touch-end can trigger long-press refresh fallback when dedicated long-press
  events are unavailable, with a hold-trigger guard preventing duplicate
  refresh/feedback for the same long-press gesture.

## Behavioral Invariants

- Periodic `wakeup()` polling runs at roughly 5 Hz only while `Display Value`
  is enabled.
- Manual long-press remains the explicit full rescan path.
- Sorting remains deterministic for stable visual ordering.
- The default layout remains four columns unless `Display Value` is enabled.
- Ethos API calls are guarded through `safeCall()` to fail soft on missing or
  variant runtime functions.
