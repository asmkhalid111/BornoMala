@echo off
title BornoMala Fcitx 5 Linux Interactive Tester (WSLg)
echo ========================================================
echo   Launching BornoMala Linux Interactive Tester...
echo   Target: Dedicated GTK3 Interface with live Bangla typing
echo ========================================================
wsl.exe -d Ubuntu -- python3 "/mnt/e/Codes/Bangla Input Platform/Borno V1/native/bornomala-fcitx5/scripts/interactive_tester.py"
