//! # Unicode Module
//!
//! Authoritative Unicode 16.0.0 character classification, standard normalization (UAX #15),
//! scoped composition sanitization, deterministic cursor mapping, and grapheme cluster helpers
//! for the Bangla script in Borno Core.

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// Operational character category used for IME state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BanglaCharCategory {
    /// Independent vowels (অ, আ, ই, etc.)
    VowelIndependent,
    /// Dependent vowel signs and length marks (া, ি, ী, ৗ, etc.)
    VowelKar,
    /// Consonants including standard, nukta-derived, Khanda-Ta, and Assamese (ক, খ, ড়, ৎ, ৰ, etc.)
    Consonant,
    /// Virama / Hasanta (্)
    Hasanta,
    /// Nukta sign (়)
    Nukta,
    /// Combining modifiers: Candrabindu (ঁ), Anusvara (ং), Visarga (ঃ)
    Modifier,
    /// Bengali decimal digits (০–৯)
    Digit,
    /// Punctuation (। U+0964, ॥ U+0965, ৽ U+09FD)
    Punctuation,
    /// Format controls: ZWNJ (U+200C) and ZWJ (U+200D)
    Joiner,
    /// Assigned Bengali characters not participating in abugida syllable bonding
    /// (e.g. Anji ঀ, Avagraha ঽ, Taka ৳, Isshar ৺, Ganda ৻, currency signs)
    UncategorizedBengali,
    /// Non-Bengali character (ASCII, Latin, other scripts, unassigned)
    NonBengali,
}

/// Checks whether a character is an unassigned code point in the Bengali block (U+0980..=U+09FF).
/// Exactly 32 code points in Unicode 16.0.0 (identical to 15.1.0).
#[inline]
pub fn is_unassigned_bangla(c: char) -> bool {
    matches!(
        c,
        '\u{0984}'
            | '\u{098D}'..='\u{098E}'
            | '\u{0991}'..='\u{0992}'
            | '\u{09A9}'
            | '\u{09B1}'
            | '\u{09B3}'..='\u{09B5}'
            | '\u{09BA}'..='\u{09BB}'
            | '\u{09C5}'..='\u{09C6}'
            | '\u{09C9}'..='\u{09CA}'
            | '\u{09CF}'..='\u{09D6}'
            | '\u{09D8}'..='\u{09DB}'
            | '\u{09DE}'
            | '\u{09E4}'..='\u{09E5}'
            | '\u{09FF}'
    )
}

/// Checks whether a character is an assigned code point in the Bengali block (U+0980..=U+09FF).
/// Exactly 96 code points in Unicode 16.0.0.
#[inline]
pub fn is_assigned_bangla(c: char) -> bool {
    ('\u{0980}'..='\u{09FF}').contains(&c) && !is_unassigned_bangla(c)
}

/// Checks whether a character is an independent vowel in the Bengali block.
/// Exactly 14 code points: 11 modern vowels + 3 historical/Sanskrit vocalic letters.
#[inline]
pub fn is_independent_vowel(c: char) -> bool {
    matches!(
        c,
        '\u{0985}'..='\u{098B}'
            | '\u{098C}'
            | '\u{098F}'..='\u{0990}'
            | '\u{0993}'..='\u{0994}'
            | '\u{09E0}'..='\u{09E1}'
    )
}

/// Checks whether a character is a consonant in the Bengali block.
/// Exactly 38 code points: 32 standard consonants in 0995..09B9, 3 nukta consonants (ড়, ঢ়, য়),
/// 1 Khanda-Ta (ৎ), and 2 Assamese consonants (ৰ, ৱ).
#[inline]
pub fn is_consonant(c: char) -> bool {
    matches!(
        c,
        '\u{0995}'..='\u{099E}'
            | '\u{099F}'..='\u{09A8}'
            | '\u{09AA}'..='\u{09AE}'
            | '\u{09AF}'
            | '\u{09B0}'
            | '\u{09B2}'
            | '\u{09B6}'..='\u{09B9}'
            | '\u{09DC}'..='\u{09DD}'
            | '\u{09DF}'
            | '\u{09CE}'
            | '\u{09F0}'..='\u{09F1}'
    )
}

/// Checks whether a character is a dependent vowel sign or length mark in the Bengali block.
/// Exactly 14 code points: 10 standard dependent vowel signs, 1 AU length mark (U+09D7),
/// and 3 historical vocalic signs (U+09C4, U+09E2, U+09E3).
#[inline]
pub fn is_vowel_sign(c: char) -> bool {
    matches!(
        c,
        '\u{09BE}'..='\u{09C0}'
            | '\u{09C1}'..='\u{09C3}'
            | '\u{09C4}'
            | '\u{09C7}'..='\u{09C8}'
            | '\u{09CB}'..='\u{09CC}'
            | '\u{09D7}'
            | '\u{09E2}'..='\u{09E3}'
    )
}

/// Checks whether a character is a combining modifier in the Bengali block.
/// Exactly 5 code points: Hasanta (U+09CD), Nukta (U+09BC), Candrabindu (U+0981),
/// Anusvara (U+0982), and Visarga (U+0983).
#[inline]
pub fn is_modifier(c: char) -> bool {
    matches!(
        c,
        '\u{0981}'..='\u{0983}' | '\u{09BC}' | '\u{09CD}'
    )
}

/// Checks whether a character is Virama / Hasanta (্ U+09CD).
#[inline]
pub fn is_hasanta(c: char) -> bool {
    c == '\u{09CD}'
}

/// Checks whether a character is Nukta (় U+09BC).
#[inline]
pub fn is_nukta(c: char) -> bool {
    c == '\u{09BC}'
}

/// Checks whether a character is a Bengali decimal digit (০–৯ U+09E6..=U+09EF).
/// Exactly 10 code points.
#[inline]
pub fn is_digit(c: char) -> bool {
    ('\u{09E6}'..='\u{09EF}').contains(&c)
}

/// Checks whether a character is punctuation relevant to Bangla typing.
/// Covers Danda (U+0964), Double Danda (U+0965), and Bengali abbreviation sign (U+09FD).
#[inline]
pub fn is_punctuation(c: char) -> bool {
    c == '\u{0964}' || c == '\u{0965}' || c == '\u{09FD}'
}

/// Checks whether a character is a format control joiner (ZWNJ U+200C or ZWJ U+200D).
#[inline]
pub fn is_joiner(c: char) -> bool {
    c == '\u{200C}' || c == '\u{200D}'
}

/// Checks whether a character is one of the 15 additional signs, currency marks, or religious symbols.
#[inline]
pub fn is_additional_sign(c: char) -> bool {
    matches!(
        c,
        '\u{0980}'
            | '\u{09BD}'
            | '\u{09F2}'..='\u{09F9}'
            | '\u{09FA}'..='\u{09FB}'
            | '\u{09FC}'
            | '\u{09FD}'
            | '\u{09FE}'
    )
}

/// Classifies any character into an operational BanglaCharCategory.
/// Provides a mutually exclusive and collectively exhaustive partition for the block.
pub fn classify_bangla_char(c: char) -> BanglaCharCategory {
    if is_hasanta(c) {
        BanglaCharCategory::Hasanta
    } else if is_nukta(c) {
        BanglaCharCategory::Nukta
    } else if c == '\u{0981}' || c == '\u{0982}' || c == '\u{0983}' {
        BanglaCharCategory::Modifier
    } else if is_independent_vowel(c) {
        BanglaCharCategory::VowelIndependent
    } else if is_vowel_sign(c) {
        BanglaCharCategory::VowelKar
    } else if is_consonant(c) {
        BanglaCharCategory::Consonant
    } else if is_digit(c) {
        BanglaCharCategory::Digit
    } else if is_joiner(c) {
        BanglaCharCategory::Joiner
    } else if is_punctuation(c) {
        BanglaCharCategory::Punctuation
    } else if is_assigned_bangla(c) {
        BanglaCharCategory::UncategorizedBengali
    } else {
        BanglaCharCategory::NonBengali
    }
}

/// Normalizes a string to Unicode Normalization Form C (NFC) according to UAX #15.
///
/// Under standard NFC:
/// - Two-part vowel signs compose: `\u{09C7} + \u{09BE} ➔ \u{09CB}` (`ো`), and `\u{09C7} + \u{09D7} ➔ \u{09CC}` (`ৌ`).
/// - Full composition exclusions decompose: `ড়`, `ঢ়`, `য়` decompose into `ড + ়`, `ঢ + ়`, `য + ়`.
/// - Pure ASCII and already-normalized strings remain byte-identical.
#[inline]
pub fn normalize_nfc(s: &str) -> String {
    s.nfc().collect::<String>()
}

/// Scoped composition sanitizer applied strictly to provisional in-composition buffers.
///
/// Invariants:
/// - Two consecutive hasantas entered during live typing (`্ + ্`) collapse to a single `্`.
///   Hasantas followed by joiners (`্ + ZWNJ` or `্ + ZWJ`) are strictly preserved.
/// - When a dependent vowel sign immediately follows another dependent vowel sign on the active
///   syllable (e.g. `কা + ি`), the new vowel replaces the preceding one (`কি`).
///   *Exception:* Two-part vowels (`\u{09C7}` followed by `\u{09BE}` or `\u{09D7}`) are kept so that
///   subsequent NFC normalization composes them into `ো` and `ৌ`.
/// - Joiners (ZWJ/ZWNJ), whitespace, ASCII characters, and external text are strictly preserved.
pub fn sanitize_composition(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_char: Option<char> = None;

    for c in s.chars() {
        if let Some(prev) = prev_char {
            // Rule 1: Collapse consecutive hasantas (্ + ্ -> ্)
            if is_hasanta(prev) && is_hasanta(c) {
                // Skip adding the second hasanta
                continue;
            }

            // Rule 2: Dependent vowel sign replacement
            // If prev is a vowel sign and c is a vowel sign:
            // Exception: \u{09C7} (ে) followed by \u{09BE} (া) or \u{09D7} (ৗ) forms standard
            // composite vowels (ো, ৌ), so do not replace.
            let is_composing_two_part = prev == '\u{09C7}' && (c == '\u{09BE}' || c == '\u{09D7}');
            if is_vowel_sign(prev) && is_vowel_sign(c) && !is_composing_two_part {
                // Replace previous vowel sign with current vowel sign
                result.pop();
                result.push(c);
                prev_char = Some(c);
                continue;
            }
        }

        result.push(c);
        prev_char = Some(c);
    }

    result
}

/// Optional compatibility helper to convert legacy `ত` + `্` + `ZWJ` sequence to Khanda-Ta (`ৎ` U+09CE).
///
/// Decoupled from live typing so that default typing preserves joiners without ambient mutation.
pub fn convert_legacy_khanda_ta(s: &str) -> String {
    s.replace("\u{09A4}\u{09CD}\u{200D}", "\u{09CE}")
}

/// Deterministically maps a logical cursor position after Unicode normalization (NFC)
/// has potentially collapsed or expanded character sequences.
///
/// Implements Left-Affinity (trailing absorption) for two-part vowel composition:
/// When the cursor sits between `\u{09C7}` (ে) and `\u{09BE}` (া) / `\u{09D7}` (ৗ),
/// the trailing mark is absorbed into the prefix, placing the cursor immediately after
/// the resulting composed vowel (`ো` / `ৌ`).
pub fn map_cursor_after_normalization(
    original: &str,
    original_cursor: usize,
    normalized: &str,
) -> usize {
    let orig_chars: Vec<char> = original.chars().collect();
    let norm_len = normalized.chars().count();
    let c = original_cursor.min(orig_chars.len());

    if c == 0 {
        return 0;
    }
    if c == orig_chars.len() {
        return norm_len;
    }

    let is_cross_boundary = c > 0
        && c < orig_chars.len()
        && orig_chars[c - 1] == '\u{09C7}'
        && (orig_chars[c] == '\u{09BE}' || orig_chars[c] == '\u{09D7}');

    let mapped_pos = if is_cross_boundary {
        // Left-affinity: include the trailing mark in the prefix so cursor lands after the composite
        let prefix: String = orig_chars[0..=c].iter().collect();
        normalize_nfc(&prefix).chars().count()
    } else {
        let prefix: String = orig_chars[0..c].iter().collect();
        normalize_nfc(&prefix).chars().count()
    };

    mapped_pos.min(norm_len)
}

/// Counts the number of extended grapheme clusters in a string according to UAX #29.
#[inline]
pub fn grapheme_count(s: &str) -> usize {
    s.graphemes(true).count()
}

/// Returns the byte offsets and string slices of each extended grapheme cluster in a string.
#[inline]
pub fn grapheme_indices(s: &str) -> Vec<(usize, &str)> {
    s.grapheme_indices(true).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exhaustive_inventory_count_and_partition() {
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
                let cat = classify_bangla_char(c);
                assert_ne!(
                    cat,
                    BanglaCharCategory::NonBengali,
                    "Assigned Bengali U+{:04X} should not classify as NonBengali",
                    code
                );
            } else {
                unassigned_count += 1;
                let cat = classify_bangla_char(c);
                assert_eq!(
                    cat,
                    BanglaCharCategory::NonBengali,
                    "Unassigned U+{:04X} should classify as NonBengali",
                    code
                );
            }
        }

        assert_eq!(assigned_count, 96, "Must have exactly 96 assigned code points");
        assert_eq!(unassigned_count, 32, "Must have exactly 32 unassigned code points");
    }

    #[test]
    fn test_category_counts() {
        let mut vowel_count = 0;
        let mut consonant_count = 0;
        let mut vowel_sign_count = 0;
        let mut modifier_count = 0;
        let mut digit_count = 0;
        let mut punctuation_count = 0;
        let mut uncategorized_count = 0;

        for code in 0x0980..=0x09FF {
            let c = char::from_u32(code).unwrap();
            if is_assigned_bangla(c) {
                match classify_bangla_char(c) {
                    BanglaCharCategory::VowelIndependent => vowel_count += 1,
                    BanglaCharCategory::Consonant => consonant_count += 1,
                    BanglaCharCategory::VowelKar => vowel_sign_count += 1,
                    BanglaCharCategory::Modifier | BanglaCharCategory::Hasanta | BanglaCharCategory::Nukta => {
                        modifier_count += 1
                    }
                    BanglaCharCategory::Digit => digit_count += 1,
                    BanglaCharCategory::Punctuation => punctuation_count += 1,
                    BanglaCharCategory::UncategorizedBengali => uncategorized_count += 1,
                    _ => panic!("Unexpected category for assigned Bengali char U+{:04X}", code),
                }
            }
        }

        assert_eq!(vowel_count, 14, "Must have exactly 14 independent vowels");
        assert_eq!(consonant_count, 38, "Must have exactly 38 consonants");
        assert_eq!(vowel_sign_count, 14, "Must have exactly 14 vowel signs & length marks");
        assert_eq!(modifier_count, 5, "Must have exactly 5 modifiers (including Hasanta and Nukta)");
        assert_eq!(digit_count, 10, "Must have exactly 10 digits");
        assert_eq!(punctuation_count, 1, "Must have exactly 1 punctuation mark in block (U+09FD)");
        assert_eq!(uncategorized_count, 14, "Must have exactly 14 remaining uncategorized Bengali characters");

        let total = vowel_count
            + consonant_count
            + vowel_sign_count
            + modifier_count
            + digit_count
            + punctuation_count
            + uncategorized_count;
        assert_eq!(total, 96, "Sum of all categories must equal exactly 96");
    }

    #[test]
    fn test_nfc_vowel_composition() {
        // Isolated
        assert_eq!(normalize_nfc("\u{09C7}\u{09BE}"), "\u{09CB}");
        assert_eq!(normalize_nfc("\u{09C7}\u{09D7}"), "\u{09CC}");
        // Attached to consonant
        assert_eq!(normalize_nfc("\u{0995}\u{09C7}\u{09BE}"), "\u{0995}\u{09CB}");
        assert_eq!(normalize_nfc("\u{0995}\u{09C7}\u{09D7}"), "\u{0995}\u{09CC}");
    }

    #[test]
    fn test_nfc_nukta_canonical_decomposition() {
        // Precomposed decomposes to canonical decomposed sequence
        assert_eq!(normalize_nfc("বাড়ি"), "বা\u{09A1}\u{09BC}ি");
        assert_eq!(normalize_nfc("\u{09DC}"), "\u{09A1}\u{09BC}");
        assert_eq!(normalize_nfc("\u{09DD}"), "\u{09A2}\u{09BC}");
        assert_eq!(normalize_nfc("\u{09DF}"), "\u{09AF}\u{09BC}");
        // Decomposed sequence remains decomposed
        assert_eq!(normalize_nfc("\u{09A1}\u{09BC}"), "\u{09A1}\u{09BC}");
    }

    #[test]
    fn test_nfc_mixed_script_preservation() {
        let original = "Node.js \u{0995}\u{09C7}\u{09BE}ড";
        let expected = "Node.js কোড";
        assert_eq!(normalize_nfc(original), expected);

        let already_norm = "Version 2.0: \u{09AC}\u{09BE}\u{0982}\u{09B2}\u{09BE}";
        assert_eq!(normalize_nfc(already_norm), already_norm);
    }

    #[test]
    fn test_joiner_preservation() {
        let zwnj = "ক\u{09CD}\u{200C}ক";
        assert_eq!(normalize_nfc(zwnj), zwnj);
        assert_eq!(sanitize_composition(zwnj), zwnj);

        let zwj = "ক\u{09CD}\u{200D}য";
        assert_eq!(normalize_nfc(zwj), zwj);
        assert_eq!(sanitize_composition(zwj), zwj);
    }

    #[test]
    fn test_legacy_khanda_ta() {
        assert_eq!(convert_legacy_khanda_ta("ত\u{09CD}\u{200D}"), "ৎ");
        assert_eq!(convert_legacy_khanda_ta("উৎসব"), "উৎসব");
    }

    #[test]
    fn test_cursor_remapping() {
        // Strictly before
        assert_eq!(map_cursor_after_normalization("কোখ", 3, "কোখ"), 2);
        // Strictly after
        assert_eq!(map_cursor_after_normalization("খকো", 1, "খকো"), 1);
        // Cross-boundary (between ে and া)
        assert_eq!(map_cursor_after_normalization("কো", 2, "কো"), 2);
        // Nukta expansion
        assert_eq!(map_cursor_after_normalization("ড়", 1, "ড়"), 2);
    }

    #[test]
    fn test_sanitization_rules() {
        // Hasanta collapse
        assert_eq!(sanitize_composition("ক\u{09CD}\u{09CD}"), "ক্");
        // Hasanta with joiner preserved
        assert_eq!(sanitize_composition("ক\u{09CD}\u{200C}"), "ক\u{09CD}\u{200C}");
        // Vowel replacement: কা + ি -> কি
        assert_eq!(sanitize_composition("কা\u{09BF}"), "কি");
        // Two-part vowel not replaced: কে + া -> কো
        assert_eq!(sanitize_composition("কে\u{09BE}"), "কো");
    }
}
