# BornoMala (বর্ণমালা) — Bangla Input Platform

[![Rust](https://img.shields.io/badge/Rust-1.99.1%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20TSF%20%7C%20Linux%20Fcitx%205-0078d7.svg)](https://github.com/asmkhalid111/BornoMala)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24c8db.svg)](https://tauri.app/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Tests](https://img.shields.io/badge/Tests-142%20Core%20%7C%20All%20Passing-brightgreen.svg)](#testing)

**BornoMala (বর্ণমালা)** is a fast, modular, offline-first, and privacy-conscious Bangla input platform for desktop operating systems.

It unifies multiple Bangla typing layouts — **Phonetic transliteration**, **National (Jatiyo)**, **Probhat**, and **Bijoy** — behind a shared, high-performance Rust core engine. BornoMala ships as:
1. **Windows Native TSF In-Process IME** (`bornomala_tsf.dll`) for system-wide typing in all Windows applications.
2. **Linux Native Fcitx 5 Addon** (`bornomala.so`) for system-wide typing across Linux desktop environments (GNOME, KDE, X11, Wayland/WSLg).
3. **Desktop Sandbox & Configuration App** (`bornomala-desktop`) built with Tauri v2 + React 18 + TypeScript.

---

## Key Features

- **⚡ Blazing Fast Rust Engine (`bornomala-core`):**
  - Sub-millisecond keystroke processing latency (< 0.25 ms average, 3.4 µs pure core).
  - Canonical Unicode NFC normalization, vowel sign collapsing, and conjunct joiner preservation (ZWJ/ZWNJ).
  - Trie-based predictive completions and Levenshtein typo-correction for Phonetic typing.
  - 100% safe Rust core with a clean C-ABI (`bornomala.h`).

- **🪟 Native Windows TSF In-Process IME (`native/borno-tsf`):**
  - Full Text Services Framework (TSF) COM in-process text service (`bornomala_tsf.dll`).
  - Works natively across Windows Notepad, Sublime Text, Office, Visual Studio Code, browsers, and terminal emulators.
  - Real-time preedit inline composition with accurate caret tracking.
  - Floating GDI candidate window with mouse and keyboard navigation (1–9 digits, arrows, space/enter commit).
  - Language bar integration and instant mode toggling (`F12`).

- **🐧 Native Linux Fcitx 5 Addon (`native/bornomala-fcitx5`):**
  - Native C++ input method engine addon (`bornomala.so`) statically linked against `libbornomala_core.a`.
  - Seamless integration with Fcitx 5 across GTK3, GTK4, Qt5, Qt6, X11, and Wayland/WSLg.
  - Inline preedit formatting with underline styling and cursor offset mapping across multi-byte UTF-8 boundaries.
  - Desktop floating candidate list with classification badges (`transliteration`, `lexicon`, `spelling`).
  - Instant layout cycling with `F12` or `F4`.

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
| 🔤 | **English** | Clean passthrough mode (`F12` or `F4` quick toggle) for zero-latency English typing. |

---

## Architecture Overview

```
                               ┌────────────────────────────────────────────────────────┐
                               │            BornoMala Bangla Input Platform             │
                               └───────────────────────────┬────────────────────────────┘
                                                           │
                        ┌──────────────────────────────────┼──────────────────────────────────┐
                        │                                  │                                  │
         ┌──────────────▼──────────────┐    ┌──────────────▼──────────────┐    ┌──────────────▼──────────────┐
         │  Windows Native TSF IME     │    │  Linux Native Fcitx 5 Addon │    │ Desktop Sandbox & Settings  │
         │  (native/borno-tsf)         │    │  (native/bornomala-fcitx5)  │    │ (src-tauri & src/ UI)       │
         │  • bornomala_tsf.dll        │    │  • bornomala.so             │    │ • Tauri v2 + React + TS     │
         │  • In-Process COM TIP       │    │  • Fcitx 5 InputMethodEngine│    │ • Virtual keyboard preview  │
         │  • Floating Candidate UI    │    │  • Preedit Underline Format │    │ • Event logging & inspector │
         │  • Language Bar Controller  │    │  • Floating Candidate Popup │    │ • Config JSON persistence   │
         └──────────────┬──────────────┘    └──────────────┬──────────────┘    └──────────────┬──────────────┘
                        │                                  │                                  │
                        │ C-ABI                            │ C-ABI                            │ Rust FFI / Tauri IPC
                        └──────────────────────────────────┼──────────────────────────────────┘
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
│       └── tests/              # 142 comprehensive unit, contract, layout, and golden tests
├── native/
│   ├── borno-tsf/              # Windows Text Services Framework (TSF) in-process C++ COM IME
│   │   ├── borno_tsf.cpp       # ITfTextInputProcessorEx, ITfKeyEventSink, COM server
│   │   ├── candidate_window.h  # Floating candidate UI with GDI rendering & caret tracking
│   │   ├── lang_bar_item.h     # Language bar status and mode button
│   │   └── scripts/            # Elevated register.bat & unregister.bat
│   └── bornomala-fcitx5/       # Linux Fcitx 5 C++ input method engine addon
│       ├── bornomala.cpp       # Fcitx 5 InputMethodEngineV2 implementation
│       ├── engine_adapter.h    # State machine binding & codepoint-to-byte offset mapper
│       ├── scripts/            # build.sh, install.sh, interactive_tester.py
│       └── tests/              # Headless verification suite (Layers 1–6)
├── src-tauri/                  # Desktop application backend (Tauri v2, config persistence)
├── src/                        # Desktop UI frontend (React 18, Vite, TypeScript, Vanilla CSS)
├── docs/                       # Architecture, engine contract, and usability backlogs
├── milestones/                 # Verified milestone specifications and completion reports (M0–M8)
├── ROADMAP.md                  # Master roadmap and milestone pathway through v1.0.0
├── AGENTS.md                   # Persistent operating rules for AI coding agents
└── LICENSE-MIT / LICENSE-APACHE # Dual open-source licenses
```

---

## Getting Started

### 1. Run Automated Rust Core Tests (142 Tests)

```powershell
cargo test -p bornomala-core
```

### 2. Windows: Build and Register Native TSF IME

```powershell
powershell -ExecutionPolicy Bypass -File "native/borno-tsf/build.ps1"
```

To register the IME system-wide:
1. Run `native/borno-tsf/scripts/register.bat` as Administrator.
2. Press `Win + Space` in any application to select **BornoMala Bangla**.
3. Press `F12` to toggle modes (Phonetic, National, Probhat, Bijoy, English).

### 3. Linux: Build and Install Fcitx 5 Addon

```bash
# Build static core & Fcitx 5 addon
cd native/bornomala-fcitx5
./scripts/build.sh

# User-mode installation (no root required)
./scripts/install.sh
```

To test interactively on Linux or Windows WSL2:
- Run `Launch-Linux-Fcitx5-Test.bat` from Windows (or `python3 native/bornomala-fcitx5/scripts/interactive_tester.py` inside Linux).
- Press `F12` or `F4` to cycle typing modes.

### 4. Launch Desktop Sandbox & Settings App

```powershell
npm run tauri dev
```

---

## Roadmap & Milestone Progress

- [x] **M0 — Runnable Foundation:** Sandbox desktop app, Rust core, and CI baseline.
- [x] **M0.1 — Native Feasibility:** Windows TSF COM architecture spike and C-ABI boundary.
- [x] **M1 — Engine Contract & Sandbox:** State machine, event dispatch, and inspectable candidate UI.
- [x] **M2 — Unicode & Composition Foundation:** Strict NFC canonical normalization, virama collapsing, and joiner safety.
- [x] **M3 — Phonetic Input v1:** Avro-compatible phonetic transliteration, candidate generation, and conjunct matrix.
- [x] **M4 — Fixed Layouts:** National (Jatiyo), Probhat, and Bijoy layouts with physical key mappings.
- [x] **M5 — Settings Foundation:** Schema-versioned JSON config with deep fallback isolation and Settings Drawer.
- [x] **M6 — Dictionary & Correction:** Trie-based prefix completions, typo corrections, and phonetic ambiguity pipeline.
- [x] **M7 — First Native Windows IME (TSF):** Full Windows Text Services Framework in-process COM IME with floating candidate UI.
- [x] **M8 — Linux Integration (Fcitx 5):** Native C++ input method engine addon for Linux desktops with candidate popups, preedit underline, and fontconfig setup.
- [ ] **M9 — Customization & Usability (Next Milestone):** Shortcut customization, candidate typography, layout export, and phonetic punctuation enhancements ([`docs/usability-and-linguistic-backlog.md`](./docs/usability-and-linguistic-backlog.md)).

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
