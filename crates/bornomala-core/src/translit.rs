//! Transliteration compatibility module for Borno Core.
//!
//! In Milestone M3, this delegates directly to the comprehensive, Avro-compatible
//! `phonetic::PhoneticEngine`.

/// Transliterates a given ASCII input string into Bangla Unicode.
pub fn transliterate(input: &str) -> String {
    crate::phonetic::transliterate(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_consonant_k() {
        assert_eq!(transliterate("k"), "ক");
    }

    #[test]
    fn test_consonant_vowel_ka() {
        assert_eq!(transliterate("ka"), "কা");
    }

    #[test]
    fn test_word_ami() {
        assert_eq!(transliterate("ami"), "আমি");
    }

    #[test]
    fn test_word_kor() {
        assert_eq!(transliterate("kor"), "কর");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(transliterate(""), "");
    }
}
