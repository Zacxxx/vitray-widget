# Bundle Release for Vitray Widget
$ErrorActionPreference = "Stop"

# Ensure setup_dev.ps1 has been run or env vars are set
if (-not $env:PKG_CONFIG_PATH) {
    Write-Warning "PKG_CONFIG_PATH is not set. Attempting to run setup_dev.ps1..."
    . "$PSScriptRoot\setup_dev.ps1"
}

Write-Host "Building Release..."
cargo build --release

$TargetDir = Join-Path $PSScriptRoot "target\release"
$ReleaseDir = Join-Path $PSScriptRoot "release_bundle"
$ExeName = "vitray-widget.exe"
$SourceExe = Join-Path $TargetDir $ExeName

if (-not (Test-Path $SourceExe)) {
    Write-Error "Build failed or executable not found at $SourceExe"
    exit 1
}

# Create Release Directory
if (Test-Path $ReleaseDir) {
    Remove-Item -Path $ReleaseDir -Recurse -Force
}
New-Item -ItemType Directory -Path $ReleaseDir | Out-Null

# Copy Executable
Copy-Item -Path $SourceExe -Destination $ReleaseDir

# Copy Assets
$AssetsDir = Join-Path $PSScriptRoot "assets"
Copy-Item -Path $AssetsDir -Destination $ReleaseDir -Recurse

# Copy DLLs
# We need to find where the DLLs are. They should be in the GTK bin directory.
# We can infer it from the setup script or assume it's in deps/gtk4/bin
$DepsGtkBin = Join-Path $PSScriptRoot "deps\gtk4\bin"

if (Test-Path $DepsGtkBin) {
    Write-Host "Copying DLLs from $DepsGtkBin..."
    # Copy all .dll files
    Get-ChildItem -Path $DepsGtkBin -Filter "*.dll" | Copy-Item -Destination $ReleaseDir
}
else {
    Write-Warning "Could not find local GTK4 bin directory at $DepsGtkBin."
    Write-Warning "You may need to manually copy DLLs if they are not in the system PATH."
}

# Create a run script for convenience (optional, but helpful if something needs setting)
$RunScript = "@echo off
start $ExeName"
Set-Content -Path (Join-Path $ReleaseDir "run.bat") -Value $RunScript

Write-Host "Release bundled at $ReleaseDir"
Write-Host "You can zip this folder and distribute it."
