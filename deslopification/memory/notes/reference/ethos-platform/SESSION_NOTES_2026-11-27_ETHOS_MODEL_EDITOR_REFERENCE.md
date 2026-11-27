# Session Notes 2026-11-27 - ETHOS model editor reference

## Note Placement

- Artifact: `reference`
- Scope: `ethos-platform`
- Concern: `implementation`
- Store this file under:
  - `deslopification/memory/notes/reference/ethos-platform/`

## What changed

- The [model editor reference](../../../../../docs/ETHOS_MODEL_EDITOR.md)
  records the Rust codec design, observed binary fields, simulator versions,
  fixture method, and staged acceptance checks.

## Why

- A standalone codec can serve the CLI, desktop editor, and a future Rust log
  explorer. The matching simulator remains available for fixtures and optional
  validation. The first target is X20 Pro AW format `0.1.37`.

## Validation run(s)

- Rechecked the observed header, CRC, and S.Port key prefix on local X20 Pro AW
  samples and prototype outputs. The reference distinguishes prototype
  observations from planned Rust and physical-radio checks.

## Follow-up items

- Use synthetic fixtures and verify record boundaries before enabling edits.
  Track format coverage and work through the linked GitHub implementation issues.

## Current-State Index

- Retrieval pointer in `CURRENT_STATE.md` updated: yes.
