# WP-03 Completion Report — Unicode-Safe Cursor Arithmetic & UTF-8 Dimension Synchronization

> **Work Package:** WP-03 (Remediation)  
> **Status:** Closed with Documented Limitations  
> **Target Package:** `crates/bornomala-core`  
> **Execution Date:** 2026-10-26  
> **Author:** Antigravity Coding Agent  

---

## 1. Executive Summary

This report concludes **WP-03: Unicode-Safe Cursor Arithmetic & UTF-8 Dimension Synchronization** pursuant to developer review and authorization.

The underlying defect in `crates/bornomala-core/src/lib.rs`—where the private `source_cursor` was treated as a scalar `char` index in byte-slice operations—has been eliminated. The engine enforces the strict invariant:
$$\text{source\_cursor} \le \text{raw\_buffer.len()} \land \text{raw\_buffer.is\_char\_boundary(source\_cursor)}$$
at all stable execution points and across every mutation path.

All 10 required acceptance criteria tests were implemented and verified. All 251 tests across all ten core test targets in `crates/bornomala-core` pass with zero failures and zero regressions. All pre-existing WP-01 and WP-02 changes remain preserved and decoupled.

---

## 2. Final Scope Reconciliation

The implementation was strictly confined to the two authorized files:

| # | File Path | Type | Nature of Change |
|---|:---|:---:|:---|
| 1 | `crates/bornomala-core/src/lib.rs` | Production & Unit Tests | Added `floor_char_boundary` helper; converted `insert_char_at_cursor`, `delete_before_cursor`, `delete_after_cursor` to boundary-safe UTF-8 byte operations; implemented bounded interpolation in `map_target_to_source`; added entry `debug_assert!` checks; implemented internal unit tests (Tests 2, 3, 4, 7, 9, 10). |
| 2 | `crates/bornomala-core/tests/contract_tests.rs` | Integration Tests | Implemented contract integration tests (Tests 1, 5, 6, 8) verifying public observables (`cursor_pos`, `preedit`, `raw_buffer`, `is_composing`, `handled`). |

### Preservation of Prior Work (Decoupled Baseline)
The following modifications from **WP-01** and **WP-02** remain untouched and preserved:
- `crates/bornomala-core/src/phonetic/candidates.rs` (WP-02 candidate integrity)
- `crates/bornomala-core/tests/engine_config_tests.rs` (WP-02 config alignment)
- `crates/bornomala-core/tests/ffi_tests.rs` (WP-01 Linux privacy invariant)
- `crates/bornomala-core/tests/phonetic_tests.rs` (WP-02 phonetic test alignment)
- `native/bornomala-fcitx5/CMakeLists.txt` (WP-01 static library resolution)
- `native/bornomala-fcitx5/bornomala.cpp` (WP-01 Fcitx 5 log redaction)
- `native/bornomala-fcitx5/scripts/install.sh` (WP-01 installer cleanup)
- `crates/bornomala-core/tests/candidate_provenance_tests.rs` (WP-02 provenance suite)

---

## 3. Invariant & Mutation Comparison

| Operation | Before WP-03 (Defective) | After WP-03 (Remediated) | Verification Evidence |
|:---|:---|:---|:---|
| **Character Insertion** | Collected to `Vec<char>`, inserted at `source_cursor.min(chars.len())`, advanced by `idx + c.len_utf8()` (mixing scalar index with byte width) | Native `String::insert(source_cursor, c)` at valid byte boundary, advancing `source_cursor += c.len_utf8()` | Test 2 (`test_internal_cursor_multibyte_insertion`) |
| **Backspace Deletion** | Removed `source_cursor - 1` from `Vec<char>`, subtracted 1 byte from `source_cursor` (corrupting multibyte continuation bytes) | Slices `..source_cursor`, extracts `chars().next_back()`, drains exact byte range `prev_start..source_cursor`, assigns `source_cursor = prev_start` | Test 3 (`test_internal_cursor_multibyte_interior_backspace`) |
| **Forward Delete** | Removed `source_cursor` from `Vec<char>`, leaving `source_cursor` desynchronized | Slices `source_cursor..`, extracts `chars().next()`, drains exact byte range `source_cursor..next_end`, preserves `source_cursor` | Test 4 (`test_internal_cursor_multibyte_forward_delete`) |
| **Target-to-Source Mapping** | Rounded candidate without boundary alignment, risking mid-character continuation byte positions | Candidate byte snapped with `floor_char_boundary` and clamped to `[source_range.start, source_range.end]` | Tests 7, 9, 10 |

---

## 4. Test Evidence Summary

### Verified Core Targets (251 Tests Passing)
All ten test targets within `crates/bornomala-core` were executed individually and passed:

1. `cargo test -p bornomala-core --lib` $\rightarrow$ **55 passed; 0 failed; 0 ignored**
2. `cargo test -p bornomala-core --test candidate_provenance_tests` $\rightarrow$ **8 passed; 0 failed; 0 ignored**
3. `cargo test -p bornomala-core --test contract_tests` $\rightarrow$ **17 passed; 0 failed; 0 ignored**
4. `cargo test -p bornomala-core --test dictionary_benchmarks` $\rightarrow$ **5 passed; 0 failed; 0 ignored**
5. `cargo test -p bornomala-core --test dictionary_tests` $\rightarrow$ **10 passed; 0 failed; 0 ignored**
6. `cargo test -p bornomala-core --test engine_config_tests` $\rightarrow$ **8 passed; 0 failed; 0 ignored**
7. `cargo test -p bornomala-core --test ffi_tests` $\rightarrow$ **8 passed; 0 failed; 0 ignored**
8. `cargo test -p bornomala-core --test layout_tests` $\rightarrow$ **57 passed; 0 failed; 0 ignored**
9. `cargo test -p bornomala-core --test phonetic_tests` $\rightarrow$ **61 passed; 0 failed; 0 ignored**
10. `cargo test -p bornomala-core --test unicode_golden_tests` $\rightarrow$ **22 passed; 0 failed; 0 ignored**

**Total Core Passing:** **251 passed; 0 failed; 0 ignored.**

Formatting check `rustfmt --edition 2021 --check crates/bornomala-core/tests/contract_tests.rs` exited 0. Modified blocks in `crates/bornomala-core/src/lib.rs` are formatted to rustfmt specification.

---

## 5. Documented Limitations

1. **Workspace Tooling Limitation (`windres`):**
   Full-workspace verification (`cargo test --workspace`) fails on Windows under the GNU toolchain during `src-tauri` (`bornomala-desktop`) build script execution due to unquoted path spacing in `tauri-winres` (`cc1.exe: fatal error: Input\: No such file or directory`). The core engine crate (`bornomala-core`) is completely isolated and verified independently.
2. **Auto-Exporting Test Runner Side-Effect:**
   `test_export_fixtures` in `crates/bornomala-core/src/layout/builtin.rs:364-376` writes reference layout JSON files to disk during `cargo test --lib`, touching LF line endings on Windows. Git diff confirms 0 content changes.

---

## 6. Next Work Package Recommendation

In accordance with `milestones/remediation/INDEX.md`, the next scheduled remediation work package is:

- **Work Package WP-04: C-ABI Header Unification**  
  **Scope:** Synchronize and unify `borno.h` with `bornomala.h` to ensure complete behavioral parity and declaration consistency across native platform wrappers.
