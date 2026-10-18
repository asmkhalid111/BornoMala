//! Phonetic tokenizer and contextual scanner with TokenSpan tracking.
//!
//! Scans Latin ASCII input greedily (longest-prefix match first) and emits
//! Bangla Unicode characters while maintaining bidirectional coordinate mapping.

use super::rules::*;
use crate::unicode;

/// A bidirectional mapping span connecting a Latin source byte range
/// to a Bangla Unicode scalar (codepoint) range in the preedit string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenSpan {
    /// Byte range [start, end) in the ASCII raw_buffer.
    pub source_range: std::ops::Range<usize>,
    /// Codepoint range [start, end) in the normalized Bangla preedit.
    pub target_range: std::ops::Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LastTokenKind {
    Consonant,
    Vowel,
    Other,
}

/// Tokenizer that parses Latin input into raw Bangla Unicode and TokenSpans.
pub struct Tokenizer;

impl Tokenizer {
    /// Tokenizes the ASCII input string into raw Bangla Unicode and a list of TokenSpans.
    pub fn tokenize(input: &str) -> (String, Vec<TokenSpan>) {
        if input.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut output = String::new();
        let mut spans = Vec::new();
        let bytes = input.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        let mut last_kind = LastTokenKind::Other;

        while i < len {
            let start_byte = i;
            let start_codepoints = output.chars().count();
            let rem = &input[i..];

            // ----------------------------------------------------------------
            // 1. Escaped tokens and boundary controls
            // ----------------------------------------------------------------

            // Khanda-Ta: "t`" (when not separating "th") or "\t"
            if (rem.starts_with("t`") && !rem.starts_with("t`h")) || rem.starts_with("\\t") {
                output.push_str(KHANDA_TA);
                let matched_len = 2;
                i += matched_len;
                last_kind = LastTokenKind::Consonant;
                let end_codepoints = output.chars().count();
                spans.push(TokenSpan {
                    source_range: start_byte..i,
                    target_range: start_codepoints..end_codepoints,
                });
                continue;
            }

            // Explicit Virama / Hasanta: ",," or "`" after consonant
            if rem.starts_with(",,") {
                output.push_str(VIRAMA);
                i += 2;
                last_kind = LastTokenKind::Consonant;
                let end_codepoints = output.chars().count();
                spans.push(TokenSpan {
                    source_range: start_byte..i,
                    target_range: start_codepoints..end_codepoints,
                });
                continue;
            }

            // Backtick token break or backtick virama
            if rem.starts_with('`') {
                if last_kind == LastTokenKind::Consonant && rem.starts_with("``") {
                    // Double backtick: explicit virama
                    output.push_str(VIRAMA);
                    i += 2;
                    last_kind = LastTokenKind::Consonant;
                    let end_codepoints = output.chars().count();
                    spans.push(TokenSpan {
                        source_range: start_byte..i,
                        target_range: start_codepoints..end_codepoints,
                    });
                    continue;
                } else if last_kind == LastTokenKind::Consonant && rem.len() == 1 {
                    // Trailing backtick after consonant -> explicit virama
                    output.push_str(VIRAMA);
                    i += 1;
                    last_kind = LastTokenKind::Consonant;
                    let end_codepoints = output.chars().count();
                    spans.push(TokenSpan {
                        source_range: start_byte..i,
                        target_range: start_codepoints..end_codepoints,
                    });
                    continue;
                } else {
                    // Single backtick token break: breaks greedy pattern and resets context
                    last_kind = LastTokenKind::Other;
                    i += 1;
                    // Token break emits 0 target codepoints
                    spans.push(TokenSpan {
                        source_range: start_byte..i,
                        target_range: start_codepoints..start_codepoints,
                    });
                    continue;
                }
            }

            // Candrabindu: "^"
            if rem.starts_with('^') {
                output.push_str(CANDRABINDU);
                i += 1;
                let end_codepoints = output.chars().count();
                spans.push(TokenSpan {
                    source_range: start_byte..i,
                    target_range: start_codepoints..end_codepoints,
                });
                continue;
            }

            // Dari / Punctuation: "||" -> Double Dari, "|" -> Dari
            if rem.starts_with("||") {
                output.push('\u{09E5}'); // ॥
                i += 2;
                last_kind = LastTokenKind::Other;
                let end_codepoints = output.chars().count();
                spans.push(TokenSpan {
                    source_range: start_byte..i,
                    target_range: start_codepoints..end_codepoints,
                });
                continue;
            }
            if rem.starts_with('|') {
                output.push('\u{09E4}'); // ।
                i += 1;
                last_kind = LastTokenKind::Other;
                let end_codepoints = output.chars().count();
                spans.push(TokenSpan {
                    source_range: start_byte..i,
                    target_range: start_codepoints..end_codepoints,
                });
                continue;
            }

            // Visarga: ":"
            if rem.starts_with(':') {
                output.push_str(VISARGA);
                i += 1;
                last_kind = LastTokenKind::Other;
                let end_codepoints = output.chars().count();
                spans.push(TokenSpan {
                    source_range: start_byte..i,
                    target_range: start_codepoints..end_codepoints,
                });
                continue;
            }

            // ----------------------------------------------------------------
            // 2. Reph: "r" followed immediately by a consonant
            // ----------------------------------------------------------------
            if rem.starts_with('r') && rem.len() > 1 && !rem.starts_with("rri") {
                let next_char = rem.as_bytes()[1] as char;
                let is_next_vowel = matches!(next_char, 'a' | 'A' | 'e' | 'E' | 'i' | 'I' | 'o' | 'O' | 'u' | 'U');
                let is_next_r = next_char == 'r' || next_char == 'R';
                let is_next_symbol = matches!(next_char, ' ' | '`' | '^' | '|' | ':' | ',' | '.' | '!' | '?');

                if !is_next_vowel && !is_next_r && !is_next_symbol {
                    // "r" preceding a consonant acts as Reph
                    output.push_str(REPH_PREFIX);
                    i += 1;
                    last_kind = LastTokenKind::Consonant;
                    let end_codepoints = output.chars().count();
                    spans.push(TokenSpan {
                        source_range: start_byte..i,
                        target_range: start_codepoints..end_codepoints,
                    });
                    continue;
                }
            }

            // ----------------------------------------------------------------
            // 3. Phalas following a consonant
            // ----------------------------------------------------------------
            if last_kind == LastTokenKind::Consonant {
                // Ya-phala: "y" or "Z" following consonant
                if rem.starts_with('y') || rem.starts_with('Z') {
                    output.push_str(YA_PHALA);
                    i += 1;
                    last_kind = LastTokenKind::Consonant;
                    let end_codepoints = output.chars().count();
                    spans.push(TokenSpan {
                        source_range: start_byte..i,
                        target_range: start_codepoints..end_codepoints,
                    });
                    continue;
                }

                // Ba-phala: "w" or "v" following consonant
                if rem.starts_with('w') || rem.starts_with('v') {
                    output.push_str(BA_PHALA);
                    i += 1;
                    last_kind = LastTokenKind::Consonant;
                    let end_codepoints = output.chars().count();
                    spans.push(TokenSpan {
                        source_range: start_byte..i,
                        target_range: start_codepoints..end_codepoints,
                    });
                    continue;
                }

                // Ra-phala: "r" following consonant (when not followed by "rri")
                if rem.starts_with('r') && !rem.starts_with("rri") {
                    output.push_str(RA_PHALA);
                    i += 1;
                    last_kind = LastTokenKind::Consonant;
                    let end_codepoints = output.chars().count();
                    spans.push(TokenSpan {
                        source_range: start_byte..i,
                        target_range: start_codepoints..end_codepoints,
                    });
                    continue;
                }
            }

            // ----------------------------------------------------------------
            // 4. Conjuncts: Greedy matching from CONJUNCT_RULES
            // ----------------------------------------------------------------
            let mut matched_conjunct = false;
            for pattern in CONJUNCT_RULES {
                if rem.starts_with(pattern.latin) {
                    output.push_str(pattern.bangla);
                    i += pattern.latin.len();
                    last_kind = LastTokenKind::Consonant;
                    let end_codepoints = output.chars().count();
                    spans.push(TokenSpan {
                        source_range: start_byte..i,
                        target_range: start_codepoints..end_codepoints,
                    });
                    matched_conjunct = true;
                    break;
                }
            }
            if matched_conjunct {
                continue;
            }

            // ----------------------------------------------------------------
            // 5. Vowels: Independent vs Dependent (Kar) Forms
            // ----------------------------------------------------------------
            let mut matched_vowel = false;
            for v_rule in VOWEL_RULES {
                if rem.starts_with(v_rule.latin) {
                    if last_kind == LastTokenKind::Consonant {
                        // After a consonant:
                        if v_rule.is_inherent_o {
                            // Inherent vowel 'o': absorbs inherent vowel without emitting a kar
                            i += v_rule.latin.len();
                            last_kind = LastTokenKind::Vowel;
                            let end_codepoints = output.chars().count();
                            spans.push(TokenSpan {
                                source_range: start_byte..i,
                                target_range: start_codepoints..end_codepoints,
                            });
                            matched_vowel = true;
                            break;
                        } else if let Some(dep) = v_rule.dependent {
                            // If output currently ends with CANDRABINDU, swap so vowel sign precedes Candrabindu
                            if output.ends_with(CANDRABINDU) {
                                output.pop(); // remove Candrabindu
                                output.push_str(dep);
                                output.push_str(CANDRABINDU);
                                i += v_rule.latin.len();
                                last_kind = LastTokenKind::Vowel;
                                let end_codepoints = output.chars().count();
                                spans.push(TokenSpan {
                                    source_range: start_byte..i,
                                    target_range: (end_codepoints - 2)..(end_codepoints - 1),
                                });
                                if spans.len() >= 2 {
                                    let prev_idx = spans.len() - 2;
                                    spans[prev_idx].target_range = (end_codepoints - 1)..end_codepoints;
                                }
                            } else {
                                output.push_str(dep);
                                i += v_rule.latin.len();
                                last_kind = LastTokenKind::Vowel;
                                let end_codepoints = output.chars().count();
                                spans.push(TokenSpan {
                                    source_range: start_byte..i,
                                    target_range: start_codepoints..end_codepoints,
                                });
                            }
                            matched_vowel = true;
                            break;
                        }
                    } else {
                        // Initial, standalone, or after another vowel: emit independent form
                        output.push_str(v_rule.independent);
                        i += v_rule.latin.len();
                        last_kind = LastTokenKind::Vowel;
                        let end_codepoints = output.chars().count();
                        spans.push(TokenSpan {
                            source_range: start_byte..i,
                            target_range: start_codepoints..end_codepoints,
                        });
                        matched_vowel = true;
                        break;
                    }
                }
            }
            if matched_vowel {
                continue;
            }

            // ----------------------------------------------------------------
            // 6. Consonants: Greedy matching from CONSONANT_RULES
            // ----------------------------------------------------------------
            let mut matched_consonant = false;
            for pattern in CONSONANT_RULES {
                if rem.starts_with(pattern.latin) {
                    output.push_str(pattern.bangla);
                    i += pattern.latin.len();
                    last_kind = LastTokenKind::Consonant;
                    let end_codepoints = output.chars().count();
                    spans.push(TokenSpan {
                        source_range: start_byte..i,
                        target_range: start_codepoints..end_codepoints,
                    });
                    matched_consonant = true;
                    break;
                }
            }
            if matched_consonant {
                continue;
            }

            // ----------------------------------------------------------------
            // 7. Non-Bangla fallback (ASCII passthrough, punctuation, spaces)
            // ----------------------------------------------------------------
            let ch = rem.chars().next().unwrap();
            output.push(ch);
            i += ch.len_utf8();
            last_kind = LastTokenKind::Other;
            let end_codepoints = output.chars().count();
            spans.push(TokenSpan {
                source_range: start_byte..i,
                target_range: start_codepoints..end_codepoints,
            });
        }

        (output, spans)
    }

    /// Realignment pass: updates target spans after Unicode normalization
    /// (e.g. collapsing two-part vowels or decomposing nukta consonants).
    pub fn realign_spans_after_nfc(
        pre_nfc_output: &str,
        normalized_output: &str,
        spans: &mut [TokenSpan],
    ) {
        if pre_nfc_output == normalized_output {
            return;
        }

        for span in spans.iter_mut() {
            let realigned_start = unicode::map_cursor_after_normalization(
                pre_nfc_output,
                span.target_range.start,
                normalized_output,
            );
            let realigned_end = unicode::map_cursor_after_normalization(
                pre_nfc_output,
                span.target_range.end,
                normalized_output,
            );
            span.target_range = realigned_start..realigned_end.max(realigned_start);
        }
    }

    /// Maps an ASCII source byte cursor into target preedit Unicode scalar offset.
    ///
    /// Applies:
    /// - End-of-buffer rule: S == raw_buffer.len() => T = preedit.chars().count()
    /// - Token boundary rule: S == token.start => T = token.target.start, S == token.end => T = token.target.end
    /// - Left affinity: token.start < S < token.end => T = token.target.start
    pub fn map_cursor_to_target(
        source_cursor: usize,
        raw_buffer: &str,
        spans: &[TokenSpan],
        preedit: &str,
    ) -> usize {
        let max_target = preedit.chars().count();
        if source_cursor == 0 || raw_buffer.is_empty() {
            return 0;
        }
        if source_cursor >= raw_buffer.len() {
            return max_target;
        }

        for span in spans {
            if source_cursor == span.source_range.start {
                return span.target_range.start.min(max_target);
            }
            if source_cursor == span.source_range.end {
                return span.target_range.end.min(max_target);
            }
            if source_cursor > span.source_range.start && source_cursor < span.source_range.end {
                // Rule 3: Left affinity for interior edits
                return span.target_range.start.min(max_target);
            }
        }

        max_target
    }
}
