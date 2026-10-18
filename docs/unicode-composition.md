# Unicode and Composition Specification for Bangla (Borno Core)

> **Document ID:** DOC-UNICODE-001  
> **Status:** Authoritative Specification (Milestone M2)  
> **Target Standard:** Unicode Standard Version 16.0.0 (Script: Bengali / Bengali-Assamese, Block: U+0980–U+09FF)  
> **Target Implementation:** `crates/borno-core/src/unicode.rs`

---

## 1. Scope and Architectural Role

This document defines the formal Unicode foundation and composition rules for the shared Borno Rust core (`crates/borno-core`).

To prevent text corruption, search indexing failures, and layout-dependent rendering bugs, the Borno engine strictly separates five distinct architectural layers:

```
┌────────────────────────────────────────────────────────────────────────┐
│ 1. Unicode Normalization (UAX #15: NFC)                                │
│    Canonical equivalence: two-part vowel composition & nukta sequences │
├────────────────────────────────────────────────────────────────────────┤
│ 2. Character Classification (UCD 16.0.0: Bengali Script Block)         │
│    96 assigned & 32 unassigned code points; normative properties       │
├────────────────────────────────────────────────────────────────────────┤
│ 3. Composition Policy (Borno Input State Machine)                      │
│    Abugida typing logic: Reph, Phala, atomic Khanda-Ta, joiner rules   │
├────────────────────────────────────────────────────────────────────────┤
│ 4. Grapheme Cluster Segmentation (UAX #29: Extended Grapheme Clusters)  │
│    User-perceived characters for host selection and cursor mapping     │
├────────────────────────────────────────────────────────────────────────┤
│ 5. Text Rendering & OpenType Font Shaping (Host OS: DirectWrite/HarfBuzz)│
│    Visual ligature formation (ক্ষ, জ্ঞ, etc.) and glyph layout         │
└────────────────────────────────────────────────────────────────────────┘
```

The engine outputs standard Unicode codepoints exclusively. It **never** emits private-use area (PUA) codes or font-specific glyph indices. Visual shaping is delegated to the host platform's OpenType text shaper.

---

## 2. Authoritative UCD 16.0.0 Inventory & Classification

### 2.1 Baseline Standard & Parity Note
The authoritative baseline is the **Unicode Character Database (UCD) Version 16.0.0**.

> **Version Comparison Note:** Between Unicode 15.1.0 and Unicode 16.0.0, the Bengali script block (`U+0980`–`U+09FF`) contains zero additions, deletions, or normative property changes. All 96 assigned code points and 32 unassigned positions are identical across both versions. Unicode 16.0.0 is adopted as the primary authoritative standard.

### 2.2 Mathematical Partition Guarantee
The 128 total code points in the block `0x0980`..`0x09FF` are partitioned into **6 mutually exclusive and collectively exhaustive assigned categories** (96 code points) and **1 unassigned set** (32 code points):

$$\text{Block Total (128)} = \sum_{i=1}^6 |\text{Category}_i| (96) + |\text{Unassigned}| (32)$$

- Pairwise intersection between any two categories is empty: $\forall i \neq j, \text{Category}_i \cap \text{Category}_j = \emptyset$.
- Zero assigned code points in `U+0980`..`U+09FF` are omitted.

```
┌────────────────────────────────────────────────────────────┬───────┐
│ Category                                                   │ Count │
├────────────────────────────────────────────────────────────┼───────┤
│ 1. Independent Vowels (Lo)                                 │    14 │
│ 2. Consonants (Lo)                                         │    38 │
│ 3. Dependent Vowel Signs & Length Marks (Mn/Mc)            │    14 │
│ 4. Combining Modifiers (Mn/Mc)                             │     5 │
│ 5. Decimal Digits (Nd)                                     │    10 │
│ 6. Additional Signs & Currency (Lo/Sc/So/No/Po/Mn)         │    15 │
├────────────────────────────────────────────────────────────┼───────┤
│ Total Assigned Code Points                                 │    96 │
│ Total Unassigned Code Points                               │    32 │
├────────────────────────────────────────────────────────────┼───────┤
│ Total Code Points in Block (0x0980..0x09FF)                │   128 │
└────────────────────────────────────────────────────────────┴───────┘
```

---

### 2.3 Detailed Category Roster

#### 1. Independent Vowels (`Lo`, 14 total)
- **11 Modern Vowels:** `অ` (0985), `আ` (0986), `ই` (0987), `ঈ` (0988), `উ` (0989), `ঊ` (098A), `ঋ` (098B), `এ` (098F), `ঐ` (0990), `ও` (0993), `ঔ` (0994).
- **3 Historical / Sanskrit Vocalic Letters:** `ঌ` (098C VOCALIC L), `ৠ` (09E0 VOCALIC RR), `ৡ` (09E1 VOCALIC LL).

#### 2. Consonants (`Lo`, 38 total)
- **32 Standard Consonants in `0995`..`09B9`:**
  - `0995`..`099E` (10): `ক`, `খ`, `গ`, `ঘ`, `ঙ`, `চ`, `ছ`, `জ`, `ঝ`, `ঞ`
  - `099F`..`09A8` (10): `ট`, `ঠ`, `ড`, `ঢ`, `ণ`, `ত`, `থ`, `দ`, `ধ`, `ন`
  - *Note:* `09A9` is unassigned.
  - `09AA`..`09AE` (5): `প`, `ফ`, `ব`, `ভ`, `ম`
  - `09AF` (1): `য`
  - `09B0` (1): `র`
  - *Note:* `09B1` is unassigned.
  - `09B2` (1): `ল`
  - *Note:* `09B3`, `09B4`, `09B5` are unassigned.
  - `09B6`..`09B9` (4): `শ`, `ষ`, `স`, `হ`
- **3 Nukta-Derived Consonants:** `ড়` (09DC), `ঢ়` (09DD), `য়` (09DF).
- **1 Khanda-Ta:** `ৎ` (09CE).
- **2 Assamese Consonants:** `ৰ` (09F0 ASSAMESE RA), `ৱ` (09F1 ASSAMESE WA).

#### 3. Dependent Vowel Signs & Length Marks (`Mn`/`Mc`, 14 total)
- **10 Standard Dependent Vowel Signs:**
  - Spacing (`Mc`): `া` (09BE), `ি` (09BF), `ী` (09C0), `ে` (09C7), `ৈ` (09C8), `ো` (09CB), `ৌ` (09CC).
  - Non-spacing (`Mn`): `ু` (09C1), `ূ` (09C2), `ৃ` (09C3).
- **1 Length Mark:** `ৗ` (09D7 AU LENGTH MARK, `Mc`).
- **3 Historical Vocalic Signs:** `ৄ` (09C4 VOCALIC RR, `Mn`), `ৢ` (09E2 VOCALIC L, `Mn`), `ৣ` (09E3 VOCALIC LL, `Mn`).

#### 4. Combining Modifiers (`Mn`/`Mc`, 5 total)
- `্` (09CD BENGALI SIGN VIRAMA / HASANTA, `Mn`, CCC = 9).
- `়` (09BC BENGALI SIGN NUKTA, `Mn`, CCC = 7).
- `ঁ` (0981 BENGALI SIGN CANDRABINDU, `Mn`, CCC = 0).
- `ং` (0982 BENGALI SIGN ANUSVARA, `Mc`, CCC = 0).
- `ঃ` (0983 BENGALI SIGN VISARGA, `Mc`, CCC = 0).

#### 5. Decimal Digits (`Nd`, 10 total)
- `০` (09E6), `১` (09E7), `২` (09E8), `৩` (09E9), `৪` (09EA), `৫` (09EB), `৬` (09EC), `৭` (09ED), `৮` (09EE), `৯` (09EF).

#### 6. Additional Signs & Currency (15 total)
- Avagraha: `ঽ` (09BD, `Lo`).
- Historical/Religious Signs: Anji `ঀ` (0980, `Lo`), Vedic Anusvara `ৼ` (09FC, `Lo`), Isshar `৺` (09FA, `So`).
- Punctuation/Grammar: Abbreviation sign `৽` (09FD, `Po`), Sandhi mark `৾` (09FE, `Mn`).
- Currency Marks: Rupee mark `৲` (09F2, `Sc`), Taka sign `৳` (09F3, `Sc`), Ganda mark `৻` (09FB, `Sc`), Currency numerators/denominators `৴`..`৹` (09F4..09F9, `No`, 6 total).

#### 7. Unassigned Code Points (Exactly 32 Exclusions)
`0984`, `098D`, `098E`, `0991`, `0992`, `09A9`, `09B1`, `09B3`, `09B4`, `09B5`, `09BA`, `09BB`, `09C5`, `09C6`, `09C9`, `09CA`, `09CF`..`09D6` (8), `09D8`..`09DB` (4), `09DE`, `09E4`, `09E5`, and `09FF`.

---

### 2.4 External Shared Indic Characters
- Danda: `।` (`U+0964`, Script = `Common`, Category = `Po`).
- Double Danda: `॥` (`U+0965`, Script = `Common`, Category = `Po`).
- Zero-Width Non-Joiner: `ZWNJ` (`U+200C`, Category = `Cf`).
- Zero-Width Joiner: `ZWJ` (`U+200D`, Category = `Cf`).

---

### 2.5 Borno IME Typing Categories
Distinct from Unicode general categories, Borno defines operational categories used for typing state transitions:

```rust
pub enum BanglaCharCategory {
    VowelIndependent,
    VowelKar,
    Consonant,
    Hasanta,
    Nukta,
    Modifier,
    Digit,
    Punctuation,
    Joiner,
    UncategorizedBengali,
    NonBengali,
}
```

Characters mapped to `UncategorizedBengali` (e.g. `ঀ`, `৺`, `৳`, `৻`) are recognized as valid assigned Bengali characters. They pass through cleanly without triggering abugida consonant-vowel bonding.

---

## 3. Unicode Normalization (NFC Policy)

### 3.1 Standards Mandate (UAX #15)
The engine strictly delegates Unicode canonical decomposition and canonical composition to the standard `unicode-normalization` crate conforming to Unicode Standard Annex #15. Custom canonical reordering algorithms are prohibited.

### 3.2 Canonical Compositions
Two-part dependent vowel signs compose canonically under NFC:
- **Isolated Signs:**
  - `\u{09C7}` (`ে`) + `\u{09BE}` (`া`) ≡ `\u{09CB}` (`ো` BENGALI VOWEL SIGN O)
  - `\u{09C7}` (`ে`) + `\u{09D7}` (`ৗ`) ≡ `\u{09CC}` (`ৌ` BENGALI VOWEL SIGN AU)
- **Attached Syllables:**
  - `\u{0995}\u{09C7}\u{09BE}` (`ক` + `ে` + `া`) ≡ `\u{0995}\u{09CB}` (`কো`)
  - `\u{0995}\u{09C7}\u{09D7}` (`ক` + `ে` + `ৗ`) ≡ `\u{0995}\u{09CC}` (`কৌ`)

### 3.3 Authoritative Nukta Decomposition in Committed Text
Under Unicode UAX #15 (`CompositionExclusions.txt`), the three precomposed nukta letters are **Full Composition Exclusions**:
- `ড়` (`U+09DC`) ≡ `ড` (`U+09A1`) + `়` (`U+09BC`)
- `ঢ়` (`U+09DD`) ≡ `ঢ` (`U+09A2`) + `়` (`U+09BC`)
- `য়` (`U+09DF`) ≡ `য` (`U+09AF`) + `়` (`U+09BC`)

Standard NFC **never** composes `ড + ়` into `ড়`. Instead, precomposed `ড়`, `ঢ়`, `য়` **decompose** to `ড + ়`, `ঢ + ়`, `য + ়`.

**Borno Committed Text Normalization Policy:**
All committed text emitted via `ProcessResult.committed_text` is strictly NFC-normalized. Consequently:
1. Committed text decomposes `ড়`, `ঢ়`, `য়` into their standard canonical sequences (`ড + ়`, `ঢ + ়`, `য + ়`).
2. Test assertions explicitly verify this: `normalize_nfc("বাড়ি") == "বা\u{09A1}\u{09BC}ি"`.
3. OpenType font shapers (HarfBuzz, DirectWrite, CoreText) render these canonical decomposed sequences identically to atomic glyphs across all modern fonts.

### 3.4 Partitioned Normalization Guarantees
1. **Pure ASCII Preservation:** `s.is_ascii() ==> NFC(s) == s`.
2. **Already-Normalized Idempotency:** `NFC(s) == s ==> NFC(NFC(s)) == NFC(s)`.
3. **Mixed-Script Normalization:**
   - *Decomposable sequence:* `"Node.js \u{0995}\u{09C7}\u{09BE}ড"` normalizes to `"Node.js কোড"`. The Latin prefix `"Node.js "` remains byte-identical while `ক + ে + া + ড` composes to `কোড`.
   - *Already-normalized sequence:* `"Version 2.0: \u{09AC}\u{09BE}\u{0982}\u{09B2}\u{09BE}"` (`"Version 2.0: বাংলা"`) remains byte-identical.

---

## 4. Borno Composition Policy (Application Rules vs. Standards)

Borno explicitly distinguishes standard Unicode equivalence from keyboard input policy:

### 4.1 Khanda-Ta (`ৎ` U+09CE) & Joiner Preservation
- **Direct Atomic Emission:** Khanda-Ta is an atomic character (`U+09CE`) with no canonical decomposition. Borno maps assigned keystrokes / transliteration rules directly to `U+09CE`.
- **Joiner Preservation by Default:** Joiners (`ZWJ` `U+200D` and `ZWNJ` `U+200C`) are strictly preserved by default across normalization and sanitization. Live typing never strips or alters joiners.
- **Isolated Legacy Compatibility Helper:** The legacy sequence `ত` (`U+09A4`) + `্` (`U+09CD`) + `ZWJ` (`U+200D`) was historically used before `U+09CE` was encoded. If conversion of this legacy sequence is needed, it is decoupled from the main engine pipeline into an explicit helper:
  ```rust
  pub fn convert_legacy_khanda_ta(s: &str) -> String
  ```
  This helper is tested independently and is **never** invoked automatically during live typing or ambient text sanitization.

### 4.2 Format Control Joining Intent
- **ZWNJ (`U+200C`, Zero-Width Non-Joiner):** Expresses a Unicode non-joining request / ligature suppression intent (e.g. `ক` + `্` + `ZWNJ` + `ক`), signaling to the text shaper not to form a conjunct ligature across the boundary. Visual presentation (whether displayed with an explicit visible hasanta or an isolated consonant shape) is governed entirely by font OpenType tables and the host text shaper, not by Borno.
- **ZWJ (`U+200D`, Zero-Width Joiner):** Expresses a Unicode joining request, used in specific Indic script contexts to request half-forms or ligated presentation.

### 4.3 Conjunct & Modifier Formation
- **Standard Conjuncts (যুক্তবর্ণ):** Encoded strictly as `Consonant + Virama (্) + Consonant` (e.g. `ক + ্ + ষ` = `ক্ষ`, `জ + ্ + ঞ` = `জ্ঞ`).
- **Reph:** Encoded as `র` (`U+09B0`) + `্` (`U+09CD`) + `Consonant`.
- **Phalas:**
  - Ya-phala: `Consonant + ্ + য` (or `Consonant + ্ + ZWJ + য` when half-form requested).
  - Ra-phala: `Consonant + ্ + র`.
  - Ba-phala: `Consonant + ্ + ব`.
- **Vowel Kar Placement:** In abugida typing, a vowel entered after a consonant/conjunct attaches as a dependent vowel sign (Kar), while a vowel entered at word start attaches as an independent vowel.

---

## 5. Scoped In-Composition Sanitization

Sanitization applies **strictly and exclusively** to active provisional composition buffers (`is_composing == true`, `raw_buffer` / `preedit`). It is **never** applied to committed document text or external text.

### 5.1 Rules
1. **Hasanta Collapsing:** Two consecutive hasantas entered during live typing (`্ + ্`) collapse to a single `্`. Hasantas followed by joiners (`্ + ZWNJ` or `্ + ZWJ`) are strictly preserved.
2. **Vowel Sign Replacement:** Entering a dependent vowel sign immediately following an existing dependent vowel sign on the active syllable replaces the preceding vowel sign (supporting typing correction).
3. **No Artificial Consonant Injection:** Orphan vowel signs in preedit are retained as valid Unicode scalars without synthesizing artificial consonants (shapers render them with dotted circles `◌া` until resolved or committed).
4. **Joiner & Mixed-Script Invariant:** All intentional ZWJ/ZWNJ characters, whitespace, Latin characters, and punctuation pass through unmodified.

---

## 6. Cursor and Editing Policy

### 6.1 Codepoint Indexing Contract
`EngineState.cursor_pos` is maintained strictly as a 0-based Unicode scalar value (codepoint) index (`0..=preedit.chars().count()`), preserving 100% backward compatibility with M1.

### 6.2 Granular Scalar Editing Inside Preedit
Inside active preedit, Backspace, Delete, and arrow navigation intentionally operate on individual Unicode scalars:
- * কা (cursor 2) + Backspace ➔ ক (cursor 1).
- * ক্ষ (`ক` + `্` + `ষ`, cursor 3) + Backspace ➔ ক্ (`ক` + `্`, cursor 2).
- * ক্ (cursor 2) + Backspace ➔ ক (cursor 1).

This provides granular typing correction during composition. Extended grapheme cluster segmentation (UAX #29) is exposed for host selection and word snapping utilities (`unicode::grapheme_count`, `unicode::grapheme_indices`), but does not govern intra-preedit caret stepping in M2.

---

## 7. Deterministic Cursor Mapping (`map_cursor_after_normalization`)

When preedit collapses under NFC normalization (such as `ে + া` fusing into `ো`), the logical cursor position must be updated deterministically.

### 7.1 Signature
```rust
pub fn map_cursor_after_normalization(original: &str, original_cursor: usize, normalized: &str) -> usize
```

### 7.2 Left-Affinity (Trailing Absorption) Algorithm
1. **Pre-Validation:** Let `orig_chars = original.chars().collect::<Vec<char>>()`, `norm_len = normalized.chars().count()`. Clamp `c = original_cursor.min(orig_chars.len())`.
   - If `c == 0`, return `0`.
   - If `c == orig_chars.len()`, return `norm_len`.
2. **Detect Cross-Boundary Two-Part Vowel Collapse:**
   In Bengali NFC, canonical composition occurs when `U+09C7` (`ে`) is immediately followed by `U+09BE` (`া`) to form `U+09CB` (`ো`), or by `U+09D7` (`ৗ`) to form `U+09CC` (`ৌ`).
   Test if the cursor sits directly between such a collapsing pair:
   $$\text{is\_cross\_boundary} = (c > 0) \land (c < \text{len}) \land (\text{orig}[c-1] == \text{'\u{09C7}'}) \land (\text{orig}[c] \in \{\text{'\u{09BE}'}, \text{'\u{09D7}'}\})$$
3. **Compute Mapped Position:**
   - **If `is_cross_boundary` is true:**
     Under **Left-Affinity**, the trailing character is absorbed into the composite so the cursor lands immediately **after** the composed vowel.
     $$\text{mapped\_pos} = \text{chars\_count}(\text{NFC}(\text{orig}[0..=c]))$$
   - **If `is_cross_boundary` is false:**
     $$\text{mapped\_pos} = \text{chars\_count}(\text{NFC}(\text{orig}[0..c]))$$
4. **Post-Validation:** Return `mapped_pos.min(norm_len)`.

### 7.3 Boundary Invariants
- *Strictly Before:* `"কোখ"` (cursor 3) ➔ `"কোখ"` (cursor 2).
- *Strictly After:* `"খকো"` (cursor 1) ➔ `"খকো"` (cursor 1).
- *Cross-Boundary:* `"কো"` (cursor 2) ➔ `"কো"` (cursor 2).
- *Nukta Invariant:* `"ড়"` (cursor 1) ➔ `"ড়"` (cursor 2). Cursor stays after the decomposed sequence.

---

## 8. Verification Matrix

| Area | Verification Test Vector | Expected Result |
|---|---|---|
| **ASCII Preservation** | `"hello world 123"` | Byte-identical |
| **Already Normalized** | `"বাংলা"` | Byte-identical (`NFC(s) == s`) |
| **Isolated Vowel Composition** | `\u{09C7}\u{09BE}` | `\u{09CB}` (`ো`) |
| **Syllable Vowel Composition** | `\u{0995}\u{09C7}\u{09BE}` | `\u{0995}\u{09CB}` (`কো`) |
| **Nukta Canonical Decomposition** | `"বাড়ি"` (`বা\u{09DC}ি`) | `"বা\u{09A1}\u{09BC}ি"` (`বাড়ি`) |
| **Mixed-Script Normalization** | `"Node.js \u{0995}\u{09C7}\u{09BE}ড"` | `"Node.js কোড"` |
| **Direct Khanda-Ta** | Keystroke for Khanda-Ta | Emits atomic `U+09CE` |
| **Legacy Khanda-Ta Helper** | `convert_legacy_khanda_ta("ত\u{09CD}\u{200D}")` | `"ৎ"` (`U+09CE`) |
| **Joiner Preservation** | `"ক\u{09CD}\u{200C}ক"` (with ZWNJ) | Strictly preserved |
| **Hasanta Collapsing** | Preedit: `"ক" + "্" + "্"` | `"ক্"` (`ক` + single `্`) |
| **Vowel Sign Replacement** | Preedit: `"কা" + "ি"` | `"কি"` (replaces preceding `া`) |
