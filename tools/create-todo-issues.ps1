param(
  [string]$Repo = "doppleganger53/sloppy-ethos"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$gh = "C:\Program Files\GitHub CLI\gh.exe"
if (-not (Test-Path $gh)) {
  throw "GitHub CLI not found at '$gh'."
}

& $gh auth status | Out-Null
if ($LASTEXITCODE -ne 0) {
  throw "GitHub CLI is not authenticated. Run '$gh auth login' first."
}

function New-Issue {
  param(
    [string]$Title,
    [string[]]$Labels,
    [string]$Body
  )
  $labelArgs = @()
  foreach ($label in $Labels) {
    $labelArgs += @("--label", $label)
  }
  $args = @(
    "issue", "create",
    "--repo", $Repo,
    "--title", $Title,
    "--body", $Body
  ) + $labelArgs
  $url = & $gh @args
  if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($url)) {
    throw "Failed to create issue: $Title"
  }
  return $url.Trim()
}

$todoRef = "Source backlog: TODO.md (TODO-03..TODO-09)."

$children = @()
$children += @{
  title = "[Enhancement] Generate roadmap prompt files for SensorList roadmap items"
  labels = @("enhancement", "docs")
  body = @"
## Problem Statement
Roadmap items exist in README but prompt artifacts are incomplete.

## Desired Outcome
Prompt files exist in deslopification/prompts for each roadmap item using the widget template structure.

## Proposed Approach
Create prompt files for sort headers, conflict display refinement, and acceptable conflicts model. Include non-goals, callback constraints, validation checklist, and acceptance criteria.

## Acceptance Criteria
1. Three roadmap prompt files are added.
2. Prompt content maps directly to README roadmap bullets.
3. Each prompt includes explicit non-goals and regression checks.

## Risks / Edge Cases
- Scope creep into implementation details.
- Prompt ambiguity for acceptable conflicts model.

$todoRef
"@
}
$children += @{
  title = "[Enhancement] Add unit tests for SensorList Lua behavior"
  labels = @("enhancement", "tests")
  body = @"
## Problem Statement
SensorList behavior is validated manually but lacks repeatable unit tests.

## Desired Outcome
Pytest-managed Lua tests validate core SensorList helper behavior and edge cases.

## Proposed Approach
Add a guarded `_test` export in `main.lua`, Lua test script with mocked Ethos APIs, and pytest wrapper to run the Lua test.

## Acceptance Criteria
1. Sensor ID parsing/formatting and normalization are covered.
2. Grouping/signature behavior is covered.
3. Scroll clamp and missing API fallback behaviors are covered.

## Risks / Edge Cases
- Lua runtime differences across developer machines.
- Mocked APIs drifting from simulator behavior.

$todoRef
"@
}
$children += @{
  title = "[Enhancement] Add unit tests for tools/build.py"
  labels = @("enhancement", "tests")
  body = @"
## Problem Statement
Build tooling has no automated regression tests for version/config/deploy behavior.

## Desired Outcome
Pytest unit tests cover core `tools/build.py` logic with mocked filesystem/process interactions.

## Proposed Approach
Add tests for version normalization, config parsing, simulator path resolution, deploy error formatting, and no-action guard path in main.

## Acceptance Criteria
1. Core pure helpers are covered with positive and negative cases.
2. Main no-op path exits with clear message.
3. Tests run without requiring real simulator paths.

## Risks / Edge Cases
- Over-mocking may hide integration issues.

$todoRef
"@
}
$children += @{
  title = "[Enhancement] Add tests for documentation command examples"
  labels = @("enhancement", "tests", "docs")
  body = @"
## Problem Statement
Documentation command snippets can drift from actual working workflows.

## Desired Outcome
Automated tests parse documented commands and validate syntax/path references, with explicit manual/environment-dependent skips.

## Proposed Approach
Add pytest docs command parser for README, DEVELOPMENT, SensorList README, and CONTRIBUTING.

## Acceptance Criteria
1. Command snippets are discovered from docs.
2. Referenced scripts/files exist.
3. Runnable commands execute or are explicitly skipped as manual.

## Risks / Edge Cases
- Some command snippets are intentionally partial in prose context.

$todoRef
"@
}
$children += @{
  title = "[Refactor] Streamline AGENTS.md and move SensorList-specific notes to memory"
  labels = @("refactor", "docs")
  body = @"
## Current Pain / Complexity
AGENTS mixes repo-wide and script-specific operational detail, causing repeated context noise.

## Refactor Boundaries
- In scope: Keep AGENTS concise and repo-level; move SensorList specifics to memory.
- Out of scope: Widget behavior changes.

## Safety Constraints
No workflow regressions; command references remain consistent with documented Python-first build flow.

## Validation Checklist
1. AGENTS startup workflow remains clear.
2. SensorList script notes are captured in `deslopification/memory/SensorList.md`.
3. No conflicting command guidance remains.

## Rollback Plan
Restore prior AGENTS content from git history and remove script-specific memory note if needed.

$todoRef
"@
}
$children += @{
  title = "[Enhancement] Refresh SensorList README with current behavior and Python-first workflow"
  labels = @("enhancement", "docs")
  body = @"
## Problem Statement
`src/scripts/SensorList/README.md` is stale and uses PowerShell-first packaging steps.

## Desired Outcome
README reflects current widget behavior and uses `python tools/build.py` as primary workflow, with PowerShell as fallback.

## Proposed Approach
Update behavior section (scroll/conflict grouping/discovery strategy) and replace install/build command examples.

## Acceptance Criteria
1. Python-first build/deploy commands are documented.
2. Scroll and conflict-group behavior are documented.
3. Wording aligns with repository README and DEVELOPMENT docs.

## Risks / Edge Cases
- Inconsistency with other docs if not updated together.

$todoRef
"@
}

$childUrls = @()
foreach ($item in $children) {
  $childUrls += New-Issue -Title $item.title -Labels $item.labels -Body $item.body
}

$childList = ($childUrls | ForEach-Object { "- $_" }) -join "`n"
$parentBody = @"
Tracks execution of TODO backlog items that remain after issue template creation.

## Child Issues
$childList

## Notes
- Linked from TODO.md item TODO-03.
- Child issues correspond to TODO-04 through TODO-09.
"@

$parentUrl = New-Issue -Title "[Enhancement] Track TODO backlog execution (tests, prompts, docs, AGENTS refactor)" -Labels @("enhancement") -Body $parentBody

Write-Output "Created parent issue: $parentUrl"
