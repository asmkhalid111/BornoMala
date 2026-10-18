//! Phonetic mapping tables and rule definitions for Borno Core.
//!
//! Grounded in published Avro Phonetic v5.x grammar and public Bengali linguistic facts.
//! All rules are compile-time constants for zero runtime allocation overhead.

/// Classification of a phonetic rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleClassification {
    AvroCompatible,
    BornoExtension,
}

/// A static phonetic replacement pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhoneticPattern {
    pub latin: &'static str,
    pub bangla: &'static str,
    pub classification: RuleClassification,
}

// ============================================================================
// 1. Vowels: Independent vs Dependent (Kar) Forms
// ============================================================================

/// Vowel mapping entry with separate independent and dependent kar forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VowelRule {
    pub latin: &'static str,
    pub independent: &'static str,
    pub dependent: Option<&'static str>,
    pub is_inherent_o: bool,
    pub classification: RuleClassification,
}

/// Ordered vowel rules: longest patterns first to ensure greedy matching.
pub static VOWEL_RULES: &[VowelRule] = &[
    // Tri-graphs
    VowelRule {
        latin: "rri",
        independent: "\u{098B}", // ঋ
        dependent: Some("\u{09C3}"), // ৃ
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    // Di-graphs
    VowelRule {
        latin: "aa",
        independent: "\u{0986}", // আ
        dependent: Some("\u{09BE}"), // া
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "ee",
        independent: "\u{0988}", // ঈ
        dependent: Some("\u{09C0}"), // ী
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "oo",
        independent: "\u{098A}", // ঊ
        dependent: Some("\u{09C2}"), // ূ
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "oi",
        independent: "\u{0990}", // ঐ
        dependent: Some("\u{09C8}"), // ৈ
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "OI",
        independent: "\u{0990}", // ঐ
        dependent: Some("\u{09C8}"), // ৈ
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "ou",
        independent: "\u{0994}", // ঔ
        dependent: Some("\u{09CC}"), // ৌ
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "OU",
        independent: "\u{0994}", // ঔ
        dependent: Some("\u{09CC}"), // ৌ
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "ae",
        independent: "\u{0985}\u{09CD}\u{09AF}\u{09BE}", // অ্যা
        dependent: Some("\u{09CD}\u{09AF}\u{09BE}"),     // ্যা
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    // Single-letters
    VowelRule {
        latin: "A",
        independent: "\u{0986}", // আ
        dependent: Some("\u{09BE}"), // া
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "a",
        independent: "\u{0986}", // আ
        dependent: Some("\u{09BE}"), // া
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "i",
        independent: "\u{0987}", // ই
        dependent: Some("\u{09BF}"), // ি
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "I",
        independent: "\u{0988}", // ঈ
        dependent: Some("\u{09C0}"), // ী
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "u",
        independent: "\u{0989}", // উ
        dependent: Some("\u{09C1}"), // ু
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "U",
        independent: "\u{098A}", // ঊ
        dependent: Some("\u{09C2}"), // ূ
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "e",
        independent: "\u{098F}", // এ
        dependent: Some("\u{09C7}"), // ে
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "E",
        independent: "\u{0985}\u{09CD}\u{09AF}\u{09BE}", // অ্যা
        dependent: Some("\u{09CD}\u{09AF}\u{09BE}"),     // ্যা
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "O",
        independent: "\u{0993}", // ও
        dependent: Some("\u{09CB}"), // ো
        is_inherent_o: false,
        classification: RuleClassification::AvroCompatible,
    },
    VowelRule {
        latin: "o",
        independent: "\u{0985}", // অ
        dependent: None,          // Inherent vowel bridge (no kar emitted)
        is_inherent_o: true,
        classification: RuleClassification::AvroCompatible,
    },
];

// ============================================================================
// 2. Curated Core Conjunct Matrix (30 High-Frequency Verified Conjuncts)
// ============================================================================

/// Ordered conjunct rules: longer patterns (4-5 chars, then 3, then 2) match first.
pub static CONJUNCT_RULES: &[PhoneticPattern] = &[
    // 5-character patterns
    PhoneticPattern {
        latin: "kShm",
        bangla: "\u{0995}\u{09CD}\u{09B7}\u{09CD}\u{09AE}", // ক্ষ্ম
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "kShN",
        bangla: "\u{0995}\u{09CD}\u{09B7}\u{09CD}\u{09A3}", // ক্ষ্ণ
        classification: RuleClassification::AvroCompatible,
    },
    // 4-character patterns
    PhoneticPattern {
        latin: "ShTh",
        bangla: "\u{09B7}\u{09CD}\u{09A0}", // ষ্ঠ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "shTh",
        bangla: "\u{09B7}\u{09CD}\u{09A0}", // ষ্ঠ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "NGch",
        bangla: "\u{099E}\u{09CD}\u{099B}", // ঞ্ছ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Ngkh",
        bangla: "\u{0999}\u{09CD}\u{0996}", // ঙ্খ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Nggh",
        bangla: "\u{0999}\u{09CD}\u{0998}", // ঙ্ঘ
        classification: RuleClassification::AvroCompatible,
    },
    // 3-character patterns
    PhoneticPattern {
        latin: "kSh",
        bangla: "\u{0995}\u{09CD}\u{09B7}", // ক্ষ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "jNG",
        bangla: "\u{099C}\u{09CD}\u{099E}", // জ্ঞ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "jgy",
        bangla: "\u{099C}\u{09CD}\u{099E}", // জ্ঞ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "cch",
        bangla: "\u{099A}\u{09CD}\u{099B}", // চ্ছ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "NGc",
        bangla: "\u{099E}\u{09CD}\u{099A}", // ঞ্চ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Nch",
        bangla: "\u{099E}\u{09CD}\u{099B}", // ঞ্ছ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Nc",
        bangla: "\u{099E}\u{09CD}\u{099A}", // ঞ্চ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "NGj",
        bangla: "\u{099E}\u{09CD}\u{099C}", // ঞ্জ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Ngk",
        bangla: "\u{0999}\u{09CD}\u{0995}", // ঙ্ক
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Ngg",
        bangla: "\u{0999}\u{09CD}\u{0997}", // ঙ্গ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "skr",
        bangla: "\u{09B8}\u{09CD}\u{0995}\u{09CD}\u{09B0}", // স্ক্র
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "str",
        bangla: "\u{09B8}\u{09CD}\u{09A4}\u{09CD}\u{09B0}", // স্ত্র
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "sTh",
        bangla: "\u{09B8}\u{09CD}\u{09A0}", // স + ঠ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "ShT",
        bangla: "\u{09B7}\u{09CD}\u{099F}", // ষ্ট
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "shT",
        bangla: "\u{09B7}\u{09CD}\u{099F}", // ষ্ট
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "STh",
        bangla: "\u{09B7}\u{09CD}\u{09A0}", // ষ্ঠ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "ShN",
        bangla: "\u{09B7}\u{09CD}\u{09A3}", // ষ্ণ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "SN",
        bangla: "\u{09B7}\u{09CD}\u{09A3}", // ষ্ণ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "nth",
        bangla: "\u{09A8}\u{09CD}\u{09A5}", // ন্থ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "ndh",
        bangla: "\u{09A8}\u{09CD}\u{09A7}", // ন্ধ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "mbh",
        bangla: "\u{09AE}\u{09CD}\u{09AD}", // ম্ভ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "sht",
        bangla: "\u{09B6}\u{09CD}\u{09A4}", // শ্ত
        classification: RuleClassification::AvroCompatible,
    },
    // 2-character patterns
    PhoneticPattern {
        latin: "xm",
        bangla: "\u{0995}\u{09CD}\u{09B7}\u{09CD}\u{09AE}", // ক্ষ্ম
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "xN",
        bangla: "\u{0995}\u{09CD}\u{09B7}\u{09CD}\u{09A3}", // ক্ষ্ণ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "nj",
        bangla: "\u{099E}\u{09CD}\u{099C}", // ঞ্জ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "sT",
        bangla: "\u{09B8}\u{09CD}\u{099F}", // স্ট (Dental Sa + Virama + Retroflex Tta)
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "ST",
        bangla: "\u{09B7}\u{09CD}\u{099F}", // ষ্ট (Retroflex Sha + Virama + Retroflex Tta)
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "st",
        bangla: "\u{09B8}\u{09CD}\u{09A4}", // স্ত (Dental Sa + Virama + Dental Ta)
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "St",
        bangla: "\u{09B6}\u{09CD}\u{09A4}", // শ্ত (Palatal Sha + Virama + Dental Ta)
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "nt",
        bangla: "\u{09A8}\u{09CD}\u{09A4}", // ন্ত
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "nd",
        bangla: "\u{09A8}\u{09CD}\u{09A6}", // ন্দ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "mp",
        bangla: "\u{09AE}\u{09CD}\u{09AA}", // ম্প
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "mb",
        bangla: "\u{09AE}\u{09CD}\u{09AC}", // ম্ব
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "hm",
        bangla: "\u{09B9}\u{09CD}\u{09AE}", // হ্ম
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "hn",
        bangla: "\u{09B9}\u{09CD}\u{09A8}", // হ্ন
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "hN",
        bangla: "\u{09B9}\u{09CD}\u{09A3}", // হ্ণ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "hl",
        bangla: "\u{09B9}\u{09CD}\u{09B2}", // হ্ল
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "hr",
        bangla: "\u{09B9}\u{09CD}\u{09B0}", // হ্র
        classification: RuleClassification::AvroCompatible,
    },
    // Geminates (identical consonant doubling)
    PhoneticPattern {
        latin: "kk",
        bangla: "\u{0995}\u{09CD}\u{0995}", // ক্ক
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "gg",
        bangla: "\u{0997}\u{09CD}\u{0997}", // গ্গ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "cc",
        bangla: "\u{099A}\u{09CD}\u{099A}", // চ্চ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "jj",
        bangla: "\u{099C}\u{09CD}\u{099C}", // জ্জ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "TT",
        bangla: "\u{099F}\u{09CD}\u{099F}", // ট্ট
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "DD",
        bangla: "\u{09A1}\u{09CD}\u{09A1}", // ড্ড
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "tt",
        bangla: "\u{09A4}\u{09CD}\u{09A4}", // ত্ত
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "dd",
        bangla: "\u{09A6}\u{09CD}\u{09A6}", // দ্দ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "pp",
        bangla: "\u{09AA}\u{09CD}\u{09AA}", // প্প
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "bb",
        bangla: "\u{09AC}\u{09CD}\u{09AC}", // ব্ব
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "mm",
        bangla: "\u{09AE}\u{09CD}\u{09AE}", // ম্ম
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "nn",
        bangla: "\u{09A8}\u{09CD}\u{09A8}", // ন্ন
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "ll",
        bangla: "\u{09B2}\u{09CD}\u{09B2}", // ল্ল
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "ss",
        bangla: "\u{09B8}\u{09CD}\u{09B8}", // স্স
        classification: RuleClassification::AvroCompatible,
    },
    // Single-character alias
    PhoneticPattern {
        latin: "x",
        bangla: "\u{0995}\u{09CD}\u{09B7}", // ক্ষ
        classification: RuleClassification::AvroCompatible,
    },
];

// ============================================================================
// 3. Consonants
// ============================================================================

/// Ordered consonant rules: multi-character patterns matched before single characters.
pub static CONSONANT_RULES: &[PhoneticPattern] = &[
    // 2-character patterns
    PhoneticPattern {
        latin: "kh",
        bangla: "\u{0996}", // খ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "gh",
        bangla: "\u{0998}", // ঘ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Ng",
        bangla: "\u{0999}", // ঙ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "ng",
        bangla: "\u{0982}", // ং (Anusvara)
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "ch",
        bangla: "\u{099B}", // ছ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Ch",
        bangla: "\u{099B}", // ছ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "jh",
        bangla: "\u{099D}", // ঝ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "NG",
        bangla: "\u{099E}", // ঞ (Palatal nasal; alias Nj removed to avoid collision with ঞ্জ)
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Th",
        bangla: "\u{09A0}", // ঠ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Dh",
        bangla: "\u{09A2}", // ঢ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "th",
        bangla: "\u{09A5}", // থ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "dh",
        bangla: "\u{09A7}", // ধ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "ph",
        bangla: "\u{09AB}", // ফ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "bh",
        bangla: "\u{09AD}", // ভ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "sh",
        bangla: "\u{09B6}", // শ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Sh",
        bangla: "\u{09B7}", // ষ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Rh",
        bangla: "\u{09DD}", // ঢ়
        classification: RuleClassification::AvroCompatible,
    },
    // Single-character patterns
    PhoneticPattern {
        latin: "k",
        bangla: "\u{0995}", // ক
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "g",
        bangla: "\u{0997}", // গ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "c",
        bangla: "\u{099A}", // চ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "j",
        bangla: "\u{099C}", // জ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "T",
        bangla: "\u{099F}", // ট
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "D",
        bangla: "\u{09A1}", // ড
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "N",
        bangla: "\u{09A3}", // ণ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "t",
        bangla: "\u{09A4}", // ত
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "d",
        bangla: "\u{09A6}", // দ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "n",
        bangla: "\u{09A8}", // ন
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "p",
        bangla: "\u{09AA}", // প
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "f",
        bangla: "\u{09AB}", // ফ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "b",
        bangla: "\u{09AC}", // ব
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "v",
        bangla: "\u{09AD}", // ভ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "m",
        bangla: "\u{09AE}", // ম
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "z",
        bangla: "\u{09AF}", // য (Antastha Za)
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "y",
        bangla: "\u{09DF}", // য় (Antastha Ya)
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "Y",
        bangla: "\u{09DF}", // য়
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "r",
        bangla: "\u{09B0}", // র
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "l",
        bangla: "\u{09B2}", // ল
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "S",
        bangla: "\u{09B6}", // শ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "s",
        bangla: "\u{09B8}", // স
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "h",
        bangla: "\u{09B9}", // হ
        classification: RuleClassification::AvroCompatible,
    },
    PhoneticPattern {
        latin: "R",
        bangla: "\u{09DC}", // ড়
        classification: RuleClassification::AvroCompatible,
    },
];

// ============================================================================
// 4. Phalas and Modifiers
// ============================================================================

pub const VIRAMA: &str = "\u{09CD}";
pub const REPH_PREFIX: &str = "\u{09B0}\u{09CD}"; // র + ্
pub const RA_PHALA: &str = "\u{09CD}\u{09B0}";     // ্ + র
pub const YA_PHALA: &str = "\u{09CD}\u{09AF}";     // ্ + য
pub const BA_PHALA: &str = "\u{09CD}\u{09AC}";     // ্ + ব
pub const CANDRABINDU: &str = "\u{0981}";          // ঁ
pub const KHANDA_TA: &str = "\u{09CE}";            // ৎ
pub const ANUSVARA: &str = "\u{0982}";             // ং
pub const VISARGA: &str = "\u{0983}";              // ঃ
