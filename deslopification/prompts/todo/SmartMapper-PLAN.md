# SmartMapper #45 Ethos 26.1 Implementation Plan

## Summary
- Goal objective: complete issue #45 by replacing the current `SmartMapper Probe` with a production `SmartMapper` Ethos widget that maps accessible model assignments and identifies unused switch-like controls.
- Work only in `C:\Users\kurtk\Documents\Workspaces\EthosLua\sloppy-ethos`; keep sibling repos read-only except the requested fast-forward of `..\ETHOS-Feedback-Community-prerelease`.
- Start from current `main`, not the stale `feature/45-smartmapper-function-mapping-script` branch. Create a fresh issue branch:
  ```powershell
  git pull --ff-only origin main
  git -C ..\ETHOS-Feedback-Community-prerelease pull --ff-only origin 26.1
  python tools/session_preflight.py --mode issue --issue-number 45 --issue-kind enhancement --slug smartmapper-ethos26-implementation
  git checkout -b feature/45-smartmapper-ethos26-implementation
  python tools/session_preflight.py --mode issue --issue-number 45 --issue-kind enhancement --slug smartmapper-ethos26-implementation
  ```

## Key Changes
- Implement `scripts/SmartMapper/main.lua` as an Ethos widget registered through `system.registerWidget`, key `smrtmpr`, with startup-safe `create`, `paint`, `wakeup`, and `event` callbacks.
- Build the mapping from proven 26.1-safe source inventory via numeric `system.getSources(CATEGORY_*)`, plus defensive model enumeration using plural APIs first, count/getter APIs second, and bounded getter scans last.
- Normalize only explicit assignment fields: `input`, `source`, `src`, `switch`, `sw`, `activeCondition`, `condition`, `control`, `trim`; include logical-switch `values()` when present, and ignore name-only channel/input records.
- Use the #45 naming priority: special-function play-audio filename, then play-text text, then mix name, then a plain subsystem fallback only when a real control assignment was found.
- Render assigned controls first, grouped by control type and sorted case-insensitively, then unused switch/function-switch entries, then API status rows. Show `No accessible model mappings found.` only when neither mappings nor useful inventory/status are available.
- Keep scans bounded: initial deep scan on create, 1s lightweight polling, no more than one deep rescan every 5s unless manually refreshed. Wheel and touch drag scroll manually; no auto-scroll.

## Public Interfaces
- Add harness probe support to `tools/sim/harness/run.py`: repeated `--probe NAME`, repeated `--expect-probe-report NAME`, suite keys `probes` and `expectProbeReports`, and result JSON `probeReports`.
- Add `tools/sim/harness/probes/SmartMapperApiProbe/main.lua` as a harness-only runtime probe that prints `[SimProbe:SmartMapperApiProbe] {json}` with category constants, `system.getSources` summaries, and candidate `model.*` read surfaces.
- Add `tools/sim/harness/suites/SmartMapper-X20RS-FCC.json` targeting `SmartMapper`, `latest-26.1`, `X20RS-FCC`, and requiring the SmartMapper probe report.
- Update `scripts/SmartMapper/README.md`, `docs/ETHOS_26_1_COMPATIBILITY.md`, `docs/REPOSITORY_LAYOUT.md`, `CHANGELOG.md`, and repo memory notes. Bump `scripts/SmartMapper/VERSION` to `0.2.0`; do not add SmartMapper to README release-download links until it is actually published.

## Test Plan
- Script tests: `luac -p scripts/SmartMapper/main.lua` and `python -m pytest scripts/SmartMapper/tests -q`.
- Harness tests: `python -m pytest tests/test_sim_harness.py -q`, covering probe staging, suite parsing, probe report parsing, and missing expected probe failure.
- Docs/memory tests: `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py tests/test_memory_catalog_sync.py -q`.
- Runtime/build checks: `python tools/build.py --project SmartMapper --dist`, `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/SmartMapper-X20RS-FCC.json`, then `python tools/build.py --project SmartMapper --deploy`.
- Final broad check after focused fixes: `python -m pytest -q`.

## Assumptions
- “Full intent” means use every readable Ethos 26.1+ API surface available at runtime, while explicitly reporting unavailable mix/trim/special-function APIs instead of fabricating mappings.
- Prior SmartMapper branches are read-only reference material; do not merge or cherry-pick their unrelated repo drift.
- If the simulator exposes less model data than hardware, the widget remains complete for simulator-supported data and documents the gap in the compatibility baseline.
