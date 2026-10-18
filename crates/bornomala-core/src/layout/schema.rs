//! Layout schema and validation definitions for Borno fixed keyboard layouts.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// 48 ANSI physical keyboard keys supported by Borno.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum PhysicalKey {
    // Row E (Number Row) - 13 keys
    Backquote = 0,
    Digit1 = 1,
    Digit2 = 2,
    Digit3 = 3,
    Digit4 = 4,
    Digit5 = 5,
    Digit6 = 6,
    Digit7 = 7,
    Digit8 = 8,
    Digit9 = 9,
    Digit0 = 10,
    Minus = 11,
    Equal = 12,

    // Row D (Upper Row) - 13 keys
    KeyQ = 13,
    KeyW = 14,
    KeyE = 15,
    KeyR = 16,
    KeyT = 17,
    KeyY = 18,
    KeyU = 19,
    KeyI = 20,
    KeyO = 21,
    KeyP = 22,
    BracketLeft = 23,
    BracketRight = 24,
    Backslash = 25,

    // Row C (Home Row) - 11 keys
    KeyA = 26,
    KeyS = 27,
    KeyD = 28,
    KeyF = 29,
    KeyG = 30,
    KeyH = 31,
    KeyJ = 32,
    KeyK = 33,
    KeyL = 34,
    Semicolon = 35,
    Quote = 36,

    // Row B (Bottom Row) - 10 keys
    KeyZ = 37,
    KeyX = 38,
    KeyC = 39,
    KeyV = 40,
    KeyB = 41,
    KeyN = 42,
    KeyM = 43,
    Comma = 44,
    Period = 45,
    Slash = 46,

    // Row A (Space Row) - 1 key
    Space = 47,
}

impl PhysicalKey {
    /// Exhaustive array of all 48 physical keys.
    pub const ALL: [PhysicalKey; 48] = [
        PhysicalKey::Backquote,
        PhysicalKey::Digit1,
        PhysicalKey::Digit2,
        PhysicalKey::Digit3,
        PhysicalKey::Digit4,
        PhysicalKey::Digit5,
        PhysicalKey::Digit6,
        PhysicalKey::Digit7,
        PhysicalKey::Digit8,
        PhysicalKey::Digit9,
        PhysicalKey::Digit0,
        PhysicalKey::Minus,
        PhysicalKey::Equal,
        PhysicalKey::KeyQ,
        PhysicalKey::KeyW,
        PhysicalKey::KeyE,
        PhysicalKey::KeyR,
        PhysicalKey::KeyT,
        PhysicalKey::KeyY,
        PhysicalKey::KeyU,
        PhysicalKey::KeyI,
        PhysicalKey::KeyO,
        PhysicalKey::KeyP,
        PhysicalKey::BracketLeft,
        PhysicalKey::BracketRight,
        PhysicalKey::Backslash,
        PhysicalKey::KeyA,
        PhysicalKey::KeyS,
        PhysicalKey::KeyD,
        PhysicalKey::KeyF,
        PhysicalKey::KeyG,
        PhysicalKey::KeyH,
        PhysicalKey::KeyJ,
        PhysicalKey::KeyK,
        PhysicalKey::KeyL,
        PhysicalKey::Semicolon,
        PhysicalKey::Quote,
        PhysicalKey::KeyZ,
        PhysicalKey::KeyX,
        PhysicalKey::KeyC,
        PhysicalKey::KeyV,
        PhysicalKey::KeyB,
        PhysicalKey::KeyN,
        PhysicalKey::KeyM,
        PhysicalKey::Comma,
        PhysicalKey::Period,
        PhysicalKey::Slash,
        PhysicalKey::Space,
    ];

    /// Convert from numerical integer keycode (0 to 47).
    pub fn from_u32(val: u32) -> Option<Self> {
        if (val as usize) < Self::ALL.len() {
            Some(Self::ALL[val as usize])
        } else {
            None
        }
    }
}

/// Modifier bitflags and state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Modifiers {
    pub shift: bool,
    pub altgr: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Modifiers {
    pub const BORNO_MOD_SHIFT: u32 = 0x01;
    pub const BORNO_MOD_ALTGR: u32 = 0x02;
    pub const BORNO_MOD_CTRL: u32 = 0x04;
    pub const BORNO_MOD_ALT: u32 = 0x08;

    pub const fn new(shift: bool, altgr: bool, ctrl: bool, alt: bool) -> Self {
        Self { shift, altgr, ctrl, alt }
    }

    /// Construct from C-ABI bitflags integer.
    pub fn from_bits(bits: u32) -> Self {
        Self {
            shift: (bits & Self::BORNO_MOD_SHIFT) != 0,
            altgr: (bits & Self::BORNO_MOD_ALTGR) != 0,
            ctrl: (bits & Self::BORNO_MOD_CTRL) != 0,
            alt: (bits & Self::BORNO_MOD_ALT) != 0,
        }
    }

    /// Convert to C-ABI bitflags integer.
    pub fn to_bits(&self) -> u32 {
        let mut bits = 0;
        if self.shift { bits |= Self::BORNO_MOD_SHIFT; }
        if self.altgr { bits |= Self::BORNO_MOD_ALTGR; }
        if self.ctrl { bits |= Self::BORNO_MOD_CTRL; }
        if self.alt { bits |= Self::BORNO_MOD_ALT; }
        bits
    }
}

/// Modifier layers supported by Borno keyboard layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierLayer {
    Normal,
    Shift,
    AltGr,
}

/// Metadata describing a keyboard layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub standard: Option<String>,
    pub author: String,
    pub description: String,
    pub supported_layers: Vec<ModifierLayer>,
}

/// Mapping of a single physical key across modifier layers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyMapping {
    pub unshifted: String,
    pub shifted: String,
    pub altgr: Option<String>,
}

impl KeyMapping {
    pub fn new(unshifted: impl Into<String>, shifted: impl Into<String>) -> Self {
        Self {
            unshifted: unshifted.into(),
            shifted: shifted.into(),
            altgr: None,
        }
    }

    pub fn with_altgr(unshifted: impl Into<String>, shifted: impl Into<String>, altgr: impl Into<String>) -> Self {
        Self {
            unshifted: unshifted.into(),
            shifted: shifted.into(),
            altgr: Some(altgr.into()),
        }
    }
}

/// Dynamic composition rules for a layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutRules {
    pub link_key: Option<PhysicalKey>,
    pub supports_link_vowel_composition: bool,
    pub link_vowel_map: BTreeMap<String, String>,
}

/// Complete, validated definition of a fixed keyboard layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutDefinition {
    pub schema_version: String,
    pub metadata: LayoutMetadata,
    pub rules: LayoutRules,
    pub keys: BTreeMap<PhysicalKey, KeyMapping>,
}

/// Validation errors encountered in layout schemas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    UnsupportedSchemaVersion { expected: String, found: String },
    InvalidMetadataId(String),
    MissingKeyMapping(PhysicalKey),
    MissingShiftMapping(PhysicalKey),
    EmptyMapping(PhysicalKey, String),
    DisallowedCodePoint { key: PhysicalKey, layer: String, ch: char, codepoint: u32 },
    UnexpectedLayerMapping { key: PhysicalKey, layer: ModifierLayer },
    DuplicateDefinition(String),
    SerializationError(String),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { expected, found } => {
                write!(f, "Unsupported schema version: expected {expected}, found {found}")
            }
            Self::InvalidMetadataId(id) => write!(f, "Invalid layout metadata id: {id}"),
            Self::MissingKeyMapping(k) => write!(f, "Missing mapping for physical key: {k:?}"),
            Self::MissingShiftMapping(k) => write!(f, "Missing shift mapping for key: {k:?}"),
            Self::EmptyMapping(k, layer) => write!(f, "Empty mapping for key {k:?} in layer {layer}"),
            Self::DisallowedCodePoint { key, layer, ch, codepoint } => {
                write!(f, "Disallowed character '{ch}' (U+{codepoint:04X}) in key {key:?} layer {layer}")
            }
            Self::UnexpectedLayerMapping { key, layer } => {
                write!(f, "Unexpected mapping for layer {layer:?} on key {key:?}")
            }
            Self::DuplicateDefinition(id) => write!(f, "Duplicate layout definition: {id}"),
            Self::SerializationError(msg) => write!(f, "Layout serialization error: {msg}"),
        }
    }
}

impl std::error::Error for LayoutError {}

impl LayoutDefinition {
    /// Validate structural and script invariants of the layout definition.
    pub fn validate(&self) -> Result<(), LayoutError> {
        // 1. Schema version check
        if self.schema_version != "1.0.0" {
            return Err(LayoutError::UnsupportedSchemaVersion {
                expected: "1.0.0".to_string(),
                found: self.schema_version.clone(),
            });
        }

        // 2. Identifier format check
        let id = &self.metadata.id;
        if id.len() < 2 || id.len() > 32 || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') {
            return Err(LayoutError::InvalidMetadataId(id.clone()));
        }

        let supports_altgr = self.metadata.supported_layers.contains(&ModifierLayer::AltGr);

        // 3. Exhaustive 48 physical keys completeness
        for key in &PhysicalKey::ALL {
            let mapping = self.keys.get(key).ok_or(LayoutError::MissingKeyMapping(*key))?;

            if mapping.unshifted.is_empty() {
                return Err(LayoutError::EmptyMapping(*key, "unshifted".to_string()));
            }

            if mapping.shifted.is_empty() {
                return Err(LayoutError::EmptyMapping(*key, "shifted".to_string()));
            }

            // 4. Per-layout AltGr enforcement
            if let Some(ref altgr_str) = mapping.altgr {
                if !supports_altgr {
                    return Err(LayoutError::UnexpectedLayerMapping {
                        key: *key,
                        layer: ModifierLayer::AltGr,
                    });
                }
                // If National (which supports AltGr), only Digit4, Digit7, Equal may have altgr
                if *key != PhysicalKey::Digit4 && *key != PhysicalKey::Digit7 && *key != PhysicalKey::Equal {
                    return Err(LayoutError::UnexpectedLayerMapping {
                        key: *key,
                        layer: ModifierLayer::AltGr,
                    });
                }
                if altgr_str.is_empty() {
                    return Err(LayoutError::EmptyMapping(*key, "altgr".to_string()));
                }
                Self::validate_codepoints(*key, "altgr", altgr_str)?;
            }

            // Validate codepoints in unshifted and shifted layers
            Self::validate_codepoints(*key, "unshifted", &mapping.unshifted)?;
            Self::validate_codepoints(*key, "shifted", &mapping.shifted)?;
        }

        Ok(())
    }

    fn validate_codepoints(key: PhysicalKey, layer: &str, s: &str) -> Result<(), LayoutError> {
        for ch in s.chars() {
            let cp = ch as u32;
            let is_allowed = (0x0980..=0x09FF).contains(&cp)      // Bengali block
                || cp == 0x0964                                   // Dari
                || cp == 0x0965                                   // Double Dari
                || cp == 0x200C                                   // ZWNJ
                || cp == 0x200D                                   // ZWJ
                || (0x0020..=0x007E).contains(&cp);               // Printable ASCII

            if !is_allowed {
                return Err(LayoutError::DisallowedCodePoint {
                    key,
                    layer: layer.to_string(),
                    ch,
                    codepoint: cp,
                });
            }
        }
        Ok(())
    }
}
