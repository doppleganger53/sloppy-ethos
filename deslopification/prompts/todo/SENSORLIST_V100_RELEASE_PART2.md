# Prompt: Complete SensorList v1.0.0 Release (Part 2)

## Mission

Finish the deferred release workflow after Part 1 branch migration is complete.
Manual testing issue #32 is the release gate and must be completed before tagging.

## Required Context

- AGENTS.md
- README.md
- docs/DEVELOPMENT.md
- CHANGELOG.md
- VERSION
- scripts/SensorList/VERSION
- deslopification/prompts/templates/RELEASE_RESOLUTION_TEMPLATE.md
- GitHub issues: #8, #9, #17, #32
- Milestone: SensorList-v1.0.0

## Startup Workflow (Required)

1. Read latest relevant note in deslopification/memory/.
2. Check branch/worktree:
   - git branch --show-current
   - git status --short --branch
3. Sync main:
   - git checkout main
   - git pull --ff-only origin main
4. Confirm migrated issue state:
   - #8 closed
   - #9 closed
   - #17 closed
   - #32 open or recently completed

## Gate 1: Complete Manual Testing (#32)

1. If #32 is still open, execute manual tests on simulator and physical radio:
   - Header sorting per column and direction toggle.
   - Header drag-cancel does not trigger sort.
   - Conflict markers/colors for high vs low severity.
   - Long-press refresh feedback behavior and fallback.
   - No duplicate refresh on touch-end after long press.
2. Record environment details and outcomes in #32 comment.
3. Close #32 only if all required checks pass.
4. Stop release if #32 remains open.

## Release Branch Preparation

1. Create release branch from main:
   - git checkout -b release/v1.0.0
2. Update files:
   - VERSION => 1.0.0
   - scripts/SensorList/VERSION => 1.0.0
   - CHANGELOG.md => add 1.0.0 section summarizing #8/#9/#17 and manual validation gate completion
3. Validate branch cleanliness:
   - git status --short --branch

## Build and Validation (Required)

1. Build:
   - python tools/build.py --project SensorList --dist
2. Confirm artifact exists:
   - dist/SensorList-1.0.0.zip
3. Run:
   - luac -p scripts/SensorList/main.lua
4. Run:
   - python -m pytest tests/test_sensorlist_widget.py -q
5. Run:
   - python -m pytest tests/test_docs_commands.py tests/test_docs_contracts.py -q
6. If a required command hangs, rerun once with higher timeout and report exact failure if still incomplete.

## PR, Tag, and Publish

1. Commit release metadata changes on release/v1.0.0.
2. Push branch and open PR to main.
3. Merge PR into main.
4. Sync main:
   - git checkout main
   - git pull --ff-only origin main
5. Verify tag does not exist:
   - git ls-remote --tags origin v1.0.0
6. Tag and push:
   - git tag v1.0.0
   - git push origin v1.0.0
7. Publish release with single SensorList artifact:
   - gh release create v1.0.0 dist/SensorList-1.0.0.zip --title "SensorList-v1.0.0" --notes "<CHANGELOG 1.0.0 notes>"
8. Verify release:
   - gh release view v1.0.0 --json tagName,name,url,isDraft,isPrerelease,publishedAt

## Close Milestone and Cleanup

1. Confirm issues #8, #9, #17, #32 are closed.
2. Close milestone SensorList-v1.0.0.
3. Delete release branch:
   - git push origin --delete release/v1.0.0
   - git branch -d release/v1.0.0
4. Delete old milestone branch:
   - git push origin --delete milestone/sensorlist-v1.0.0
   - git branch -d milestone/sensorlist-v1.0.0

## Session Memory Sync (Required)

Create a session note in deslopification/memory/ with:
- what changed
- validation commands and results
- release URL/tag
- milestone closure confirmation
- branch cleanup confirmation
