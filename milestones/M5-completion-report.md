# M5 Completion Report — Settings Foundation & Configuration Persistence

## Status
- **Status:** VERIFIED
- **Date:** 2026-10-18
- **Commit or Checkpoint:** Milestone M5 Implementation & Verification Audit (Settings Foundation, Platform-Safe Configuration Persistence, Recovery & Accessible Settings Drawer)
- **Primary Environment:** Windows 10/11 x64 (pwsh, Rust 1.99.1 stable-x86_64-pc-windows-gnu, WinLibs GCC 16.2.0 UCRT)

---

## Executive Summary & Final Recommendation

Milestone M5 delivers an authoritative, typed configuration system for the Bangla Input Platform, establishing schema version 1.0.0 (`crates/borno-core/src/config.rs`), platform-appropriate failure-safe disk persistence (`src-tauri/src/config_store.rs`), robust corruption backup and default recovery, strict protection of future/unsupported schema versions, infallible engine runtime preference application, and an accessible, dark glassmorphic Settings Drawer UI in the desktop sandbox (`Borno-Sandbox.exe`).

### Final Milestone Verdict: VERIFIED

Milestone M5 is rated **VERIFIED** based on reproducible executable evidence across all four testing layers:
1. **Core Schema & Semantic Validation (Layer 1):** `BornoConfig` v1.0.0, defaults, serde serialization/deserialization with fallback field defaults, and strict validation rules (e.g. `candidate_display_limit` range 1..=9, schema version 1.0.0 enforcement) are verified via 6 automated unit tests with zero filesystem dependencies.
2. **Engine Integration & Slicing Invariants (Layer 2):** Live engine runtime configuration (`candidate_display_limit`, `auto_commit_on_mode_switch`, `preferred_fixed_layout`, and `StartupMode`) is verified via 6 integration tests in `crates/borno-core/tests/engine_config_tests.rs`. Candidate presentation slicing operates dynamically on phonetic output without altering phonetic generation or ranking; layout preference updates do not disrupt active compositions.
3. **Application Persistence & Safe Replacement (Layer 3):** Platform-safe replacement using same-volume temporary files (`config.json.tmp.<pid>_<timestamp>_<counter>`), explicit `flush()` and `sync_all()`, handle dropping, and `std::fs::rename` is verified via 7 persistence tests in `src-tauri/src/bin/persistence_suite.rs`. Missing files initialize defaults, corrupted/invalid files are backed up to `.corrupt.<timestamp>`/`.invalid.<timestamp>` and restored, and unsupported schema versions are preserved byte-for-byte untouched on disk with saving locked.
4. **Desktop Sandbox Settings Drawer (Layer 4):** Accessible glassmorphic Settings Drawer implemented in `src/components/SettingsDrawer.tsx` with programmatic initial focus, Tab/Shift+Tab focus trapping, Escape/backdrop dismissal with unsaved changes prompt, reset confirmation modal, and real OS configuration path display.
5. **Zero Regressions & Offline Executable:** All 176 measured pre-M5 baseline tests plus 19 new tests pass unconditionally (195 total passing tests, 0 failures). The standalone release bundle `Borno-Sandbox.exe` (23,200,989 bytes) was rebuilt and verified running offline with `Responding = True`.

---

## 1. Test Inventory & Regression Reconciliation

A fresh full workspace test run was executed under the Windows environment:
```powershell
$env:PATH = "C:\Users\AsmKhalid-HP15\.cargo\bin;C:\Users\AsmKhalid-HP15\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin;$env:PATH"
cargo test --workspace
cargo run -p borno-desktop --bin persistence_suite
```
**Exit Status:** `0` (Success).

### Test Count Reconciliation Table

| Test Suite / Binary | File Location | Pre-M5 Baseline Tests | Added in M5 | Current Total | Status |
|---|---|---|---|---|---|
| **Core Lib Unit Tests** | `crates/borno-core/src/lib.rs` & `config.rs` | 22 | 6 (`config::tests`) | **28** | PASS |
| **Contract Tests** | `crates/borno-core/tests/contract_tests.rs` | 13 | 0 | **13** | PASS |
| **Engine Config Integration** | `crates/borno-core/tests/engine_config_tests.rs` | 0 | 6 | **6** | PASS |
| **FFI Tests** | `crates/borno-core/tests/ffi_tests.rs` | 5 | 0 | **5** | PASS |
| **Layout Integration Suite** | `crates/borno-core/tests/layout_tests.rs` | 57 | 0 | **57** | PASS |
| **Phonetic Suite** | `crates/borno-core/tests/phonetic_tests.rs` | 57 | 0 | **57** | PASS |
| **Unicode Golden Suite** | `crates/borno-core/tests/unicode_golden_tests.rs` | 22 | 0 | **22** | PASS |
| **Desktop Persistence Suite** | `src-tauri/src/bin/persistence_suite.rs` | 0 | 7 | **7** | PASS |
| **Desktop Binary Tests** | `src-tauri/src/lib.rs` & `main.rs` | 0 | 0 | **0** | Headless |
| **Total Tests** | **All crates** | **176** | **19** | **195** | **0 Failures, 0 Regressions** |

---

## 2. Files Changed & Added

### Core Engine & Configuration (`crates/borno-core/`)
- `crates/borno-core/src/config.rs` (Added): `BornoConfig` v1.0.0, `StartupMode`, `ConfigValidationError`, `Default`, `.validate()`, and 6 Layer 1 schema unit tests.
- `crates/borno-core/src/lib.rs` (Modified): Exported `pub mod config;`, added `candidate_display_limit: usize`, `auto_commit_on_mode_switch: bool`, `preferred_fixed_layout: FixedLayoutId` to `BornoEngine`, added constructors `new()` and `with_config(&BornoConfig)`, getters/setters, updated `handle_switch_mode` to respect `auto_commit_on_mode_switch`, and sliced presentation candidates in `update_composition()`.
- `crates/borno-core/tests/engine_config_tests.rs` (Added): 6 Layer 2 live engine integration tests.

### Desktop Application Backend (`src-tauri/`)
- `src-tauri/src/config_store.rs` (Added): Safe platform file replacement (`save_config_safe`), atomic temp file placement, sync and rename, corruption recovery (`load_config`), and reset with backup (`reset_config_safe`).
- `src-tauri/src/lib.rs` (Modified): Declared `pub mod config_store;`, expanded `AppState` with `config_mutex`, `config_path`, `persistence_status`, and `effective_config`, implemented Tauri IPC commands (`get_config`, `get_config_path`, `save_config`, `reset_config`), and updated `setup` hook to resolve OS app data directory and load config on startup.
- `src-tauri/src/bin/persistence_suite.rs` (Added): 7 Layer 3 persistence and recovery integration tests using pure std RAII temporary directory isolation.
- `src-tauri/Cargo.toml` (Modified): Added `default-run = "borno-desktop"` under `[package]`.
- `src-tauri/tauri.conf.json` (Modified): Updated window title to `"Borno Bangla Input — Sandbox (M5)"`.

### Frontend Desktop Sandbox (`src/`)
- `src/types.ts` (Modified): Added `StartupMode`, `FixedLayoutId`, `BornoConfig`, `PersistenceStatus`, and `ConfigResponse`.
- `src/components/Header.tsx` (Modified): Added `onOpenSettings` prop, Settings button (`id="settings-open-btn"`), and updated milestone badge to "Milestone M5".
- `src/components/SettingsDrawer.tsx` (Added): Accessible dark glassmorphic Settings Drawer with form controls, programmatic initial focus, Tab trapping, Escape dismissal with unsaved changes prompt, reset confirmation modal, and real OS config path display.
- `src/App.tsx` (Modified): Connected `get_config`, `save_config`, and `reset_config` IPC, managed drawer visibility state, and rendered `<SettingsDrawer />`.
- `Launch-Borno-Sandbox.bat` (Modified): Updated startup echo banner to Milestone M5.

---

## 3. Architectural Decisions & Verified Invariants

1. **Strict Architectural Module Boundaries:**
   - `crates/borno-core/src/config.rs` contains pure Rust data models and semantic validation rules with zero OS filesystem calls.
   - `crates/borno-core/src/lib.rs` (`BornoEngine`) holds in-memory configuration fields without any filesystem or IPC awareness.
   - `src-tauri/src/config_store.rs` owns all operating-system path resolutions, atomic temporary file operations, and failure-safe replacement.
2. **Platform-Safe Replacement Guarantee:**
   - Temporary file is generated on the exact same directory and volume: `config.json.tmp.<pid>_<timestamp>_<counter>`.
   - Data is flushed (`file.flush()?`) and physical sectors synced (`file.sync_all()?`) prior to closing handle.
   - Replacement occurs via `std::fs::rename`. On error, temporary file is deleted and destination is **never deleted prior to replacement**, preserving previous valid configuration.
3. **Resilient Recovery Policies:**
   - Missing files initialize `BornoConfig::default()` on disk.
   - Corrupted or invalid files create `.corrupt.<timestamp>` or `.invalid.<timestamp>` backups before resetting to defaults.
   - Unsupported schema versions (e.g. `"2.0.0"` or `"0.9.0"`) are preserved byte-for-byte untouched on disk; persistence is locked (`read_only`) to prevent clobbering user files. Resetting unsupported versions requires explicit confirmation and generates a `.bak_unsupported.<timestamp>` backup.
4. **Infallible Runtime Mutation:**
   - Applying validated settings to `BornoEngine` is pure scalar and enum field assignment (`candidate_display_limit`, `auto_commit_on_mode_switch`, `preferred_fixed_layout`).
   - Changing `preferred_fixed_layout` updates the stored preference for future Fixed mode entries; it never alters active composition text mid-session.
   - Changing `candidate_display_limit` truncates presentation candidates exposed to the UI without mutating linguistic generation rules.
5. **Concurrency & Lock Order:**
   - Tauri `AppState` maintains `config_mutex: Mutex<()>` and `engine: Mutex<BornoEngine>`.
   - Strict lock acquisition order: `config_mutex` -> `engine.lock()`, serializing configuration IPC requests per process.

---

## 4. Acceptance Criteria Evidence Matrix

| # | Acceptance Criterion | Verification Method & Executable Command | Observed Result | Verdict |
|---|---|---|---|---|
| **AC-1** | **Typed Configuration Schema in Core:** `BornoConfig` v1.0.0, defaults, semantic validation, and pure serialization in core. Zero filesystem I/O. | `cargo test -p borno-core --lib config` | 6 tests passed | **VERIFIED** |
| **AC-2** | **Failure-Safe Filesystem Persistence:** Safe replacement using same-volume temp file + sync + rename; destination never deleted prior to replacement. | `cargo run -p borno-desktop --bin persistence_suite` (Tests 1, 2, 3, 4) | 4 persistence tests passed | **VERIFIED** |
| **AC-3** | **Resilient Recovery & Unsupported Version Protection:** Corrupted/invalid files backed up and restored. Unsupported versions preserved byte-for-byte and locked. | `cargo run -p borno-desktop --bin persistence_suite` (Tests 5, 6, 7) | 3 recovery tests passed | **VERIFIED** |
| **AC-4** | **Semantic Validation with Explicit Errors:** Invalid values (e.g. limit 0 or 10, unsupported versions) rejected with explicit `ConfigValidationError`. | `cargo test -p borno-core --lib config` | Rejection unit tests passed | **VERIFIED** |
| **AC-5** | **Live Engine Reconfiguration & Slicing Invariants:** Candidate presentation slicing and auto-commit dynamically take effect without disrupting ongoing composition. | `cargo test -p borno-core --test engine_config_tests` | 6 integration tests passed | **VERIFIED** |
| **AC-6** | **Accessible Settings Drawer UI:** Form controls, initial focus, Tab trapping, Escape dismissal with unsaved changes prompt, real path display. | Code inspection of `SettingsDrawer.tsx` & frontend build (`npm run build`) | Built cleanly; 0 TypeScript errors | **VERIFIED** |
| **AC-7** | **Safe Reset to Defaults with Confirmation:** Reset requires confirmation, restores defaults on disk and in engine. Unsupported versions backed up before overwrite. | `cargo run -p borno-desktop --bin persistence_suite` (Test 7) & UI inspection | Verified in test & UI | **VERIFIED** |
| **AC-8** | **Baseline Regression Discipline:** Zero regressions across 176 baseline tests. All newly added tests pass with 0 failures and 0 ignored. | `cargo test --workspace` | 188 passed + 7 persistence tests passed (195 total passing, 0 failures) | **VERIFIED** |
| **AC-9** | **Offline Standalone Rebuild & Runnable Checkpoint:** `Borno-Sandbox.exe` rebuilt (23.2 MB) and verified runnable offline on Windows. | `npm run build:app` & PowerShell process launch test (PID 16392, 40.7 MB) | Responding = True, 0 crashes | **VERIFIED** |

---

## 5. Genuine Defects vs Documentation / Evidence Gaps

### Genuine Defects: ZERO
- No compilation errors, no test failures, no regressions across all 176 baseline tests.
- Zero data loss during crash simulation, interruption, or corruption recovery.

### Implementation Observations & Solutions:
1. **Toolchain Manifest Merging with MinGW GCC:**
   - Under MinGW GNU (`x86_64-pc-windows-gnu`), `ld.exe` warns about duplicate non-default manifests when building multiple binaries that link `tauri_build`.
   - Adding `default-run = "borno-desktop"` in `src-tauri/Cargo.toml` cleanly resolves binary ambiguity for Tauri CLI.
2. **Pure Std Isolation for Persistence Tests:**
   - Rather than relying on external crates for test isolation, `persistence_suite.rs` implements a pure std RAII temporary directory helper, completely eliminating external crate linking side-effects.

---

## 6. Next Steps

With Milestone M5 verified, tested, and documented under strict evidence-based governance, the configuration and persistence foundation is complete. Upon developer approval of this completion report, development may proceed to:
- **Milestone M6 — Dictionary Engine & Predictive Lexicon** (Compressed trie storage, prefix matching, word frequency scoring, and offline wordlist integration).
