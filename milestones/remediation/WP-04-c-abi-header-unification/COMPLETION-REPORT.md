# WP-04 Completion Report — C-ABI Header Unification (`borno.h` vs `bornomala.h`)

> **Work Package:** WP-04 (Remediation)  
> **Status:** Closed — Verified  
> **Target Package:** `crates/bornomala-core` (`include/`, `tests/c_abi/`, `tests/c_abi_layout_tests.rs`), `docs/engine-contract.md`  
> **Execution Date:** 2026-11-03  
> **Author:** Antigravity Coding Agent  
> **Operating Environment:** Windows 11 (x64) with pwsh, Rust 1.99.1 (`x86_64-pc-windows-gnu`), MinGW-w64 GCC/G++ 16.2.0 (`x86_64-w64-mingw32`)

---

## 1. Executive Summary

This report concludes **WP-04: C-ABI Header Unification (`borno.h` vs `bornomala.h`)** pursuant to developer review and authorization.

The four-dimensional header discrepancy between `borno.h` and `bornomala.h`—comprising API content disparity (missing error codes `-5`/`-6` and `borno_engine_configure_json` in `borno.h`), preprocessor guard collisions (`BORNO_H` shadowing), metadata drift (`@file` tag errors), and redundant dual-declaration maintenance—has been resolved following the canonical master + forwarding header pattern (Option A).

All ten acceptance criteria (AC-1 through AC-10) have been implemented, executed, and verified:
- **Level 1 (Compile-Only Inclusion & Visibility):** All 4 C99 and 2 C++17 translation units compiled cleanly with zero errors and zero warnings under strict `-Wall -Wextra -Werror -pedantic`.
- **Level 2 (Linkage & ABI Layout Parity):** C and C++ linkage test binaries linked cleanly against the dynamically resolved `libbornomala_core.a` static archive and ran to clean exit (0). Struct layout parity between C and Rust `#[repr(C)]` representations was verified dynamically at the byte level by `c_abi_layout_tests.rs`.
- **Level 3 (Baseline Regression & Consumer Compatibility):** All 251 pre-existing core tests across 10 targets continue to pass with 0 regressions, plus the new 11th target `c_abi_layout_tests.rs` (252 total passing tests). Windows TSF consumer compatibility was verified through isolated Stage A compilation and Stage B shared library linkage (`test_bornomala_tsf.dll`, 7,950,932 bytes) with zero test harness execution, zero DLL overwriting, and zero COM/registry mutation.

---

## 2. Final Scope Reconciliation

The implementation was confined to the authorized C-ABI headers, contract documentation, and dedicated C/C++ verification translation units:

| # | File Path | Type | Nature of Change |
|---|:---|:---:|:---|
| 1 | `crates/bornomala-core/include/bornomala.h` | Canonical Master Header | Corrected `@file bornomala.h`; established primary guard `#ifndef BORNOMALA_H`; added backward-compatibility alias `#define BORNO_H`; documented memory ownership, double-free prevention, and out-parameter reuse lifecycle; declared full C-ABI surface including `-5`, `-6`, and `borno_engine_configure_json`. |
| 2 | `crates/bornomala-core/include/borno.h` | Forwarding Compatibility Header | Converted from redundant 314-line duplicate to documented thin forwarding header under `#ifndef BORNO_LEGACY_FORWARDING_H`, `#include "bornomala.h"`. |
| 3 | `docs/engine-contract.md` | Engine Contract Documentation | Aligned §7 with canonical header location (`bornomala.h`), forwarding header deprecation guidance, status codes `-5`/`-6`, string borrowing rules, schema migration, and double-free prevention. |
| 4 | `crates/bornomala-core/tests/c_abi/tu_c99_borno_then_bornomala.c` | Level 1 C99 Test | Forward inclusion order test (`borno.h` $\to$ `bornomala.h`) under C99. |
| 5 | `crates/bornomala-core/tests/c_abi/tu_c99_bornomala_then_borno.c` | Level 1 C99 Test | Reverse inclusion order test (`bornomala.h` $\to$ `borno.h`) under C99. |
| 6 | `crates/bornomala-core/tests/c_abi/tu_c99_repeat_inclusion.c` | Level 1 C99 Test | Multi-inclusion idempotency test under C99. |
| 7 | `crates/bornomala-core/tests/c_abi/tu_c99_constants_and_types.c` | Level 1 C99 Test | C99 compile-time assertions for all error codes, modes, candidate kinds, and key codes. |
| 8 | `crates/bornomala-core/tests/c_abi/tu_cpp17_borno_then_bornomala.cpp` | Level 1 C++17 Test | Forward inclusion order and declaration visibility (`borno_engine_configure_json`) under C++17. |
| 9 | `crates/bornomala-core/tests/c_abi/tu_cpp17_bornomala_then_borno.cpp` | Level 1 C++17 Test | Reverse inclusion order and declaration visibility under C++17. |
| 10 | `crates/bornomala-core/tests/c_abi/test_c_linkage.c` | Level 2 C Linkage Test | C binary linking against static library; verifies `borno_engine_new`, `configure_json`, and `borno_engine_free`. |
| 11 | `crates/bornomala-core/tests/c_abi/test_cpp_linkage.cpp` | Level 2 C++ Linkage Test | C++ binary linking against static library; confirms unmangled `extern "C"` linkage. |
| 12 | `crates/bornomala-core/tests/c_abi/report_c_layout.c` | Level 2 Layout Reporter | Emits pointer width, struct sizes, and member offsets via standard `sizeof` and `offsetof`. |
| 13 | `crates/bornomala-core/tests/c_abi_layout_tests.rs` | Level 2 Layout Parity Test | Compiles and runs `report_c_layout.c` and compares output against Rust `core::mem::size_of` and `core::mem::offset_of!`. |
| 14 | `milestones/remediation/WP-04-c-abi-header-unification/SPECIFICATION.md` | Work Package Specification | Updated verification commands with COM callback warning exceptions and checked off verified criteria. |

### Preservation of Prior Working-Tree Changes
In strict accordance with `AGENTS.md` §1.4, all pre-existing tracked and untracked changes were preserved:
- `crates/bornomala-core/src/lib.rs` (WP-03 cursor arithmetic)
- `crates/bornomala-core/src/phonetic/candidates.rs` (WP-02 candidate integrity)
- `crates/bornomala-core/tests/contract_tests.rs` (WP-03 contract tests)
- `crates/bornomala-core/tests/engine_config_tests.rs` (WP-02 config tests)
- `crates/bornomala-core/tests/ffi_tests.rs` (WP-01 Linux privacy)
- `crates/bornomala-core/tests/phonetic_tests.rs` (WP-02 phonetic test alignment)
- `native/bornomala-fcitx5/CMakeLists.txt` (WP-01 CMake discovery)
- `native/bornomala-fcitx5/bornomala.cpp` (WP-01 Fcitx 5 privacy redaction)
- `native/bornomala-fcitx5/scripts/install.sh` (WP-01 installer cleanup)
- `crates/bornomala-core/tests/candidate_provenance_tests.rs` (WP-02 provenance suite)
- `milestones/AUDIT-2026-10-23.md`, `milestones/remediation/`, `milestones/revisions/` (planning records)
- Reference layout JSON fixtures (`bijoy_reference.json`, `national_reference.json`, `probhat_reference.json`)

---

## 3. Acceptance Criteria Evidence Matrix

| Criterion | Description | Command Executed | Actual Result | Status |
|:---|:---|:---|:---|:---:|
| **AC-1** | Authoritative Canonical Header Structure | File inspection (`bornomala.h`) | Primary guard `BORNOMALA_H`, alias `BORNO_H`, full C-ABI surface (`-5`, `-6`, `configure_json`), complete lifecycle comments. | **Verified** |
| **AC-2** | Forwarding Legacy Header Structure | File inspection (`borno.h`) | Guard `BORNO_LEGACY_FORWARDING_H`, `#include "bornomala.h"`, clean deprecation comment. | **Verified** |
| **AC-3** | C99 Inclusion Order & Declarations (Level 1) | `gcc -std=c99 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c crates/bornomala-core/tests/c_abi/tu_c99_*.c` | All 4 translation units compiled with 0 errors and 0 warnings (exit code 0). | **Verified** |
| **AC-4** | C++17 Declaration Visibility (Level 1) | `g++ -std=c++17 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c crates/bornomala-core/tests/c_abi/tu_cpp17_*.cpp` | Both translation units compiled with 0 errors and 0 warnings (exit code 0). | **Verified** |
| **AC-5** | Target-Aware C Linkage (Level 2) | `gcc ... test_c_linkage.c $staticLib -luserenv -lws2_32 -lbcrypt -lntdll -o .../test_c_linkage.exe` | Linked cleanly; executed binary output: `Level 2 C linkage test passed.` (exit code 0). | **Verified** |
| **AC-6** | Target-Aware C++ Linkage (Level 2) | `g++ ... test_cpp_linkage.cpp $staticLib -luserenv -lws2_32 -lbcrypt -lntdll -o .../test_cpp_linkage.exe` | Linked cleanly; executed binary output: `Level 2 C++ linkage test passed.` (exit code 0). | **Verified** |
| **AC-7** | Target-Aware C/Rust Struct Layout Parity (Level 2) | `cargo test -p bornomala-core --test c_abi_layout_tests --target x86_64-pc-windows-gnu` | `report_c_layout.c` compiled and executed; verified byte-for-byte size and offset parity: `test test_c_abi_layout_parity ... ok`. | **Verified** |
| **AC-8** | Documentation Alignment | Inspection of `docs/engine-contract.md` §7 | Synchronized with `bornomala.h`, `-5`/`-6`, string borrowing, schema migration, and double-free prevention. | **Verified** |
| **AC-9** | Rust Core Baseline Regression Gate (Level 3) | `cargo test -p bornomala-core --target x86_64-pc-windows-gnu` | **251 baseline tests passed across 10 targets (0 failures, 0 regressions)**; 11th target `c_abi_layout_tests` passed (252 total passed). | **Verified** |
| **AC-10** | Windows TSF Consumer Compatibility (Level 3) | Stage A: `g++ -c -std=c++17 ...` on 3 TSF files; Stage B: `g++ -shared ... -o test_bornomala_tsf.dll` | Stage A: 3 units compiled cleanly (exit code 0); Stage B: `test_bornomala_tsf.dll` linked cleanly (7,950,932 bytes, exit code 0). Zero registry or harness side effects. | **Verified** |

---

## 4. Detailed Test Execution Evidence

### 4.1 Target Resolution & Archive Prerequisite
- **Rust Target Triple:** `x86_64-pc-windows-gnu`
- **Compiler Machine (`gcc -dumpmachine`):** `x86_64-w64-mingw32`
- **Profile:** `release`
- **Static Library Location:** `E:\Codes\Bangla Input Platform\Borno V1\target\x86_64-pc-windows-gnu\release\libbornomala_core.a`
- **Archive Size:** 19,893,512 bytes
- **Symbol Inspection:** `nm -g libbornomala_core.a` confirmed presence of `0000000000002a20 T borno_engine_configure_json`.

### 4.2 Level 1 Verification (Inclusion Order & Declaration Visibility)
```
[AC-3: C99 Compilation]
Compiling crates/bornomala-core/tests/c_abi/tu_c99_borno_then_bornomala.c -> PASS (exit code 0)
Compiling crates/bornomala-core/tests/c_abi/tu_c99_bornomala_then_borno.c -> PASS (exit code 0)
Compiling crates/bornomala-core/tests/c_abi/tu_c99_repeat_inclusion.c -> PASS (exit code 0)
Compiling crates/bornomala-core/tests/c_abi/tu_c99_constants_and_types.c -> PASS (exit code 0)

[AC-4: C++17 Compilation]
Compiling crates/bornomala-core/tests/c_abi/tu_cpp17_borno_then_bornomala.cpp -> PASS (exit code 0)
Compiling crates/bornomala-core/tests/c_abi/tu_cpp17_bornomala_then_borno.cpp -> PASS (exit code 0)
```

### 4.3 Level 2 Verification (Static Linkage & ABI Layout Parity)
```
[AC-5: C Linkage Execution]
Linking target/c_abi_tests/test_c_linkage.exe against libbornomala_core.a -> PASS (exit code 0)
Output: "Level 2 C linkage test passed."

[AC-6: C++ Linkage Execution]
Linking target/c_abi_tests/test_cpp_linkage.exe against libbornomala_core.a -> PASS (exit code 0)
Output: "Level 2 C++ linkage test passed."

[AC-7: Layout Parity Verification]
Running tests\c_abi_layout_tests.rs
C-ABI layout parity verified successfully: C vs Rust struct layout identical for target 'x86_64-pc-windows-gnu'.
test test_c_abi_layout_parity ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
```

#### Layout Comparison Details (`x86_64-pc-windows-gnu`):
- `Pointer Width`: 64 bits (C: 64, Rust: 64)
- `BornoEventC`: Size = 32 bytes; Offsets: `event_type` = 0, `char_codepoint` = 4, `string_payload` = 8, `index_payload` = 16, `mode_payload` = 24.
- `BornoStatusC`: Size = 32 bytes; Offsets: `name` = 0, `version` = 8, `active_mode` = 16, `ready` = 24.
- `BornoStateC`: Size = 48 bytes; Offsets: `preedit` = 0, `raw_buffer` = 8, `cursor_pos` = 16, `candidates_count` = 24, `candidates` = 32, `selected_candidate_index` = 40, `is_composing` = 44, `mode` = 45.
- `BornoProcessResultC`: Size = 64 bytes; Offsets: `state` = 0, `committed_text` = 48, `cancelled` = 56, `handled` = 57.

### 4.4 Level 3 Verification (Baseline Regression Gate & TSF Compatibility)

#### Rust Core Test Suite (AC-9):
Executed: `cargo test -p bornomala-core --target x86_64-pc-windows-gnu`

| # | Test Target | Tests Passed | Status |
|---|:---|:---:|:---:|
| 1 | `src/lib.rs` (Unit tests) | 55 / 55 | **PASSED** |
| 2 | `tests/c_abi_layout_tests.rs` (WP-04 C-ABI Layout Parity) | 1 / 1 | **PASSED** |
| 3 | `tests/candidate_provenance_tests.rs` (Candidate provenance) | 8 / 8 | **PASSED** |
| 4 | `tests/contract_tests.rs` (Engine contracts & cursor tests) | 17 / 17 | **PASSED** |
| 5 | `tests/dictionary_benchmarks.rs` (Dictionary performance) | 5 / 5 | **PASSED** |
| 6 | `tests/dictionary_tests.rs` (Dictionary lookup & phonetic scoring) | 10 / 10 | **PASSED** |
| 7 | `tests/engine_config_tests.rs` (JSON configuration & migration) | 8 / 8 | **PASSED** |
| 8 | `tests/ffi_tests.rs` (FFI boundaries, privacy, status codes) | 8 / 8 | **PASSED** |
| 9 | `tests/layout_tests.rs` (Fixed layouts & Latin pass-through) | 57 / 57 | **PASSED** |
| 10 | `tests/phonetic_tests.rs` (Phonetic parser, grammar, conjuncts) | 61 / 61 | **PASSED** |
| 11 | `tests/unicode_golden_tests.rs` (Unicode NFC, Indic golden tests) | 22 / 22 | **PASSED** |
| **Total** | **All 11 targets** | **252 / 252** | **PASSED (0 Failures, 0 Regressions)** |

#### Windows TSF Consumer Compatibility (AC-10):
- **Stage A (Isolated Translation Unit Compilation):**
  - `native/borno-tsf/borno_tsf.cpp` $\to$ `target/tsf_compat_test/borno_tsf.o` (exit code 0)
  - `native/borno-tsf/candidate_window.cpp` $\to$ `target/tsf_compat_test/candidate_window.o` (exit code 0)
  - `native/borno-tsf/lang_bar_item.cpp` $\to$ `target/tsf_compat_test/lang_bar_item.o` (exit code 0)
- **Stage B (Isolated Shared Library Linkage):**
  - Linked `target/tsf_compat_test/test_bornomala_tsf.dll` (7,950,932 bytes, exit code 0)
- **Safety Invariant Verification:**
  - `native/borno-tsf/build.ps1` was NOT executed.
  - `native/borno-tsf/test_harness.exe` was NOT compiled or executed.
  - In-tree DLL `native/borno-tsf/bornomala_tsf.dll` was NOT overwritten.
  - No COM registration or registry modifications were made (`DllRegisterServer` / `DllUnregisterServer` were never invoked).

---

## 5. Architectural & Technical Decisions

1. **Option A (Canonical Master + Forwarding Wrapper):**
   `bornomala.h` is the authoritative single source of truth for the BornoMala Core C-ABI. `borno.h` is preserved as a clean forwarding wrapper including `bornomala.h` under `#ifndef BORNO_LEGACY_FORWARDING_H`. This completely eliminates dual-file drift while guaranteeing 100% source-level compatibility for existing consumers.
2. **Backward-Compatibility Guard Alias (`#define BORNO_H`):**
   `bornomala.h` defines `BORNO_H` internally if not already defined. This ensures that legacy preprocessor feature-detection checks (`#ifdef BORNO_H`) evaluate to true regardless of which header was included.
3. **Out-Parameter Memory Ownership & Double-Free Prevention:**
   Documented explicit memory ownership rules directly in `bornomala.h` and `docs/engine-contract.md`:
   - Preferred method: structure cleanup helpers (`borno_status_free_members`, `borno_state_free_members`, `borno_process_result_free_members`).
   - If callers individually release structure string members with `borno_string_free()`, they MUST immediately assign that field to `NULL`. Callers must never free a member individually and subsequently invoke the structure cleanup helper while that pointer remains dangling.
   - Candidate array inner strings and the outer pointer array MUST be managed exclusively via `borno_state_free_members()`; `borno_string_free()` must never be used on the outer `char**` array.
4. **Isolated TSF Verification with Warning Exceptions:**
   `native/borno-tsf` implements standard Windows COM interface callbacks (`ITfTextInputProcessorEx`, `ITfThreadMgrEventSink`, `ITfKeyEventSink`) which specify parameters unused by the minimalist text service. To maintain `-Wall -Wextra -Werror -pedantic` rigor without modifying pre-existing consumer files, `-Wno-unused-parameter` and `-Wno-missing-field-initializers` were specified for Stage A.

---

## 6. Documented Limitations

1. **Workspace Tooling Limitation (`tauri-winres`):**
   Full-workspace `cargo test --workspace` fails on Windows under the MinGW GNU toolchain due to path spacing in `tauri-winres` when building the desktop app GUI (`bornomala-desktop`). The core engine crate (`bornomala-core`) builds and verifies completely independently.
2. **Auto-Exporting Test Runner Side-Effect:**
   `test_export_fixtures` in `crates/bornomala-core/src/layout/builtin.rs:364-376` rewrites reference layout JSON fixtures (`bijoy_reference.json`, `national_reference.json`, `probhat_reference.json`) during `cargo test -p bornomala-core`, updating LF line endings on Windows. Git diff confirms 0 content changes.

---

## 7. Next Work Package Recommendation

In accordance with `milestones/remediation/INDEX.md`, the next scheduled remediation work package is:

- **Work Package WP-05: Desktop App Identifier & Configuration Migration**  
  **Scope:** Unify application identifiers (`com.bornomala.desktop` vs `com.borno.desktop`), configuration directory paths, and settings schema keys across Tauri and native platform adapters.

