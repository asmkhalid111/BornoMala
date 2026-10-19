# Usability and Linguistic Backlog

> **Purpose:** Formal tracking and acceptance test backlog for usability, punctuation, and linguistic refinements identified during interactive testing.  
> **Master Source of Truth:** [`ROADMAP.md`](../ROADMAP.md)  
> **Governance:** Items in this backlog are strictly mapped to their designated milestones. When a target milestone begins, its implementation plan must include the designated backlog items in its acceptance test gates.

---

## 1. Backlog Summary Table

| ID | Title | Discovered In | Target Milestone | Severity / Type | Current Status |
| :---: | :--- | :---: | :---: | :---: | :---: |
| [`BACKLOG-01`](#backlog-01-bengali-dáñṛi--phonetic-input--codepoint-typo) | Bengali Dáñṛi (`।`) Phonetic Input & Codepoint Bug | M8 | **M9** | Bug & Usability | `Deferred to M9` |
| [`BACKLOG-02`](#backlog-02-casual-vowel-ambiguity-auto-suggestion-tomake--তোমাকে) | Casual Vowel Ambiguity Auto-Suggestion (`tomake` → `তোমাকে`) | M8 | **M9 / M10** | Usability / Phonetics | `Deferred to M9/M10` |
| [`BACKLOG-03`](#backlog-03-n-twa-bidhan-nasal-suggestion-bornomala--বর্ণমালা) | ণ-ত্ব বিধান Nasal Homophone Suggestion (`bornomala` → `বর্ণমালা`) | M8 | **M9 / M10** | Usability / Lexicon | `Deferred to M9/M10` |

---

## 2. Detailed Item Specifications & Verification Tests

### BACKLOG-01: Bengali Dáñṛi (`।`) Phonetic Input & Codepoint Typo

- **Discovery Context:**  
  During Milestone M8 interactive testing in Ubuntu WSL2, typing `আমি খালিদ বলছি.` in phonetic mode emitted an ASCII period (`.`) rather than a Bengali Dáñṛi (`।`).
- **Technical Analysis:**
  1. **Codepoint Typo:** In [`crates/bornomala-core/src/phonetic/tokenizer.rs:L141`](../crates/bornomala-core/src/phonetic/tokenizer.rs#L141), the tokenizer emits `\u{09E4}` (an unassigned/currency sign in Unicode) instead of `\u{0964}` (Bengali Danda / Dari `।`), and `\u{09E5}` instead of `\u{0965}` (Double Dari `॥`).
  2. **Typing Convention:** Users expect typing `..` (double period) to emit `।` (matching standard Avro conventions), with an optional user configuration setting to auto-convert single `.` to `।`.
- **Target Milestone:** **Milestone M9 (Customization & Usability)**.
- **Resolution Plan:**
  1. Correct `\u{09E4}` $\to$ `\u{0964}` and `\u{09E5}` $\to$ `\u{0965}` in `tokenizer.rs`.
  2. Implement `..` $\to$ `।` tokenization rule in phonetic mode.
  3. Provide a configuration toggle in `BornoConfig`: `auto_convert_period_to_dari: bool` (default: false for double-dot, true if opted-in).
- **Acceptance Verification Test:**
  - **Test Case 1 (Double Dot):** Type `ami khalid bolchhi..` $\to$ Expect output `আমি খালিদ বলছি।`
  - **Test Case 2 (Single Dot with Setting Disabled):** Type `ami khalid bolchhi.` $\to$ Expect output `আমি খালিদ বলছি.`
  - **Test Case 3 (Single Dot with Setting Enabled):** Type `ami khalid bolchhi.` $\to$ Expect output `আমি খালিদ বলছি।`

---

### BACKLOG-02: Casual Vowel Ambiguity Auto-Suggestion (`tomake` → `তোমাকে`)

- **Discovery Context:**  
  Typing `tomake` emits `তমাকে` with no candidate suggestion for `তোমাকে`, even though `তোমাকে` exists in `lexicon.tsv` (rank 8644).
- **Technical Analysis:**
  1. In Avro phonetic grammar ([`rules.rs:L166-L172`](../crates/bornomala-core/src/phonetic/rules.rs#L166-L172)), lowercase `o` is the *inherent vowel bridge* (`অ`, `dependent: None`). Lowercase `o` emits no dependent vowel kar.
  2. The explicit o-kar (`ো`) is mapped to uppercase `O` (`tOmake` $\to$ `তোমাকে`).
  3. Milestone M6 introduced the *Phonetic Ambiguity Pipeline* ([`ambiguity.rs`](../crates/bornomala-core/src/dictionary/ambiguity.rs)), but restricted rewrites to consonant homophones (`kh` $\to$ `ক্ষ`, `gg` $\to$ `জ্ঞ`, `sh` $\to$ `ষ`) to prevent combinatorial latency explosion.
- **Target Milestone:** **Milestone M9 / M10 (Customization / Prediction)**.
- **Resolution Plan:**
  1. Add a bounded vowel-alternation rewrite rule: where an inherent `o` follows a consonant, generate a candidate variant with o-kar (`O` $\to$ `ো`).
  2. Gate the variant against the embedded trie (`PrefixTrie::exact_search`). Because `তোমাকে` exists in the lexicon, it will surface with origin `CandidateKind::ExactLexiconMatch` (badge `lexicon`).
- **Acceptance Verification Test:**
  - **Test Case 1:** Type `tomake` in phonetic mode.
  - **Candidate 0:** `তমাকে` (primary transliteration).
  - **Candidate 1:** `তোমাকে` (verified lexicon candidate).
  - **Commit Verification:** Pressing `2` or selecting candidate 1 commits `তোমাকে`.

---

### BACKLOG-03: ণ-ত্ব বিধান Nasal Homophone Suggestion (`bornomala` → `বর্ণমালা`)

- **Discovery Context:**  
  In standard Bengali grammar (ণ-ত্ব বিধান / *N-twa Bidhan*), Tatsama words with Reph (`র্`) require মূর্ধন্য-ণ (`ণ`), making **`বর্ণমালা`** the grammatically correct spelling. However, typing casual lowercase `bornomala` emits `বর্নমালা` (dental `ন`), requiring the user to know that uppercase `N` must be typed (`borNomala`).
- **Technical Analysis:**
  1. Lowercase `n` maps to `ন` ([`rules.rs:L628`](../crates/bornomala-core/src/phonetic/rules.rs#L628)).
  2. Uppercase `N` maps to `ণ` ([`rules.rs:L613`](../crates/bornomala-core/src/phonetic/rules.rs#L613)).
  3. Casual users frequently type all-lowercase Latin strings for Bengali words with `ণ`.
- **Target Milestone:** **Milestone M9 / M10 (Customization / Prediction)**.
- **Resolution Plan:**
  1. Add a phonetic ambiguity rewrite rule for post-Reph nasals: rewrite `rn` $\to$ `rN` (`borNomala`).
  2. Validate against the lexicon trie: `বর্ণমালা` exists in the dictionary, so `বর্ণমালা` will be surfaced as a verified lexicon candidate.
- **Acceptance Verification Test:**
  - **Test Case 1:** Type `bornomala` in phonetic mode.
  - **Candidate 0:** `বর্নমালা` (primary transliteration).
  - **Candidate 1:** `বর্ণমালা` (verified lexicon candidate with origin `lexicon`).
  - **Commit Verification:** Selecting candidate 1 commits `বর্ণমালা`.

---

## 3. Verification Protocol

When closing any target milestone:
1. Open this file.
2. Execute the automated test command:
   ```bash
   cargo test -p bornomala-core
   ```
3. Run the interactive manual test in both Windows TSF and Linux Fcitx 5.
4. Update the item's status in Section 1 to `Verified` with date and commit hash.
