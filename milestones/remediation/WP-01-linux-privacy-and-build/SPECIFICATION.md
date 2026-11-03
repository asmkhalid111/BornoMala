# Work Package WP-01: Linux Fcitx 5 Privacy & Build Integrity — Specification

> **Work Package ID:** `WP-01`  
> **Status:** Implemented & Verified  
> **Date:** 2026-10-26  
> **Target Scope:** Linux Fcitx 5 Addon (`native/bornomala-fcitx5/`)

---

## 1. Objective

1. Remove or redact all logging calls in the Linux Fcitx 5 native adapter that expose raw keysyms, preedit composition text, or committed user text strings.
2. Replace hardcoded developer paths and workspace target assumptions in `CMakeLists.txt` with multi-tier discovery.
3. Align `scripts/install.sh` to use the project root Cargo target directory and pass explicit paths to CMake.
4. Add automated static regression tests to guarantee zero keystroke or text logging and clean portability.

---

## 2. Invariants & Constraints

- **Zero Keystroke Retention:** Keystrokes and text payloads must never be written to logs.
- **Portability:** CMake must not rely on `/home/asmkhalid/...` or custom user directories.
- **Public API Preservation:** Fcitx 5 addon interface and C-ABI contracts remain completely unchanged.
- **Zero Core Regressions:** All existing unit, contract, and golden tests must continue to pass.

---

## 3. Files Impacted

1. `native/bornomala-fcitx5/bornomala.cpp` (Logging redaction)
2. `native/bornomala-fcitx5/CMakeLists.txt` (Multi-tier `libbornomala_core.a` discovery)
3. `native/bornomala-fcitx5/scripts/install.sh` (Target path alignment)
4. `crates/bornomala-core/tests/ffi_tests.rs` (Automated static regression test)
