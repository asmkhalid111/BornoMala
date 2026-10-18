//! # Unicode Golden Test Suite (Milestone M2)
//!
//! Comprehensive layered test suite verifying:
//! - Layer 1: Unicode Normalization (UAX #15: NFC, two-part vowels, nukta decomposition, mixed-script)
//! - Layer 2: Character Classification (UCD 16.0.0, 96 assigned, 32 unassigned, zero overlap)
//! - Layer 3: Grapheme Cluster Segmentation (UAX #29) & Deterministic Cursor Remapping
//! - Layer 4: Composition Policy (Primary conjuncts, Reph, Phalas, Khanda-Ta, joiner preservation)
//! - Layer 5: In-Composition Sanitization & Core Engine Integration

use bornomala_core::unicode::*;
use bornomala_core::{BornoEngine, EngineEvent, InputMode};
use unicode_segmentation::UnicodeSegmentation;

// ============================================================================
// Layer 1: Unicode Normalization Tests (UAX #15)
// ============================================================================

#[test]
fn test_layer1_pure_ascii_preservation() {
    let ascii_samples = [
        "",
        "hello world",
        "The quick brown fox jumps over the lazy dog 1234567890",
        "!@#$%^&*()-_=+[]{}|;:',.<>/?~`\"",
        "printf(\"Hello, %s\\n\", name);\r\n",
    ];

    for s in ascii_samples {
        assert_eq!(
            normalize_nfc(s),
            s,
            "Pure ASCII string must remain byte-identical under NFC"
        );
    }
}

#[test]
fn test_layer1_already_normalized_idempotency() {
    let bangla_nfc_samples = [
        "বাংলা",
        "ঢাকা",
        "কোড",
        "বৃষ্টি",
        "বিজ্ঞান",
        "ক্ষমা",
        "উৎসব",
        "বা\u{09A1}\u{09BC}ি", // বাড়ি in canonical NFC (decomposed nukta)
        "সম\u{09AF}\u{09BC}", // সময় in canonical NFC (decomposed nukta)
    ];

    for s in bangla_nfc_samples {
        let once = normalize_nfc(s);
        let twice = normalize_nfc(&once);
        assert_eq!(once, s, "Already-normalized string must be byte-identical");
        assert_eq!(twice, once, "NFC must be idempotent: NFC(NFC(s)) == NFC(s)");
    }
}

#[test]
fn test_layer1_isolated_vowel_composition() {
    // E-kar (U+09C7) + AA-kar (U+09BE) -> O-kar (U+09CB)
    let e_plus_aa = "\u{09C7}\u{09BE}";
    assert_eq!(normalize_nfc(e_plus_aa), "\u{09CB}");

    // E-kar (U+09C7) + AU-length mark (U+09D7) -> AU-kar (U+09CC)
    let e_plus_au = "\u{09C7}\u{09D7}";
    assert_eq!(normalize_nfc(e_plus_au), "\u{09CC}");
}

#[test]
fn test_layer1_syllable_vowel_composition() {
    // ক + ে + া -> কো
    let ko_decomposed = "\u{0995}\u{09C7}\u{09BE}";
    assert_eq!(normalize_nfc(ko_decomposed), "\u{0995}\u{09CB}");

    // ক + ে + ৗ -> কৌ
    let kau_decomposed = "\u{0995}\u{09C7}\u{09D7}";
    assert_eq!(normalize_nfc(kau_decomposed), "\u{0995}\u{09CC}");

    // গ + ে + া -> গো
    let go_decomposed = "\u{0997}\u{09C7}\u{09BE}";
    assert_eq!(normalize_nfc(go_decomposed), "\u{0997}\u{09CB}");
}

#[test]
fn test_layer1_nukta_canonical_decomposition() {
    // Precomposed nukta consonants canonically decompose under strict NFC
    // RRA ড় (U+09DC) -> DDA ড (U+09A1) + Nukta (U+09BC)
    assert_eq!(normalize_nfc("\u{09DC}"), "\u{09A1}\u{09BC}");
    // RHA ঢ় (U+09DD) -> DDHA ঢ (U+09A2) + Nukta (U+09BC)
    assert_eq!(normalize_nfc("\u{09DD}"), "\u{09A2}\u{09BC}");
    // YYA য় (U+09DF) -> YA য (U+09AF) + Nukta (U+09BC)
    assert_eq!(normalize_nfc("\u{09DF}"), "\u{09AF}\u{09BC}");

    // Words with precomposed nuktas decompose canonically
    assert_eq!(normalize_nfc("বাড়ি"), "বা\u{09A1}\u{09BC}ি");
    assert_eq!(normalize_nfc("আষাঢ়"), "আষা\u{09A2}\u{09BC}");
    assert_eq!(normalize_nfc("সময়"), "সম\u{09AF}\u{09BC}");

    // Decomposed sequences remain decomposed
    assert_eq!(normalize_nfc("\u{09A1}\u{09BC}"), "\u{09A1}\u{09BC}");
    assert_eq!(normalize_nfc("\u{09A2}\u{09BC}"), "\u{09A2}\u{09BC}");
    assert_eq!(normalize_nfc("\u{09AF}\u{09BC}"), "\u{09AF}\u{09BC}");
}

#[test]
fn test_layer1_mixed_script_normalization() {
    // Decomposable sequence inside mixed script: Latin prefix remains byte-identical,
    // Bangla syllable with two-part vowel canonically composes to কোড.
    let mixed_decomposable = "Node.js \u{0995}\u{09C7}\u{09BE}ড";
    let expected_composed = "Node.js কোড";
    assert_eq!(normalize_nfc(mixed_decomposable), expected_composed);

    // Latin prefix with punctuation remains unchanged
    let prefix = &normalize_nfc(mixed_decomposable)[0..8];
    assert_eq!(prefix, "Node.js ");

    // Already-normalized mixed script satisfies NFC(s) == s
    let already_norm_mixed = "Version 2.0: \u{09AC}\u{09BE}\u{0982}\u{09B2}\u{09BE}";
    assert_eq!(normalize_nfc(already_norm_mixed), already_norm_mixed);

    // Multi-script sentence
    let multi_script = "User typed 'কোড' in VS Code version 1.90";
    assert_eq!(normalize_nfc(multi_script), multi_script);
}

// ============================================================================
// Layer 2: Character Classification Tests (UCD 16.0.0)
// ============================================================================

#[test]
fn test_layer2_exhaustive_96_assigned_code_points() {
    let mut assigned_count = 0;
    let mut unassigned_count = 0;

    for code in 0x0980..=0x09FF {
        let c = char::from_u32(code).unwrap();
        let is_assigned = is_assigned_bangla(c);
        let is_unassigned = is_unassigned_bangla(c);

        assert_ne!(
            is_assigned, is_unassigned,
            "Code point U+{:04X} must be strictly assigned OR unassigned",
            code
        );

        if is_assigned {
            assigned_count += 1;
            assert_ne!(
                classify_bangla_char(c),
                BanglaCharCategory::NonBengali,
                "Assigned Bengali U+{:04X} must not classify as NonBengali",
                code
            );
        } else {
            unassigned_count += 1;
            assert_eq!(
                classify_bangla_char(c),
                BanglaCharCategory::NonBengali,
                "Unassigned U+{:04X} must classify as NonBengali",
                code
            );
        }
    }

    assert_eq!(assigned_count, 96, "UCD 16.0.0 Bengali block has exactly 96 assigned code points");
    assert_eq!(unassigned_count, 32, "UCD 16.0.0 Bengali block has exactly 32 unassigned code points");
}

#[test]
fn test_layer2_zero_category_overlap_and_omissions() {
    let mut vowels = 0;
    let mut consonants = 0;
    let mut vowel_signs = 0;
    let mut modifiers = 0;
    let mut digits = 0;
    let mut punctuation = 0;
    let mut uncategorized = 0;

    for code in 0x0980..=0x09FF {
        let c = char::from_u32(code).unwrap();
        if is_assigned_bangla(c) {
            match classify_bangla_char(c) {
                BanglaCharCategory::VowelIndependent => vowels += 1,
                BanglaCharCategory::Consonant => consonants += 1,
                BanglaCharCategory::VowelKar => vowel_signs += 1,
                BanglaCharCategory::Modifier | BanglaCharCategory::Hasanta | BanglaCharCategory::Nukta => {
                    modifiers += 1
                }
                BanglaCharCategory::Digit => digits += 1,
                BanglaCharCategory::Punctuation => punctuation += 1,
                BanglaCharCategory::UncategorizedBengali => uncategorized += 1,
                other => panic!("Unexpected category {:?} for assigned character U+{:04X}", other, code),
            }
        }
    }

    assert_eq!(vowels, 14, "14 independent vowels (11 modern + 3 historical)");
    assert_eq!(consonants, 38, "38 consonants (32 standard + 3 nukta + 1 khanda-ta + 2 assamese)");
    assert_eq!(vowel_signs, 14, "14 vowel signs (10 standard + 1 length mark + 3 historical)");
    assert_eq!(modifiers, 5, "5 modifiers (hasanta, nukta, candrabindu, anusvara, visarga)");
    assert_eq!(digits, 10, "10 decimal digits (০–৯)");
    assert_eq!(punctuation, 1, "1 punctuation mark in block (U+09FD)");
    assert_eq!(uncategorized, 14, "14 uncategorized assigned characters");

    let sum = vowels + consonants + vowel_signs + modifiers + digits + punctuation + uncategorized;
    assert_eq!(sum, 96, "Sum of assigned categories must equal exactly 96 with 0 omissions");
}

#[test]
fn test_layer2_historical_vowels_and_special_signs() {
    // Historical vowels
    assert!(is_independent_vowel('\u{098C}')); // ঌ VOCALIC L
    assert!(is_independent_vowel('\u{09E0}')); // ৠ VOCALIC RR
    assert!(is_independent_vowel('\u{09E1}')); // ৡ VOCALIC LL

    // Special religious / historical symbols
    assert!(is_additional_sign('\u{0980}')); // ঀ ANJI
    assert!(is_additional_sign('\u{09BD}')); // ঽ AVAGRAHA
    assert!(is_additional_sign('\u{09FA}')); // ৺ ISSHAR
    assert!(is_additional_sign('\u{09FB}')); // ৻ GANDA MARK
    assert!(is_additional_sign('\u{09FC}')); // ৼ VEDIC ANUSVARA
    assert!(is_additional_sign('\u{09FD}')); // ৽ ABBREVIATION SIGN
    assert!(is_additional_sign('\u{09FE}')); // ৾ SANDHI MARK

    // Currency signs
    assert!(is_additional_sign('\u{09F2}')); // ৲ RUPEE MARK
    assert!(is_additional_sign('\u{09F3}')); // ৳ TAKA SIGN
    for code in 0x09F4..=0x09F9 {
        assert!(is_additional_sign(char::from_u32(code).unwrap()));
    }
}

#[test]
fn test_layer2_external_indic_punctuation_and_joiners() {
    // Shared Indic punctuation in Devanagari block
    assert!(is_punctuation('\u{0964}')); // । DANDA
    assert!(is_punctuation('\u{0965}')); // ॥ DOUBLE DANDA

    // Format control joiners
    assert!(is_joiner('\u{200C}')); // ZWNJ
    assert!(is_joiner('\u{200D}')); // ZWJ

    assert_eq!(classify_bangla_char('\u{0964}'), BanglaCharCategory::Punctuation);
    assert_eq!(classify_bangla_char('\u{0965}'), BanglaCharCategory::Punctuation);
    assert_eq!(classify_bangla_char('\u{200C}'), BanglaCharCategory::Joiner);
    assert_eq!(classify_bangla_char('\u{200D}'), BanglaCharCategory::Joiner);
}

// ============================================================================
// Layer 3: Grapheme Segmentation (UAX #29) & Deterministic Cursor Remapping
// ============================================================================

#[test]
fn test_layer3_extended_grapheme_clusters() {
    // বাংলা (ব + া + ং, ল + া) = 2 extended grapheme clusters: ["বাং", "লা"]
    // Because Anusvara (U+0982) has GCB = SpacingMark (GB9a), bonding with preceding base + vowel.
    let clusters: Vec<&str> = "বাংলা".graphemes(true).collect();
    assert_eq!(clusters, vec!["বাং", "লা"]);
    assert_eq!(grapheme_count("বাংলা"), 2);

    // Simple word: ঢাকা (ঢ + া, ক + া) = 2 grapheme clusters
    assert_eq!(grapheme_count("ঢাকা"), 2);

    // Complex conjunct: ক্ষ (ক + ্ + ষ) = 1 grapheme cluster
    let kkha = "\u{0995}\u{09CD}\u{09B7}";
    assert_eq!(kkha.chars().count(), 3);
    assert_eq!(grapheme_count(kkha), 1);

    // Conjunct with vowel: ক্ষমা (ক্ষ, ম + া) = 2 grapheme clusters
    assert_eq!(grapheme_count("ক্ষমা"), 2);

    // Syllable with ZWNJ: ক্‌ক (ক + ্ + ZWNJ + ক) = 2 grapheme clusters
    let zwnj_seq = "\u{0995}\u{09CD}\u{200C}\u{0995}";
    assert_eq!(grapheme_count(zwnj_seq), 2);
}

#[test]
fn test_layer3_deterministic_cursor_remapping_all_conditions() {
    // Condition 1: Cursor strictly before collapse
    // Original: "কোখ" (chars: ক [0], ে [1], া [2], খ [3])
    // Normalized: "কোখ" (chars: ক [0], ো [1], খ [2])
    assert_eq!(map_cursor_after_normalization("কোখ", 0, "কোখ"), 0);
    assert_eq!(map_cursor_after_normalization("কোখ", 1, "কোখ"), 1);

    // Condition 2: Cursor cross-boundary (between ে and া)
    // Left-affinity: trailing mark া is absorbed, placing cursor after composed ো (index 2)
    assert_eq!(map_cursor_after_normalization("কোখ", 2, "কোখ"), 2);

    // Condition 3: Cursor strictly after collapse
    // Cursor at index 3 (after া, before খ) maps to index 2 (after ো, before খ)
    assert_eq!(map_cursor_after_normalization("কোখ", 3, "কোখ"), 2);
    // Cursor at index 4 (end of string) maps to end of normalized string (index 3)
    assert_eq!(map_cursor_after_normalization("কোখ", 4, "কোখ"), 3);

    // Condition 4: Collapse strictly after cursor
    // Original: "খকো" (cursor at 1, after খ) -> Normalized: "খকো"
    assert_eq!(map_cursor_after_normalization("খকো", 1, "খকো"), 1);

    // Condition 5: AU-kar cross-boundary collapse
    // Original: "কৌ" (cursor between ে and ৗ, pos 2) -> Normalized: "কৌ" (length 2)
    assert_eq!(map_cursor_after_normalization("কৌ", 2, "কৌ"), 2);

    // Condition 6: Nukta canonical decomposition cursor mapping
    // Original: "ড়" (length 1, cursor at 1) -> Normalized: "ড়" (ড + ়, length 2)
    assert_eq!(map_cursor_after_normalization("ড়", 0, "ড়"), 0);
    assert_eq!(map_cursor_after_normalization("ড়", 1, "ড়"), 2);
}

// ============================================================================
// Layer 4: Composition Policy Tests
// ============================================================================

#[test]
fn test_layer4_primary_conjunct_sequences() {
    let conjuncts = [
        ("ক্ষ", "\u{0995}\u{09CD}\u{09B7}"), // ক + ্ + ষ
        ("জ্ঞ", "\u{099C}\u{09CD}\u{099E}"), // জ + ্ + ঞ
        ("ত্র", "\u{09A4}\u{09CD}\u{09B0}"), // ত + ্ + র
        ("ঙ্ক", "\u{0999}\u{09CD}\u{0995}"), // ঙ + ্ + ক
        ("ঙ্গ", "\u{0999}\u{09CD}\u{0997}"), // ঙ + ্ + গ
        ("ঞ্চ", "\u{099E}\u{09CD}\u{099A}"), // ঞ + ্ + চ
        ("ঞ্জ", "\u{099E}\u{09CD}\u{099C}"), // ঞ + ্ + জ
        ("ণ্ড", "\u{09A3}\u{09CD}\u{09A1}"), // ণ + ্ + ড
        ("ন্ত", "\u{09A8}\u{09CD}\u{09A4}"), // ন + ্ + ত
        ("ন্দ", "\u{09A8}\u{09CD}\u{09A6}"), // ন + ্ + দ
        ("ম্প", "\u{09AE}\u{09CD}\u{09AA}"), // ম + ্ + প
        ("ম্ব", "\u{09AE}\u{09CD}\u{09AC}"), // ম + ্ + ব
        ("ষ্ট", "\u{09B7}\u{09CD}\u{099F}"), // ষ + ্ + ট
        ("স্থ", "\u{09B8}\u{09CD}\u{09A5}"), // স + ্ + থ
        ("স্প", "\u{09B8}\u{09CD}\u{09AA}"), // স + ্ + প
        ("স্ক্র", "\u{09B8}\u{09CD}\u{0995}\u{09CD}\u{09B0}"), // স + ্ + ক + ্ + র
        ("হ্ম", "\u{09B9}\u{09CD}\u{09AE}"), // হ + ্ + ম
    ];

    for (name, expected_seq) in conjuncts {
        assert_eq!(
            normalize_nfc(expected_seq),
            expected_seq,
            "Conjunct {} sequence must be canonically stable in NFC",
            name
        );
    }
}

#[test]
fn test_layer4_reph_and_phalas() {
    // Reph: র + ্ + consonant
    let reph_k = "\u{09B0}\u{09CD}\u{0995}"; // র্ক
    assert_eq!(normalize_nfc(reph_k), reph_k);

    // Ya-phala: consonant + ্ + য
    let ya_phala = "\u{0995}\u{09CD}\u{09AF}"; // ক্য
    assert_eq!(normalize_nfc(ya_phala), ya_phala);

    // Ra-phala: consonant + ্ + র
    let ra_phala = "\u{0995}\u{09CD}\u{09B0}"; // ক্র
    assert_eq!(normalize_nfc(ra_phala), ra_phala);

    // Ba-phala: consonant + ্ + ব
    let ba_phala = "\u{0995}\u{09CD}\u{09AC}"; // ক্ব
    assert_eq!(normalize_nfc(ba_phala), ba_phala);
}

#[test]
fn test_layer4_khanda_ta_atomic_emission() {
    // Khanda-Ta is atomic U+09CE
    let khanda_ta = "\u{09CE}";
    assert_eq!(normalize_nfc(khanda_ta), "\u{09CE}");
    assert_eq!(classify_bangla_char('\u{09CE}'), BanglaCharCategory::Consonant);
}

#[test]
fn test_layer4_joiner_preservation() {
    // ZWNJ expresses non-joining intent (ligature suppression)
    let zwnj_seq = "ক\u{09CD}\u{200C}ক";
    assert_eq!(normalize_nfc(zwnj_seq), zwnj_seq);
    assert_eq!(sanitize_composition(zwnj_seq), zwnj_seq);

    // ZWJ expresses joining request
    let zwj_seq = "ক\u{09CD}\u{200D}য";
    assert_eq!(normalize_nfc(zwj_seq), zwj_seq);
    assert_eq!(sanitize_composition(zwj_seq), zwj_seq);
}

#[test]
fn test_layer4_isolated_legacy_khanda_ta_helper() {
    // Decoupled compatibility helper converts legacy ত + ্ + ZWJ to ৎ
    let legacy = "ত\u{09CD}\u{200D}";
    assert_eq!(convert_legacy_khanda_ta(legacy), "ৎ");

    let legacy_word = "উৎসব";
    assert_eq!(convert_legacy_khanda_ta(legacy_word), "উৎসব");

    // Regular ত + ্ (without ZWJ) must not convert to ৎ
    let regular_hasanta = "ত\u{09CD}ম";
    assert_eq!(convert_legacy_khanda_ta(regular_hasanta), "ত\u{09CD}ম");
}

// ============================================================================
// Layer 5: In-Composition Sanitization & Engine Integration
// ============================================================================

#[test]
fn test_layer5_hasanta_collapsing() {
    // Two hasantas collapse to one
    assert_eq!(sanitize_composition("ক\u{09CD}\u{09CD}"), "ক্");
    // Three hasantas collapse to one
    assert_eq!(sanitize_composition("ক\u{09CD}\u{09CD}\u{09CD}"), "ক্");
    // Hasanta followed by ZWNJ or ZWJ is preserved
    assert_eq!(sanitize_composition("ক\u{09CD}\u{200C}"), "ক\u{09CD}\u{200C}");
    assert_eq!(sanitize_composition("ক\u{09CD}\u{200D}"), "ক\u{09CD}\u{200D}");
}

#[test]
fn test_layer5_vowel_sign_replacement() {
    // Entering a new vowel sign on the same syllable replaces the previous vowel sign
    assert_eq!(sanitize_composition("কা\u{09BF}"), "কি"); // কা + ি -> কি
    assert_eq!(sanitize_composition("কি\u{09C1}"), "কু"); // কি + ু -> কু
    assert_eq!(sanitize_composition("কু\u{09C7}"), "কে"); // কু + ে -> কে

    // Two-part vowel sequence (ে + া -> কো) must NOT be replaced; it forms composite ো in NFC
    assert_eq!(sanitize_composition("কে\u{09BE}"), "কো");
    assert_eq!(normalize_nfc(&sanitize_composition("কে\u{09BE}")), "কো");

    // Two-part vowel sequence (ে + ৗ -> কৌ) must NOT be replaced; it forms composite ৌ in NFC
    assert_eq!(sanitize_composition("কে\u{09D7}"), "কৌ");
    assert_eq!(normalize_nfc(&sanitize_composition("কে\u{09D7}")), "কৌ");
}

#[test]
fn test_layer5_core_engine_nfc_committed_text() {
    let mut engine = BornoEngine::new();

    // Type "ami" -> preedit "আমি"
    engine.process_event(EngineEvent::InputChar('a'));
    engine.process_event(EngineEvent::InputChar('m'));
    engine.process_event(EngineEvent::InputChar('i'));
    assert_eq!(engine.state().preedit, "আমি");

    // Commit -> committed text is strictly NFC
    let res = engine.process_event(EngineEvent::Commit);
    assert_eq!(res.committed_text, Some("আমি".to_string()));
    assert_eq!(
        res.committed_text.as_ref().unwrap(),
        &normalize_nfc("আমি")
    );
}

#[test]
fn test_layer5_core_engine_switch_mode_nfc_commit() {
    let mut engine = BornoEngine::new();

    // Type "k" -> preedit "ক"
    engine.process_event(EngineEvent::InputChar('k'));
    assert_eq!(engine.state().preedit, "ক");

    // Switch mode while composing -> auto-commits with strictly NFC-normalized text
    let res = engine.process_event(EngineEvent::SwitchMode(InputMode::EnglishPassthrough));
    assert_eq!(res.committed_text, Some("ক".to_string()));
    assert_eq!(engine.state().mode, InputMode::EnglishPassthrough);
}

#[test]
fn test_layer5_core_engine_preserves_joiners_without_silent_mutation() {
    let mut engine = BornoEngine::new();

    // Directly input a string containing ZWNJ
    let zwnj_str = "ক\u{09CD}\u{200C}ক".to_string();
    engine.process_event(EngineEvent::InputString(zwnj_str.clone()));

    // Verify preedit preserves ZWNJ
    assert!(engine.state().preedit.contains('\u{200C}'));
    let res = engine.process_event(EngineEvent::Commit);
    assert_eq!(res.committed_text, Some(zwnj_str));
}
