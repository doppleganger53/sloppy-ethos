# Current-State Index

Use this index when prior decisions or unresolved work affect the task. Read
only the relevant reference or matching catalog note, and verify facts that can
drift against source, current versions, and runtime evidence.

| Task needs | Read |
| --- | --- |
| Operational policy or required checks | [AGENTS.md](../../AGENTS.md) |
| Build, deploy, or release procedures | [Development notes](../../docs/DEVELOPMENT.md) |
| Ethos 26.1 compatibility and follow-up gates | [Compatibility baseline](../../docs/ETHOS_26_1_COMPATIBILITY.md) |
| Runtime/API/simulator behavior | [Ethos platform reference](notes/reference/ethos-platform/EthosPlatform.md) |
| Standalone ETHOS model editor architecture and binary format evidence | [Model editor reference](../../docs/ETHOS_MODEL_EDITOR.md) |
| SensorList behavior | [SensorList reference](notes/reference/sensorlist/SensorList.md) |
| Script-specific behavior or version | The affected script's README and VERSION in scripts/ |
| Prior decisions or unresolved handoff | Search [CATALOG.md](CATALOG.md) by Scope, Concern, and topic |
| Note placement and catalog maintenance | [Memory guide](README.md) |
| The previous state snapshot | [July 17 archive](notes/reference/memory/CURRENT_STATE_2026-07-17_ARCHIVE.md) |

Use dated notes as evidence rather than instructions to replay. In particular,
match Ethos API claims to the exact firmware/runtime under test; release assets
may contain API documentation missing from reference checkout examples.
