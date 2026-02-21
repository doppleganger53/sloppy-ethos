param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
  [string]$ProjectName = "SensorList",
  [string]$Version = "",
  [string]$OutDir = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($OutDir)) {
  $OutDir = Join-Path $RepoRoot "dist"
}

$sourceDir = Join-Path $RepoRoot "src/scripts/$ProjectName"
if (-not (Test-Path $sourceDir)) {
  throw "Script directory '$sourceDir' does not exist."
}

$luac = Get-Command luac -ErrorAction SilentlyContinue
if (-not $luac) {
  throw "Could not find 'luac' on PATH. Install Lua and make sure 'luac' is available."
}

if ([string]::IsNullOrWhiteSpace($Version)) {
  $versionFile = Join-Path $RepoRoot "VERSION"
  if (-not (Test-Path $versionFile)) {
    throw "Version file '$versionFile' does not exist."
  }
  $Version = (Get-Content -Path $versionFile -TotalCount 1).Trim()
}

if ([string]::IsNullOrWhiteSpace($Version)) {
  throw "Version cannot be empty."
}

if ($Version -notmatch '^[0-9A-Za-z][0-9A-Za-z._-]*$') {
  throw "Invalid version '$Version'. Use only letters, numbers, '.', '-', '_' and no spaces."
}

Write-Host "Checking Lua syntax with $($luac.Source)..."
$luaFiles = Get-ChildItem -Path $sourceDir -Recurse -File -Filter *.lua
foreach ($file in $luaFiles) {
  & $luac.Source -p $file.FullName
  if ($LASTEXITCODE -ne 0) {
    throw "Syntax check failed: $($file.FullName)"
  }
}

$zipName = "$ProjectName-$Version.zip"
$zipPath = Join-Path $OutDir $zipName

$stagingRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("ethos-build-" + [System.Guid]::NewGuid().ToString("N"))
$packageRoot = Join-Path $stagingRoot "package"
$packageScriptDir = Join-Path $packageRoot "scripts/$ProjectName"

try {
  New-Item -ItemType Directory -Path $packageScriptDir -Force | Out-Null
  New-Item -ItemType Directory -Path $OutDir -Force | Out-Null

  Copy-Item -Path (Join-Path $sourceDir "*") -Destination $packageScriptDir -Recurse -Force

  $packageReadme = Join-Path $packageScriptDir "README.md"
  if (-not (Test-Path $packageReadme)) {
    @"
# $ProjectName

Install with Ethos Suite:

1. Open Ethos Suite.
2. Use the Lua script install/import function.
3. Select this ZIP file.
4. Sync/transfer to the radio.

This package installs to: scripts/$ProjectName
"@ | Set-Content -Path $packageReadme -Encoding UTF8
  }

  if (Test-Path $zipPath) {
    Remove-Item -Path $zipPath -Force
  }

  Compress-Archive -Path (Join-Path $packageRoot "*") -DestinationPath $zipPath -Force
  Write-Host "Created package: $zipPath"
}
finally {
  if (Test-Path $stagingRoot) {
    Remove-Item -Path $stagingRoot -Recurse -Force
  }
}
