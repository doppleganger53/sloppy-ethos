# SensorList Architecture

This document captures the runtime lifecycle, state model, and event/data flow
for `scripts/SensorList/main.lua`.

## Widget Lifecycle

The widget registers through `init()` and uses the standard Ethos callbacks:

- `create()`: allocates per-instance widget state and performs initial deep
  sensor refresh.
- `paint(widget)`: draws table headers and visible sensor rows.
- `wakeup(widget, event)`: rate-limited invalidate scheduling; does not perform
  periodic sensor refresh.
- `event(widget, category, value, x, y)`: dispatches touch/long-press input for
  manual refresh and scrolling.

## State Model

Each widget instance owns state in the `widget` table, including:

- sensor and display data:
  - `sensors`, `groups`, `colorCache`, `lastSignature`, `lastRawCount`, `lastDebug`
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
   physical ID, application ID, then name.
3. Build signature (`buildSignature`) and only update widget tables when the
   signature changes.
4. Recompute conflict group mapping (`buildPhysicalGroups`) and reset color cache.
5. Clamp `scrollOffset` against current visible row count.

`allowDeepScan` gates expensive category scans:

- `true`: allows dynamic category discovery and full scan.
- `false`: avoids full scan and defers deeper work.

## Event And Input Flow

The `event()` callback handles user-driven interactions:

- Long press (`EVT_TOUCH_LONG` or equivalent category) triggers explicit full
  sensor refresh.
- Touch phases are resolved through `resolveTouchPhase()` to normalize Ethos
  event code variants across runtime versions.
- `handleTouchScroll()` accumulates movement and applies row-based scrolling.
- Scroll processing is protected by per-event caps to avoid extreme jumps.
- Touch-end can trigger long-press refresh fallback when dedicated long-press
  events are unavailable.

## Behavioral Invariants

- No periodic sensor polling in `wakeup()`.
- Manual refresh happens on `create()` and explicit long-press only.
- Sorting remains deterministic for stable visual ordering.
- Ethos API calls are guarded through `safeCall()` to fail soft on missing or
  variant runtime functions.
