# Pull Request Template

## Summary

-

## Changes

-

## Verification

- [ ] `luac -p src/scripts/SensorList/main.lua`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-package.ps1 -ProjectName SensorList`
- [ ] If docs changed: `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- [ ] Manual simulator check performed

## Notes

-
