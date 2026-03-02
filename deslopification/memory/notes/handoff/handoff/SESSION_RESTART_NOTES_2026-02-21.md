# Session Restart Notes (2026-02-21)

## Note Placement

- Artifact: `handoff`
- Scope: `handoff`
- Concern: `workflow`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

## Current Objective

- Keep iterating on `SensorList` while maturing build/deploy workflow.
- Use `deslopification/` as the new home for prompt and handoff artifacts.

## Important Repo State

- Working tree currently shows moved/deleted files from old locations:
  - `docs/HANDOFF_2026-02-21.md` deleted (moved under `deslopification/` flow)
  - `prompts/EthosWidgetPromptTemplate.md` deleted (moved)
  - `prompts/SensorList.md` deleted (moved)
  - `deslopification/` untracked/new structure
- Before committing, verify moves are intentional and stage as renames if desired.

## Build/Deploy Status

- Cross-platform build script: `tools/build.py`
- Dist build command:
  - `python tools/build.py --project SensorList --dist`
- Deploy command:
  - `python tools/build.py --project SensorList --deploy`
- Local simulator config file (gitignored):
  - `tools/deploy.config.json`
  - key: `ETHOS_SIM_PATH`

## Deploy Caveat (Important)

- Previous deploy logic was changed to avoid deleting `SensorList` first.
- Current deploy updates in place via:
  - `shutil.copytree(..., dirs_exist_ok=True)`
- If deploy fails with `Permission denied`, likely causes:
  - simulator/radio process still locking files
  - no write access for current shell/session to target path

## Quick Validation Steps After Restart

1. Check Python:
   - `python --version`
2. Check Lua parser:
   - `luac -p src/scripts/SensorList/main.lua`
3. Run dist build:
   - `python tools/build.py --project SensorList --dist`
4. Run deploy:
   - `python tools/build.py --project SensorList --deploy`
5. If deploy fails, confirm path and permissions:
   - verify `tools/deploy.config.json`
   - close Ethos simulator fully and retry

## Next Good Tasks

- Confirm moved files in `deslopification/` and commit the reorg cleanly.
- Add a small deploy preflight command to `build.py`:
  - check target writability and print explicit diagnosis before copy.
- Add a minimal deploy troubleshooting section to README/DEVELOPMENT docs.