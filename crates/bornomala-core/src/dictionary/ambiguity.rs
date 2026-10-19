//! Phonetic Ambiguity Resolution Pipeline for Borno Core.
//!
//! Generates secondary dictionary candidates for common homophone variations
//! (e.g. `khoma` -> "ক্ষমা", `biggan` -> "বিজ্ঞান", `bhasha` -> "ভাষা") without altering Candidate 0.

use crate::dictionary::trie::PrefixTrie;
use crate::phonetic::PhoneticEngine;
use crate::unicode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbiguityCondition {
    WordInitial,
    Medial,
    Anywhere,
}

pub struct AmbiguityRule {
    pub pattern: &'static str,
    pub replace_with: &'static str,
    pub condition: AmbiguityCondition,
}

/// Curated, ordered phonetic ambiguity rewrite rules.
pub static PHONETIC_AMBIGUITY_RULES: &[AmbiguityRule] = &[
    // 1. Initial kh -> kSh (e.g. khoma -> kShoma -> ক্ষমা)
    AmbiguityRule {
        pattern: "kh",
        replace_with: "kSh",
        condition: AmbiguityCondition::WordInitial,
    },
    // 2. Medial/anywhere kkh -> kSh (e.g. opekkha -> opekSha -> অপেক্ষা, porikkha -> পরীক্ষা)
    AmbiguityRule {
        pattern: "kkh",
        replace_with: "kSh",
        condition: AmbiguityCondition::Anywhere,
    },
    // 3. Medial gg -> jNG (e.g. biggan -> bijNGan -> বিজ্ঞান)
    AmbiguityRule {
        pattern: "gg",
        replace_with: "jNG",
        condition: AmbiguityCondition::Medial,
    },
    // 4. Medial sh -> Sh (e.g. bhasha -> bhaSha -> ভাষা)
    AmbiguityRule {
        pattern: "sh",
        replace_with: "Sh",
        condition: AmbiguityCondition::Medial,
    },
    // 5. Sibilant homophones: s -> Sh
    AmbiguityRule {
        pattern: "s",
        replace_with: "Sh",
        condition: AmbiguityCondition::Anywhere,
    },
    // 6. Sibilant homophones: s -> sh
    AmbiguityRule {
        pattern: "s",
        replace_with: "sh",
        condition: AmbiguityCondition::Anywhere,
    },
    // 7. Sibilant homophones: sh -> s
    AmbiguityRule {
        pattern: "sh",
        replace_with: "s",
        condition: AmbiguityCondition::Anywhere,
    },
    // 8. Post-Reph nasal alternation (ণ-ত্ব বিধান): rn -> rN (e.g. bornomala -> borNomala -> বর্ণমালা)
    AmbiguityRule {
        pattern: "rn",
        replace_with: "rN",
        condition: AmbiguityCondition::Anywhere,
    },
    // 9. Medial vowel ambiguity: o -> O (e.g. tomake -> tOmake -> তোমাকে)
    AmbiguityRule {
        pattern: "o",
        replace_with: "O",
        condition: AmbiguityCondition::Medial,
    },
];

/// Maximum number of distinct rewritten raw variants generated per active token.
pub const MAX_AMBIGUITY_VARIANTS: usize = 4;

/// Generates at most `MAX_AMBIGUITY_VARIANTS` candidate rewrites from the raw ASCII buffer.
pub fn generate_raw_variants(raw_buffer: &str) -> Vec<String> {
    if raw_buffer.is_empty() {
        return Vec::new();
    }

    let mut variants: Vec<String> = Vec::with_capacity(MAX_AMBIGUITY_VARIANTS);

    for rule in PHONETIC_AMBIGUITY_RULES {
        if variants.len() >= MAX_AMBIGUITY_VARIANTS {
            break;
        }

        match rule.condition {
            AmbiguityCondition::WordInitial => {
                if raw_buffer.starts_with(rule.pattern) {
                    let rewritten = format!("{}{}", rule.replace_with, &raw_buffer[rule.pattern.len()..]);
                    if rewritten != raw_buffer && !variants.contains(&rewritten) {
                        variants.push(rewritten);
                    }
                }
            }
            AmbiguityCondition::Medial => {
                if let Some(pos) = raw_buffer.find(rule.pattern) {
                    // Medial: strictly between first character and end of string
                    if pos > 0 && pos + rule.pattern.len() <= raw_buffer.len() {
                        let rewritten = format!(
                            "{}{}{}",
                            &raw_buffer[..pos],
                            rule.replace_with,
                            &raw_buffer[pos + rule.pattern.len()..]
                        );
                        if rewritten != raw_buffer && !variants.contains(&rewritten) {
                            variants.push(rewritten);
                        }
                    }
                }
            }
            AmbiguityCondition::Anywhere => {
                if let Some(pos) = raw_buffer.find(rule.pattern) {
                    let rewritten = format!(
                        "{}{}{}",
                        &raw_buffer[..pos],
                        rule.replace_with,
                        &raw_buffer[pos + rule.pattern.len()..]
                    );
                    if rewritten != raw_buffer && !variants.contains(&rewritten) {
                        variants.push(rewritten);
                    }
                }
            }
        }
    }

    variants
}

/// Resolves phonetic ambiguity variants by checking transliteration against the lexicon trie.
///
/// Returns verified dictionary words that differ from the primary transliteration.
pub fn resolve_ambiguity(
    raw_buffer: &str,
    primary_transliteration: &str,
    trie: &PrefixTrie,
) -> Vec<(String, u32)> {
    let variants = generate_raw_variants(raw_buffer);
    let norm_primary = unicode::normalize_nfc(primary_transliteration);
    let mut verified = Vec::new();

    for variant in variants {
        let (translit, _, _) = PhoneticEngine::transliterate(&variant);
        let norm_translit = unicode::normalize_nfc(&translit);

        // Discard if identical to Candidate 0
        if norm_translit == norm_primary {
            continue;
        }

        // Lexicon validation gate: MUST exist in the trie
        if let Some(freq) = trie.get_frequency(&norm_translit) {
            if !verified.iter().any(|(w, _)| w == &norm_translit) {
                verified.push((norm_translit, freq));
            }
        }
    }

    verified
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_variant_generation() {
        let vars_khoma = generate_raw_variants("khoma");
        assert!(vars_khoma.contains(&"kShoma".to_string()));

        let vars_biggan = generate_raw_variants("biggan");
        assert!(vars_biggan.contains(&"bijNGan".to_string()));

        let vars_bhasha = generate_raw_variants("bhasha");
        assert!(vars_bhasha.contains(&"bhaSha".to_string()));

        let vars_opekkha = generate_raw_variants("opekkha");
        assert!(vars_opekkha.contains(&"opekSha".to_string()));
    }

    #[test]
    fn test_resolve_ambiguity_lexicon_validated() {
        let mut trie = PrefixTrie::new();
        trie.insert("ক্ষমা", 165);
        trie.insert("বিজ্ঞান", 356);
        trie.insert("অপেক্ষা", 305);

        // khoma primary is "খমা"; "ক্ষমা" is verified from lexicon
        let results = resolve_ambiguity("khoma", "খমা", &trie);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "ক্ষমা");
        assert_eq!(results[0].1, 165);

        // biggan primary is "বিগ্গান"; "বিজ্ঞান" is verified from lexicon
        let results_biggan = resolve_ambiguity("biggan", "বিগ্গান", &trie);
        assert_eq!(results_biggan.len(), 1);
        assert_eq!(results_biggan[0].0, "বিজ্ঞান");
        assert_eq!(results_biggan[0].1, 356);

        // opekkha primary is "অপেক্কহা"; "অপেক্ষা" is verified from lexicon
        let results_opekkha = resolve_ambiguity("opekkha", "অপেক্কহা", &trie);
        assert_eq!(results_opekkha.len(), 1);
        assert_eq!(results_opekkha[0].0, "অপেক্ষা");
        assert_eq!(results_opekkha[0].1, 305);

        // Missing word in lexicon: should yield 0 results
        let results_unverified = resolve_ambiguity("bhasha", "ভাশা", &trie);
        assert!(results_unverified.is_empty());
    }
}
