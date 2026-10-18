//! Offline dictionary engine, predictive lexicon, and phonetic ambiguity pipeline.

pub mod trie;
pub mod distance;
pub mod ambiguity;
pub mod ranker;

pub use trie::PrefixTrie;
pub use ambiguity::PHONETIC_AMBIGUITY_RULES;
pub use distance::grapheme_damerau_levenshtein;

use std::sync::OnceLock;
use crate::CandidateDetail;

/// Compile-time embedded binary lexicon generated from Leipzig CC BY 4.0 corpus.
pub static EMBEDDED_LEXICON_BYTES: &[u8] = include_bytes!("../../data/lexicon.bin");

static DEFAULT_ENGINE: OnceLock<DictionaryEngine> = OnceLock::new();

/// High-performance offline dictionary engine.
pub struct DictionaryEngine {
    trie: PrefixTrie,
}

impl Default for DictionaryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DictionaryEngine {
    /// Initializes a new DictionaryEngine by parsing the embedded binary lexicon.
    pub fn new() -> Self {
        let trie = PrefixTrie::from_binary(EMBEDDED_LEXICON_BYTES)
            .expect("Failed to initialize DictionaryEngine from embedded BLX1 binary");
        Self { trie }
    }

    /// Accesses the global shared reference to the DictionaryEngine (lazily initialized).
    pub fn global() -> &'static DictionaryEngine {
        DEFAULT_ENGINE.get_or_init(Self::new)
    }

    /// Returns a reference to the underlying prefix trie.
    pub fn trie(&self) -> &PrefixTrie {
        &self.trie
    }

    /// Primary query method: evaluates ambiguity, completions, or corrections,
    /// and returns ranked candidates and details adhering to all M6 invariants.
    pub fn query_candidates(
        &self,
        raw_buffer: &str,
        primary_transliteration: &str,
        m3_alternates: Vec<String>,
        enable_dictionary: bool,
        enable_completion: bool,
        max_dict_suggestions: usize,
        display_limit: usize,
    ) -> (Vec<String>, Vec<CandidateDetail>) {
        if !enable_dictionary {
            return ranker::rank_candidates(
                primary_transliteration.to_string(),
                m3_alternates,
                vec![],
                vec![],
                vec![],
                false,
                false,
                0,
                display_limit,
            );
        }

        // Step 1: Phonetic Ambiguity Resolution
        let ambiguity_matches = ambiguity::resolve_ambiguity(
            raw_buffer,
            primary_transliteration,
            &self.trie,
        );

        // Step 2: Prefix Completion Eligibility & Trie completions check
        let is_completion_eligible =
            primary_transliteration.chars().count() >= 2 || raw_buffer.len() >= 2;

        let has_trie_completions = if is_completion_eligible {
            !self.trie.find_completions(primary_transliteration, 1).is_empty()
        } else {
            false
        };

        let completions = if enable_completion && is_completion_eligible {
            self.trie.find_completions(primary_transliteration, max_dict_suggestions)
        } else {
            Vec::new()
        };

        // Step 3: Spelling Correction Eligibility
        // Invariant: Active ONLY when character length >= 3 AND the trie has 0 completions for this prefix
        let corrections = if !has_trie_completions && primary_transliteration.chars().count() >= 3 {
            distance::search_spelling_corrections(&self.trie, primary_transliteration, max_dict_suggestions)
        } else {
            Vec::new()
        };

        // Step 4: Rank and merge
        ranker::rank_candidates(
            primary_transliteration.to_string(),
            m3_alternates,
            ambiguity_matches,
            completions,
            corrections,
            enable_dictionary,
            enable_completion,
            max_dict_suggestions,
            display_limit,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CandidateKind;

    #[test]
    fn test_dictionary_engine_loads_embedded_lexicon() {
        let engine = DictionaryEngine::global();
        assert!(engine.trie().word_count >= 25000);

        // Target words presence
        assert!(engine.trie().contains("ক্ষমা"));
        assert!(engine.trie().contains("বিজ্ঞান"));
        assert!(engine.trie().contains("ভাষা"));
        assert!(engine.trie().contains("বাংলাদেশ"));
        assert!(engine.trie().contains("বাংলা"));
        assert!(engine.trie().contains("বাঙালি"));
        assert!(engine.trie().contains("কষ্ট"));
        assert!(engine.trie().contains("বৃষ্টি"));
    }

    #[test]
    fn test_query_candidates_khoma_ambiguity() {
        let engine = DictionaryEngine::global();
        let (cands, details) = engine.query_candidates(
            "khoma",
            "খমা",
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
    }

    #[test]
    fn test_query_candidates_bang_completions() {
        let engine = DictionaryEngine::global();
        let (cands, details) = engine.query_candidates(
            "bang",
            "বাং",
            vec![],
            true,
            true,
            3,
            5,
        );

        assert_eq!(cands[0], "বাং");
        assert_eq!(details[0].kind, CandidateKind::PrimaryTransliteration);
        assert!(cands.len() > 1);
        assert_eq!(details[1].kind, CandidateKind::PrefixCompletion);
        assert!(cands.contains(&"বাংলাদেশ".to_string()));
    }
}
