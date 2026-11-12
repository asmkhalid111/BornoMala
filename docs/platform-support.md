# Platform Support Matrix — BornoMala

> **Evidence Policy:** Status entries reflect directly observed build, automated test, and runtime evidence as of `2026-11-12` (baseline `249bcbc` + `M8-WP1`–`WP3` working-tree validation). Unverified environments are explicitly marked `Unverified` or `Deferred`.

| Platform / Configuration | Builds | Installs | Types in Real Apps | Core Features Tested | Status / Limitations |
|---|---|---|---|---|---|
| **Windows 11 x64 — Desktop Sandbox (`Borno-Sandbox.exe`)** | Verified (`WP-I`) | Portable Bundle | Sandbox Verified | Phonetic, Fixed (National/Probhat/Bijoy), Dictionary, Settings | Primary supported desktop sandbox (`FINAL-REMEDIATION-CLOSURE-REPORT.md`). |
| **Windows 11 x64 — Native TSF IME (`bornomala_tsf.dll`)** | Verified (`M9`) | Local/Dev Harness | Historical `M7` Verified; Post-M9 Live System COM Unre-tested | TSF edit session, GDI candidate scaling, hotkeys | System-wide COM registration requires elevation; post-remediation live COM retest excluded from closure scope. |
| **Linux — Ubuntu 26.04.1 LTS (WSL2/WSLg, Fcitx 5.1.19, XWayland `--disable waylandim`)** | Verified (`Release` CMake + PIC) | Verified in Isolated Temp `HOME` (`install.sh` / `uninstall.sh`) | Verified (`Gtk.Entry`, GNOME `gedit`) | Phonetic, cancel + idle `Escape` passthrough, WP-C digit, multi-candidate digit selection, `F12` National mode + dual-process mode/XDG config reload | **Partially Verified (WSLg XWayland only).** Live pointer clicking on candidate popup unverified (`POPUP_WINS: []`); `waylandim` disabled; Qt/GTK4 live typing unverified. See [`milestones/M8-revalidation-report.md`](../milestones/M8-revalidation-report.md). |
| **Linux — GitHub Actions `ubuntu-24.04` (Fcitx 5.1.x dev headers)** | CI Workflow Configured | Staged `DESTDIR` + Temp-`HOME` E2E Configured | Unavailable (Headless `exit 77`) | `265` core tests, `4/4` Release `CTest` + negative canary, `12/12` `test_scripts.py`, static privacy scan | Workflow authored in [`.github/workflows/ci.yml`](../.github/workflows/ci.yml); awaits push/PR for remote GitHub Actions execution. |
| **Linux — Native Desktop X11 / Native Wayland (`waylandim`) / Other Distros** | Unverified | Unverified | Unverified | Unknown | Not tested on physical/VM native Linux desktop outside WSL2/WSLg; native Wayland `waylandim` unverified. |
| **Linux — IBus** | Out of Scope | Out of Scope | Out of Scope | Out of Scope | M8 scopes Fcitx 5 only. |
| **macOS** | Unknown | Unknown | Unknown | Unknown | **Deferred / Unverified** (requires real macOS build and runtime environment). |

