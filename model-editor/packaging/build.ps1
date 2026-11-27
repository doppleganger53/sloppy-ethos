[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$project = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$oldEncodedFlags = $env:CARGO_ENCODED_RUSTFLAGS
$oldStrip = $env:CARGO_PROFILE_RELEASE_STRIP
Push-Location $project
try {
    $hostTarget = ((& rustc -vV) | Select-String '^host: ').Line.Substring(6)
    if ($hostTarget -notmatch '^x86_64-pc-windows-(msvc|gnu)$') {
        throw "Build this Windows x64 package on a supported host. Current host is $hostTarget."
    }
    # Rust panic locations may include absolute dependency and workspace paths.
    # Cargo's encoded form also preserves remappings for directories with spaces.
    $separator = [char]31
    if ($oldEncodedFlags) {
        $flags = @($oldEncodedFlags.Split($separator))
    } else {
        $baseFlags = $env:RUSTFLAGS
        if (-not $baseFlags) {
            $name = 'CARGO_TARGET_' + $hostTarget.ToUpperInvariant().Replace('-', '_') + '_RUSTFLAGS'
            $baseFlags = [Environment]::GetEnvironmentVariable($name)
        }
        $flags = @($baseFlags -split ' ' | Where-Object { $_ })
    }
    $prefixes = @(
        @{ Path = $env:USERPROFILE; Replacement = '/user' }
        @{ Path = $env:CARGO_HOME; Replacement = '/cargo' }
        @{ Path = $env:RUSTUP_HOME; Replacement = '/rustup' }
        @{ Path = $project; Replacement = '/ethos-model-editor' }
    )
    foreach ($prefix in $prefixes) {
        if ($prefix.Path) {
            $native = [IO.Path]::GetFullPath($prefix.Path)
            foreach ($spelling in @($native, $native.Replace('\', '/')) | Select-Object -Unique) {
                $flags += "--remap-path-prefix=$spelling=$($prefix.Replacement)"
            }
        }
    }
    $env:CARGO_ENCODED_RUSTFLAGS = $flags -join $separator
    $env:CARGO_PROFILE_RELEASE_STRIP = 'symbols'
    & cargo build --locked --release -p ethos-model-desktop -p ethos-model-cli
    if ($LASTEXITCODE -ne 0) { throw 'Desktop build failed.' }
    $metadata = (& cargo metadata --locked --format-version 1 | Out-String) | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Unable to resolve the workspace for packaging.' }
    $desktop = $metadata.packages | Where-Object name -eq 'ethos-model-desktop'
    $version = $desktop.version
    if ($env:GITHUB_REF_TYPE -eq 'tag' -and $env:GITHUB_REF_NAME -ne "ethos-model-editor-v$version") {
        throw "Release tag must match desktop version $version."
    }
    $outputDir = Join-Path $metadata.target_directory 'package'
    [IO.Directory]::CreateDirectory($outputDir) | Out-Null
    $zip = Join-Path $outputDir "ethos-model-editor-$version-windows-x64.zip"
    $temporaryZip = Join-Path $outputDir ([guid]::NewGuid().ToString() + '.tmp')
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::Open($temporaryZip, [IO.Compression.ZipArchiveMode]::Create)
    function Add-PackageFile($source, $entry) {
        [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $source, $entry, [IO.Compression.CompressionLevel]::Optimal) | Out-Null
    }
    try {
        foreach ($program in 'ethos-model-desktop.exe', 'ethos-model.exe') {
            Add-PackageFile (Join-Path $metadata.target_directory "release/$program") $program
        }
        foreach ($doc in 'README.md', 'FORMAT.md') {
            Add-PackageFile (Join-Path $project $doc) $doc
        }
        Add-PackageFile (Join-Path $project '../LICENSE') 'LICENSE.txt'
        foreach ($example in 'throttle-before.bin', 'throttle-after.bin', 'sensor-variants.bin') {
            Add-PackageFile (Join-Path $project "fixtures/$example") "examples/$example"
        }
        foreach ($adapter in 'runtime.cjs', 'worker.cjs', 'validate.cjs') {
            Add-PackageFile (Join-Path $project "simulator/$adapter") "simulator/$adapter"
        }
        foreach ($package in $metadata.packages | Where-Object source) {
            $root = Split-Path $package.manifest_path
            foreach ($license in (Get-ChildItem -LiteralPath $root -File | Where-Object Name -Match '^(LICENSE|COPYING|NOTICE)([-.].*)?$')) {
                Add-PackageFile $license.FullName ("licenses/{0}-{1}/{2}" -f $package.name, $package.version, $license.Name)
            }
        }
    } finally {
        $archive.Dispose()
    }
    Move-Item -LiteralPath $temporaryZip -Destination $zip -Force
    Write-Output $zip
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $oldEncodedFlags
    $env:CARGO_PROFILE_RELEASE_STRIP = $oldStrip
    if ($temporaryZip -and (Test-Path -LiteralPath $temporaryZip -PathType Leaf)) { Remove-Item -LiteralPath $temporaryZip }
    Pop-Location
}
