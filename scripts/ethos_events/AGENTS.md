# AGENTS.md (ethos_events)

This file augments root `AGENTS.md` for work under `scripts/ethos_events/`.

## Scope

- Applies to `scripts/ethos_events/**`.
- Preserve `ethos_events` diagnostic intent: high-signal event visibility without destabilizing system navigation.

## Implementation guidance

- Keep event logging paths lightweight and safe on simulator and radio.
- Avoid introducing default behavior that swallows or overrides unrelated key events unless explicitly requested.
- Keep output readability and prefix/tag consistency when changing UI text.

## Validation additions

- Parse-check all touched Lua entrypoints in this project with `luac -p`.
- Run project and cross-cutting pytest coverage as required by root `AGENTS.md`.
