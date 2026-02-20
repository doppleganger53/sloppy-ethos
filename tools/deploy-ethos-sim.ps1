param(
  [string]$RepoRoot = (Get-Location),
  [string]$ProjectName = 'SensorList'
)

$scriptDir = Join-Path $RepoRoot \"src/scripts/$ProjectName\"
Write-Host \"Deploying $ProjectName from $scriptDir to the Ethos simulator...\"
# TODO: replace this placeholder logic with the real copy/deployment steps
if (-not (Test-Path $scriptDir)) {
  Write-Error \"Script directory '$scriptDir' does not exist.\"
  exit 1
}

Write-Host \"Mock deploy completed (adjust script for your simulator setup).\"
