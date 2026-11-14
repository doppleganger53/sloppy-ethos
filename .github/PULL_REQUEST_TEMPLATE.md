# Pull Request Template

## Summary

What changed, and what can a pilot or contributor do now?

## How to try it

Which build or commit should a reviewer use? Give the shortest steps to repeat
the check and the result to look for.

- What you observed (name the simulator or radio, model, and ETHOS version if applicable):
- What still needs testing:

## Verification

Check all items that apply to touched files (see `AGENTS.md` validation matrix):

- [ ] Lua changes: list the affected script, files parsed, local tests run
  (e.g. `python -m pytest scripts/SensorList/tests -q` for SensorList), and
  the matching runtime check or coverage gap.
- [ ] Documentation changes:
  - `python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q`
- [ ] Build/tooling changes:
  - `python -m pytest tests/test_build_py.py -q`
- [ ] Broad/cross-cutting changes:
  - `python -m pytest -q`
- [ ] Packaging/build command run for touched project(s), as needed:
  - `python tools/build.py --project SensorList --dist`
  - `python tools/build.py --project ethos_events --dist`
- [ ] UI/input changes: describe the interaction and observed result on the
  named simulator or radio, or identify what still needs checking.

## Notes

- Linked issue(s):
- Expected merge strategy: `{squash|merge commit}`
- If `merge commit`, reason:
- Risks / follow-up items:
