# WP-03 Implementation Report — Unicode-Safe Cursor Arithmetic & UTF-8 Dimension Synchronization

> **Work Package:** WP-03 (Remediation)  
> **Status:** Implemented & Verified — Awaiting Formal Closure  
> **Target Package:** `crates/bornomala-core`  
> **Execution Date:** 2026-10-26  
> **Author:** Antigravity Coding Agent  

---

## 1. Executive Summary

This report documents the implementation and verification of **WP-03: Unicode-Safe Cursor Arithmetic & UTF-8 Dimension Synchronization** in accordance with `milestones/remediation/WP-03-cursor-arithmetic/SPECIFICATION.md`.

The remediation eliminates the dimensional defect in `crates/bornomala-core/src/lib.rs` where the private `source_cursor` was treated as a scalar `char` index in byte-slice operations, which corrupted UTF-8 continuation bytes and led to invalid string slices during in-composition typing and deletion of multibyte characters.

The implementation strictly maintains the separation of coordinate systems:
- `source_cursor`: UTF-8 **byte offset** in `state.raw_buffer` ($0 \le \text{source\_cursor} \le \text{raw\_buffer.len()}$).
- `state.cursor_pos`: Unicode scalar-value (codepoint) index in `state.preedit` ($0 \le \text{cursor\_pos} \le \text{preedit.chars().count()}$).

All 10 required acceptance criteria tests were implemented and verified. All 251 tests across all 10 core test targets in `bornomala-core` pass with zero failures and zero regressions.

---

## 2. Summary of Changes

### A. Core Engine (`crates/bornomala-core/src/lib.rs`)

1. **Portable UTF-8 Boundary Helper (`floor_char_boundary`):**
   - Implemented private inline helper function `floor_char_boundary(s: &str, mut idx: usize) -> usize` (`lib.rs:1475-1483`).
   - Guarantees finding the greatest valid UTF-8 character boundary $\le \text{idx}$ without depending on unpinned compiler features.

2. **Invariant Debug Assertions:**
   - Added `debug_assert!(self.source_cursor <= self.state.raw_buffer.len());` and `debug_assert!(self.state.raw_buffer.is_char_boundary(self.source_cursor));` unconditionally at the entry point of all three core mutation methods (`insert_char_at_cursor`, `delete_before_cursor`, `delete_after_cursor`).

3. **Byte-Safe Character Insertion (`insert_char_at_cursor`):**
   - Replaced scalar `Vec<char>` mutation with native string byte insertion: `self.state.raw_buffer.insert(self.source_cursor, c)` (`lib.rs:1325`).
   - Incremented cursor by exact UTF-8 byte length: `self.source_cursor += c.len_utf8()` (`lib.rs:1326`).
   - Preserves character boundary by construction.

4. **Boundary-Safe Backspace Deletion (`delete_before_cursor`):**
   - Slices `raw_buffer[..self.source_cursor]` and extracts the preceding scalar character via `.chars().next_back()` (`lib.rs:1343-1346`).
   - Calculates the exact character starting byte: `prev_start = self.source_cursor - prev_char.len_utf8()`.
   - Drains the complete multi-byte sequence: `self.state.raw_buffer.drain(prev_start..self.source_cursor)` (`lib.rs:1348`).
   - Positions `self.source_cursor = prev_start` (`lib.rs:1349`).

5. **Boundary-Safe Forward Delete (`delete_after_cursor`):**
   - Slices `raw_buffer[self.source_cursor..]` and extracts the following scalar character via `.chars().next()` (`lib.rs:1368`).
   - Calculates the next character ending byte: `next_end = self.source_cursor + next_char.len_utf8()`.
   - Drains `self.state.raw_buffer.drain(self.source_cursor..next_end)` (`lib.rs:1370`).
   - Leaves `self.source_cursor` at its valid byte position without shifting.

6. **Target-to-Source Proportional Interpolation (`map_target_to_source`):**
   - Preserved fast paths for `target_pos == 0 || self.spans.is_empty()` and `target_pos >= max_target` (`lib.rs:1288-1294`).
   - For interior target positions in multi-codepoint spans, computes the proportional candidate byte offset and snaps it backward using `floor_char_boundary` clamped to source span bounds (`lib.rs:1310-1314`):
     ```rust
     let candidate = span.source_range.start + offset;
     let aligned = floor_char_boundary(&self.state.raw_buffer, candidate);
     return aligned
         .max(span.source_range.start)
         .min(span.source_range.end);
     ```

### B. Internal Unit Tests (`crates/bornomala-core/src/lib.rs` under `mod tests`)
Implemented 6 focused private unit tests asserting exact byte arithmetic and invariants:
- **Test 2 (`test_internal_cursor_multibyte_insertion`):** Multibyte insertion of `'ক'` (3 bytes) and `'b'` (1 byte) verifies `source_cursor == 4` on character boundary.
- **Test 3 (`test_internal_cursor_multibyte_interior_backspace`):** Deleting 3-byte `'ক'` before `'b'` in `"কb"` at byte offset 3 verifies `source_cursor = 0` and `raw_buffer = "b"`.
- **Test 4 (`test_internal_cursor_multibyte_forward_delete`):** Forward deleting `'ক'` from `"কb"` at byte offset 0 verifies `source_cursor = 0` and `raw_buffer = "b"`.
- **Test 7 (`test_internal_cursor_pure_mapping_digraph`):** Digraph `"kh"` $\rightarrow$ `"খ"` verifies forward mapping left-affinity collapse ($0 \rightarrow 0, 1 \rightarrow 0, 2 \rightarrow 1$) and inverse mapping ($0 \rightarrow 0, 1 \rightarrow 2$).
- **Test 9 (`test_internal_cursor_proportional_interpolation_synthetic`):** Multibyte synthetic fixture `"কa"` with target span `0..2` verifies candidate byte 2 collapses to byte 0 via `floor_char_boundary`. ASCII uniform fixture `"abcdef"` verifies proportional offsets 0, 2, 4, 6.
- **Test 10 (`test_internal_cursor_empty_states_and_fast_paths`):** Verifies fast-path precedence for empty source, empty spans, target position 0, and out-of-range targets.

### C. Public Contract Integration Tests (`crates/bornomala-core/tests/contract_tests.rs`)
Implemented 4 integration tests verifying event handling and public observables:
- **Test 1 (`test_contract_cursor_ascii_baseline_interior_backspace`):** `"ami"` $\rightarrow$ `MoveCursorLeft` $\rightarrow$ `Backspace` deletes `'m'`, leaving `"ai"` and visual preedit `"আই"` at `cursor_pos = 1`.
- **Test 5 (`test_contract_cursor_boundary_deletion_handled_semantics`):** Exhaustive 6-step boundary event sequence asserting all 5 observable properties (`handled`, `raw_buffer`, `preedit`, `cursor_pos`, `is_composing`) after every event.
- **Test 6 (`test_contract_cursor_in_preedit_navigation_and_interior_insertion`):** In-preedit navigation in `"bha"` $\rightarrow$ `MoveCursorLeft` $\rightarrow$ inserting `'k'` produces `"bhka"` $\rightarrow$ `"ভকা"` at `cursor_pos = 2`.
- **Test 8 (`test_contract_cursor_public_ingress_clamping`):** Public ingress `SetCursor(1000)` clamps to preedit length 3, and subsequent `Backspace` deletes `'i'`, yielding `"am"` $\rightarrow$ `"আম"` at `cursor_pos = 2`.

---

## 3. Invariant Verification Across All Writers

| Source Cursor Writer | Location | Buffer Mutation | Invariant Guarantee |
|:---|:---|:---|:---|
| `BornoEngine::new()` | `lib.rs:255` | `raw_buffer = String::new()` | `source_cursor = 0` ($0 \le 0 \land \text{is\_char\_boundary}(0)$) |
| `BornoEngine::with_options(...)` | `lib.rs:292` | `raw_buffer = String::new()` | `source_cursor = 0` ($0 \le 0 \land \text{is\_char\_boundary}(0)$) |
| `EngineEvent::MoveCursorLeft` | `lib.rs:801` | None | `source_cursor = self.map_target_to_source(self.state.cursor_pos)` |
| `EngineEvent::MoveCursorRight` | `lib.rs:823` | None | `source_cursor = self.map_target_to_source(self.state.cursor_pos)` |
| `EngineEvent::SetCursor(pos)` | `lib.rs:845` | None | `source_cursor = self.map_target_to_source(clamped)` |
| `clear_composition()` | `lib.rs:1278` | `raw_buffer.clear()` | `source_cursor = 0` |
| `insert_char_at_cursor(c)` | `lib.rs:1325-1326` | `raw_buffer.insert(pos, c)` | `source_cursor += c.len_utf8()` (advances to next boundary) |
| `delete_before_cursor()` | `lib.rs:1348-1349` | `raw_buffer.drain(prev_start..pos)` | `source_cursor = prev_start` (valid boundary by construction) |
| `delete_after_cursor()` | `lib.rs:1370` | `raw_buffer.drain(pos..next_end)` | `source_cursor = pos` (trailing bytes shifted left to boundary) |

---

## 4. Acceptance Criteria Verification Matrix

| Acceptance Test | Scope & File | Expected Result | Actual Result | Verification Status |
|:---|:---|:---|:---|:---:|
| **Test 1** | Integration (`contract_tests.rs`) | Deleting `'m'` in `"ami"` leaves `"ai"`, preedit `"আই"`, `cursor_pos = 1` | `raw_buffer == "ai"`, `preedit == "আই"`, `cursor_pos == 1`, `is_composing == true` | **Verified** |
| **Test 2** | Internal Unit (`src/lib.rs`) | Inserting `'ক'` then `'b'` sets `len = 4`, `source_cursor = 4` on char boundary | `raw_buffer == "কb"`, `source_cursor == 4`, `is_char_boundary(4) == true`, `cursor_pos == 2` | **Verified** |
| **Test 3** | Internal Unit (`src/lib.rs`) | Backspace before `'b'` in `"কb"` removes 3-byte `'ক'`, sets `source_cursor = 0` | `raw_buffer == "b"`, `source_cursor == 0`, `is_char_boundary(0) == true`, `preedit == "ব"` | **Verified** |
| **Test 4** | Internal Unit (`src/lib.rs`) | Forward delete of `'ক'` in `"কb"` leaves `"b"`, sets `source_cursor = 0` | `raw_buffer == "b"`, `source_cursor == 0`, `is_char_boundary(0) == true`, `preedit == "ব"` | **Verified** |
| **Test 5** | Integration (`contract_tests.rs`) | Full 6-step boundary sequence across start/end no-ops and clearing | All 5 properties match verified expectations after each event | **Verified** |
| **Test 6** | Integration (`contract_tests.rs`) | In-preedit navigation in `"bha"` + insert `'k'` produces `"ভকা"`, `cursor_pos = 2` | `raw_buffer == "bhka"`, `preedit == "ভকা"`, `cursor_pos == 2`, `is_composing == true` | **Verified** |
| **Test 7** | Internal Unit (`src/lib.rs`) | Digraph `"kh"` forward maps $0 \rightarrow 0, 1 \rightarrow 0, 2 \rightarrow 1$; inverse maps $0 \rightarrow 0, 1 \rightarrow 2$ | Forward and inverse mappings verified on character boundaries | **Verified** |
| **Test 8** | Integration (`contract_tests.rs`) | `SetCursor(1000)` clamps to 3; Backspace removes `'i'`, leaving `"am"` | `raw_buffer == "am"`, `preedit == "আম"`, `cursor_pos == 2`, `is_composing == true` | **Verified** |
| **Test 9** | Internal Unit (`src/lib.rs`) | Synthetic `"কa"` candidate byte 2 collapses to byte 0; ASCII `"abcdef"` maps proportionally | Mapped offsets verified within source span bounds and on character boundaries | **Verified** |
| **Test 10** | Internal Unit (`src/lib.rs`) | Fast-path precedence: empty source, empty spans, target 0, target $\ge$ max | Fast-path returns verified against mapper precedence | **Verified** |

---

## 5. Test Execution Results Across All 10 Core Targets

All ten core test targets in `bornomala-core` were executed in the development environment and passed cleanly:

```powershell
cargo check -p bornomala-core
cargo test -p bornomala-core --lib
cargo test -p bornomala-core --test candidate_provenance_tests
cargo test -p bornomala-core --test contract_tests
cargo test -p bornomala-core --test dictionary_benchmarks
cargo test -p bornomala-core --test dictionary_tests
cargo test -p bornomala-core --test engine_config_tests
cargo test -p bornomala-core --test ffi_tests
cargo test -p bornomala-core --test layout_tests
cargo test -p bornomala-core --test phonetic_tests
cargo test -p bornomala-core --test unicode_golden_tests
rustfmt --edition 2021 --check crates/bornomala-core/tests/contract_tests.rs
```

### Target Execution Summary

| # | Test Target | Binary / Target Name | Passed | Failed | Ignored | Result |
|---|:---|:---|:---:|:---:|:---:|:---:|
| 1 | Core Library Unit Tests | `src/lib.rs` | 55 | 0 | 0 | **PASSED** |
| 2 | Candidate Provenance Tests | `tests/candidate_provenance_tests.rs` | 8 | 0 | 0 | **PASSED** |
| 3 | Contract Integration Tests | `tests/contract_tests.rs` | 17 | 0 | 0 | **PASSED** |
| 4 | Dictionary Benchmarks | `tests/dictionary_benchmarks.rs` | 5 | 0 | 0 | **PASSED** |
| 5 | Dictionary Tests | `tests/dictionary_tests.rs` | 10 | 0 | 0 | **PASSED** |
| 6 | Engine Config Tests | `tests/engine_config_tests.rs` | 8 | 0 | 0 | **PASSED** |
| 7 | FFI Tests | `tests/ffi_tests.rs` | 8 | 0 | 0 | **PASSED** |
| 8 | Fixed Layout Tests | `tests/layout_tests.rs` | 57 | 0 | 0 | **PASSED** |
| 9 | Phonetic Tests | `tests/phonetic_tests.rs` | 61 | 0 | 0 | **PASSED** |
| 10 | Unicode Golden Tests | `tests/unicode_golden_tests.rs` | 22 | 0 | 0 | **PASSED** |
| — | **Total Core Engine Tests** | | **251** | **0** | **0** | **100% PASSED** |

Formatting check on `crates/bornomala-core/tests/contract_tests.rs` passed with exit code 0 and zero diffs. Modified blocks in `crates/bornomala-core/src/lib.rs` are formatted to rustfmt specification.

---

## 6. Scope & Working Tree Audit

### Modified Files for WP-03
Only the two permitted implementation files were altered:
- `crates/bornomala-core/src/lib.rs` (+366 lines / -40 lines)
- `crates/bornomala-core/tests/contract_tests.rs` (+186 lines / -38 lines)

### Preserved Work from Prior Packages
All modifications from prior remediation phases remain preserved and untracked files are untouched:
- `crates/bornomala-core/src/phonetic/candidates.rs` (WP-02)
- `crates/bornomala-core/tests/engine_config_tests.rs` (WP-02)
- `crates/bornomala-core/tests/ffi_tests.rs` (WP-01/WP-02)
- `crates/bornomala-core/tests/phonetic_tests.rs` (WP-02)
- `native/bornomala-fcitx5/CMakeLists.txt` (WP-01)
- `native/bornomala-fcitx5/bornomala.cpp` (WP-01)
- `native/bornomala-fcitx5/scripts/install.sh` (WP-01)
- `crates/bornomala-core/tests/candidate_provenance_tests.rs` (Untracked WP-02 test suite)
- `milestones/` directory and specifications

### Generated Fixtures Notice
Running `cargo test -p bornomala-core --lib` triggers `test_export_fixtures` in `layout/builtin.rs`, which writes reference JSON files to `crates/bornomala-core/tests/fixtures/*.json`. On Windows, Git marks these modified due to line endings (LF vs CRLF), with 0 content changes. In accordance with non-destructive preservation rules, these files have not been reset or reverted.

---

## 7. Conclusion & Next Steps

WP-03 is fully implemented and verified. `crates/bornomala-core` is ready for formal completion review and closure in `milestones/remediation/INDEX.md`.
