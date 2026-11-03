# BornoMala — Final Implementation-Readiness Review

> **Document Type:** Architectural Review & Risk Assessment  
> **Date:** 2026-10-26  
> **Commit Inspected:** `2105591`  
> **Status:** Authoritative Baseline for Remediation

---

## 1. Findings That Remain Confirmed

1. **Test-Only Fixtures in Production Engine Code:**
   - **Location:** `crates/bornomala-core/src/phonetic/candidates.rs:75-80`
   - **Evidence:** Explicitly commented as `// Backward Compatibility Fixtures for Engine Contract Tests`. Hardcodes `"k"` -> `["খ"]` and `"ka"` -> `["ক্যা", "খা"]`.
   - **Impact:** Directly satisfies `tests/contract_tests.rs:148-151` artificially while bypassing the real dictionary and phonetic pipeline.

2. **Isolated Ad-Hoc Literal in Phonetic Rules:**
   - **Location:** `crates/bornomala-core/src/phonetic/candidates.rs:27-28`
   - **Evidence:** `if raw_buffer == "tumi" { Self::push_if_unique(&mut candidates, "তূমি".to_string()); }` is placed inside Rule ALT-1 ("Word-Final High Vowel Orthographic Alternation") despite being a stem/medial vowel alteration.

3. **Active Keystroke/Committed Text Logging in Linux Adapter:**
   - **Location:** `native/bornomala-fcitx5/bornomala.cpp:184, 405-408`
   - **Evidence:** `FCITX_INFO() << "Committed: " << text;` and `FCITX_INFO() << "Preedit updated: " << ...;` write raw typed content and keystrokes to standard system logs.
   - **Impact:** Violates the mandatory offline/privacy requirement in AGENTS.md Section 3.4 and ROADMAP.md Section 1.1 Goal 8.

4. **Configuration Path Disconnection Between Tauri and Native Adapters:**
   - **Location:** `src-tauri/tauri.conf.json:5` vs `native/borno-tsf/borno_tsf.cpp:86` and `native/bornomala-fcitx5/bornomala.cpp:15`
   - **Evidence:** Tauri uses identifier `"com.bornomala.sandbox"`, writing to `%APPDATA%\com.bornomala.sandbox\config.json` (confirmed present on host). Both native adapters read strictly from `%APPDATA%\com.bornomala.desktop\config.json` (confirmed absent on host).
   - **Impact:** Settings saved from the desktop GUI never reach the native Windows TSF or Linux Fcitx 5 engines.

5. **Header Discrepancy (`borno.h` vs `bornomala.h`):**
   - **Location:** `crates/bornomala-core/include/borno.h:23-28` vs `crates/bornomala-core/include/bornomala.h:23-29`
   - **Evidence:** `bornomala.h` defines `BORNO_ERR_INVALID_CONFIG` (`-5`), `BORNO_ERR_UNSUPPORTED_SCHEMA` (`-6`), and declares `borno_engine_configure_json`. `borno.h` lacks them completely. Both native adapters include `bornomala.h`.

6. **Dimensional Cursor Arithmetic Defect:**
   - **Location:** `crates/bornomala-core/src/lib.rs:1316`
   - **Evidence:** `self.source_cursor = idx + c.len_utf8();` adds UTF-8 byte length to `idx` (a `Vec<char>` scalar index). In contrast, line 1330 decrements by scalar count `saturating_sub(1)`.

7. **Linux Build Artifact Path Hardcoding:**
   - **Location:** `native/bornomala-fcitx5/CMakeLists.txt:15`
   - **Evidence:** Hardcodes `/home/asmkhalid/...` as primary library path, with fallback to `../../crates/bornomala-core/target/release/` instead of the root Cargo workspace target `../../target/release/`.

8. **Third-Party Attribution Notice Omission:**
   - **Location:** `docs/resources-and-licenses.md:29` and `LICENSE-MIT`
   - **Evidence:** Documentation claims Leipzig CC BY 4.0 attribution is included in `LICENSE`, but `LICENSE-MIT` contains only standard MIT copyright text. No `THIRD-PARTY-NOTICES.md` exists.

---

## 2. Findings Downgraded, Qualified, or Requiring More Evidence

1. **FFI Error Code `-5` Status (Corrected from "Prescribed Change" to "Existing ABI"):**
   - Error codes `-5` (`BORNO_ERR_INVALID_CONFIG`) and `-6` (`BORNO_ERR_UNSUPPORTED_SCHEMA`) are already implemented in `ffi.rs:29-30` and declared in `bornomala.h:28-29`. Legacy `borno.h` is merely un-synchronized.
2. **FFI Struct Destruction on Early Error (Qualified):**
   - On early error returns, Rust does not touch `out_result`. Callers must zero-initialize output structs to avoid double-free or dereferencing garbage memory if destructors are invoked.
3. **`catch_unwind` Safety Boundaries (Qualified):**
   - `catch_unwind` protects against unwinding panics in Rust; it does not protect against invalid, misaligned, or dangling pointers provided by C/C++ callers.
4. **Benchmark Assertion in Debug Mode (Qualified):**
   - `assert!(p95 <= 50.0)` in `tests/dictionary_benchmarks.rs:54` is gated by `#[cfg(not(debug_assertions))]`. This is an intentional design choice; performance verification must run via `cargo test --release`.
5. **M8 Classification (Qualified):**
   - M8 is not "Blocked solely because testing requires Linux." It is `Implemented — not verified (environment-dependent)` with blocking code defects (privacy logging and CMake path hardcoding).

---

## 3. Work Package Sequencing

```
[M9.1 Remediation & Alignment]
  WP-01: Linux Fcitx 5 Privacy & Build Integrity (COMPLETED)
  WP-02: Candidate Generator & Contract Test Realignment (NEXT)
  WP-03: Unicode In-Composition Cursor Arithmetic Correction
  WP-04: C-ABI Header Unification (borno.h -> bornomala.h)
  WP-05: Desktop App Identifier Synchronization & Safe Migration
  WP-06: Licensing Compliance (Leipzig CC BY 4.0 Notice)
         │
         ▼ [GATE: All tests passing, clean launch, no regressions]
[M9.2 Customization & Usability (Original M9 Scope)]
  WP-07: Punctuation & Auto-Dari Toggle
  WP-08: Candidate Presentation & Themes
  WP-09: Shortcut Key Configuration
```

---

## 4. Criterion-Specific Status Recommendations for M1–M8

- **M0, M0.1, M4:** Retain as `Verified`.
- **M1:** `Implemented — not verified` (candidate navigation contract tests rely on test fixtures).
- **M2:** `Implemented — not verified` (non-ASCII in-composition cursor arithmetic defect).
- **M3:** `Implemented — not verified` (ALT-1 contains ad-hoc `"tumi"` literal).
- **M5:** `Implemented — not verified` (settings path mismatch prevents native adapters from receiving UI config).
- **M6:** `Implemented — not verified` (Leipzig CC BY 4.0 attribution notice missing from license file).
- **M7:** `Implemented — not verified` (requires config synchronization to canonical path and elevated installer verification).
- **M8:** `Implemented — not verified (environment-dependent)` (requires live Linux runtime verification).
