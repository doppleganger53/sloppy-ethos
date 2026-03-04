# Ethos Platform Operating Notes

## Note Placement

- Artifact: `reference`
- Scope: `ethos-platform`
- Concern: `implementation`
- Store this file under:
  - `deslopification/memory/notes/{artifact}/{scope}/`

Reusable guidance for Ethos runtime behavior, API quirks, simulator differences, and cross-script implementation patterns.

## Runtime And API Constraints

- Treat sensor/source member access as unsafe across simulator and radio targets; use guarded probing (`pcall`) before reading candidate fields or invoking method-backed accessors.
- Physical radios may expose function-valued accessors such as `source:subId()` where the simulator permits direct field reads.
- Keep callback paths defensive (`create()`, `paint()`, `wakeup()`, `event()`) and prefer fail-soft logging over silent widget failure.

## Performance Patterns

- Keep expensive source discovery out of a single callback when possible; stage broad scans across later ticks to respect the on-device instruction budget.
- Prefer queued refresh work over long inline refresh operations inside touch callbacks.

## Simulator And Layout Differences

- Simulator touch and icon behavior can diverge from the radio, so validate event mappings and resource assumptions on hardware before treating simulator behavior as authoritative.
- Tool and icon assets may need Ethos-specific dimensions or placement to load consistently across radio and simulator contexts.

## File And Deploy Conventions

- Keep simulator-path configuration in `tools/deploy.config.json` or environment variables only; do not commit machine-specific paths.
- Reuse `tools/build.py` deploy and dist workflows instead of ad hoc copy scripts so install paths stay consistent.

## Related Notes

- SensorList-specific operating guidance: `notes/reference/sensorlist/SensorList.md`
