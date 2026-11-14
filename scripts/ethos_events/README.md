# ethos_events

System-tool wrapper around the upstream Ethos event helper from:

- `https://github.com/FrSkyRC/ETHOS-Feedback-Community/tree/1.6/lua/ethos_events`

## What is included

- `scripts/ethos_events/main.lua`: local entrypoint compatible with this repository's `tools/build.py`.
- `scripts/ethos_events/ethos_events.lua`: upstream helper module.
- `scripts/ethos_events/example.lua`: upstream example script.
- `scripts/ethos_events/UPSTREAM_README.md`: adapted usage notes for this repo's self-contained folder layout.

## Try the tool

From the repository folder, open an isolated simulator:

```powershell
$preview = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
python tools/sim/harness/run.py gui --project ethos_events --radio X20RS-FCC --ethos-version 1.6.6 --persist-dir "$preview"
```

Open **Ethos Events** in the simulator's system tools. Tap the
`throttleSame` row and check that its status changes.

To build an install ZIP, run `python tools/build.py --project ethos_events --dist`.
When a task calls for updating a live Suite installation, verify the
configured path and name the radio, for example
`python tools/build.py --project ethos_events --deploy --sim-radio X20RS`.

## Runtime behavior

- The tool now shows recent formatted events directly in the tool UI.
- Console/debug output is still printed for each event.
- Duplicate consecutive lines are throttled by default (`throttleSame` starts `ON`).
- Toggle `throttleSame` at runtime by tapping the `ThrottleSame` status row in the UI.

Artifact version source: `scripts/ethos_events/VERSION`.
