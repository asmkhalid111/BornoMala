@echo off
setlocal
title BornoMala Fcitx 5 Linux Gedit Session (Isolated WSLg)
echo ========================================================
echo   Launching BornoMala in Isolated GNOME Gedit Session...
echo   Tip: Press 'F4' or 'F12' to cycle modes.
echo ========================================================
set "REPO_DIR=%~dp0"
if "%REPO_DIR:~-1%"=="\" set "REPO_DIR=%REPO_DIR:~0,-1%"
wsl.exe -d Ubuntu --cd "%REPO_DIR%" -- bash "native/bornomala-fcitx5/scripts/launch_test_editor.sh" %*
set "EXIT_CODE=%ERRORLEVEL%"
if not "%EXIT_CODE%"=="0" (
    echo [ERROR] Linux Gedit test launcher exited with code %EXIT_CODE%.
)
exit /b %EXIT_CODE%
