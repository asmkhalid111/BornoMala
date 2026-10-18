# build.ps1 - Builds and tests the native Borno TSF in-process prototype
$ErrorActionPreference = "Stop"

$root = Resolve-Path "$PSScriptRoot\..\.."
$env:PATH = "C:\Users\AsmKhalid-HP15\.cargo\bin;C:\Users\AsmKhalid-HP15\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin;$env:PATH"

Write-Host "=== 1. Building BornoMala Core Static Library (Release) ===" -ForegroundColor Cyan
Push-Location $root
try {
    cargo build --release -p bornomala-core
    $targetDir = (cargo metadata --format-version 1 --no-deps | ConvertFrom-Json).target_directory
} finally {
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
} else {
    $gpp = "C:\Users\AsmKhalid-HP15\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin\g++.exe"
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
} catch {
    Write-Host "Note: borno_tsf.dll is locked by another process (IME in use). bornomala_tsf.dll updated." -ForegroundColor Yellow
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
} finally {
    Pop-Location
}
