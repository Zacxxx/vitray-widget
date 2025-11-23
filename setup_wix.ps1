# Setup WiX Toolset (Portable)
$ErrorActionPreference = "Stop"

$DepsDir = Join-Path $PSScriptRoot "deps"
if (-not (Test-Path $DepsDir)) {
    New-Item -ItemType Directory -Path $DepsDir | Out-Null
}

$WixDir = Join-Path $DepsDir "wix"
$WixZip = Join-Path $DepsDir "wix311-binaries.zip"
$WixUrl = "https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip"

if (-not (Test-Path $WixDir)) {
    Write-Host "Downloading WiX Toolset v3.11..."
    if (Test-Path $WixZip) { Remove-Item $WixZip }
    
    # Use curl for reliability
    curl.exe -L $WixUrl -o $WixZip
    
    Write-Host "Extracting WiX Toolset..."
    New-Item -ItemType Directory -Path $WixDir -Force | Out-Null
    
    # Expand-Archive is usually fine for standard zips, but tar is safer if available.
    # WiX binaries zip is standard.
    Expand-Archive -Path $WixZip -DestinationPath $WixDir -Force
    
    Remove-Item $WixZip
    Write-Host "WiX Toolset installed to $WixDir"
}
else {
    Write-Host "WiX Toolset already installed."
}

# Add to PATH for this session
$env:PATH = "$WixDir;$env:PATH"
Write-Host "WiX added to PATH for this session."
candle.exe -? 
