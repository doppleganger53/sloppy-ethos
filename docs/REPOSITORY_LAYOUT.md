# Repository Layout Guide

This document explains the purpose of the top-level folders and key root files in `sloppy-ethos`.

## Top-Level Directories

### `scripts/`

- Source for Ethos Lua projects.
- Current active widget: `scripts/SensorList/`.

### `tools/`

- Build, packaging, and deployment tooling.
- Key scripts:
  - `tools/build.py`: canonical cross-platform build/deploy entrypoint.
  - `tools/create_todo_issues.py`: utility to create TODO-tracking GitHub issues.
  - `tools/deploy.config.example.json`: template for local simulator-path config.
  - `tools/config/stylua.toml`: Lua formatting configuration for StyLua workflows.

### `requirements/`

- Python dependency manifests used by contributor and CI workflows.
- Key file:
  - `requirements/dev.txt`: Python test dependencies.

### `tests/`

- Automated validation for Python tooling, Lua behavior, and documentation contracts.
- Key areas:
  - `tests/test_build_py.py`: behavior tests for `tools/build.py`.
  - `tests/test_sensorlist_widget.py` + `tests/lua/test_sensorlist.lua`: Lua-facing widget tests.
  - `tests/test_docs_commands.py`: verifies documented commands remain valid/safely marked manual.
  - `tests/test_docs_contracts.py`: enforces documentation consistency and required references.

### `docs/`

- Contributor/developer-facing documentation.
- Primary guide: `docs/DEVELOPMENT.md`.
- SensorList architecture reference: `docs/SensorList/SENSORLIST_ARCHITECTURE.md`.
- This file (`docs/REPOSITORY_LAYOUT.md`) documents repository structure and root artifacts.

### `deslopification/`

- Project memory and prompt artifacts that preserve implementation context.
- Subfolders:
  - `deslopification/memory/`: session notes, handoffs, and project-specific operating context.
  - `deslopification/prompts/`: source prompts and roadmap prompt variants.

## Root Artifacts

- `README.md`: user-facing entrypoint with quick-start and collaboration links.
- `CONTRIBUTING.md`: contributor workflow and coding guidance.
- `AGENTS.md`: coding-agent operating instructions for this repository.
- `TODO.md`: current backlog and follow-up actions.
- `VERSION`: package version source of truth.
- `LICENSE`: project license (GPLv3).
- `requirements/dev.txt`: Python test dependencies.
- `.editorconfig`: repository-wide formatting defaults.
- `.gitignore`: ignored local artifacts (dist output, local config, caches, virtualenv, etc.).
- `tools/config/stylua.toml`: Lua formatting configuration.
