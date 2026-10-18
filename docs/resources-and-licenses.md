# Resources and Licenses

> **Project:** Bangla Input Platform (Borno)  
> **Milestone:** M6 — Dictionary Engine & Predictive Lexicon  
> **Last Updated:** 2026-10-18  

This document provides complete provenance, licensing verification, and redistribution terms for all external data resources utilized within the Borno project, strictly adhering to Section 6.4 of `ROADMAP.md`.

---

## 1. Bengali Vocabulary and Frequency Lexicon

### 1.1 Source & Provenance
- **Dataset:** Leipzig Corpora Collection — Bengali News 2020 (`ben_news_2020_300K.tar.gz`)
- **Originating Institution:** Institut für Informatik, Abteilung Automatische Sprachverarbeitung, Universität Leipzig
- **Source URL:** [https://wortschatz.uni-leipzig.de/en/download/Bengali](https://wortschatz.uni-leipzig.de/en/download/Bengali)
- **Direct Download:** `https://downloads.wortschatz-leipzig.de/corpora/ben_news_2020_300K.tar.gz`
- **Source Archive SHA-256:** `7624300acee975a4344510bdacb000dd63c6709aa76ef3e922805c031bc22b49`
- **Extraction Timestamp:** 2026-10-18

### 1.2 License & Redistribution Rights
- **License:** **Creative Commons Attribution 4.0 International (CC BY 4.0)**
- **License URL:** [https://creativecommons.org/licenses/by/4.0/](https://creativecommons.org/licenses/by/4.0/)
- **Grant:** Under Section 3(a) of CC BY 4.0, the Licensor grants worldwide, royalty-free, non-sublicensable, non-exclusive license to:
  1. Reproduce and Share the Licensed Material, in whole or in part; and
  2. Produce, reproduce, and Share Adapted Material.
- **Redistribution Compliance:**
  - Borno compiles the derived vocabulary and frequency scores into a binary prefix trie artifact (`crates/borno-core/data/lexicon.bin`).
  - Attribution to Universität Leipzig is explicitly retained in this documentation, in `docs/lexicon_provenance.json`, and in the application `LICENSE`.

### 1.3 Preprocessing and Curation Pipeline
The raw Leipzig word corpus is processed using `scripts/prepare_lexicon.py`:
1. **Unicode NFC Normalization (UAX #15):** Every raw token is normalized to standard Unicode NFC.
2. **Script Validation:** Retains only words composed strictly of assigned Bengali Unicode code points (U+0980..=U+09FF) and valid joiners (ZWJ U+200D, ZWNJ U+200C). ASCII tokens, numbers, and non-Bengali symbols are discarded.
3. **Frequency Aggregation:** Identical NFC strings have their observed frequencies aggregated.
4. **Hapax Legomena Elimination:** Infrequent words with frequency $< 3$ are purged.
5. **Final Curation:** The top 40,000 frequency-ranked words are packed into `lexicon.bin`.

---

## 2. Offline Orthographic Validation Oracle (Hunspell bn_BD)

### 2.1 Boundary & License Isolation
- **Source:** LibreOffice Bengali Dictionary (`bn_BD.dic`, `bn_BD.aff`)
- **Upstream Repository:** [LibreOffice Dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/bn_BD)
- **License:** GNU General Public License v2.0+ / GNU Lesser General Public License v2.1+ / Mozilla Public License 1.1 tri-license.
- **Strict Architectural Boundary:**
  - **No Hunspell data is copied or redistributed into the Borno runtime binary.**
  - `bn_BD.dic` is utilized exclusively as an **offline validation oracle** during preprocessing in `scripts/prepare_lexicon.py`.
  - It cross-validates that 39.55% of high-frequency Leipzig words correspond to canonical dictionary entries.
  - The compiled runtime binary (`lexicon.bin`) and runtime data structures contain exclusively CC BY 4.0 derived data from the Leipzig Corpora Collection.

---

## 3. Provenance Checksums Summary

| Artifact | Location | Size | SHA-256 Checksum | License |
|---|---|---|---|---|
| Raw Leipzig Archive | `scripts/.cache/ben_news_2020_300K.tar.gz` | ~16 MB | `7624300acee975a4344510bdacb000dd63c6709aa76ef3e922805c031bc22b49` | CC BY 4.0 |
| Compiled Binary Lexicon | `crates/borno-core/data/lexicon.bin` | `1,116,596` bytes | `97db33d26ec1a5b358103f5945f95b0a7d9610db979991f1f7a47fe94f383288` | CC BY 4.0 (Derived) |
| Lexicon Manifest | `docs/lexicon_provenance.json` | - | - | CC BY 4.0 / MIT |
