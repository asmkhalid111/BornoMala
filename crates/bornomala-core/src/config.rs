//! Configuration models, defaults, and semantic validation for Borno.
//!
//! Provides pure, in-memory configuration schemas strictly decoupled from any
//! specific filesystem, host OS, or GUI framework.

use serde::{Deserialize, Serialize};
use crate::layout::FixedLayoutId;

/// Authoritative configuration schema for Borno (Schema Version 1.0.0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BornoConfig {
    /// Schema format version (strictly "1.0.0" for M5).
    pub schema_version: String,

    /// Mode to initialize when the engine starts up.
    #[serde(default = "default_startup_mode")]
    pub default_startup_mode: StartupMode,

    /// Preferred fixed keyboard layout when switching to Fixed mode.
    #[serde(default = "default_preferred_fixed_layout")]
    pub preferred_fixed_layout: FixedLayoutId,

    /// Maximum number of candidates presented in candidate window (1..9, default 5).
    #[serde(default = "default_candidate_display_limit")]
    pub candidate_display_limit: usize,

    /// Auto-commit active preedit composition when switching input modes.
    #[serde(default = "default_auto_commit_on_mode_switch")]
    pub auto_commit_on_mode_switch: bool,

    // --- M6 Dictionary Assistance Additions ---
    /// Master toggle for offline dictionary suggestions, phonetic ambiguity, and word completions.
    #[serde(default = "default_enable_dictionary_assistance")]
    pub enable_dictionary_assistance: bool,

    /// Sub-toggle enabling prefix word completions while typing.
    #[serde(default = "default_enable_word_completion")]
    pub enable_word_completion: bool,

    /// Maximum number of dictionary suggestions allowed in the candidate list (1..=5, default 3).
    #[serde(default = "default_max_dictionary_suggestions")]
    pub max_dictionary_suggestions: usize,
}

fn default_startup_mode() -> StartupMode {
    StartupMode::Phonetic
}

fn default_preferred_fixed_layout() -> FixedLayoutId {
    FixedLayoutId::National
}

fn default_candidate_display_limit() -> usize {
    5
}

fn default_auto_commit_on_mode_switch() -> bool {
    true
}

fn default_enable_dictionary_assistance() -> bool {
    true
}

fn default_enable_word_completion() -> bool {
    true
}

fn default_max_dictionary_suggestions() -> usize {
    3
}

/// Permitted startup modes for BornoEngine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StartupMode {
    Phonetic,
    Fixed,
    EnglishPassthrough,
}

impl Default for BornoConfig {
    fn default() -> Self {
        Self {
            schema_version: "1.1.0".to_string(),
            default_startup_mode: default_startup_mode(),
            preferred_fixed_layout: default_preferred_fixed_layout(),
            candidate_display_limit: default_candidate_display_limit(),
            auto_commit_on_mode_switch: default_auto_commit_on_mode_switch(),
            enable_dictionary_assistance: default_enable_dictionary_assistance(),
            enable_word_completion: default_enable_word_completion(),
            max_dictionary_suggestions: default_max_dictionary_suggestions(),
        }
    }
}

/// Validation errors for configuration invariants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValidationError {
    UnsupportedSchemaVersion(String),
    InvalidCandidateDisplayLimit(usize),
    InvalidMaxDictionarySuggestions(usize),
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(ver) => {
                write!(f, "Unsupported schema version '{ver}'; this application supports '1.0.0' and '1.1.0'")
            }
            Self::InvalidCandidateDisplayLimit(val) => {
                write!(f, "Candidate display limit must be between 1 and 9 (received {val})")
            }
            Self::InvalidMaxDictionarySuggestions(val) => {
                write!(f, "Maximum dictionary suggestions must be between 1 and 5 (received {val})")
            }
        }
    }
}

impl std::error::Error for ConfigValidationError {}

impl BornoConfig {
    /// Strictly validates semantic invariants of the configuration.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.schema_version != "1.0.0" && self.schema_version != "1.1.0" {
            return Err(ConfigValidationError::UnsupportedSchemaVersion(
                self.schema_version.clone(),
            ));
        }
        if self.candidate_display_limit < 1 || self.candidate_display_limit > 9 {
            return Err(ConfigValidationError::InvalidCandidateDisplayLimit(
                self.candidate_display_limit,
            ));
        }
        if self.max_dictionary_suggestions < 1 || self.max_dictionary_suggestions > 5 {
            return Err(ConfigValidationError::InvalidMaxDictionarySuggestions(
                self.max_dictionary_suggestions,
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = BornoConfig::default();
        assert_eq!(config.schema_version, "1.1.0");
        assert_eq!(config.default_startup_mode, StartupMode::Phonetic);
        assert_eq!(config.preferred_fixed_layout, FixedLayoutId::National);
        assert_eq!(config.candidate_display_limit, 5);
        assert!(config.auto_commit_on_mode_switch);
        assert!(config.enable_dictionary_assistance);
        assert!(config.enable_word_completion);
        assert_eq!(config.max_dictionary_suggestions, 3);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_json_serialization_roundtrip() {
        let config = BornoConfig {
            schema_version: "1.1.0".to_string(),
            default_startup_mode: StartupMode::Fixed,
            preferred_fixed_layout: FixedLayoutId::Probhat,
            candidate_display_limit: 3,
            auto_commit_on_mode_switch: false,
            enable_dictionary_assistance: false,
            enable_word_completion: true,
            max_dictionary_suggestions: 2,
        };

        let json = serde_json::to_string(&config).expect("Serialization failed");
        let deserialized: BornoConfig = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(config, deserialized);
        assert!(deserialized.validate().is_ok());
    }

    #[test]
    fn test_config_validation_accepts_v1_0_0_and_v1_1_0() {
        let mut config = BornoConfig::default();
        config.schema_version = "1.0.0".to_string();
        assert!(config.validate().is_ok());

        config.schema_version = "1.1.0".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_rejects_candidate_limit_bounds() {
        let mut config = BornoConfig::default();
        config.candidate_display_limit = 0;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidCandidateDisplayLimit(0))
        );

        config.candidate_display_limit = 10;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidCandidateDisplayLimit(10))
        );
    }

    #[test]
    fn test_config_validation_rejects_max_dictionary_suggestions_bounds() {
        let mut config = BornoConfig::default();
        config.max_dictionary_suggestions = 0;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidMaxDictionarySuggestions(0))
        );

        config.max_dictionary_suggestions = 6;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidMaxDictionarySuggestions(6))
        );

        config.max_dictionary_suggestions = 1;
        assert!(config.validate().is_ok());

        config.max_dictionary_suggestions = 5;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_rejects_unsupported_schema_version() {
        let mut config = BornoConfig::default();
        config.schema_version = "2.0.0".to_string();
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::UnsupportedSchemaVersion("2.0.0".to_string()))
        );

        config.schema_version = "0.9.0".to_string();
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::UnsupportedSchemaVersion("0.9.0".to_string()))
        );
    }

    #[test]
    fn test_config_deserialization_handles_missing_fields_safely() {
        // Minimal JSON containing only legacy 1.0.0 schema_version
        let json = r#"{"schema_version": "1.0.0"}"#;
        let config: BornoConfig = serde_json::from_str(json).expect("Deserialization should succeed with defaults");

        assert_eq!(config.schema_version, "1.0.0");
        assert_eq!(config.default_startup_mode, StartupMode::Phonetic);
        assert_eq!(config.preferred_fixed_layout, FixedLayoutId::National);
        assert_eq!(config.candidate_display_limit, 5);
        assert!(config.auto_commit_on_mode_switch);
        assert!(config.enable_dictionary_assistance);
        assert!(config.enable_word_completion);
        assert_eq!(config.max_dictionary_suggestions, 3);
        assert!(config.validate().is_ok());
    }
}
