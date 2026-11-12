@echo off
setlocal
title BornoMala Fcitx 5 Linux Interactive Tester (Isolated WSLg)
echo ========================================================
echo   Launching BornoMala Isolated Linux Interactive Tester...
echo   Target: Dedicated GTK3 Interface in Private Fcitx Session
echo ========================================================
set "REPO_DIR=%~dp0"
if "%REPO_DIR:~-1%"=="\" set "REPO_DIR=%REPO_DIR:~0,-1%"
wsl.exe -d Ubuntu --cd "%REPO_DIR%" -- python3 -B "native/bornomala-fcitx5/scripts/interactive_tester.py" %*
set "EXIT_CODE=%ERRORLEVEL%"
if not "%EXIT_CODE%"=="0" (
    echo [ERROR] Linux interactive tester exited with code %EXIT_CODE%.
)
exit /b %EXIT_CODE%
