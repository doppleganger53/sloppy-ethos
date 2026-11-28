# Session Notes 2026-11-27 - Generate and validate X20 Pro AW legacy models

- Artifact: `reference`
- Scope: `ethos-platform`
- Concern: `testing`

## What changed

For issue #123, the X20 Pro AW FCC 1.6.8 WebSimulator generated synthetic
legacy models using its native telemetry menu and documented Lua model APIs.
Script registration runs before model load. Opening the fixture tool after
preflight ensures its actions apply to the selected model.

## Validation

The 1.6.8 firmware wrote and reopened format `0.1.37` in disposable storage.
An independent 26.1.2 JSON read-back confirmed the four sensor identities,
duplicate labels, module selection and a timer's telemetry source index.
The newer runtime persists the converted model in `0.2.22`. Receipts, the
fixture generator and regeneration commands are in the
[format evidence](../../../../../model-editor/FORMAT.md).

## Follow-up

Additional record variants, receiver routing choices and hardware behavior
require their own evidence. The current-state index already points to the
model editor reference.
