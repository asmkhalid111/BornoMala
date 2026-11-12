# BornoMala (বর্ণমালা) — Bangla Input Platform

[![Rust](https://img.shields.io/badge/Rust-1.99.1%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20TSF%20%7C%20Linux%20Fcitx%205-0078d7.svg)](https://github.com/asmkhalid111/BornoMala)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24c8db.svg)](https://tauri.app/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Core Tests](https://img.shields.io/badge/Core%20Tests-265%20Core%20Tests-blue.svg)](#getting-started)
[![UX Baseline](https://img.shields.io/badge/UX%20Baseline-UX%200.1.0-blueviolet.svg)](docs/design-system-and-usability.md)
[![Config Schema](https://img.shields.io/badge/Config%20Schema-v1.2.0-informational.svg)](crates/bornomala-core/src/config.rs)

**BornoMala (বর্ণমালা)** is a fast, modular, offline-first, and privacy-conscious Bangla input platform for desktop operating systems.

It unifies multiple Bangla typing layouts — **Phonetic transliteration**, **National (Jatiyo)**, **Probhat**, and **Bijoy** — behind a shared, high-performance Rust core engine. BornoMala provides:
1. **Windows Native TSF In-Process IME** (`bornomala_tsf.dll`) for system-wide typing on Windows (verified historically in M7; post-M9 live system COM re-testing remains outstanding).
2. **Linux Native Fcitx 5 Addon** (`bornomala.so`) — **partially verified** under Ubuntu 26.04.1 LTS on WSL2/WSLg (`XWayland` with `--disable waylandim`, GTK3, and GNOME `gedit`). Native Linux desktop sessions, native Wayland (`waylandim`), Qt5/Qt6, GTK4, live candidate-popup pointer clicks, and other distributions remain **unverified** (see [`docs/platform-support.md`](./docs/platform-support.md)).
3. **Desktop Sandbox & Configuration App** (`bornomala-desktop` / local `Borno-Sandbox.exe` build target) built with Tauri v2 + React 18 + TypeScript. *(Prebuilt Windows binaries are not distributed in Git or GitHub Releases yet; run from source during development.)*

The project follows a **three-tier decoupled versioning architecture**:
- **Product Milestone Pathway:** Targeted towards `v1.0.0` release per [`ROADMAP.md`](./ROADMAP.md).
- **Design System & UX Framework:** Baseline `UX 0.1.0` defined in [`docs/design-system-and-usability.md`](./docs/design-system-and-usability.md).
- **Configuration Schema:** Schema `v1.2.0` with atomic failure-safe persistence in `crates/bornomala-core/src/config.rs`.

---

## Key Features

- **⚡ Blazing Fast Rust Engine (`bornomala-core`):**
  - Sub-millisecond keystroke processing latency (< 0.25 ms average, 3.4 µs pure core).
  - Canonical Unicode NFC normalization, vowel sign collapsing, and conjunct joiner preservation (ZWJ/ZWNJ).
  - Trie-based predictive completions and Levenshtein typo-correction for Phonetic typing.
  - 100% safe Rust core with a clean C-ABI (`bornomala.h`).

- **🪟 Native Windows TSF In-Process IME (`native/borno-tsf`):**
  - Full Text Services Framework (TSF) COM in-process text service (`bornomala_tsf.dll`).
  - Historical M7 verification covered Windows Notepad, Sublime Text, Office, and Visual Studio Code; post-M9 / post-remediation live system COM re-testing remains outstanding.
  - Real-time preedit inline composition with accurate caret tracking.
  - Floating GDI candidate window with mouse and keyboard navigation (1–9 digits, arrows, space/enter commit).
  - Language bar integration and instant mode toggling (`F12`).

- **🐧 Native Linux Fcitx 5 Addon (`native/bornomala-fcitx5`):**
  - Native C++20 `InputMethodEngineV2` addon (`bornomala.so`) statically linked against PIC `libbornomala_core.a`.
  - **Partially verified live scope:** Ubuntu 26.04.1 LTS under WSL2/WSLg (`Fcitx 5.1.19`, `XWayland` with `--disable waylandim`), GTK3 (`Gtk.Entry`), and GNOME `gedit`.
  - Inline preedit formatting with underline styling, multi-byte UTF-8 cursor mapping, Fcitx candidate list comments (`transliteration`, `lexicon`, `spelling`), and `F12` / `F4` mode switching.
  - **Explicitly unverified:** Native Linux desktop sessions, native Wayland (`waylandim`), Qt5/Qt6, GTK4 live typing, live candidate-popup pointer selection, other Linux distributions, and hosted GitHub Actions `ubuntu-24.04` CI (workflow configured in [`.github/workflows/ci.yml`](./.github/workflows/ci.yml), with no verified hosted run yet; current Linux evidence is based on WSL2/WSLg and local CI-mirror verification). See [`docs/platform-support.md`](./docs/platform-support.md) and [`milestones/M8-revalidation-report.md`](./milestones/M8-revalidation-report.md).

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
│       ├── include/            # C headers (bornomala.h canonical master, borno.h forwarding)
│       ├── src/                # Engine, normalizer, translit, layout, and dictionary code
│       └── tests/              # 265 unit, contract, layout, golden, provenance, licensing, and C-ABI tests (12 targets)
├── native/
│   ├── borno-tsf/              # Windows Text Services Framework (TSF) in-process C++ COM IME
│   │   ├── borno_tsf.cpp       # ITfTextInputProcessorEx, ITfKeyEventSink, COM server
│   │   ├── candidate_window.h  # Floating candidate UI with GDI rendering & caret tracking
│   │   ├── lang_bar_item.h     # Language bar status and mode button
│   │   ├── README.md           # Build, manual COM registration, and safety warnings
│   │   └── scripts/            # Elevated register.bat & unregister.bat (see README warning)
│   └── bornomala-fcitx5/       # Linux Fcitx 5 C++20 input method engine addon
│       ├── bornomala.cpp       # Fcitx 5 InputMethodEngineV2 implementation
│       ├── engine.h            # Engine, context property, and candidate word declarations
│       ├── adapter_helpers.h   # Per-context state, mode sync, and XDG config reload helpers
│       ├── adapter_helpers.cpp # Mode sync, C-ABI consumption guard, and XDG reload logic
│       ├── README.md           # Prerequisites, Release CMake/CTest build, install, and uninstall guide
│       ├── scripts/            # common.sh, install.sh, uninstall.sh, fcitx_test_env.py, interactive_tester.py, launch_test_editor.sh
│       └── tests/              # 4 Release-active C++ CTest targets, 16-test script safety suite, and isolated GTK3/Gedit runners
├── src-tauri/                  # Desktop application backend (Tauri v2, config persistence)
├── src/                        # Desktop UI frontend (React 18, Vite, TypeScript, Vanilla CSS)
├── docs/                       # Specifications, architecture, support matrix, and usability backlogs
│   ├── design-system-and-usability.md      # Living UX/design spec, tokens, and a11y criteria
│   ├── usability-and-linguistic-backlog.md # Usability bug tracking & ergonomic test cases
│   ├── platform-support.md                 # Authoritative current platform capability & verification matrix
│   ├── engine-contract.md                  # State machine and event dispatch contract
│   ├── unicode-composition.md              # Unicode normalization & grapheme cluster rules
│   └── windows-tsf-architecture.md         # Native Windows TSF COM architecture
├── milestones/                 # Milestone specifications (M0–M9), completion & revalidation reports
│   ├── M8-revalidation-report.md # Current M8 Linux Fcitx 5 evidence & blocked hosted CI record
│   ├── remediation/            # Remediation specs and closure reports (WP-01–WP-I)
│   └── revisions/              # Architectural revisions, hardening roadmaps, and governance reviews
├── ROADMAP.md                  # Active master roadmap and milestone pathway through v1.0.0
├── AGENTS.md                   # Persistent operating rules for AI coding agents
└── LICENSE-MIT / LICENSE-APACHE # Dual open-source licenses
```

*(Prebuilt Windows binaries such as `Borno-Sandbox.exe` are intentionally gitignored and not distributed in a clean Git checkout or GitHub Release.)*

---

## Getting Started

### 1. Run Automated Rust Core Tests (265 Tests, 12 Targets)

```powershell
cargo test -p bornomala-core
```

*(On Linux/WSL2, the separate Fcitx 5 adapter also includes `4` Release CMake/CTest suites and `16` packaging/harness tests in `native/bornomala-fcitx5/tests/test_scripts.py`.)*

### 2. Launch Desktop Sandbox from Source (Windows / Tauri Dev)

Prebuilt Windows executables (`Borno-Sandbox.exe`) are intentionally excluded from Git, and no GitHub Release binary asset is currently published. With Rust and Node.js installed, run the desktop sandbox directly from source:

```powershell
npm install
npm run tauri dev
```

*(Note on Windows GNU builds: workspace paths containing spaces require an external spaceless `CARGO_TARGET_DIR` for full release builds; `npm run tauri dev` or `npm run build` runs TypeScript + Vite checks directly.)*

### 3. Windows: Build and Register Native TSF IME (Read Warning First)

> **Warning — COM/TSF Registry Mutation:** Do **not** treat `native/borno-tsf/build.ps1` as a routine read-only build command. Every run builds and automatically executes `test_harness.exe`, which invokes `DllRegisterServer()` and then `DllUnregisterServer()` for CLSID `{8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}`—wiping any existing COM/TSF registration on that machine if run elevated. Read [`native/borno-tsf/README.md`](./native/borno-tsf/README.md) before running `build.ps1` or using `scripts/register.bat` / `scripts/unregister.bat`.

```powershell
powershell -ExecutionPolicy Bypass -File "native/borno-tsf/build.ps1"
```

After reading [`native/borno-tsf/README.md`](./native/borno-tsf/README.md), manual elevated registration uses `native/borno-tsf/scripts/register.bat`, and `F12` cycles modes (Phonetic, National, Probhat, Bijoy, English).

### 4. Linux / WSL2: Build and Install Fcitx 5 Addon (User Mode)

See [`native/bornomala-fcitx5/README.md`](./native/bornomala-fcitx5/README.md) for required Ubuntu/Debian Fcitx 5 development packages (`cmake`, `g++`, `pkg-config`, `make`, `fcitx5`, `libfcitx5core-dev`, `libfcitx5config-dev`, `libfcitx5utils-dev`, `fcitx5-modules-dev`), Release CMake/CTest commands, troubleshooting, and clean removal.

From the repository root:

```bash
# Builds bornomala-core (PIC) + Fcitx 5 addon and installs into user-owned ~/.local paths
bash native/bornomala-fcitx5/scripts/install.sh
```

`install.sh` runs strictly in user mode without `sudo`, preserves your existing `~/.config/fcitx5/profile` by default (opt-in `--register-profile` creates a backup first), and never kills or automatically restarts running Fcitx 5 daemons. It also accepts `--skip-build --build-dir DIR` to install an existing CMake build, and `native/bornomala-fcitx5/scripts/uninstall.sh` safely removes only BornoMala-owned files.

To test interactively in an isolated Fcitx session without touching your desktop profile:
- Run `Launch-Linux-Fcitx5-Test.bat` from Windows (or `python3 -B native/bornomala-fcitx5/scripts/interactive_tester.py` inside Linux/WSL2 after building), and press `F12` or `F4` to cycle typing modes.

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
- [x] **M7 — First Native Windows IME (TSF):** Historical verification (`milestones/M7_COMPLETION_REPORT.md`); post-M9 live system COM re-testing remains outstanding.
- [ ] **M8 — Linux Integration (Fcitx 5):** **Partially Verified** — Historical record (`2026-10-19`) in [`milestones/M8-completion-report.md`](./milestones/M8-completion-report.md); current `M8-WP1`–`WP3` revalidation in [`milestones/M8-revalidation-report.md`](./milestones/M8-revalidation-report.md) and [`docs/platform-support.md`](./docs/platform-support.md). WSL2/WSLg (`Fcitx 5.1.19`, XWayland `--disable waylandim`, GTK3, Gedit), Release CTest + canary, `16/16` script safety suite, local CI-mirror checks, and two-process XDG reload pass; native Linux desktop, native Wayland (`waylandim`), Qt/GTK4 live typing, live pointer popup click, and hosted Ubuntu 24.04 CI execution (configured in [`.github/workflows/ci.yml`](./.github/workflows/ci.yml), with no verified hosted run yet) remain unverified.
- [x] **M9 — Customization & Usability:** Completed and signed off (`2026-10-19`) in [`milestones/M9-completion-report.md`](./milestones/M9-completion-report.md) (`v1.2.0` schema, themes, `UX 0.1.0` design baseline, Dari punctuation toggle). Follow-on UI/ergonomic refinements remain tracked in [`docs/usability-and-linguistic-backlog.md`](./docs/usability-and-linguistic-backlog.md).
- [x] **Remediation & Post-M9 Hardening (`WP-01`–`WP-04`, `WP-05`–`WP-I`, `M8-WP1`–`WP3`):**
  - **WP-01 – WP-04:** Privacy redaction & CMake discovery (`WP-01`), candidate contract alignment (`WP-02`), Unicode scalar/byte cursor safety (`WP-03`), and C-ABI header unification (`WP-04`), tracked in [`milestones/remediation/INDEX.md`](./milestones/remediation/INDEX.md).
  - **WP-05 – WP-I:** App identifier/config unification (`com.bornomala.desktop`), licensing notices (`THIRD-PARTY-NOTICES.md`, w/ open `RV-11` MinGW `libmingwex.a` audit), settings/ellipsis/digit fixes, Windows GNU PE manifest, and UI text selectability, closed in [`milestones/remediation/FINAL-REMEDIATION-CLOSURE-REPORT.md`](./milestones/remediation/FINAL-REMEDIATION-CLOSURE-REPORT.md).
  - **M8-WP1 – WP3:** Fcitx 5 per-context mode sync, XDG reload, Release CTest + canary enforcement, non-destructive install/uninstall/GUI harnesses, and `.github/workflows/ci.yml` (`linux-fcitx5`), documented in [`milestones/M8-revalidation-report.md`](./milestones/M8-revalidation-report.md).
- [ ] **M10 — Contextual Prediction (Optional, Evidence-Gated):** Reproducible Python evaluation pipeline, offline next-word prediction quality/latency benchmarks, and candidate integration assessment.

> **Note on UX Evolution:** Ongoing visual and ergonomic refinements (e.g. multi-accent color selectors, light theme contrast enhancements, and 1-press down-arrow candidate selection) are systematically cataloged in [`docs/usability-and-linguistic-backlog.md`](./docs/usability-and-linguistic-backlog.md) and governed by [`docs/design-system-and-usability.md`](./docs/design-system-and-usability.md).

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
