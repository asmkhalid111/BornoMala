# BornoMala Windows Text Services Framework (TSF) Module

This directory contains the native Windows Text Services Framework (TSF) in-process COM DLL for BornoMala (`borno-tsf` / `bornomala_tsf.dll`), enabling system-wide Bengali input on Windows applications alongside the shared Rust core engine.

> **Status & Scope Notice:** `build.ps1` and `scripts/*.bat` are **developer build and manual COM registration scripts**, **not** a production installer. No MSI, WiX, Inno Setup, NSIS, or automated ZIP packager currently exists in this repository for standalone TSF binary distribution, and `bornomala_tsf.dll` is not bundled into the `bornomala-desktop` Tauri application installer.

---

## 1. Architecture & Shared Core Relationship

- **In-Process COM / TSF Text Service (`borno_tsf.cpp`, `borno_tsf.def`):**
  Implements `ITfTextInputProcessorEx` / `ITfTextInputProcessor`, `ITfKeyEventSink`, `ITfCompositionSink`, and `ITfThreadMgrEventSink` under COM CLSID:
  - **CLSID:** `{8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}` (`"Borno Bangla Text Service"`)
  - **Profiles:** Bengali (Bangladesh) (`0x0845`) and Bengali (India) (`0x0445`)
  - **UI & Session Helpers:** `candidate_window.cpp`, `lang_bar_item.cpp`, `edit_session.h`, `key_mapper.h`
  - **Exported COM Entry Points (`borno_tsf.def`):** `DllGetClassObject`, `DllCanUnloadNow`, `DllRegisterServer`, `DllUnregisterServer`
- **Shared Rust Core (`bornomala-core`):**
  `bornomala_tsf.dll` statically links `bornomala-core` (`crates/bornomala-core`, `crate-type = ["rlib", "staticlib"]`) and calls its C ABI (`crates/bornomala-core/include/bornomala.h`). The compiled Bengali lexicon trie (`crates/bornomala-core/data/lexicon.bin`, 40,001 prefix-trie entries) is embedded inside `bornomala-core`.
  Unlike `bornomala-desktop`, `bornomala-core` does **not** depend on `option-ext` (its direct dependencies in `crates/bornomala-core/Cargo.toml` are `serde`, `serde_json`, `unicode-normalization`, and `unicode-segmentation`).

---

## 2. Building & Verification Harness (`build.ps1`)

Run `native/borno-tsf/build.ps1` using MinGW-w64 GCC/G++ and Rust/Cargo to produce the native TSF binaries:

1. Builds `bornomala-core` as a static library.
2. Compiles and links the C++17 TSF DLL with MinGW-w64 G++ (`-shared -std=c++17 -static -static-libgcc -static-libstdc++`), producing **`bornomala_tsf.dll`** (primary output).
3. Copies `bornomala_tsf.dll` to **`borno_tsf.dll`** as a legacy compatibility copy.
4. Compiles **`test_harness.exe`** (`test_harness.cpp`) and executes `.\test_harness.exe` automatically.

### CRITICAL WARNING: `build.ps1` and `test_harness.exe` Modify/Unregister COM & TSF State

Do **not** treat `build.ps1` or `test_harness.exe` as registry-read-only:

- `build.ps1` automatically runs `.\test_harness.exe` at the end of every build.
- At Level 4 (`test_harness.cpp`), `test_harness.exe` explicitly invokes **`DllRegisterServer()`** against CLSID `{8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}` (attempting `HKEY_LOCAL_MACHINE\Software\Classes\CLSID\{8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}` first and falling back to `HKEY_CURRENT_USER` if HKLM fails) and then **unconditionally calls `DllUnregisterServer()`** to clean up.
- Running `build.ps1` or `test_harness.exe` **elevated (as Administrator)** will register and immediately delete the HKLM/HKCU COM registration and TSF profiles for `{8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}`—**wiping out any previously registered live Borno TSF installation on that machine**.
- Running `build.ps1` or `test_harness.exe` **non-elevated** fails HKLM TSF profile registration (`ITfInputProcessorProfiles::Register`) and invokes `DllUnregisterServer()` rollback/cleanup against both HKLM and HKCU CLSID keys.
- If you previously registered `bornomala_tsf.dll` for live Windows typing and subsequently run `build.ps1`, you must re-register afterward using `scripts\register.bat` from an elevated shell.

---

## 3. Manual Registration & Unregistration (`scripts/`)

Manual COM and TSF profile registration uses the batch scripts in `native/borno-tsf/scripts/`:

- **Register (`scripts/register.bat`):**
  1. Open an **Administrator** Command Prompt or PowerShell window (`register.bat` requires elevation checked via `net session`).
  2. Run `scripts\register.bat` (or change into `scripts\` and run `register.bat`).
  3. `register.bat` keeps relative script resolution: it looks for `..\bornomala_tsf.dll` first and falls back to `..\borno_tsf.dll`, invoking `regsvr32.exe /s`.
- **Unregister (`scripts/unregister.bat` — KNOWN SYNTAX DEFECT / UNVERIFIED):**
  `scripts/unregister.bat` is intended to run `regsvr32` unregistration on `..\bornomala_tsf.dll` and `..\borno_tsf.dll` when executed elevated. **However, `native/borno-tsf/scripts/unregister.bat` currently contains an unverified `) else (` syntax block without a matching opening `if (` block (`lines 27-30`), so unregistration via `unregister.bat` is NOT verified in the current repository snapshot.** Treat `unregister.bat` as broken/unverified until fixed separately; manual cleanup, if ever required in an elevated test environment, would invoke `regsvr32.exe /u /s <path-to-dll>` directly.

---

## 4. Mandatory Standalone Distribution & Licensing Rule

The repository root [`THIRD-PARTY-NOTICES.md`](../../THIRD-PARTY-NOTICES.md) is the **single authoritative third-party notices and license disclosure file**. Do **not** invent a separate trimmed or rewritten TSF-only license file.

Whenever `bornomala_tsf.dll` (and/or `borno_tsf.dll`) is packaged, archived, or distributed outside this source tree (for example in manual ZIP archives or future Windows installers), you **MUST**:

1. Copy the repository root [`THIRD-PARTY-NOTICES.md`](../../THIRD-PARTY-NOTICES.md) unmodified **directly beside `bornomala_tsf.dll`**.
2. Preserve the `scripts/` subdirectory relative to the DLL so `register.bat` / `unregister.bat` resolve `..\bornomala_tsf.dll` cleanly:

```text
<tsf-distribution-root>/
├── bornomala_tsf.dll          # Primary TSF COM DLL (optionally also borno_tsf.dll)
├── THIRD-PARTY-NOTICES.md     # Exact unmodified copy of root THIRD-PARTY-NOTICES.md
└── scripts/
    ├── register.bat
    └── unregister.bat
```

### Licensing Summary & Open Limitations (`bornomala_tsf.dll`)

Refer to root [`THIRD-PARTY-NOTICES.md`](../../THIRD-PARTY-NOTICES.md) for complete texts, disclaimers, and upstream repository links governing components compiled into `bornomala_tsf.dll`:

- **Linguistic Lexicon (`crates/bornomala-core/data/lexicon.bin`):** Derived from the Leipzig Corpora Collection (`ben_news_2020_300K`) under **Creative Commons Attribution 4.0 International (CC BY 4.0)**, including required attribution and CC BY 4.0 §5 warranty disclaimer.
- **Rust Core Dependencies (`bornomala-core`):** Locked Cargo workspace dependencies (`serde`, `serde_json`, `unicode-segmentation` under `MIT OR Apache-2.0`, MIT elected; `unicode-normalization` under `(MIT OR Apache-2.0) AND Unicode-3.0`, MIT elected and Unicode-3.0 License Agreement reproduced; transitive permissive crates resolved in `Cargo.lock`). Contains no Hunspell or GPL/LGPL dependencies.
- **GCC Runtime Library Exception (Version 3.1):** `build.ps1` statically links `libgcc` and `libstdc++` (`-static -static-libgcc -static-libstdc++`) under the GCC Runtime Library Exception (Version 3.1).
- **MinGW-w64 C Runtime (`libmingw32.a`, `libmingwex.a`) & Open `RV-11` Limitation:** Upstream MinGW-w64 manifests document Public Domain (`_mingw.h` / `DISCLAIMER.PD`), Zope Public License Version 2.1 (`COPYING.MinGW-w64`), and BSD-style permissive terms. **Known limitation (`RV-11`):** exhaustive object-by-object license and attribution verification of the statically linked `libmingwex.a` archive has **not** yet been completed; root `THIRD-PARTY-NOTICES.md` reflects established upstream manifests and does not constitute a completed object-level archive audit.

