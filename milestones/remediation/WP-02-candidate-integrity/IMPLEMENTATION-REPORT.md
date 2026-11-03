# WP-02 Implementation Report — Candidate Integrity & Contract Test Alignment

> **Work Package:** WP-02 (Remediation)  
> **Status:** Implemented — Awaiting Review  
> **Target Package:** `crates/bornomala-core`  
> **Execution Date:** 2026-10-26  
> **Author:** Antigravity Coding Agent  

---

## 1. Executive Summary

This report documents the implementation of **WP-02: Candidate Integrity & Contract Test Alignment** in accordance with `milestones/remediation/WP-02-candidate-integrity/SPECIFICATION.md`.

All candidate-fabrication mechanisms identified during the architectural audit have been purged from `crates/bornomala-core/src/phonetic/candidates.rs`. The dependent contract, configuration, and phonetic test suites have been re-baselined to use genuine, rule-derived outputs (`"shanti"`, `"tumi"`), and a dedicated provenance test suite (`candidate_provenance_tests.rs`) has been introduced to permanently guard against future candidate fabrication.

Scope has been strictly preserved: exactly four existing files were modified, and one new test file was created. No production code outside `candidates.rs` was altered.

---

## 2. Summary of Changes

### A. Production Engine (`crates/bornomala-core/src/phonetic/candidates.rs`)
1. **Removed Backward Compatibility Fixtures:** Purged lines 73–80, which previously injected synthetic candidate lists for `"k"` (`["খ"]`) and `"ka"` (`["ক্যা", "খা"]`).
2. **Removed Literal Branch in ALT-1:** Purged lines 27–28 (`if raw_buffer == "tumi" { push "তূমি" }`), restoring pure rule-based word-final high-vowel alternation (`ি` -> `ী` for words ending in `'i'`, and `ু` -> `ূ` for words ending in `'u'`).

### B. Dependent Test Suites
1. **Contract Tests (`crates/bornomala-core/tests/contract_tests.rs`):**
   - Re-baselined `test_candidate_navigation_wrapping_and_selection` from fabricated `"ka"` to genuine rule-derived `"shanti"` (`["শান্তি", "শান্তী", "সান্তি"]`). Verified forward wrapping (`0 -> 1 -> 2 -> 0`), reverse wrapping (`0 -> 2 -> 1 -> 0`), and committing highlighted index 1 (`"শান্তী"`).
   - Re-baselined `test_candidate_direct_selection_and_invalid_index` to `"shanti"`. Verified out-of-bounds rejection (index 99) and direct candidate selection (index 2 commits `"সান্তি"`).
   - Updated `test_c_abi_process_event_parity_and_cleanup` assertion for candidate index 1 selection from deleted mock fixture `"ক্যা"` to genuine lexicon candidate `"কাজ"`.
2. **Configuration Tests (`crates/bornomala-core/tests/engine_config_tests.rs`):**
   - Re-baselined `test_engine_candidate_display_limit_presentation_slicing` from `"ka"` to `"shanti"`. Verified that candidate display limit 2 slices to `["শান্তি", "শান্তী"]` and display limit 5 expands to all 3 rule-derived candidates `["শান্তি", "শান্তী", "সান্তি"]`.
3. **Phonetic Tests (`crates/bornomala-core/tests/phonetic_tests.rs`):**
   - Updated `test_candidate_alt1_tumi` to assert genuine rule output `vec!["তুমি", "তুমী"]`, eliminating the expectation of the fabricated initial vowel variant `"তূমি"`.

### C. Automated Provenance & Anti-Fabrication Suite (`crates/bornomala-core/tests/candidate_provenance_tests.rs`)
Created a comprehensive test suite enforcing:
- Fixture eradication: verifies `"k"` strictly yields `["ক"]` and `"ka"` strictly yields `["কা"]` with dictionary disabled.
- Behavioral provenance for Rule ALT-1 (word-final high vowels) with positive controls (`"ami"`, `"tumi"`, `"pakhi"`, `"babu"`) and negative controls (`"amra"`, `"kolom"`).
- Behavioral provenance for Rule ALT-2 (sibilant alternation: initial ALT-2a homophones and medial ALT-2b retroflex clusters) with positive controls (`"shanto"`, `"shanti"`, `"shandho"`, `"kosTo"`) and negative controls (`"shob"`, `"sundor"`, `"kosto"`).
- Candidate metadata lockstep: verifies 1:1 text correspondence between `candidates` and `candidate_details`, Slot 0 `PrimaryTransliteration` invariant, and `PhoneticAlternative` classification.
- Invariant bounds: strict <= 5 capacity cap and deduplication across all candidate lists.
- Narrow static source guard tripwire: verifies `candidates.rs` contains zero literal comparisons against `raw_buffer`.

---

## 3. Before-and-After Candidate Output Comparison

| Input | Engine Configuration | Before WP-02 (Fabricated) | After WP-02 (Rule-Derived) | Verification Evidence |
| :--- | :--- | :--- | :--- | :--- |
| `"k"` | Dict Assistance: OFF | `["ক", "খ"]` | `["ক"]` | `test_candidate_fixtures_purged` |
| `"ka"` | Dict Assistance: OFF | `["কা", "ক্যা", "খা"]` | `["কা"]` | `test_candidate_fixtures_purged` |
| `"tumi"` | Dict Assistance: OFF | `["তুমি", "তূমি"]` | `["তুমি", "তুমী"]` | `test_candidate_alt1_tumi`, `test_candidate_alt1_word_final_provenance_and_controls` |
| `"shanti"` | Dict Assistance: OFF | `["শান্তি", "শান্তী", "সান্তি"]` | `["শান্তি", "শান্তী", "সান্তি"]` | `test_candidate_navigation_wrapping_and_selection`, `test_candidate_alt2_sibilant_provenance_and_controls` |

---

## 4. Candidate Metadata Alignment & Provenance Evidence

The engine's candidate generation and ranking pipeline maintains a parallel metadata vector `candidate_details: Vec<CandidateDetail>` in exact 1:1 lockstep with `candidates: Vec<String>`.

Evidence collected from `candidate_provenance_tests.rs`:

```rust
// Test: test_candidate_metadata_lockstep_without_dictionary
// Input: "shanti", Dictionary Assistance: OFF
candidates:        ["শান্তি", "শান্তী", "সান্তি"]
candidate_details: [
    CandidateDetail { text: "শান্তি", kind: CandidateKind::PrimaryTransliteration, score: 0 },
    CandidateDetail { text: "শান্তী", kind: CandidateKind::PhoneticAlternative,     score: 0 },
    CandidateDetail { text: "সান্তি", kind: CandidateKind::PhoneticAlternative,     score: 0 },
]
// 1:1 Lockstep Text: state.candidate_details[i].text == state.candidates[i] (Passed)
// Slot 0 Invariant: state.candidate_details[0].kind == PrimaryTransliteration (Passed)
```

With dictionary assistance enabled:
```rust
// Test: test_candidate_metadata_lockstep_with_dictionary
// Input: "k", Dictionary Assistance: ON
candidates[0]:         "ক"
candidate_details[0]:  CandidateDetail { text: "ক", kind: CandidateKind::PrimaryTransliteration, score: 0 }
candidates[1..]:       Lexicon completions (ExactLexiconMatch / PrefixCompletion)
// 1:1 Lockstep Text & Slot 0 Preedit Alignment: (Passed)
```

---

## 5. Test Execution Results

### Focused Test Suites
1. **Candidate Provenance Tests:**
   - Command: `cargo test -p bornomala-core --test candidate_provenance_tests`
   - Exit Code: `0`
   - Result: `ok. 8 passed; 0 failed; 0 ignored; finished in 0.15s`
2. **Contract Tests:**
   - Command: `cargo test -p bornomala-core --test contract_tests`
   - Exit Code: `0`
   - Result: `ok. 13 passed; 0 failed; 0 ignored; finished in 0.03s`
3. **Engine Config Tests:**
   - Command: `cargo test -p bornomala-core --test engine_config_tests`
   - Exit Code: `0`
   - Result: `ok. 8 passed; 0 failed; 0 ignored; finished in 0.04s`
4. **Phonetic Tests:**
   - Command: `cargo test -p bornomala-core --test phonetic_tests`
   - Exit Code: `0`
   - Result: `ok. 61 passed; 0 failed; 0 ignored; finished in 123.43s`
5. **Layout Tests:**
   - Command: `cargo test -p bornomala-core --test layout_tests`
   - Exit Code: `0`
   - Result: `ok. 57 passed; 0 failed; 0 ignored; finished in 0.18s`
6. **Dictionary Tests:**
   - Command: `cargo test -p bornomala-core --test dictionary_tests`
   - Exit Code: `0`
   - Result: `ok. 10 passed; 0 failed; 0 ignored; finished in 0.08s`
7. **FFI Tests:**
   - Command: `cargo test -p bornomala-core --test ffi_tests`
   - Exit Code: `0`
   - Result: `ok. 8 passed; 0 failed; 0 ignored; finished in 0.05s`
8. **Unit Tests (`lib.rs`):**
   - Command: `cargo test -p bornomala-core --lib`
   - Exit Code: `0`
   - Result: `ok. 49 passed; 0 failed; 0 ignored; finished in 0.05s`

**Total `bornomala-core` automated tests executed and passing:** **214 passed; 0 failed.**

### Workspace Test Command Execution
- Command: `cargo test --workspace`
- Exit Code: `1`
- Detailed Finding: As documented in previous milestone reports and `SPECIFICATION.md`, `cargo test --workspace` fails on the Windows GNU toolchain during the build script of the second workspace member (`bornomala-desktop` / `src-tauri`) because `tauri-winres` passes the workspace directory path `E:\Codes\Bangla Input Platform\Borno V1\` unquoted to GNU `windres`, which splits on the spaces in `Input Platform\Borno V1`.
- The Rust core engine package compiles cleanly and passes 100% of its tests via `cargo test -p bornomala-core`.

---

## 6. Git Status and Diff Summary

### Git Status (`git status --porcelain`)
```text
 M crates/bornomala-core/src/phonetic/candidates.rs
 M crates/bornomala-core/tests/contract_tests.rs
 M crates/bornomala-core/tests/engine_config_tests.rs
 M crates/bornomala-core/tests/ffi_tests.rs            (WP-01)
 M crates/bornomala-core/tests/phonetic_tests.rs
 M native/bornomala-fcitx5/CMakeLists.txt              (WP-01)
 M native/bornomala-fcitx5/bornomala.cpp               (WP-01)
 M native/bornomala-fcitx5/scripts/install.sh          (WP-01)
?? crates/bornomala-core/tests/candidate_provenance_tests.rs
?? milestones/AUDIT-2026-10-23.md
?? milestones/remediation/
?? milestones/revisions/
```

### Git Diff Stat (`git diff --stat crates/bornomala-core/`)
```text
 crates/bornomala-core/src/phonetic/candidates.rs   | 13 +-------
 crates/bornomala-core/tests/contract_tests.rs      | 36 +++++++++++-----------
 crates/bornomala-core/tests/engine_config_tests.rs | 14 ++++-----
 crates/bornomala-core/tests/ffi_tests.rs           | 27 ++++++++++++++++ (WP-01)
 crates/bornomala-core/tests/phonetic_tests.rs      |  2 +-
 5 files changed, 54 insertions(+), 38 deletions(-)
```

---

## 7. Unresolved Issues, Deviations, or Limitations

1. **No Production Architecture Creep:** Zero production logic outside `candidates.rs` was modified.
2. **Deterministic Multi-Candidate Selection:** Using `"shanti"` for contract navigation, wrapping, and selection was confirmed to satisfy all contract requirements without any artificial fixtures or loose assertions.
3. **Workspace Path Spacing Incompatibility in `bornomala-desktop`:** `tauri-winres` failure under Windows GNU toolchain when the workspace path contains spaces remains an external tooling issue confined to `src-tauri` desktop packaging, not a defect in `bornomala-core`.
4. **Governance Status:** In accordance with instructions, WP-02 is NOT marked as complete or verified. This report is submitted for review.
