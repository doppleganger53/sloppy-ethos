# Session Notes 2026-02-26 - ethos_events-v0.1.0 Release

## Note Placement

- Artifact: `session`
- Scope: `ethos-events`
- Concern: `release`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`


## What changed

- Committed script artifact version update for `ethos_events`:
  - `scripts/ethos_events/VERSION`: `0.1.0`
- Built release artifact from repository root:
  - `dist/ethos_events-0.1.0.zip`
- Published GitHub release and tag:
  - tag: `ethos_events-v0.1.0`
  - release title: `ethos_events-v0.1.0`
  - release URL: `https://github.com/doppleganger53/sloppy-ethos/releases/tag/ethos_events-v0.1.0`

## Validation run(s)

- `python tools/build.py --project ethos_events --dist`
  - result: passed
  - artifact: `dist/ethos_events-0.1.0.zip`

## Follow-up items

- Verify release asset download/install flow in Ethos Suite from the published release page.