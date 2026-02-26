# ethos_events

System-tool wrapper around the upstream Ethos event helper from:

- `https://github.com/FrSkyRC/ETHOS-Feedback-Community/tree/1.6/lua/ethos_events`

## What is included

- `scripts/ethos_events/main.lua`: local entrypoint compatible with this repository's `tools/build.py`.
- `scripts/ethos_events/ethos_events.lua`: upstream helper module.
- `scripts/ethos_events/example.lua`: upstream example script.
- `scripts/ethos_events/UPSTREAM_README.md`: adapted usage notes for this repo's self-contained folder layout.

## Build and deploy

From repository root:

```powershell
python tools/build.py --project ethos_events --dist
python tools/build.py --project ethos_events --deploy
```

The deployed tool appears in Ethos system tools as `Ethos Events`.
Artifact version source: `scripts/ethos_events/VERSION`.
