//! Comprehensive, Avro-compatible phonetic input engine for Borno Core.
//!
//! Replaces the minimal M0 transliteration stub with a production-grade,
//! deterministic phonetic tokenizer, rule matching tables, bidirectional
//! span tracking, and bounded candidate generation.

pub mod candidates;
pub mod rules;
pub mod tokenizer;

pub use candidates::CandidateGenerator;
pub use rules::{RuleClassification, PhoneticPattern};
pub use tokenizer::{Tokenizer, TokenSpan};

use crate::unicode;

/// The primary phonetic engine interface for transliteration and cursor tracking.
#[derive(Debug, Clone, Copy, Default)]
pub struct PhoneticEngine;

impl PhoneticEngine {
    /// Transliterates ASCII raw input into primary normalized Bangla preedit,
    /// a list of bidirectional TokenSpans, and ranked candidates.
    pub fn transliterate(input: &str) -> (String, Vec<TokenSpan>, Vec<String>) {
        if input.is_empty() {
            return (String::new(), Vec::new(), Vec::new());
        }

        let (raw_bangla, mut spans) = Tokenizer::tokenize(input);
        let sanitized = unicode::sanitize_composition(&raw_bangla);
        let normalized = unicode::normalize_nfc(&sanitized);

        Tokenizer::realign_spans_after_nfc(&raw_bangla, &normalized, &mut spans);

        let candidates = CandidateGenerator::generate_candidates(input, &normalized);

        (normalized, spans, candidates)
    }

    /// Maps an ASCII source byte cursor into target preedit Unicode scalar offset.
    pub fn map_cursor_to_target(
        source_cursor: usize,
        raw_buffer: &str,
        spans: &[TokenSpan],
        preedit: &str,
    ) -> usize {
        Tokenizer::map_cursor_to_target(source_cursor, raw_buffer, spans, preedit)
    }
}

/// Convenience standalone transliteration function matching the original translit API.
pub fn transliterate(input: &str) -> String {
    let (preedit, _, _) = PhoneticEngine::transliterate(input);
    preedit
}
