# Build Windows Installer (MSI)
$ErrorActionPreference = "Stop"

# 1. Setup Environment
$Root = $PSScriptRoot
$DepsDir = Join-Path $Root "deps"
$WixDir = Join-Path $DepsDir "wix"

# Ensure WiX is in PATH
if (Test-Path $WixDir) {
    $env:PATH = "$WixDir;$env:PATH"
    Write-Host "Added WiX to PATH: $WixDir"
}
else {
    Write-Warning "WiX Toolset not found in $WixDir. Assuming it's in system PATH."
}

# 1.1 Setup Dev Env (PKG_CONFIG_PATH)
. "$Root\setup_dev.ps1"

# 2. Prepare DLLs list for WiX
# We need to inject <File> elements for all DLLs in deps/gtk4/bin into main.wxs
# This is a bit hacky but effective for automation without manual XML editing every time.

$WxsFile = Join-Path $Root "wix\main.wxs"
$GtkBin = Join-Path $DepsDir "gtk4\bin"

if (-not (Test-Path $GtkBin)) {
    Write-Error "GTK4 bin directory not found at $GtkBin. Run setup_dev.ps1 first."
    exit 1
}

Write-Host "Generating DLL list from $GtkBin..."
$Dlls = Get-ChildItem -Path $GtkBin -Filter "*.dll"
$DllXml = ""
foreach ($Dll in $Dlls) {
    $Id = "dll_" + $Dll.Name.Replace(".", "_").Replace("-", "_")
    $DllXml += "                <File Id='$Id' Name='$($Dll.Name)' Source='$($Dll.FullName)' KeyPath='no' />`n"
}

# 3. Patch main.wxs
# We look for a marker or a specific component to insert our DLLs.
# cargo-wix generates a 'binary0' component. We can append to it or add a new component.
# Let's read the file and insert our DLLs into the main Component.

$Content = Get-Content $WxsFile -Raw

# Check if we already patched it (simple check)
if ($Content -match "dll_gtk_4_1_dll") {
    Write-Host "main.wxs seems to be already patched with DLLs. Skipping patch."
}
else {
    Write-Host "Patching main.wxs with DLLs..."
    # Regex to find the closing tag of the main component which usually contains the exe
    # cargo-wix default: <Component Id="binary0" Guid="..."> <File Id="exe0" ... /> </Component>
    # We want to insert before </Component>
    
    # Use (?s) for single-line mode (dot matches newline) to handle multi-line File tags
    $Pattern = "(?s)(<File Id='exe0'.*?/>)"
    if ($Content -match $Pattern) {
        $Replacement = "$1`n$DllXml"
        $NewContent = $Content -replace $Pattern, $Replacement
        Set-Content -Path $WxsFile -Value $NewContent
        Write-Host "main.wxs patched successfully."
    }
    else {
        Write-Warning "Could not find insertion point in main.wxs. DLLs might not be bundled."
    }
}

# 4. Build MSI
Write-Host "Building MSI..."
# Add --nocapture to see rustc output if it fails
cargo wix -v --nocapture

# 5. Verify
$TargetWix = Join-Path $Root "target\wix"
$Msi = Get-ChildItem -Path $TargetWix -Filter "*.msi" | Select-Object -First 1

if ($Msi) {
    Write-Host "=========================================="
    Write-Host "Installer created successfully!"
    Write-Host "Location: $($Msi.FullName)"
    Write-Host "=========================================="
}
else {
    Write-Error "MSI creation failed."
}
