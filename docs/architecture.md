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
|   - Windows: Text Services Framework (TSF) (C++ / Rust)     |
|   - Linux: Fcitx 5 module (C++ / Rust)                      |
|   - macOS: InputMethodKit (Obj-C / Swift / Rust)            |
+-------------------------------------------------------------+
                              ^
                              | (FFI / Direct Engine API)
+-------------------------------------------------------------+
|                  Shared Core: borno-core (Rust)             |
|   - Event normalizer & state machine                        |
|   - Phonetic transliterator & rule table                    |
|   - Fixed layout mappings (National, Bijoy, etc.)           |
|   - Unicode normalization & composition rules               |
|   - Candidate generator & ranking                           |
+-------------------------------------------------------------+
                              ^
                              | (Tauri IPC Invocation)
+-------------------------------------------------------------+
|             Desktop Sandbox & Settings UI                   |
|   - Tauri v2 + React + TypeScript                           |
|   - Real-time preedit and candidate inspection              |
|   - Configuration and diagnostics                           |
+-------------------------------------------------------------+
```

## 2. Shared Core (`crates/borno-core`)
The shared core contains all language and input logic. It is purely written in Rust with zero GUI or OS-specific dependencies, ensuring deterministic testing and cross-platform reusability.

## 3. Desktop Sandbox (`borno-desktop`)
The sandbox application provides a persistent debugging, testing, and settings environment that remains functional regardless of native OS IME integration status.
