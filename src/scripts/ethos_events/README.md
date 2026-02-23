# ethos_events

System-tool wrapper around the upstream Ethos event helper from:

- `https://github.com/FrSkyRC/ETHOS-Feedback-Community/tree/1.6/lua/ethos_events`

## What is included

- `src/scripts/ethos_events/main.lua`: local entrypoint compatible with this repository's `tools/build.py`.
- `src/scripts/ethos_events/ethos_events.lua`: upstream helper module.
- `src/scripts/ethos_events/example.lua`: upstream example script.
- `src/scripts/ethos_events/UPSTREAM_README.md`: upstream usage notes.

## Build and deploy

From repository root:

```powershell
python tools/build.py --project ethos_events --dist
python tools/build.py --project ethos_events --deploy
```

The deployed tool appears in Ethos system tools as `Ethos Events`.
