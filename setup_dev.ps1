# Setup Development Environment for Vitray Widget on Windows
$ErrorActionPreference = "Stop"

$DepsDir = Join-Path $PSScriptRoot "deps"
if (-not (Test-Path $DepsDir)) {
    New-Item -ItemType Directory -Path $DepsDir | Out-Null
}

# 1. Install pkg-config-lite
$PkgConfigUrl = "https://sourceforge.net/projects/pkgconfiglite/files/0.28-1/pkg-config-lite-0.28-1_bin-win32.zip/download"
$PkgConfigZip = Join-Path $DepsDir "pkg-config-lite.zip"
$PkgConfigDir = Join-Path $DepsDir "pkg-config-lite"

if (-not (Test-Path $PkgConfigDir)) {
    Write-Host "Downloading pkg-config-lite..."
    if (Test-Path $PkgConfigZip) { Remove-Item $PkgConfigZip }
    
    # Use curl if available for better redirect handling
    curl.exe -L $PkgConfigUrl -o $PkgConfigZip
    
    Write-Host "Extracting pkg-config-lite..."
    tar -xf $PkgConfigZip -C $DepsDir
    
    # Rename the extracted folder to a fixed name if needed, but it usually extracts with version
    # Let's find the extracted folder
    $Extracted = Get-ChildItem -Path $DepsDir -Filter "pkg-config-lite-*" | Where-Object { $_.PSIsContainer } | Select-Object -First 1
    if ($Extracted) {
        Rename-Item -Path $Extracted.FullName -NewName "pkg-config-lite"
    }
    if (Test-Path $PkgConfigZip) { Remove-Item $PkgConfigZip }
}
else {
    Write-Host "pkg-config-lite already installed."
}

# 2. Install GTK4 Runtime (from wingtk/gvsbuild)
# Fetch latest release URL via GitHub API to be robust
$GtkRepo = "wingtk/gvsbuild"
$GtkZip = Join-Path $DepsDir "gtk4-runtime.zip"
$GtkDir = Join-Path $DepsDir "gtk4"

if (-not (Test-Path $GtkDir)) {
    Write-Host "Downloading GTK4 runtime..."
    $GtkUrl = "https://github.com/wingtk/gvsbuild/releases/download/2025.11.0/GTK4_Gvsbuild_2025.11.0_x64.zip"
    
    if (Test-Path $GtkZip) { Remove-Item $GtkZip }
    
    curl.exe -L $GtkUrl -o $GtkZip
    
    Write-Host "Extracting GTK4 runtime..."
    New-Item -ItemType Directory -Path $GtkDir -Force | Out-Null
    tar -xf $GtkZip -C $GtkDir
    if (Test-Path $GtkZip) { Remove-Item $GtkZip }
}
else {
    Write-Host "GTK4 runtime already installed."
}

# 3. Configure Environment Variables for the current session
$PkgConfigBin = Join-Path $PkgConfigDir "bin"
$GtkBin = Join-Path $GtkDir "bin"
$GtkLib = Join-Path $GtkDir "lib"
$GtkInclude = Join-Path $GtkDir "include"

# Update PATH
$env:PATH = "$PkgConfigBin;$GtkBin;$env:PATH"

# Set PKG_CONFIG_PATH
# pkg-config needs to find .pc files. They are in lib/pkgconfig
$PkgConfigPath = Join-Path $GtkLib "pkgconfig"
$env:PKG_CONFIG_PATH = $PkgConfigPath

Write-Host "Environment configured for this session."
Write-Host "PATH added: $PkgConfigBin"
Write-Host "PATH added: $GtkBin"
Write-Host "PKG_CONFIG_PATH set to: $PkgConfigPath"

# Verify
Write-Host "Verifying pkg-config..."
pkg-config --version
Write-Host "Verifying gtk4..."
pkg-config --modversion gtk4

Write-Host "Setup complete! You can now run 'cargo build' in this terminal."
