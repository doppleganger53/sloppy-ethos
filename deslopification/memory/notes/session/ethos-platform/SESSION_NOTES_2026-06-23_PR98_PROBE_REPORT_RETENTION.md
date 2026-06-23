# Session Notes 2026-06-23 - PR #98 Probe Report Retention

## Note Placement

- Artifact: `session`
- Scope: `ethos-platform`
- Concern: `testing`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## What changed

- Addressed the unresolved PR `#98` review comment about simulator probe
  reports being parsed only after `websim_runner.js` truncated captured stdout.
- `websim_runner.js` now attaches parsed `probeReports` from its full captured
  simulator stdout before trimming the returned `stdout` tail.
- `run.py` now preserves runner-attached probe reports and also parses any
  full raw stdout lines captured before the final structured result.
- Added regression tests for both runner-attached reports and raw captured
  stdout lines.
- Files touched:
  - `tools/sim/harness/websim_runner.js`
  - `tools/sim/harness/run.py`
  - `tests/test_sim_harness.py`

## Why

- Root cause or objective:
  - Activated probe workflows could emit `[SimProbe:NAME]` before more than
    100 later simulator stdout lines, causing `--expect-probe-report` to report
    `missingProbeReports` even though the probe had actually emitted.
- Scope guardrails:
  - This fix only changes simulator harness probe-report collection. It does
    not alter SmartMapper widget behavior or probe activation rules.

## Validation run(s)

- `python -m pytest tests/test_sim_harness.py -q`
  - result: pass (`33 passed, 1 warning`)
- `python -m pytest tests/test_build_py.py -q`
  - result: pass (`70 passed`)
- `node --check tools/sim/harness/websim_runner.js`
  - result: pass
- `python -m pytest tests/test_memory_catalog_sync.py -q`
  - result: pass (`27 passed`)
- `git diff --check`
  - result: pass

## Follow-up items

- None.

## Current State Sync

- `CURRENT_STATE.md` updated: no
- If `no`, reason: bug fix to existing harness behavior, not a new durable
  workflow or operating decision.
