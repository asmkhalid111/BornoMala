# BornoMala (বর্ণমালা) — Bangla Input Platform

[![Rust](https://img.shields.io/badge/Rust-1.99.1%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20TSF-0078d7.svg)](https://docs.microsoft.com/en-us/windows/win32/tsf/text-services-framework)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24c8db.svg)](https://tauri.app/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Tests](https://img.shields.io/badge/Tests-229%20Passing-brightgreen.svg)](#testing)

**BornoMala (বর্ণমালা)** is a fast, modular, offline-first, and privacy-conscious Bangla input platform for desktop operating systems.

It unifies multiple Bangla typing layouts — **Phonetic transliteration**, **National (Jatiyo)**, **Probhat**, and **Bijoy** — behind a shared, high-performance Rust core engine. BornoMala ships both as a **native Windows Text Services Framework (TSF) in-process IME** for system-wide typing in any Windows application, and as an interactive **desktop sandbox & configuration application**.

---

## Key Features

- **⚡ Blazing Fast Rust Engine (`bornomala-core`):**
  - Sub-millisecond keystroke processing latency (< 0.25 ms average).
  - Canonical Unicode NFC normalization, vowel sign collapsing, and conjunct joiner preservation (ZWJ/ZWNJ).
  - Trie-based predictive completions and Levenshtein typo-correction for Phonetic typing.
  - 100% safe Rust core with a clean C-ABI (`bornomala.h`).

- **🪟 Native Windows TSF In-Process IME (`bornomala_tsf.dll`):**
  - Full Text Services Framework (TSF) COM in-process text service.
  - Works natively across Windows Notepad, Sublime Text, Office, Visual Studio Code, browsers, and terminal emulators.
  - Real-time preedit inline composition with accurate caret tracking.
  - Floating GDI candidate window with mouse and keyboard navigation (1–9 digits, arrows, space/enter commit).
  - Language bar integration and instant mode toggling (`F12`).

- **🎨 Desktop Sandbox & Configuration App (`bornomala-desktop`):**
  - Built with Tauri v2 + React 18 + TypeScript.
  - Interactive virtual keyboard with live Shift / AltGr key-mapping preview.
  - Deterministic event inspector and automated test suite runner.
  - Live configuration drawer for candidate limits, dictionary settings, and startup modes.

- **🔒 100% Offline & Private:**
  - Zero cloud dependencies, zero telemetry, and zero keystroke logging.
  - Everything runs completely locally on your hardware.

---

## Supported Typing Layouts

| Mode | Layout Name | Description |
|:---:|:---|:---|
| 🇧🇩 | **Phonetic** | Avro-compatible phonetic transliteration (`ami` → `আমি`, `khoroch` → `খরচ`) with predictive suggestions and auto-spelling correction. |
| 🏛️ | **National (Jatiyo)** | Bangladesh Standard BDS 1738:2018 layout with `Link (G)` conjunct chaining and AltGr currency symbols. |
| 🌅 | **Probhat** | Established open-source layout with direct independent vowel access and standard conjuncts. |
| ⌨️ | **Bijoy** | Widely used legacy layout with `Link (G)` conjunct sequencing, emitting standardized Unicode NFC. |
| 🔤 | **English** | Clean passthrough mode (`F12` quick toggle) for zero-latency English typing. |

---

## Architecture Overview

```
                               ┌──────────────────────────────────────────────┐
                               │       BornoMala Bangla Input Platform        │
                               └──────────────────────┬───────────────────────┘
                                                      │
                       ┌──────────────────────────────┴──────────────────────────────┐
                       │                                                             │
        ┌──────────────▼──────────────┐                               ┌──────────────▼──────────────┐
        │  Windows Native TSF IME     │                               │ Desktop Sandbox & Settings  │
        │  (native/borno-tsf)         │                               │ (src-tauri & src/ UI)       │
        │  • bornomala_tsf.dll        │                               │ • Tauri v2 + React + TS     │
        │  • In-Process COM TIP       │                               │ • Virtual keyboard preview  │
        │  • Floating Candidate UI    │                               │ • Event logging & inspector │
        │  • Language Bar Controller  │                               │ • Config JSON persistence   │
        └──────────────┬──────────────┘                               └──────────────┬──────────────┘
                       │                                                             │
                       │ C-ABI (crates/bornomala-core/include/bornomala.h)            │ Rust FFI / Tauri IPC
                       └──────────────────────────────┬──────────────────────────────┘
                                                      │
                                       ┌──────────────▼──────────────┐
                                       │    bornomala-core (Rust)    │
                                       │  • Phonetic Transliteration │
                                       │  • National / Probhat /     │
                                       │    Bijoy Fixed Layouts      │
                                       │  • Unicode NFC Normalizer   │
                                       │  • Trie & Typo Dictionary   │
                                       │  • State Machine & C-ABI    │
                                       └─────────────────────────────┘
```

---

## Repository Structure

```
.
├── crates/
│   └── bornomala-core/         # Pure Rust core input engine, layouts, dictionary, and C-ABI
│       ├── include/            # C headers (bornomala.h, borno.h)
│       ├── src/                # Engine, normalizer, translit, layout, and dictionary code
│       └── tests/              # 221 comprehensive unit, contract, layout, and golden tests
├── native/
│   └── borno-tsf/              # Windows Text Services Framework (TSF) in-process C++ COM IME
│       ├── borno_tsf.cpp       # ITfTextInputProcessorEx, ITfKeyEventSink, COM server
│       ├── candidate_window.h  # Floating candidate UI with GDI rendering & caret tracking
│       ├── lang_bar_item.h     # Language bar status and mode button
│       ├── key_mapper.h        # Virtual-key to Borno physical-key translation
│       ├── test_harness.cpp    # Standalone native Level 1–4 verification harness
│       ├── build.ps1           # Automated build script (MinGW-w64 G++ & cargo)
│       └── scripts/            # Elevated register.bat & unregister.bat
├── src-tauri/                  # Desktop application backend (Tauri v2, config persistence)
├── src/                        # Desktop UI frontend (React 18, Vite, TypeScript, Vanilla CSS)
├── milestones/                 # Verified milestone specifications and completion reports (M0–M7)
├── ROADMAP.md                  # Master roadmap and milestone pathway through v1.0.0
├── AGENTS.md                   # Persistent operating rules for AI coding agents
└── LICENSE-MIT / LICENSE-APACHE # Dual open-source licenses
```

---

## Getting Started

### Prerequisites (Windows x64)

- **Rust Toolchain:** `1.99.1+` (`x86_64-pc-windows-gnu` via rustup)
- **C++ Compiler:** MinGW-w64 GCC (`16.2.0` or similar with UCRT, e.g. via WinLibs)
- **Node.js:** `v20+` or `v22+` with `npm`
- **WebView2:** Microsoft Edge WebView2 Runtime (preinstalled on Windows 10/11)

### 1. Run Automated Test Suite (229 Tests)

Run all 221 engine tests in the Rust core:
```powershell
cargo test -p bornomala-core
```

Run persistence and integration tests in the desktop runner:
```powershell
cargo run -p bornomala-desktop --bin persistence_suite
```

### 2. Build and Verify the Native Windows TSF IME

Compile the Rust static library, native C++ in-process DLL, and run the native verification harness:
```powershell
powershell -ExecutionPolicy Bypass -File "native/borno-tsf/build.ps1"
```

To register the IME system-wide:
1. Open `native/borno-tsf/scripts/`.
2. Right-click `register.bat` and choose **"Run as administrator"**.
3. Open Windows Notepad, Sublime Text, or any editor, press `Win + Space` to select **BornoMala Bangla**, and start typing!
4. Press `F12` to cycle between Phonetic, National, Probhat, Bijoy, and English modes.

To unregister:
- Right-click `native/borno-tsf/scripts/unregister.bat` and choose **"Run as administrator"**.

### 3. Launch the Desktop Sandbox & Settings App

Run the development sandbox:
```powershell
npm run tauri dev
```

Or build the release desktop binary:
```powershell
npm run build:app
```

---

## Roadmap

- [x] **M0 — Runnable Foundation:** Sandbox desktop app, Rust core, and CI baseline.
- [x] **M1 — Engine Contract & Deterministic Sandbox:** Formal engine state machine, event streams, and C-ABI.
- [x] **M2 — Phonetic Engine:** Avro-compatible phonetic transliteration, candidate generation, and conjunct parser.
- [x] **M3 — Unicode & Bangla Script Normalization:** Strict NFC canonical normalization, virama collapsing, and joiner safety.
- [x] **M4 — Fixed Keyboard Layouts:** National (Jatiyo), Probhat, and Bijoy layouts with physical key mappings.
- [x] **M5 — Dictionary Engine & Predictive Lexicon:** Trie-based prefix completions, typo corrections, and frequency ranking.
- [x] **M6 — Settings Persistence & User Customization:** Schema-versioned JSON config with deep fallback isolation.
- [x] **M7 — First Native Windows IME (TSF):** Full Windows Text Services Framework in-process COM IME with candidate UI.
- [ ] **M8 — Next Milestone:** Continued enhancements, packaging, and platform expansion.

See [`ROADMAP.md`](./ROADMAP.md) for the complete engineering pathway.

---

## Contributing & Collaboration

BornoMala is currently in an **active foundational build phase** led by the creator following the structured milestones in [`ROADMAP.md`](./ROADMAP.md).

- **Forking & Experimenting:** You are warmly encouraged to fork the repository and experiment freely under our open-source licenses!
- **Direct Contributions:** To prevent conflicting architectural shifts and ensure your time is valued, please reach out first before submitting large pull requests. Check our [**Contributing Guidelines (`CONTRIBUTING.md`)**](./CONTRIBUTING.md) for details on roadmap alignment, proposal discussions, and collaborator policies.
- **Contact:** You can open a GitHub Discussion / Issue or contact the author at **`asm.khalid111@gmail.com`**.

---

## License

This project is dual-licensed under either:

- **MIT License** ([`LICENSE-MIT`](./LICENSE-MIT))
- **Apache License, Version 2.0** ([`LICENSE-APACHE`](./LICENSE-APACHE))

at your option.
