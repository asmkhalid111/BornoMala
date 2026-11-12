# Final Remediation Closure & Artifact Reconciliation Report

> **Project:** BornoMala (বর্ণমালা) — Bangla Input Platform  
> **Document Role:** Master Closure Report for Post-M9 Remediation Work Packages (`WP-05` through `WP-I` + Cursor Audit)  
> **Closure Date:** 2026-11-12  
> **Git Baseline:** `master` at commit `223b7a39a6927765e3509742bdad988d862e4a7b` (1 commit ahead of `origin/master`, local changes uncommitted)  
> **Primary Environment:** Windows 11 (x64)

---

## 1. Executive Closure Summary

This report formally closes the BornoMala remediation cycle spanning **WP-05**, **WP-06**, the **WP-A / WP-B / WP-C** post-QA behavioral fixes, **WP-D through WP-G** Windows GNU application-manifest reconciliation, **UI Text Selectability Correction**, and the **WP-H / WP-I** artifact verification and canonical executable reconciliation.

All authorized remediation changes have been consolidated into the verified WP-I Windows GNU release build and reconciled to the project's canonical portable desktop launcher target:

- **Canonical Desktop Executable:** `Borno-Sandbox.exe` (repository root, replacing the obsolete `com.bornomala.sandbox` build `C26EF3D52839CB0541036AD14E4914BB0A33A38DC55200888E841801A6E8626E`)
- **Co-Located Dependency:** `WebView2Loader.dll` (repository root, SHA-256 `86545B66CDB0603BC26B626FB9AD610CB6E71F28D468F5EA66DF23B03DDA96D5`)
- **Canonical Binary SHA-256 (WP-I Build):** `52A739E54AFA028BD6A34A136BC393BA6455666816269913169BBC54837C8F45` (`24,805,742` bytes)

Evidence across automated builds/tests, static PE inspection, and user-reported manual QA is recorded below with strict separation between verified facts, user-reported manual checks, and unverified platform/deferred items.

---

## 2. Work Package Closure Status (`WP-05` through `WP-I`)

| Work Package | Scope | Status | Verified Build / Test Evidence | Manual QA / Closure Distinctions |
|---|---|---|---|---|
| **WP-05** | Desktop application identifier migration (`com.bornomala.desktop`), persistence unification, symlink/junction guards, SemVer schema handling | **Closed (Committed Locally)** | Committed at `223b7a39a6927765e3509742bdad988d862e4a7b` on local `master`. Automated `persistence_suite` (`78/78`) and `bornomala-core` suite passed at commit. | Local commit is **not pushed** to `origin/master`. Live `%APPDATA%` / TSF harness host tests remain gated outside automated CI. |
| **WP-06** | Licensing & attribution compliance (`THIRD-PARTY-NOTICES.md`, `docs/resources-and-licenses.md`, Tauri IPC + About tab, TSF/Fcitx packaging rules, `licensing_tests.rs`) | **Closed (w/ Documented Open Risk)** | All 5 implementation tasks completed in working tree. `crates/bornomala-core/tests/licensing_tests.rs` (`7/7` tests) and `test_get_third_party_notices_returns_embedded_root_notice` passed. | User manually verified About & Notices visibility, persistence across Settings tab switching, and text select/copy behavior. Linux CMake/installer checks are structural/configuration-level only (no live Fcitx runtime install tested). Independent `RV-11` MinGW `libmingwex.a` object audit remains an open release-engineering risk; project is **not** claimed to be legally certified across all possible distribution obligations. |
| **WP-A** | Settings Drawer draft lifecycle (`toFormData` reset on open, dirty tracking, Discard & Close, Save & Exit) | **Closed (Verified)** | Implemented in `src/components/SettingsDrawer.tsx`; compiles under TypeScript/Vite (`npm run build`). | User manually verified the Settings draft lifecycle, discard reset behavior, and Save & Exit flow. |
| **WP-B** | Phonetic ellipsis (`...` $\to$ literal `...`) and single-period auto-danda alignment | **Closed (Verified)** | Implemented in `crates/bornomala-core/src/phonetic/tokenizer.rs`, `src/lib.rs`, `milestones/M9.md`, and `tests/phonetic_tests.rs` (`cargo test -p bornomala-core` `265/265` passing). | User manually verified Phonetic punctuation behavior. |
| **WP-C** | Single-candidate numeric input collision fix (`candidates.len() > 1` guard) across React, Windows TSF, and Linux Fcitx 5 | **Closed (w/ Bounded Manual Scope)** | Updated `src/components/SandboxTypingArea.tsx`, `native/borno-tsf/borno_tsf.cpp`, `native/bornomala-fcitx5/bornomala.cpp`, and `tests/phonetic_tests.rs`. Rust automated tests and compile checks passed. | User manually verified numeric typing sequences in the Windows desktop sandbox. Multi-candidate numeric selection was **not** separately manually tested; live TSF COM and Fcitx 5 runtime numeric typing were not re-executed here. |
| **WP-D** | Consolidated regression build (`target/release/bornomala-desktop.exe`, SHA-256 `E77587F7...`) | **Superseded Reference Artifact** | Release compilation and test baseline succeeded, but surfaced `ld.exe: .rsrc merge failure: multiple non-default manifests`. | Preserved as historical reference only; superseded by `WP-G` and `WP-I` builds. |
| **WP-E** | Read-only forensic investigation of Windows PE duplicate manifest warning | **Closed (Diagnostic Only)** | Confirmed collision between Tauri (`tauri-winres`) `RT_MANIFEST` and MinGW implicit `default-manifest.o`. | Diagnostic evidence only; no repository changes. |
| **WP-F** | Isolated external feasibility experiment for `-specs=` manifest reconciliation | **Closed (Experimental Prototype Only)** | Verified in `%TEMP%` that omitting `<?xml ...?>` + overriding GCC `*endfile:` yields a single valid manifest without warnings. | Isolated prototype outside repository; not production code. |
| **WP-G** | Production manifest reconciliation in `src-tauri/build.rs` | **Closed (Build & PE Verified)** | `src-tauri/build.rs` unit harness (`12/12`), `cargo check --workspace`, `cargo test -p bornomala-desktop --lib` (`1/1`), `cargo test -p bornomala-core` (`265/265`), and external release build (`0` warnings/errors, single valid `RT_MANIFEST`) passed. | Establishes build-time PE/manifest integrity only; did **not** perform runtime execution or claim runtime activation on its own. |
| **WP-H** | Read-only verification and manual QA handoff for `WP-G` build | **Closed (Handoff Completed)** | Verified external `WP-G` binary hashes (`9502188E...`) and AMD64 PE32+ headers cleanly. | Handoff artifact audit only; did not execute the desktop runtime. |
| **UI Selectability** | Lift global `body { user-select: none; }` (`RV-08` `U-A11Y-1` / `WP-8.6` item 4) while protecting interactive controls | **Closed (Implemented & Bundled in WP-I)** | Updated `src/index.css` and Shift/AltGr badges (`className="no-select"`) in `src/components/VirtualKeyboard.tsx`; `npm run build` produced `index-B2tzsf9h.css` without `user-select:none` on `body`. | Bundled and validated manually under WP-I below. |
| **WP-I** | Fresh isolated release build incorporating UI selectability + canonical `Borno-Sandbox.exe` reconciliation | **Closed (PE Verified & User Sandbox QA Passed)** | Isolated build (`%LOCALAPPDATA%\Temp\bornomala-selectability-qa-target\release\bornomala-desktop.exe`, `24,805,742` bytes, SHA-256 `52A739E54AFA028BD6A34A136BC393BA6455666816269913169BBC54837C8F45`) completed with `0` linker/compiler warnings; PE analysis confirmed AMD64 PE32+ GUI, `1` valid `RT_MANIFEST` (`(24, 1, 0x0409)` Common Controls v6 + `asInvoker` + `longPathAware=true`), `6` `RT_ICON`, `1` `RT_GROUP_ICON`, `1` `RT_VERSION`, `com.bornomala.desktop`, embedded `THIRD-PARTY-NOTICES`, `BLX1`, and `index-B2tzsf9h.css`/`index-y2E1tHcU.js`. Reconciled to root `Borno-Sandbox.exe` (`52A739E5...`). | **Runtime QA (User-Reported):** User manually verified text selection/copying, committed Bangla output copying, and About & Notices copying, with no other QA issues reported; Active Preedit & in-composition cursor remain intentionally non-selectable. **Distinction:** PE/binary artifact verification is separate from user runtime QA. |

---

## 3. Canonical Executable Reconciliation Record

- **Previous Root Canonical Binary (Backed Up Externally):**
  - Original path: `E:\Codes\Bangla Input Platform\Borno V1\Borno-Sandbox.exe` (`24,601,680` bytes, dated `2026-10-20`, embedded legacy identifier `com.bornomala.sandbox`)
  - Original SHA-256: `C26EF3D52839CB0541036AD14E4914BB0A33A38DC55200888E841801A6E8626E`
  - Backup location: `%TEMP%\bornomala-backup-Borno-Sandbox-2026-11-12\Borno-Sandbox.exe` (and co-located `WebView2Loader.dll`, `86545B66CDB0603BC26B626FB9AD610CB6E71F28D468F5EA66DF23B03DDA96D5`)
- **Reconciled Canonical Desktop Binary:**
  - Source artifact: `%LOCALAPPDATA%\Temp\bornomala-selectability-qa-target\release\bornomala-desktop.exe`
  - Destination path: `Borno-Sandbox.exe` (repository root, unchanged binary contents, filename aligned with `Launch-Borno-Sandbox.bat`, `Launch-BornoMala-Sandbox.bat`, and `README.md`)
  - Verified destination SHA-256: `52A739E54AFA028BD6A34A136BC393BA6455666816269913169BBC54837C8F45`
  - Co-located `WebView2Loader.dll` SHA-256: `86545B66CDB0603BC26B626FB9AD610CB6E71F28D468F5EA66DF23B03DDA96D5`
- **Launch Workflow Alignment:** Both `Launch-Borno-Sandbox.bat` and `Launch-BornoMala-Sandbox.bat` execute `start "" "%~dp0Borno-Sandbox.exe"`, resolving directly to the updated WP-I executable.

---

## 4. Cursor Synchronization Audit Disposition (Deferred)

- **Finding:** In `src/components/SandboxTypingArea.tsx`, browser caret changes (`onClick`, `onSelect`, `Home`/`End`) inside `<input id="sandbox-keystroke-input">` do not synchronize back to `BornoEngine::source_cursor` / Active Preedit `cursor_pos`, and React does not call `setSelectionRange` on `<input>` after engine `MoveCursorLeft`/`MoveCursorRight`/`SetCursor` updates (`EngineState` currently omits `source_cursor`, while `WP-03` restricts external ingress to target `SetCursor(pos)`).
- **Boundary Assessment:** This is strictly a **frontend desktop sandbox integration limitation** (and sandbox-to-core event contract scope), **not** a regression or defect in the shared engine’s internal cursor arithmetic or the native Windows TSF / Linux Fcitx 5 adapters.
- **Disposition:** Formally deferred to coordinate cleanly with [`RV-08.1`](../revisions/RV-08.1-ime-and-focus.md) (`IME Composition, Paste & Focus Retention`). It does not block closure of `WP-05`–`WP-I`.

---

## 5. Platform Verification Boundaries & Remaining Open Risks

1. **Windows x64:** Desktop sandbox build, PE manifest/resource checks, `bornomala-core` tests, `src-tauri/build.rs` unit tests, and user-reported desktop sandbox manual QA are verified. Live system-wide Windows TSF COM host registration/testing (`native/borno-tsf/`) was intentionally excluded from this remediation closure pass.
2. **Linux x64:** Fcitx 5 source edits (`bornomala.cpp`) and CMake/script notice-install rules (`CMakeLists.txt`, `scripts/install.sh`) were inspected structurally, alongside pre-existing `WP-01` WSL2 evidence; **no live Linux desktop / Fcitx 5 runtime test** of `WP-06` or `WP-C` was performed.
3. **macOS:** Untested and unverified across all milestones and remediation packages.
4. **Open Release-Engineering Risk (`RV-11`):** Independent object-level auditing of MinGW `libmingwex.a` runtime licensing obligations remains deferred to `RV-11`.
5. **Pre-existing Path-with-Spaces Constraint:** Running `cargo` directly into workspace `target/` under `E:\Codes\Bangla Input Platform\Borno V1\` hits GNU `windres` failure on spaces in `OUT_DIR` inside `tauri-winres`; spaceless external `CARGO_TARGET_DIR` (or short 8.3 path) is required for Windows GNU builds of `bornomala-desktop`.

