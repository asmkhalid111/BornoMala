//! Deterministic candidate ranker and lockstep metadata synchronizer.

use crate::{CandidateDetail, CandidateKind};
use crate::unicode;

/// Merges, deduplicates, and ranks candidate suggestions according to M6 governance rules.
///
/// Invariants:
/// 1. Slot 0 is ALWAYS the raw deterministic primary transliteration (zero hijacking).
/// 2. Primary transliteration does NOT count toward `max_dict_suggestions`.
/// 3. Secondary candidates are merged in priority order:
///    - ExactLexiconMatch (from Phonetic Ambiguity pipeline)
///    - PhoneticAlternative (from M3 rule engine)
///    - PrefixCompletion (from Trie prefix search)
///    - SpellingCorrection (from Bounded Grapheme Search, only if completions == 0)
/// 4. Deduplication: NFC normalized comparison against already admitted candidate texts.
/// 5. Lockstep slicing: candidates and candidate_details are sliced to `display_limit` synchronously.
pub fn rank_candidates(
    primary: String,
    m3_alternates: Vec<String>,
    ambiguity_matches: Vec<(String, u32)>,
    completions: Vec<(String, u32)>,
    corrections: Vec<(String, u32, usize)>,
    enable_dictionary: bool,
    enable_completion: bool,
    max_dict_suggestions: usize,
    display_limit: usize,
) -> (Vec<String>, Vec<CandidateDetail>) {
    let norm_primary = unicode::normalize_nfc(&primary);

    let mut candidates = Vec::with_capacity(display_limit);
    let mut details = Vec::with_capacity(display_limit);

    // Slot 0 Invariant: User's raw transliteration is always Candidate 0
    candidates.push(norm_primary.clone());
    details.push(CandidateDetail {
        text: norm_primary.clone(),
        kind: CandidateKind::PrimaryTransliteration,
        score: 0,
    });

    // Helper closure to push if not duplicate
    let mut push_candidate = |text: String, kind: CandidateKind, score: u32, dict_count: &mut usize| {
        let norm_text = unicode::normalize_nfc(&text);
        if !candidates.iter().any(|c| c == &norm_text) {
            candidates.push(norm_text.clone());
            details.push(CandidateDetail {
                text: norm_text,
                kind,
                score,
            });
            match kind {
                CandidateKind::ExactLexiconMatch
                | CandidateKind::PrefixCompletion
                | CandidateKind::SpellingCorrection => {
                    *dict_count += 1;
                }
                _ => {}
            }
        }
    };

    let mut dict_count = 0;

    if enable_dictionary {
        // Priority 1: ExactLexiconMatch from Phonetic Ambiguity
        for (word, freq) in ambiguity_matches {
            if dict_count >= max_dict_suggestions {
                break;
            }
            push_candidate(word, CandidateKind::ExactLexiconMatch, freq, &mut dict_count);
        }

        // Priority 2: M3 Rule-based Phonetic Alternatives (do NOT consume dict budget)
        for alt in m3_alternates {
            push_candidate(alt, CandidateKind::PhoneticAlternative, 0, &mut dict_count);
        }

        // Priority 3: Prefix Completions
        if enable_completion {
            for (word, freq) in completions {
                if dict_count >= max_dict_suggestions {
                    break;
                }
                push_candidate(word, CandidateKind::PrefixCompletion, freq, &mut dict_count);
            }
        }

        // Priority 4: Spelling Corrections (active ONLY if no completions and budget permits)
        for (word, freq, _dist) in corrections {
            if dict_count >= max_dict_suggestions {
                break;
            }
            push_candidate(word, CandidateKind::SpellingCorrection, freq, &mut dict_count);
        }
    } else {
        // Dictionary disabled: only M3 rule-based alternatives permitted
        for alt in m3_alternates {
            push_candidate(alt, CandidateKind::PhoneticAlternative, 0, &mut dict_count);
        }
    }

    // Lockstep slicing to display_limit (1..=9)
    if candidates.len() > display_limit {
        candidates.truncate(display_limit);
        details.truncate(display_limit);
    }

    (candidates, details)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot0_invariant_strictly_preserved() {
        let (cands, details) = rank_candidates(
            "খমা".to_string(),
            vec![],
            vec![("ক্ষমা".to_string(), 165)],
            vec![],
            vec![],
            true,
            true,
            3,
            5,
        );

        assert_eq!(cands[0], "খমা");
        assert_eq!(details[0].kind, CandidateKind::PrimaryTransliteration);
        assert_eq!(cands[1], "ক্ষমা");
        assert_eq!(details[1].kind, CandidateKind::ExactLexiconMatch);
        assert_eq!(cands.len(), details.len());
    }

    #[test]
    fn test_lockstep_slicing_k1() {
        let (cands, details) = rank_candidates(
            "বাং".to_string(),
            vec![],
            vec![],
            vec![("বাংলাদেশ".to_string(), 4987), ("বাংলা".to_string(), 690)],
            vec![],
            true,
            true,
            3,
            1, // Display limit K = 1
        );

        assert_eq!(cands.len(), 1);
        assert_eq!(details.len(), 1);
        assert_eq!(cands[0], "বাং");
        assert_eq!(details[0].kind, CandidateKind::PrimaryTransliteration);
    }

    #[test]
    fn test_dictionary_disabled_isolation() {
        let (cands, details) = rank_candidates(
            "khoma".to_string(),
            vec!["খমা".to_string()],
            vec![("ক্ষমা".to_string(), 165)],
            vec![("বাংলাদেশ".to_string(), 4987)],
            vec![],
            false, // Dictionary disabled
            true,
            3,
            5,
        );

        assert_eq!(cands.len(), 2);
        assert_eq!(details[0].kind, CandidateKind::PrimaryTransliteration);
        assert_eq!(details[1].kind, CandidateKind::PhoneticAlternative);
        assert!(!cands.contains(&"ক্ষমা".to_string()));
        assert!(!cands.contains(&"বাংলাদেশ".to_string()));
    }
}
