//! Configuration models, defaults, and semantic validation for Borno.
//!
//! Provides pure, in-memory configuration schemas strictly decoupled from any
//! specific filesystem, host OS, or GUI framework.

use serde::{Deserialize, Serialize};
use crate::layout::FixedLayoutId;

/// Authoritative configuration schema for Borno (Schema Version 1.2.0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BornoConfig {
    /// Schema format version (strictly "1.2.0" for M9).
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

    // --- M9 Customization & Usability Additions ---
    /// Auto-convert single ASCII period ('.') to Bengali Dáñṛi ('।') in phonetic mode.
    #[serde(default = "default_auto_convert_period_to_dari")]
    pub auto_convert_period_to_dari: bool,

    /// Candidate window presentation layout orientation.
    #[serde(default = "default_candidate_orientation")]
    pub candidate_orientation: CandidateOrientation,

    /// Candidate window font size in points (10..=24 pt, default: 14).
    #[serde(default = "default_candidate_font_size")]
    pub candidate_font_size: u32,

    /// Visual appearance theme identifier.
    #[serde(default = "default_active_theme")]
    pub active_theme: ThemeId,

    /// Mode toggle primary shortcut key.
    #[serde(default = "default_mode_toggle_key")]
    pub mode_toggle_key: ModeToggleKey,
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

fn default_auto_convert_period_to_dari() -> bool {
    false
}

fn default_candidate_orientation() -> CandidateOrientation {
    CandidateOrientation::Vertical
}

fn default_candidate_font_size() -> u32 {
    14
}

fn default_active_theme() -> ThemeId {
    ThemeId::DarkGlass
}

fn default_mode_toggle_key() -> ModeToggleKey {
    ModeToggleKey::F12
}

/// Permitted startup modes for BornoEngine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StartupMode {
    Phonetic,
    Fixed,
    EnglishPassthrough,
}

/// Candidate window presentation layout orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CandidateOrientation {
    Vertical,
    Horizontal,
}

/// Visual theme identifiers for Borno candidate presentation and UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeId {
    #[serde(rename = "dark-glass")]
    DarkGlass,
    #[serde(rename = "light-glass")]
    LightGlass,
    #[serde(rename = "high-contrast")]
    HighContrast,
}

/// Permitted mode toggle primary shortcut keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModeToggleKey {
    F12,
    F4,
}

impl Default for BornoConfig {
    fn default() -> Self {
        Self {
            schema_version: "1.2.0".to_string(),
            default_startup_mode: default_startup_mode(),
            preferred_fixed_layout: default_preferred_fixed_layout(),
            candidate_display_limit: default_candidate_display_limit(),
            auto_commit_on_mode_switch: default_auto_commit_on_mode_switch(),
            enable_dictionary_assistance: default_enable_dictionary_assistance(),
            enable_word_completion: default_enable_word_completion(),
            max_dictionary_suggestions: default_max_dictionary_suggestions(),
            auto_convert_period_to_dari: default_auto_convert_period_to_dari(),
            candidate_orientation: default_candidate_orientation(),
            candidate_font_size: default_candidate_font_size(),
            active_theme: default_active_theme(),
            mode_toggle_key: default_mode_toggle_key(),
        }
    }
}

/// Validation errors for configuration invariants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValidationError {
    UnsupportedSchemaVersion(String),
    InvalidCandidateDisplayLimit(usize),
    InvalidMaxDictionarySuggestions(usize),
    InvalidCandidateFontSize(u32),
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(ver) => {
                write!(f, "Unsupported schema version '{ver}'; this application supports '1.0.0', '1.1.0', and '1.2.0'")
            }
            Self::InvalidCandidateDisplayLimit(val) => {
                write!(f, "Candidate display limit must be between 1 and 9 (received {val})")
            }
            Self::InvalidMaxDictionarySuggestions(val) => {
                write!(f, "Maximum dictionary suggestions must be between 1 and 5 (received {val})")
            }
            Self::InvalidCandidateFontSize(val) => {
                write!(f, "Candidate font size must be between 10 and 24 pt (received {val})")
            }
        }
    }
}

impl std::error::Error for ConfigValidationError {}

/// High-level errors encountered during configuration parsing, migration, and validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    MalformedJson(String),
    UnsupportedSchemaVersion(String),
    Validation(ConfigValidationError),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedJson(msg) => write!(f, "Malformed configuration JSON: {msg}"),
            Self::UnsupportedSchemaVersion(ver) => write!(f, "Unsupported schema version '{ver}'"),
            Self::Validation(err) => write!(f, "Configuration validation error: {err}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<ConfigValidationError> for ConfigError {
    fn from(err: ConfigValidationError) -> Self {
        Self::Validation(err)
    }
}

impl BornoConfig {
    /// Strictly validates semantic invariants of the configuration.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.schema_version != "1.0.0" && self.schema_version != "1.1.0" && self.schema_version != "1.2.0" {
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
        if self.candidate_font_size < 10 || self.candidate_font_size > 24 {
            return Err(ConfigValidationError::InvalidCandidateFontSize(
                self.candidate_font_size,
            ));
        }
        Ok(())
    }

    /// Parses, migrates, and validates a JSON string into BornoConfig v1.2.0.
    ///
    /// Preserves legacy configurations ("1.0.0" and "1.1.0") by migrating them to v1.2.0 defaults.
    /// Strictly rejects unsupported schema versions without mutation.
    pub fn from_json_str(json: &str) -> Result<Self, ConfigError> {
        let mut raw_val: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| ConfigError::MalformedJson(e.to_string()))?;

        let version_str = raw_val
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        match version_str.as_str() {
            "1.0.0" => {
                // Migrate from 1.0.0 to 1.2.0
                if let Some(obj) = raw_val.as_object_mut() {
                    obj.entry("enable_dictionary_assistance").or_insert_with(|| serde_json::json!(default_enable_dictionary_assistance()));
                    obj.entry("enable_word_completion").or_insert_with(|| serde_json::json!(default_enable_word_completion()));
                    obj.entry("max_dictionary_suggestions").or_insert_with(|| serde_json::json!(default_max_dictionary_suggestions()));
                    obj.entry("auto_convert_period_to_dari").or_insert_with(|| serde_json::json!(default_auto_convert_period_to_dari()));
                    obj.entry("candidate_orientation").or_insert_with(|| serde_json::json!(default_candidate_orientation()));
                    obj.entry("candidate_font_size").or_insert_with(|| serde_json::json!(default_candidate_font_size()));
                    obj.entry("active_theme").or_insert_with(|| serde_json::json!(default_active_theme()));
                    obj.entry("mode_toggle_key").or_insert_with(|| serde_json::json!(default_mode_toggle_key()));
                    obj.insert("schema_version".to_string(), serde_json::json!("1.2.0"));
                }
            }
            "1.1.0" => {
                // Migrate from 1.1.0 to 1.2.0
                if let Some(obj) = raw_val.as_object_mut() {
                    obj.entry("auto_convert_period_to_dari").or_insert_with(|| serde_json::json!(default_auto_convert_period_to_dari()));
                    obj.entry("candidate_orientation").or_insert_with(|| serde_json::json!(default_candidate_orientation()));
                    obj.entry("candidate_font_size").or_insert_with(|| serde_json::json!(default_candidate_font_size()));
                    obj.entry("active_theme").or_insert_with(|| serde_json::json!(default_active_theme()));
                    obj.entry("mode_toggle_key").or_insert_with(|| serde_json::json!(default_mode_toggle_key()));
                    obj.insert("schema_version".to_string(), serde_json::json!("1.2.0"));
                }
            }
            "1.2.0" => {
                // Target schema
            }
            other => {
                return Err(ConfigError::UnsupportedSchemaVersion(other.to_string()));
            }
        }

        let config: BornoConfig = serde_json::from_value(raw_val)
            .map_err(|e| ConfigError::MalformedJson(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = BornoConfig::default();
        assert_eq!(config.schema_version, "1.2.0");
        assert_eq!(config.default_startup_mode, StartupMode::Phonetic);
        assert_eq!(config.preferred_fixed_layout, FixedLayoutId::National);
        assert_eq!(config.candidate_display_limit, 5);
        assert!(config.auto_commit_on_mode_switch);
        assert!(config.enable_dictionary_assistance);
        assert!(config.enable_word_completion);
        assert_eq!(config.max_dictionary_suggestions, 3);
        assert!(!config.auto_convert_period_to_dari);
        assert_eq!(config.candidate_orientation, CandidateOrientation::Vertical);
        assert_eq!(config.candidate_font_size, 14);
        assert_eq!(config.active_theme, ThemeId::DarkGlass);
        assert_eq!(config.mode_toggle_key, ModeToggleKey::F12);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_json_serialization_roundtrip() {
        let config = BornoConfig {
            schema_version: "1.2.0".to_string(),
            default_startup_mode: StartupMode::Fixed,
            preferred_fixed_layout: FixedLayoutId::Probhat,
            candidate_display_limit: 3,
            auto_commit_on_mode_switch: false,
            enable_dictionary_assistance: false,
            enable_word_completion: true,
            max_dictionary_suggestions: 2,
            auto_convert_period_to_dari: true,
            candidate_orientation: CandidateOrientation::Horizontal,
            candidate_font_size: 16,
            active_theme: ThemeId::HighContrast,
            mode_toggle_key: ModeToggleKey::F4,
        };

        let json = serde_json::to_string(&config).expect("Serialization failed");
        let deserialized: BornoConfig = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(config, deserialized);
        assert!(deserialized.validate().is_ok());
    }

    #[test]
    fn test_config_validation_accepts_v1_0_0_v1_1_0_and_v1_2_0() {
        let mut config = BornoConfig::default();
        config.schema_version = "1.0.0".to_string();
        assert!(config.validate().is_ok());

        config.schema_version = "1.1.0".to_string();
        assert!(config.validate().is_ok());

        config.schema_version = "1.2.0".to_string();
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
    fn test_config_validation_rejects_candidate_font_size_bounds() {
        let mut config = BornoConfig::default();
        config.candidate_font_size = 9;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidCandidateFontSize(9))
        );

        config.candidate_font_size = 25;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidCandidateFontSize(25))
        );

        config.candidate_font_size = 10;
        assert!(config.validate().is_ok());

        config.candidate_font_size = 24;
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
    fn test_config_from_json_str_migrates_v1_0_0() {
        let json = r#"{"schema_version": "1.0.0", "candidate_display_limit": 4}"#;
        let config = BornoConfig::from_json_str(json).expect("Migration from 1.0.0 should succeed");

        assert_eq!(config.schema_version, "1.2.0");
        assert_eq!(config.candidate_display_limit, 4);
        assert!(config.enable_dictionary_assistance);
        assert_eq!(config.max_dictionary_suggestions, 3);
        assert!(!config.auto_convert_period_to_dari);
        assert_eq!(config.candidate_orientation, CandidateOrientation::Vertical);
        assert_eq!(config.candidate_font_size, 14);
        assert_eq!(config.active_theme, ThemeId::DarkGlass);
        assert_eq!(config.mode_toggle_key, ModeToggleKey::F12);
    }

    #[test]
    fn test_config_from_json_str_migrates_v1_1_0() {
        let json = r#"{"schema_version": "1.1.0", "max_dictionary_suggestions": 4, "enable_dictionary_assistance": false}"#;
        let config = BornoConfig::from_json_str(json).expect("Migration from 1.1.0 should succeed");

        assert_eq!(config.schema_version, "1.2.0");
        assert_eq!(config.max_dictionary_suggestions, 4);
        assert!(!config.enable_dictionary_assistance);
        assert!(!config.auto_convert_period_to_dari);
        assert_eq!(config.candidate_orientation, CandidateOrientation::Vertical);
        assert_eq!(config.candidate_font_size, 14);
    }

    #[test]
    fn test_config_from_json_str_rejects_unsupported_version() {
        let json = r#"{"schema_version": "1.3.0"}"#;
        let err = BornoConfig::from_json_str(json).expect_err("Should reject version 1.3.0");
        assert_eq!(err, ConfigError::UnsupportedSchemaVersion("1.3.0".to_string()));
    }

    #[test]
    fn test_config_from_json_str_rejects_malformed_json() {
        let json = r#"{"schema_version": "1.2.0", invalid_syntax"#;
        let err = BornoConfig::from_json_str(json).expect_err("Should reject malformed JSON");
        match err {
            ConfigError::MalformedJson(_) => (),
            other => panic!("Expected MalformedJson, got {:?}", other),
        }
    }
}
