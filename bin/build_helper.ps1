# Build the specflow-helper binary used by the Vim plugin (Windows).
#
# Run once after cloning the plugin (and after pulling updates to the Rust
# crate). The resulting binary lands at <plugin>/bin/specflow-helper.exe,
# which is where the VimScript shim looks for it by default.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File .\bin\build_helper.ps1

$ErrorActionPreference = 'Stop'

$pluginDir = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$crateDir  = Join-Path $pluginDir 'rust\specflow-helper'
$outBin    = Join-Path $pluginDir 'bin\specflow-helper.exe'

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "specflow-helper: cargo not found in PATH. Install Rust from https://rustup.rs"
    exit 1
}

Write-Host "specflow-helper: building (cargo build --release)..."
Push-Location $crateDir
try {
    & cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed (exit $LASTEXITCODE)"
    }
} finally {
    Pop-Location
}

$built = Join-Path $crateDir 'target\release\specflow-helper.exe'
Copy-Item -Force $built $outBin

Write-Host "specflow-helper: installed at $outBin"
& $outBin --version
