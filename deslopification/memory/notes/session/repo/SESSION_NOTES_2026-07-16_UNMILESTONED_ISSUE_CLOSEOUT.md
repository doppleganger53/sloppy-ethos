# Session Notes 2026-07-16 - Unmilestoned issue closeout

## Note Placement

- Artifact: `session`
- Scope: `repo`
- Concern: `implementation`

## What changed

- Resolved the remaining unmilestoned BoundryMap work:
  - synchronized shared GPS latitude/longitude query names with each widget's
    configured source, including alternating instances, cleared settings, and
    persisted settings (`#68`);
  - locked the existing split between live 3D distance and stale 2D ground
    distance with acceptance coverage and complete reset cleanup (`#69`);
  - added an optional selected speed source and 1-to-10-second projected
    boundary warning with fresh-data, valid-heading, outbound-path, unit
    conversion, overlay, and feedback-cadence guards (`#74`).
- Bumped BoundryMap to `0.1.8` and documented the new settings, stale-distance
  semantics, and GPS-ground-speed versus airspeed tradeoff.
- Revalidated the prior X20S SensorList startup report (`#30`) on clean Ethos
  1.6.4 and 1.6.6 simulator deployments with no startup errors.
- Confirmed the PowerShell-only coverage task reported by `#60` had already
  been removed by the shell-agnostic coverage workflow delivered in PR `#73`.
- Files touched:
  - `scripts/BoundryMap/main.lua`
  - `scripts/BoundryMap/tests/lua/test_boundrymap.lua`
  - `scripts/BoundryMap/README.md`
  - `scripts/BoundryMap/VERSION`
  - `docs/ETHOS_26_1_COMPATIBILITY.md`
  - `CHANGELOG.md`
  - `deslopification/memory/CURRENT_STATE.md`

## Why

- Root cause or objective:
  - close every open issue without a milestone using current implementation and
    runtime evidence rather than leaving already-completed work open.
- Scope guardrails:
  - changes remained inside `sloppy-ethos`; sibling reference checkouts and the
    parent workspace repository were read-only.

## Validation run(s)

- `luac -p scripts/BoundryMap/main.lua`
  - result: pass
- `python -m pytest scripts/BoundryMap/tests -q`
  - result: pass (`3 passed`)
- `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
  - result: pass (`184 passed, 26 skipped`)
- `python -m pytest -q`
  - result: pass (`365 passed, 26 skipped`)
- `python tools/build.py --project BoundryMap --dist`
  - result: pass; packaged `dist/BoundryMap-0.1.8.zip`
- `python tools/build.py --project BoundryMap --deploy`
  - result: pass; deployed to the configured Ethos Suite simulator persist path
- `python tools/sim/harness/run.py headless --suite tools/sim/harness/suites/BoundryMap-X20RS-FCC-1.6.6.json --no-download --run-dir tools/sim/runs/issues-68-69-74-boundrymap-x20rs-166 --timeout-ms 20000`
  - result: pass; `status=success`, `started=true`, no errors
- SensorList X20S-FCC Ethos 1.6.4 and 1.6.6 clean-start smoke runs
  - result: pass; `status=success`, `started=true`, no errors

## Follow-up items

- None for issues `#30`, `#60`, `#68`, `#69`, and `#74`.

## Current State Sync

- `CURRENT_STATE.md` updated: yes
