# Work Package WP-02: Candidate Integrity & Contract Test Alignment — Specification

> **Work Package ID:** `WP-02`  
> **Status:** Specified — Ready for Implementation  
> **Date:** 2026-10-26  
> **Target Scope:** Core Engine Phonetic Candidate Generator (`crates/bornomala-core/src/phonetic/candidates.rs`) & Dependent Contract/Config Tests  
> **Source Documents:** `milestones/remediation/FINAL-IMPLEMENTATION-READINESS-REVIEW.md` (Finding 1 & 2), `milestones/revisions/RV-01.1-candidate-integrity.md`  

---

## 1. Objectives

1. **Purge Test Fixtures from Production Engine Code:**
   - Remove the `"Backward Compatibility Fixtures for Engine Contract Tests"` block in `crates/bornomala-core/src/phonetic/candidates.rs:75-80` that hardcodes:
     - `"k"` -> `["খ"]`
     - `"ka"` -> `["ক্যা", "খা"]`
2. **Purge Ad-Hoc Literal from Phonetic Rules:**
   - Remove `if raw_buffer == "tumi" { push("তূমি") }` from Rule ALT-1 in `candidates.rs:27-28`.
   - Restore Rule ALT-1 as a pure rule-based trailing high-vowel diacritic alternation (`.ends_with('i')` / `.ends_with('u')`).
3. **Re-Baseline Contract & Configuration Tests onto Genuine Rule-Derived Outputs:**
   - Re-baseline `crates/bornomala-core/tests/contract_tests.rs:148-186` (`test_candidate_navigation_wrapping_and_selection` and `test_candidate_direct_selection_and_invalid_index`) onto genuine multi-candidate engine output (`"shanti"` -> `["শান্তি", "শান্তী", "সান্তি"]`).
   - Re-baseline `crates/bornomala-core/tests/engine_config_tests.rs:11-32` (`test_engine_candidate_display_limit_presentation_slicing`) using `"shanti"` to preserve the 3-candidate presentation limit slicing verification.
   - Update `crates/bornomala-core/tests/phonetic_tests.rs:376-379` (`test_candidate_alt1_tumi`) to assert real rule-derived candidates `["তুমি", "তুমী"]`.
4. **Implement Behavioral Provenance and Anti-Regression Tests:**
   - Add dedicated provenance and regression checks in `crates/bornomala-core/tests/candidate_provenance_tests.rs` validating:
     - Primary transliteration invariant (Slot 0).
     - Strict 5-candidate cap and deduplication.
     - Behavioral provenance of ALT-1 and ALT-2 alternations.
     - Absence of historical fixture fabrications (`"k"`, `"ka"`, `"tumi"`).
     - Static guard tripwire asserting no literal string comparisons on `raw_buffer`.

---

## 2. Invariants & Governance Rules

- **Zero Decorative Placeholders (AGENTS.md §1.3):** Production candidate generation must be a pure, deterministic function of the phonetic rules and dictionary layers. No test fixtures or ad-hoc input bypasses.
- **No Production `#[cfg(test)]` Bypasses:** The production engine binary under test must be the exact binary that ships to users.
- **Preservation of Documented Contracts:** Re-baselining means aligning test expectations with documented product contracts and genuine rule outputs—not weakening assertions or skipping navigation branches.
- **Candidate Ordering & Capacity Semantics:**
  - Candidate 0 is always the primary NFC-normalized transliteration (Slot 0 Invariant).
  - Total candidate count is strictly capped at maximum 5.
  - Deduplication via `push_if_unique` is preserved.
- **Continuous Regression Discipline:** All pre-existing workspace tests must pass.

---

## 3. Detailed Technical Analysis

### A. Candidate Provenance Architecture
Candidate provenance is established through existing core engine contracts without introducing artificial production architecture:
1. **Existing Lockstep Metadata (`CandidateKind`):**
   `EngineState` already maintains `candidate_details: Vec<CandidateDetail>` in 1:1 lockstep with `candidates: Vec<String>`. Every candidate emitted by `BornoEngine` is explicitly classified:
   - `CandidateKind::PrimaryTransliteration`: Deterministic primary transliteration from phonetic rules (Slot 0).
   - `CandidateKind::PhoneticAlternative`: Rule-based alternates from M3 candidate generator (ALT-1, ALT-2).
   - `CandidateKind::ExactLexiconMatch`: Lexicon trie match from phonetic ambiguity rewrites.
   - `CandidateKind::PrefixCompletion`: Predictive lexicon trie completion.
   - `CandidateKind::SpellingCorrection`: Bounded Damerau-Levenshtein edit-distance suggestion.
2. **Behavioral Mechanism Isolation:**
   Rather than relying solely on string output matching, tests isolate specific phonetic mechanisms:
   - **ALT-1 Isolation (Positive & Negative):**
     - Input `"ami"` (ends with `'i'`): Primary is `"আমি"`, alternate is `"আমী"` (verified: root `"আম"` invariant; trailing diacritic transforms U+09BF -> U+09C0).
     - Input `"babu"` (ends with `'u'`): Primary is `"বাবু"`, alternate is `"বাবূ"` (verified: root `"বাব"` invariant; trailing diacritic transforms U+09C1 -> U+09C2).
     - Input `"amra"` (ends with `'a'`): Negative control. ALT-1 must not trigger. Exactly 1 candidate (`"আমরা"`).
   - **ALT-2a Isolation (Initial Sibilant Homophone):**
     - Input `"shanto"` (starts with `"shan"` + primary starts with `শ`): Primary is `"শান্ত"`, alternate is `"সান্ত"` (verified: leading sibilant transforms U+09B6 -> U+09B8; cluster `"ন্ত"` invariant).
     - Input `"shob"` (does not match `"shan"`/`"shant"`/`"shand"`): Negative control. ALT-2a must not trigger.
   - **ALT-2b Isolation (Medial Sibilant Cluster):**
     - Input `"kosTo"` (contains `"sT"`): Primary contains `"স্ট"` / `"ষ্ট"`, alternate contains the sibilant cluster swap.

### B. Validation of `"tumi"` Expected Behavior
- **Phonetic Pipeline Trace:**
  - Raw input: `"tumi"`
  - Tokenization: `t` (`ত`) + `u` (`ু`, U+09C1) + `m` (`ম`) + `i` (`ি`, U+09BF) -> `"তুমি"`.
  - Primary transliteration: `"তুমি"` (Candidate 0).
- **Rule ALT-1 Definition:**
  Rule ALT-1 is titled `Word-Final High Vowel Orthographic Alternation` and conditioned on `ends_with('i')` or `ends_with('u')`. Its linguistic purpose is swapping word-final hrasva/dirgha high vowels (`ি` <-> `ী` and `ু` <-> `ূ`).
- **Dissection of Legacy Fabrication (`"তূমি"`):**
  The legacy branch `if raw_buffer == "tumi" { push("তূমি") }` modified the **medial/stem** vowel on `t` (`ু` -> `ূ`) while leaving the word-final vowel `ি` untouched. This directly contradicted Rule ALT-1's own definition and scope.
- **Genuine Output:**
  When the ad-hoc literal is removed, `"tumi"` naturally triggers `ends_with('i')`. The trailing `ি` alternates to `ী`, producing `"তুমী"`.
  - Candidate 0: `"তুমি"` (`PrimaryTransliteration`)
  - Candidate 1: `"তুমী"` (`PhoneticAlternative`)
  - Expected output without dictionary assistance: `["তুমি", "তুমী"]`.

### C. Contract Test Realignment (Preserving Full Test Contracts)
Existing contract tests (`contract_tests.rs` and `engine_config_tests.rs`) were written under M1 assuming `"ka"` produces 3 candidates (`["কা", "ক্যা", "খা"]`).
- When fixtures are removed, `"ka"` produces exactly 1 candidate (`["কা"]`), because `'a'` does not trigger ALT-1 or ALT-2. A 1-candidate list cannot test 3-item navigation or wraparound.
- **Linguistic Multi-Candidate Replacement (`"shanti"`):**
  Input `"shanti"` rule-derivatively triggers both ALT-1 and ALT-2a without dictionary assistance:
  1. Candidate 0: `"শান্তি"` (Primary transliteration)
  2. Candidate 1: `"শান্তী"` (ALT-1 word-final `'i'` diacritic alternation `ি` -> `ী`)
  3. Candidate 2: `"সান্তি"` (ALT-2a initial sibilant homophone alternation `শ` -> `স`)
- **Contract Preservation:**
  Using `"shanti"` preserves 100% of the navigation contract:
  - `NextCandidate`: `None` -> `Some(0)` (`"শান্তি"`) -> `Some(1)` (`"শান্তী"`) -> `Some(2)` (`"সান্তি"`) -> wraps to `Some(0)`.
  - `PrevCandidate`: `Some(0)` -> wraps to `Some(2)` -> `Some(1)` -> `Some(0)`.
  - Direct selection: `SelectCandidate(1)` highlights `"শান্তী"`.
  - Invalid index: `SelectCandidate(99)` is rejected.
  - Commit: committing at index 1 commits `"শান্তী"`.
  - Presentation limit slicing (`engine_config_tests.rs`): limit 2 slices to `["শান্তি", "শান্তী"]`; limit 5 displays all 3.

### D. Scoping the Static Guard vs Behavioral Protection
- **Role of the Static Guard:**
  A test reading `candidates.rs` via `include_str!` asserting `!content.contains("raw_buffer ==")` acts as a lightweight, immediate tripwire against accidental reintroduction of hardcoded literal branches.
- **Limitations of Static Inspection:**
  A static check cannot verify runtime correctness, cannot detect indirect lookup tables, and does not validate whether candidate generation satisfies linguistic rules.
- **Principal Regression Protection:**
  The primary protection is provided by behavioral tests that assert specific rule triggers, negative controls, de-fixture checks, and `CandidateKind` metadata integrity.

---

## 4. Before-and-After Output Comparison

| Input | Engine Configuration | Before WP-02 (Fabricated) | After WP-02 (Rule-Derived) | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `"k"` | Dict Assistance: OFF | `["ক", "খ"]` | `["ক"]` | Removed fixture `"খ"`. |
| `"ka"` | Dict Assistance: OFF | `["কা", "ক্যা", "খা"]` | `["কা"]` | Removed fixtures `"ক্যা"`, `"খা"`. |
| `"tumi"` | Dict Assistance: OFF | `["তুমি", "তূমি"]` | `["তুমি", "তুমী"]` | Removed ad-hoc `"তূমি"`; genuine ALT-1 yields `"তুমী"`. |
| `"shanti"` | Dict Assistance: OFF | `["শান্তি", "শান্তী", "সান্তি"]` | `["শান্তি", "শান্তী", "সান্তি"]` | Proven multi-candidate rule derivation for contract tests. |

---

## 5. Files Impacted

1. `crates/bornomala-core/src/phonetic/candidates.rs`:
   - Delete lines 27–28 (`raw_buffer == "tumi"`).
   - Delete lines 73–80 (`// Backward Compatibility Fixtures ...`).
2. `crates/bornomala-core/tests/contract_tests.rs`:
   - Re-baseline `test_candidate_navigation_wrapping_and_selection` using `"shanti"`.
   - Re-baseline `test_candidate_direct_selection_and_invalid_index` using `"shanti"`.
3. `crates/bornomala-core/tests/engine_config_tests.rs`:
   - Re-baseline `test_engine_candidate_display_limit_presentation_slicing` using `"shanti"`.
4. `crates/bornomala-core/tests/phonetic_tests.rs`:
   - Re-baseline `test_candidate_alt1_tumi` to expect `["তুমি", "তুমী"]`.
5. `crates/bornomala-core/tests/candidate_provenance_tests.rs`:
   - New automated provenance, invariant, and tripwire suite.

---

## 6. Acceptance Criteria

| # | Criterion | Verification Method |
|---|---|---|
| **AC-1** | `candidates.rs` contains zero comparisons of `raw_buffer` to string literals. | Source inspection & static guard test in `candidate_provenance_tests.rs` |
| **AC-2** | Input `"ka"` produces strictly `["কা"]` without dictionary assistance (no fabricated `"ক্যা"` or `"খা"`). | Unit test assertion in `candidate_provenance_tests.rs` |
| **AC-3** | Input `"k"` produces strictly `["ক"]` without dictionary assistance (no fabricated `"খ"`). | Unit test assertion in `candidate_provenance_tests.rs` |
| **AC-4** | Input `"tumi"` produces strictly `["তুমি", "তুমী"]` without dictionary assistance (no fabricated `"তূমি"`). | Unit test assertion in `phonetic_tests.rs` & provenance tests |
| **AC-5** | Candidate navigation, wrapping, direct selection, and invalid index handling pass using rule-derived `"shanti"`. | `cargo test -p bornomala-core --test contract_tests` |
| **AC-6** | Candidate display limit presentation slicing passes using rule-derived `"shanti"`. | `cargo test -p bornomala-core --test engine_config_tests` |
| **AC-7** | Candidate provenance test suite validates Slot 0 invariant, 5-candidate cap, deduplication, and lockstep `CandidateKind` metadata. | `cargo test -p bornomala-core --test candidate_provenance_tests` |
| **AC-8** | Full workspace test suite passes with zero regressions. | `cargo test --workspace` |
| **AC-9** | Git diff review confirms changes are strictly confined to the 5 targeted files with no architecture creep. | `git diff --stat` inspection |
