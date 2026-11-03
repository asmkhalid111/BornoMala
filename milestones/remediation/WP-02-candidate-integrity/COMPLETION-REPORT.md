# WP-02 Completion Report — Candidate Integrity & Contract Test Alignment

> **Work Package:** WP-02 (Remediation)  
> **Status:** Closed with Documented Limitations  
> **Target Package:** `crates/bornomala-core`  
> **Execution Date:** 2026-10-26  
> **Author:** Antigravity Coding Agent  

---

## 1. Executive Summary

This report concludes **WP-02: Candidate Integrity & Contract Test Alignment** pursuant to the user closure decision (Option A: Close WP-02 as-is with documented limitations).

All synthetic candidate-fabrication mechanisms identified during the architectural audit have been permanently eliminated from `crates/bornomala-core/src/phonetic/candidates.rs`. Dependent contract, configuration, and phonetic test suites have been re-baselined to use genuine rule-derived outputs (`"shanti"`, `"tumi"`), and a dedicated provenance test suite (`candidate_provenance_tests.rs`) was established.

All 214 core engine tests across eight individually executed test targets pass with zero regressions. WP-01 changes and pre-existing audit artifacts remain strictly preserved and decoupled.

---

## 2. Final Scope Reconciliation

The WP-02 implementation is strictly confined to the five approved files:

| # | File Path | Type | Nature of Change |
|---|:---|:---:|:---|
| 1 | `crates/bornomala-core/src/phonetic/candidates.rs` | Production | Deleted backward-compatibility test fixtures (`"k"`, `"ka"`) and ad-hoc `"tumi"` literal check in ALT-1. |
| 2 | `crates/bornomala-core/tests/contract_tests.rs` | Test | Re-baselined candidate navigation, wrapping, and selection from `"ka"` to `"shanti"`; updated line 442 C-ABI assertion from `"ক্যা"` to `"কাজ"`. |
| 3 | `crates/bornomala-core/tests/engine_config_tests.rs` | Test | Re-baselined candidate display limit presentation slicing from `"ka"` to `"shanti"`. |
| 4 | `crates/bornomala-core/tests/phonetic_tests.rs` | Test | Re-baselined `test_candidate_alt1_tumi` to expect genuine rule output `["তুমি", "তুমী"]`. |
| 5 | `crates/bornomala-core/tests/candidate_provenance_tests.rs` | Test (New) | Dedicated provenance suite enforcing Slot 0 invariant, ALT-1/ALT-2 controls, 5-candidate bounds, deduplication, and static guard. |

### Decoupling from WP-01 and Pre-existing Work
The following files modified under **WP-01** remain entirely separate and were neither reverted nor modified during WP-02:
- `crates/bornomala-core/tests/ffi_tests.rs` (+27 lines: Linux privacy invariant test)
- `native/bornomala-fcitx5/CMakeLists.txt` (CMake static library resolution)
- `native/bornomala-fcitx5/bornomala.cpp` (Fcitx 5 privacy log redaction)
- `native/bornomala-fcitx5/scripts/install.sh` (Installer path cleanup)

---

## 3. Candidate Outputs Before and After WP-02

| Input | Engine Configuration | Before WP-02 (Fabricated) | After WP-02 (Rule-Derived) | Verification Evidence |
| :--- | :--- | :--- | :--- | :--- |
| `"k"` | Dict Assistance: OFF | `["ক", "খ"]` | `["ক"]` | `test_candidate_fixtures_purged` |
| `"ka"` | Dict Assistance: OFF | `["কা", "ক্যা", "খা"]` | `["কা"]` | `test_candidate_fixtures_purged` |
| `"tumi"` | Dict Assistance: OFF | `["তুমি", "তূমি"]` | `["তুমি", "তুমী"]` | `test_candidate_alt1_tumi`, `test_candidate_alt1_word_final_provenance_and_controls` |
| `"shanti"` | Dict Assistance: OFF | `["শান্তি", "শান্তী", "সান্তি"]` | `["শান্তি", "শান্তী", "সান্তি"]` | `test_candidate_navigation_wrapping_and_selection`, `test_candidate_alt2_sibilant_provenance_and_controls` |

---

## 4. Test Evidence Summary

### Verified Core Targets (214 Tests Passing)
All eight test targets within `crates/bornomala-core` were executed individually and passed:

1. `cargo test -p bornomala-core --test candidate_provenance_tests` -> **8 passed; 0 failed** (0.15s)
2. `cargo test -p bornomala-core --test contract_tests` -> **13 passed; 0 failed** (0.03s)
3. `cargo test -p bornomala-core --test engine_config_tests` -> **8 passed; 0 failed** (0.04s)
4. `cargo test -p bornomala-core --test phonetic_tests` -> **61 passed; 0 failed** (123.43s)
5. `cargo test -p bornomala-core --test layout_tests` -> **57 passed; 0 failed** (0.18s)
6. `cargo test -p bornomala-core --test dictionary_tests` -> **10 passed; 0 failed** (0.08s)
7. `cargo test -p bornomala-core --test ffi_tests` -> **8 passed; 0 failed** (0.05s)
8. `cargo test -p bornomala-core --lib` -> **49 passed; 0 failed** (0.05s)

**Total Core Passing:** **214 passed; 0 failed.**

### Documented Coverage Distinction
As confirmed during the final check, the wider test suite explicitly verifies dictionary-assisted candidate metadata variants in the live `BornoEngine` pipeline:
- `CandidateKind::ExactLexiconMatch`: verified in `dictionary_tests.rs` (`test_layer3_phonetic_ambiguity_khoma_to_k_shoma`, `test_layer3_phonetic_ambiguity_biggan_and_bhasha`) and `phonetic_tests.rs` (`test_gate_03_ambiguity_vowel_tomake_ranked_slot_1`, `test_gate_04_ambiguity_nasal_bornomala_and_negative_cases`).
- `CandidateKind::PrefixCompletion`: verified in `dictionary_tests.rs` (`test_layer3_prefix_completion_and_single_consonant_protection`).
- `CandidateKind::SpellingCorrection`: verified in `dictionary_tests.rs` (`test_layer5_typo_correction_fixture_brristi`).

The newly established `candidate_provenance_tests.rs` focuses specifically on rule-based provenance (ALT-1, ALT-2a, ALT-2b), the Slot 0 primary transliteration invariant, text-to-metadata lockstep equality, 5-candidate bounds, and anti-fabrication static tripwires. Under closure Option A, no redundant assertions were added to `candidate_provenance_tests.rs`.

---

## 5. Documented Limitations

1. **Workspace Build-Script Path Spacing Limitation:**  
   The aggregated workspace test command `cargo test --workspace` fails on Windows under the GNU toolchain during `src-tauri` (`bornomala-desktop`) build-script execution:
   ```text
   cc1.exe: fatal error: Input\: No such file or directory
   windres: preprocessing failed.
   ```
   This occurs because `tauri-winres` passes the path `E:\Codes\Bangla Input Platform\Borno V1\` unquoted to GNU `windres`. Consequently, the full workspace test suite cannot run to completion in this environment. Full-workspace verification remains incomplete and is **not** claimed as verified.
2. **Aggregated Core Command Execution:**  
   The core engine was verified target-by-target across all eight constituent test binaries (214 tests passing) rather than in a single uninterrupted invocation of `cargo test -p bornomala-core` due to the ~2-minute execution duration of the unoptimized debug benchmark in `phonetic_tests.rs`.

---

## 6. Next Work Package Recommendation

In accordance with `milestones/remediation/INDEX.md`, the next scheduled remediation work package is:

- **Work Package WP-03: Unicode In-Composition Cursor Arithmetic**  
  **Scope:** Correct `lib.rs:1316` where cursor positioning uses `self.source_cursor = idx + c.len_utf8()` (mixing UTF-8 byte lengths with character scalar counts) during in-composition cursor navigation and text editing.

Work on WP-03 will await user review and explicit authorization.
