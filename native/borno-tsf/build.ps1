# build.ps1 - Builds and tests the native BornoMala Windows TSF in-process text service
$ErrorActionPreference = "Stop"

$root = Resolve-Path "$PSScriptRoot\..\.."

# Ensure cargo and MinGW-w64 are in PATH
if ($env:USERPROFILE -and (Test-Path "$env:USERPROFILE\.cargo\bin")) {
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
}
$wingetMinGW = "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin"
if (Test-Path $wingetMinGW) {
    $env:PATH = "$wingetMinGW;$env:PATH"
}
$env:PATH = $env:PATH + ';' + [Environment]::GetEnvironmentVariable('PATH', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('PATH', 'User')

Write-Host "=== 1. Building BornoMala Core Static Library (Release) ===" -ForegroundColor Cyan
Push-Location $root
try {
    cargo build --release -p bornomala-core
    $targetDir = (cargo metadata --format-version 1 --no-deps | ConvertFrom-Json).target_directory
}
finally {
    Pop-Location
}

$staticLib = Join-Path $targetDir "release\libbornomala_core.a"
if (-not (Test-Path $staticLib)) {
    $staticLib = Join-Path $targetDir "debug\libbornomala_core.a"
}
if (-not (Test-Path $staticLib)) {
    $staticLib = Join-Path $targetDir "release\libborno_core.a"
}
if (-not (Test-Path $staticLib)) {
    throw "libbornomala_core.a was not found at $staticLib"
}
Write-Host "Using static library: $staticLib" -ForegroundColor Green

$gppCmd = Get-Command "g++.exe" -ErrorAction SilentlyContinue
if ($gppCmd) {
    $gpp = $gppCmd.Source
}
elseif (Test-Path "$wingetMinGW\g++.exe") {
    $gpp = "$wingetMinGW\g++.exe"
}
else {
    throw "g++.exe not found in PATH or standard MinGW-w64 locations. Please ensure MinGW-w64 GCC is installed."
}
Write-Host "Using C++ compiler: $gpp" -ForegroundColor Green

Write-Host "`n=== 2. Compiling Native TSF COM In-Process DLL (bornomala_tsf.dll) ===" -ForegroundColor Cyan
& $gpp -shared -std=c++17 -static -static-libgcc -static-libstdc++ `
    "$PSScriptRoot\borno_tsf.cpp" `
    "$PSScriptRoot\candidate_window.cpp" `
    "$PSScriptRoot\lang_bar_item.cpp" `
    "$PSScriptRoot\borno_tsf.def" `
    $staticLib `
    -o "$PSScriptRoot\bornomala_tsf.dll" `
    -lole32 -loleaut32 -luuid -ladvapi32 -luser32 -lgdi32 -luserenv -lws2_32 -lntdll -lbcrypt

# Copy to borno_tsf.dll for backward compatibility
try {
    Copy-Item "$PSScriptRoot\bornomala_tsf.dll" "$PSScriptRoot\borno_tsf.dll" -Force -ErrorAction Stop
}
catch {
    Write-Host "Note: borno_tsf.dll is locked by another process (IME currently in use). bornomala_tsf.dll updated." -ForegroundColor Yellow
}

Write-Host "`n=== 3. Compiling Standalone TSF Feasibility Test Harness (test_harness.exe) ===" -ForegroundColor Cyan
& $gpp -std=c++17 -static -static-libgcc -static-libstdc++ `
    "$PSScriptRoot\test_harness.cpp" `
    $staticLib `
    -o "$PSScriptRoot\test_harness.exe" `
    -lole32 -loleaut32 -luuid -ladvapi32 -luser32 -lgdi32 -luserenv -lws2_32 -lntdll -lbcrypt

Write-Host "`n=== 4. Executing TSF Feasibility Test Harness ===" -ForegroundColor Cyan
Push-Location $PSScriptRoot
try {
    .\test_harness.exe
}
finally {
    Pop-Location
}
