# Vitray Widget - Windows Installer

$ErrorActionPreference = "Stop"

$RepoUrl = "https://github.com/zacxxx/vitray-widget/releases/latest/download/vitray-widget.exe"
$InstallDir = "$env:LOCALAPPDATA\VitrayWidget"
$ExePath = "$InstallDir\vitray-widget.exe"

Write-Host "🪟 Vitray Widget Installer" -ForegroundColor Cyan
Write-Host "=========================="
Write-Host ""

# Create installation directory
if (-not (Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
    Write-Host "Created directory: $InstallDir"
}

# Download executable (Placeholder URL)
# Write-Host "⬇️  Downloading vitray-widget..."
# Invoke-WebRequest -Uri $RepoUrl -OutFile $ExePath

# For development, we assume the user builds it or copies it.
# This script is a template for the final release.

Write-Host "⚠️  This is a template installer. In a real release, it would download the binary."
Write-Host "For now, please build with: cargo build --release"
Write-Host ""

# Create Shortcut
$WshShell = New-Object -comObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Vitray Widget.lnk")
$Shortcut.TargetPath = $ExePath
$Shortcut.Save()

Write-Host "✅ Shortcut created in Start Menu"
Write-Host ""
Write-Host "Installation (template) complete!"
