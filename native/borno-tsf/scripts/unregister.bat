@echo off
setlocal
echo =======================================================
echo   Unregistering BornoMala Windows TSF Text Service
echo =======================================================

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Administrator privileges are required to unregister system-wide TSF text services.
    echo Please right-click this script and select 'Run as administrator'.
    pause
    exit /b 1
)

set DLL_PATH=%~dp0..\bornomala_tsf.dll
if exist "%DLL_PATH%" (
    echo Unregistering %DLL_PATH%...
    regsvr32.exe /u /s "%DLL_PATH%"
)

set OLD_DLL_PATH=%~dp0..\borno_tsf.dll
if exist "%OLD_DLL_PATH%" (
    echo Unregistering %OLD_DLL_PATH%...
    regsvr32.exe /u /s "%OLD_DLL_PATH%"
)

echo SUCCESS: BornoMala TSF Text Service unregistered successfully!
) else (
    echo ERROR: regsvr32 returned error code %errorlevel%
)
pause
