$ErrorActionPreference = "Continue"
$Root = $PSScriptRoot
$DepsDir = Join-Path $Root "deps"
$PkgConfigLite = Join-Path $DepsDir "pkg-config-lite\bin"
$GtkBin = Join-Path $DepsDir "gtk4\bin"
$GtkPkgConfig = Join-Path $DepsDir "gtk4\lib\pkgconfig"

$env:PATH = "$PkgConfigLite;$GtkBin;$env:PATH"
$env:PKG_CONFIG_PATH = $GtkPkgConfig

cargo check --release > build_log_full.txt 2>&1
