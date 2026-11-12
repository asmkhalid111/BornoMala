# System Architecture — Bangla Input Platform

## 1. Overview

The Bangla Input Platform is designed as an offline-first, modular desktop input platform. The system is split into three main tiers:

```
+-------------------------------------------------------------+
|             User Applications (Word, Browser, etc.)         |
+-------------------------------------------------------------+
                              ^
                              | (OS Input Protocol)
+-------------------------------------------------------------+
|               Platform Native Adapters                      |
|   - Windows: Text Services Framework (TSF)                  |
|              (native/borno-tsf/, C++ / Rust)                |
|   - Linux:   Fcitx 5 Addon                                  |
|              (native/bornomala-fcitx5/, C++20 / Rust)       |
|   - macOS:   InputMethodKit (Planned / deferred —           |
|              not yet implemented or verified)               |
+-------------------------------------------------------------+
                              ^
                              | (C-ABI / Direct Engine API)
+-------------------------------------------------------------+
|               Shared Core: bornomala-core (Rust)            |
|   - Event normalizer & state machine                        |
|   - Phonetic transliterator & rule table                    |
|   - Fixed layout mappings (National, Probhat, Bijoy)        |
|   - Unicode normalization & composition rules               |
|   - Candidate generator & ranking                           |
+-------------------------------------------------------------+
                              ^
                              | (Tauri IPC Invocation)
+-------------------------------------------------------------+
|       Desktop Sandbox & Settings UI (bornomala-desktop)     |
|   - Tauri v2 + React + TypeScript (src-tauri/ & src/)       |
|   - Real-time preedit and candidate inspection              |
|   - Configuration and diagnostics                           |
+-------------------------------------------------------------+
```

## 2. Shared Core (`crates/bornomala-core/`)
The shared core (`bornomala-core` in `crates/bornomala-core/`) contains all language and input logic. It is purely written in Rust with zero GUI or OS-specific dependencies, ensuring deterministic testing and cross-platform reusability across `native/borno-tsf/`, `native/bornomala-fcitx5/`, and `src-tauri/`.

## 3. Desktop Sandbox (`bornomala-desktop`)
The Tauri desktop application (`bornomala-desktop` in `src-tauri/` with React/TypeScript UI in `src/`) provides a persistent debugging, testing, and settings environment that remains functional regardless of native OS IME integration status.

## 4. Platform Architecture vs. Verification Status
The diagram above represents the intended modular system architecture. Actual build, installation, and runtime verification boundaries across Windows, WSL2/WSLg Linux (`Fcitx 5`), unverified native Linux desktop / Wayland sessions, and deferred/unverified macOS are documented separately in [`docs/platform-support.md`](platform-support.md).
