# Pull Request Template

## Summary

-

## Changes

-

## Verification

Check all items that apply to touched files (see `AGENTS.md` validation matrix):

- [ ] Lua script behavior changes:
  - `luac -p scripts/SensorList/main.lua`
  - `python -m pytest tests/test_sensorlist_widget.py -q`
- [ ] Documentation changes:
  - `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- [ ] Build/tooling changes:
  - `python -m pytest tests/test_build_py.py -q`
- [ ] Broad/cross-cutting changes:
  - `python -m pytest -q`
- [ ] Packaging/build command run for touched project(s), as needed:
  - `python tools/build.py --project SensorList --dist`
  - `python tools/build.py --project ethos_events --dist`
- [ ] Manual simulator/radio check performed (required for UI/input behavior changes)

## Notes

- Linked issue(s):
- Expected merge strategy: `{squash|merge commit}`
- If `merge commit`, reason:
- Risks / follow-up items:
