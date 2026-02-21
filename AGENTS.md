# AGENTS.md

Repository-level operating notes for future Codex sessions in `sloppy-ethos`.

## Primary Goal

Maintain and evolve the Ethos Lua `SensorList` widget with fast, safe iteration and minimal rediscovery between sessions.

## Startup Workflow (Do First)

1. Read latest memory notes from `deslopification/memory/` before making changes.
2. Scan current repo status:
   - `git status --short --branch`
3. Confirm active version:
   - `VERSION`
4. If touching widget behavior, read:
   - `src/scripts/SensorList/main.lua`
   - `src/scripts/SensorList/README.md`

## Memory Rule

- Treat `deslopification/memory/` as the canonical session history.
- After meaningful work, add or update a concise memory note with:
  - what changed
  - why it changed
  - validation run
  - next-session follow-ups

## Build, Test, Deploy

From repo root:

- Dist build:
  - `python tools/build.py --project SensorList --dist`
- Simulator deploy:
  - `python tools/build.py --project SensorList --deploy`
- Lua syntax check only:
  - `luac -p src/scripts/SensorList/main.lua`

Notes:
- Deploy path comes from `tools/deploy.config.json` or `ETHOS_SIM_PATH`.
- `tools/deploy.config.json` is local-only and gitignored.

## Packaging + Versioning

- Version source of truth is `VERSION`.
- Dist filename format must remain:
  - `dist/{ProjectName}-{version}.zip`
- When releasing:
  1. Update `VERSION`
  2. Build dist ZIP
  3. Commit release changes
  4. Tag `v{version}`
  5. Push `main` and tag
  6. Create GitHub release and attach ZIP

## Widget Behavior Constraints

- Do not filter out duplicate sensor names; duplicate names are valid telemetry configurations.
- Keep list rendering deterministic and readable.
- Prefer low-frequency polling/deep-scan strategy to avoid instruction-budget issues.
- Keep touch scrolling behavior stable across simulator/radio differences.

## Code Change Guidelines

- Keep changes scoped; avoid unrelated refactors.
- Preserve Ethos widget lifecycle correctness (`create`, `paint`, `wakeup`, `event`, etc.).
- Add comments only for non-obvious logic.
- If changing discovery logic, validate both:
  - populated sensor path
  - empty-state path

## Documentation Sync

When behavior or tooling changes, update docs in same session:

- `README.md` (user-facing usage/status/roadmap)
- `docs/DEVELOPMENT.md` (developer workflow details)
- `deslopification/memory/*.md` (handoff context)

## Done Criteria (Default)

A task is complete when all are true:

1. Code updated and syntactically valid.
2. Relevant build/deploy command run successfully.
3. Docs updated if behavior/workflow changed.
4. Memory note captured for future sessions.
