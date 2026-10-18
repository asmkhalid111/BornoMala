@echo off
setlocal
echo =======================================================
echo   Registering BornoMala Windows TSF Text Service
echo =======================================================

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Administrator privileges are required to register system-wide TSF text services.
    echo Please right-click this script and select 'Run as administrator'.
    pause
    exit /b 1
)

set DLL_PATH=%~dp0..\bornomala_tsf.dll
if not exist "%DLL_PATH%" (
    set DLL_PATH=%~dp0..\borno_tsf.dll
)
if not exist "%DLL_PATH%" (
    echo ERROR: bornomala_tsf.dll or borno_tsf.dll not found at %~dp0..
    pause
    exit /b 2
)

echo Registering %DLL_PATH%...
regsvr32.exe /s "%DLL_PATH%"
if %errorlevel% equ 0 (
    echo SUCCESS: BornoMala TSF Text Service registered successfully!
    echo Bengali (Bangladesh) profile [0x0845] and Bengali (India) profile [0x0445] enabled.
) else (
    echo ERROR: regsvr32 returned error code %errorlevel%
)
pause
