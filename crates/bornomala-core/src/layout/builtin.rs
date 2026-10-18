//! Built-in compile-time keyboard layout definitions for Borno.
//!
//! Provides static/lazily instantiated definitions for:
//! 1. National (Jatiya) — Provisional BCC Baseline
//! 2. Probhat — Verified Upstream Conformance
//! 3. Bijoy-Compatible — Provisional Unicode Baseline

use std::collections::BTreeMap;
use std::sync::OnceLock;
use crate::layout::schema::{KeyMapping, LayoutDefinition, LayoutMetadata, LayoutRules, ModifierLayer, PhysicalKey};

/// Identifier enum for built-in fixed layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FixedLayoutId {
    National,
    Probhat,
    Bijoy,
}

impl FixedLayoutId {
    pub const ALL: [FixedLayoutId; 3] = [
        FixedLayoutId::National,
        FixedLayoutId::Probhat,
        FixedLayoutId::Bijoy,
    ];

    pub fn id_str(&self) -> &'static str {
        match self {
            Self::National => "national",
            Self::Probhat => "probhat",
            Self::Bijoy => "bijoy",
        }
    }

    pub fn from_id_str(id: &str) -> Option<Self> {
        match id {
            "national" => Some(Self::National),
            "probhat" => Some(Self::Probhat),
            "bijoy" => Some(Self::Bijoy),
            _ => None,
        }
    }
}

/// Link-key to independent vowel mappings for National & Bijoy layouts.
fn standard_link_vowel_map() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert("া".to_string(), "আ".to_string());
    m.insert("ি".to_string(), "ই".to_string());
    m.insert("ী".to_string(), "ঈ".to_string());
    m.insert("ু".to_string(), "উ".to_string());
    m.insert("ূ".to_string(), "ঊ".to_string());
    m.insert("ৃ".to_string(), "ঋ".to_string());
    m.insert("ে".to_string(), "এ".to_string());
    m.insert("ৈ".to_string(), "ঐ".to_string());
    m.insert("ো".to_string(), "ও".to_string());
    m.insert("ৌ".to_string(), "ঔ".to_string());
    m
}

/// Build the National / Jatiya Layout definition (Provisional BCC Baseline).
pub fn build_national_layout() -> LayoutDefinition {
    let mut keys = BTreeMap::new();

    // Row E
    keys.insert(PhysicalKey::Backquote, KeyMapping::new("`", "~"));
    keys.insert(PhysicalKey::Digit1, KeyMapping::new("১", "!"));
    keys.insert(PhysicalKey::Digit2, KeyMapping::new("২", "@"));
    keys.insert(PhysicalKey::Digit3, KeyMapping::new("৩", "#"));
    keys.insert(PhysicalKey::Digit4, KeyMapping::with_altgr("৪", "৳", "$"));
    keys.insert(PhysicalKey::Digit5, KeyMapping::new("৫", "%"));
    keys.insert(PhysicalKey::Digit6, KeyMapping::new("৬", "^"));
    keys.insert(PhysicalKey::Digit7, KeyMapping::with_altgr("৭", "ঁ", "&"));
    keys.insert(PhysicalKey::Digit8, KeyMapping::new("৮", "*"));
    keys.insert(PhysicalKey::Digit9, KeyMapping::new("৯", "("));
    keys.insert(PhysicalKey::Digit0, KeyMapping::new("০", ")"));
    keys.insert(PhysicalKey::Minus, KeyMapping::new("-", "_"));
    keys.insert(PhysicalKey::Equal, KeyMapping::with_altgr("=", "+", "॥"));

    // Row D
    keys.insert(PhysicalKey::KeyQ, KeyMapping::new("ঙ", "ং"));
    keys.insert(PhysicalKey::KeyW, KeyMapping::new("য", "য়"));
    keys.insert(PhysicalKey::KeyE, KeyMapping::new("ড", "ঢ"));
    keys.insert(PhysicalKey::KeyR, KeyMapping::new("প", "ফ"));
    keys.insert(PhysicalKey::KeyT, KeyMapping::new("ট", "ঠ"));
    keys.insert(PhysicalKey::KeyY, KeyMapping::new("চ", "ছ"));
    keys.insert(PhysicalKey::KeyU, KeyMapping::new("জ", "ঝ"));
    keys.insert(PhysicalKey::KeyI, KeyMapping::new("হ", "ঞ"));
    keys.insert(PhysicalKey::KeyO, KeyMapping::new("গ", "ঘ"));
    keys.insert(PhysicalKey::KeyP, KeyMapping::new("ড়", "ঢ়"));
    keys.insert(PhysicalKey::BracketLeft, KeyMapping::new("ে", "ৈ"));
    keys.insert(PhysicalKey::BracketRight, KeyMapping::new("ো", "ৌ"));
    keys.insert(PhysicalKey::Backslash, KeyMapping::new("ৎ", "ঃ"));

    // Row C
    keys.insert(PhysicalKey::KeyA, KeyMapping::new("ৃ", "ঋ"));
    keys.insert(PhysicalKey::KeyS, KeyMapping::new("ু", "ূ"));
    keys.insert(PhysicalKey::KeyD, KeyMapping::new("ি", "ী"));
    keys.insert(PhysicalKey::KeyF, KeyMapping::new("া", "অ"));
    keys.insert(PhysicalKey::KeyG, KeyMapping::new("্", "ঽ"));
    keys.insert(PhysicalKey::KeyH, KeyMapping::new("ব", "ভ"));
    keys.insert(PhysicalKey::KeyJ, KeyMapping::new("ক", "খ"));
    keys.insert(PhysicalKey::KeyK, KeyMapping::new("ত", "থ"));
    keys.insert(PhysicalKey::KeyL, KeyMapping::new("দ", "ধ"));
    keys.insert(PhysicalKey::Semicolon, KeyMapping::new(";", ":"));
    keys.insert(PhysicalKey::Quote, KeyMapping::new("'", "\""));

    // Row B
    keys.insert(PhysicalKey::KeyZ, KeyMapping::new("্য", "্র"));
    keys.insert(PhysicalKey::KeyX, KeyMapping::new("শ", "ষ"));
    keys.insert(PhysicalKey::KeyC, KeyMapping::new("স", "স"));
    keys.insert(PhysicalKey::KeyV, KeyMapping::new("র", "ল"));
    keys.insert(PhysicalKey::KeyB, KeyMapping::new("ন", "ণ"));
    keys.insert(PhysicalKey::KeyN, KeyMapping::new("ম", "ণ"));
    keys.insert(PhysicalKey::KeyM, KeyMapping::new("ল", "শ"));
    keys.insert(PhysicalKey::Comma, KeyMapping::new(",", "<"));
    keys.insert(PhysicalKey::Period, KeyMapping::new(".", "।"));
    keys.insert(PhysicalKey::Slash, KeyMapping::new("/", "?"));

    // Row A
    keys.insert(PhysicalKey::Space, KeyMapping::new(" ", " "));

    LayoutDefinition {
        schema_version: "1.0.0".to_string(),
        metadata: LayoutMetadata {
            id: "national".to_string(),
            name: "National (Jatiya)".to_string(),
            version: "1.0.0".to_string(),
            standard: Some("BCC Baseline / xkeyboard-config symbols/bd basic".to_string()),
            author: "Bangladesh Computer Council / Jamil Ahmed".to_string(),
            description: "Standard National (Jatiya) keyboard layout for Bengali".to_string(),
            supported_layers: vec![ModifierLayer::Normal, ModifierLayer::Shift, ModifierLayer::AltGr],
        },
        rules: LayoutRules {
            link_key: Some(PhysicalKey::KeyG),
            supports_link_vowel_composition: true,
            link_vowel_map: standard_link_vowel_map(),
        },
        keys,
    }
}

/// Build the Probhat Layout definition (Verified Upstream Conformance).
pub fn build_probhat_layout() -> LayoutDefinition {
    let mut keys = BTreeMap::new();

    // Row E
    keys.insert(PhysicalKey::Backquote, KeyMapping::new("\u{200D}", "~"));
    keys.insert(PhysicalKey::Digit1, KeyMapping::new("১", "!"));
    keys.insert(PhysicalKey::Digit2, KeyMapping::new("২", "@"));
    keys.insert(PhysicalKey::Digit3, KeyMapping::new("৩", "#"));
    keys.insert(PhysicalKey::Digit4, KeyMapping::new("৪", "৳"));
    keys.insert(PhysicalKey::Digit5, KeyMapping::new("৫", "%"));
    keys.insert(PhysicalKey::Digit6, KeyMapping::new("৬", "^"));
    keys.insert(PhysicalKey::Digit7, KeyMapping::new("৭", "ঞ"));
    keys.insert(PhysicalKey::Digit8, KeyMapping::new("৮", "ৎ"));
    keys.insert(PhysicalKey::Digit9, KeyMapping::new("৯", "("));
    keys.insert(PhysicalKey::Digit0, KeyMapping::new("০", ")"));
    keys.insert(PhysicalKey::Minus, KeyMapping::new("-", "_"));
    keys.insert(PhysicalKey::Equal, KeyMapping::new("=", "+"));

    // Row D
    keys.insert(PhysicalKey::KeyQ, KeyMapping::new("দ", "ধ"));
    keys.insert(PhysicalKey::KeyW, KeyMapping::new("ূ", "ঊ"));
    keys.insert(PhysicalKey::KeyE, KeyMapping::new("ী", "ঈ"));
    keys.insert(PhysicalKey::KeyR, KeyMapping::new("র", "ড়"));
    keys.insert(PhysicalKey::KeyT, KeyMapping::new("ট", "ঠ"));
    keys.insert(PhysicalKey::KeyY, KeyMapping::new("এ", "ঐ"));
    keys.insert(PhysicalKey::KeyU, KeyMapping::new("ু", "উ"));
    keys.insert(PhysicalKey::KeyI, KeyMapping::new("ি", "ই"));
    keys.insert(PhysicalKey::KeyO, KeyMapping::new("ও", "ঔ"));
    keys.insert(PhysicalKey::KeyP, KeyMapping::new("প", "ফ"));
    keys.insert(PhysicalKey::BracketLeft, KeyMapping::new("ে", "ৈ"));
    keys.insert(PhysicalKey::BracketRight, KeyMapping::new("ো", "ৌ"));
    keys.insert(PhysicalKey::Backslash, KeyMapping::new("\u{200C}", "॥"));

    // Row C
    keys.insert(PhysicalKey::KeyA, KeyMapping::new("া", "অ"));
    keys.insert(PhysicalKey::KeyS, KeyMapping::new("স", "ষ"));
    keys.insert(PhysicalKey::KeyD, KeyMapping::new("ড", "ঢ"));
    keys.insert(PhysicalKey::KeyF, KeyMapping::new("ত", "থ"));
    keys.insert(PhysicalKey::KeyG, KeyMapping::new("গ", "ঘ"));
    keys.insert(PhysicalKey::KeyH, KeyMapping::new("হ", "ঃ"));
    keys.insert(PhysicalKey::KeyJ, KeyMapping::new("জ", "ঝ"));
    keys.insert(PhysicalKey::KeyK, KeyMapping::new("ক", "খ"));
    keys.insert(PhysicalKey::KeyL, KeyMapping::new("ল", "ং"));
    keys.insert(PhysicalKey::Semicolon, KeyMapping::new(";", ":"));
    keys.insert(PhysicalKey::Quote, KeyMapping::new("'", "\""));

    // Row B
    keys.insert(PhysicalKey::KeyZ, KeyMapping::new("য়", "য"));
    keys.insert(PhysicalKey::KeyX, KeyMapping::new("শ", "ঢ়"));
    keys.insert(PhysicalKey::KeyC, KeyMapping::new("চ", "ছ"));
    keys.insert(PhysicalKey::KeyV, KeyMapping::new("আ", "ঋ"));
    keys.insert(PhysicalKey::KeyB, KeyMapping::new("ব", "ভ"));
    keys.insert(PhysicalKey::KeyN, KeyMapping::new("ন", "ণ"));
    keys.insert(PhysicalKey::KeyM, KeyMapping::new("ম", "ঙ"));
    keys.insert(PhysicalKey::Comma, KeyMapping::new(",", "ৃ"));
    keys.insert(PhysicalKey::Period, KeyMapping::new("।", "ঁ"));
    keys.insert(PhysicalKey::Slash, KeyMapping::new("্", "?"));

    // Row A
    keys.insert(PhysicalKey::Space, KeyMapping::new(" ", " "));

    LayoutDefinition {
        schema_version: "1.0.0".to_string(),
        metadata: LayoutMetadata {
            id: "probhat".to_string(),
            name: "Probhat".to_string(),
            version: "1.0.0".to_string(),
            standard: Some("GNU m17n-db bn-probhat.mim / xkeyboard-config symbols/in ben_probhat".to_string()),
            author: "Golam Mohammad Murtuza (2001)".to_string(),
            description: "Probhat phonetic-geometry layout for Bengali".to_string(),
            supported_layers: vec![ModifierLayer::Normal, ModifierLayer::Shift],
        },
        rules: LayoutRules {
            link_key: Some(PhysicalKey::Slash),
            supports_link_vowel_composition: false,
            link_vowel_map: BTreeMap::new(),
        },
        keys,
    }
}

/// Build the Bijoy-Compatible Layout definition (Provisional Unicode Baseline).
pub fn build_bijoy_layout() -> LayoutDefinition {
    let mut keys = BTreeMap::new();

    // Row E
    keys.insert(PhysicalKey::Backquote, KeyMapping::new("`", "~"));
    keys.insert(PhysicalKey::Digit1, KeyMapping::new("১", "!"));
    keys.insert(PhysicalKey::Digit2, KeyMapping::new("২", "@"));
    keys.insert(PhysicalKey::Digit3, KeyMapping::new("৩", "#"));
    keys.insert(PhysicalKey::Digit4, KeyMapping::new("৪", "৳"));
    keys.insert(PhysicalKey::Digit5, KeyMapping::new("৫", "%"));
    keys.insert(PhysicalKey::Digit6, KeyMapping::new("৬", "^"));
    keys.insert(PhysicalKey::Digit7, KeyMapping::new("৭", "ঁ"));
    keys.insert(PhysicalKey::Digit8, KeyMapping::new("৮", "*"));
    keys.insert(PhysicalKey::Digit9, KeyMapping::new("৯", "("));
    keys.insert(PhysicalKey::Digit0, KeyMapping::new("০", ")"));
    keys.insert(PhysicalKey::Minus, KeyMapping::new("-", "_"));
    keys.insert(PhysicalKey::Equal, KeyMapping::new("=", "+"));

    // Row D
    keys.insert(PhysicalKey::KeyQ, KeyMapping::new("ঙ", "ং"));
    keys.insert(PhysicalKey::KeyW, KeyMapping::new("য", "য়"));
    keys.insert(PhysicalKey::KeyE, KeyMapping::new("ড", "ঢ"));
    keys.insert(PhysicalKey::KeyR, KeyMapping::new("প", "ফ"));
    keys.insert(PhysicalKey::KeyT, KeyMapping::new("ট", "ঠ"));
    keys.insert(PhysicalKey::KeyY, KeyMapping::new("চ", "ছ"));
    keys.insert(PhysicalKey::KeyU, KeyMapping::new("জ", "ঝ"));
    keys.insert(PhysicalKey::KeyI, KeyMapping::new("হ", "ঞ"));
    keys.insert(PhysicalKey::KeyO, KeyMapping::new("গ", "ঘ"));
    keys.insert(PhysicalKey::KeyP, KeyMapping::new("ড়", "ঢ়"));
    keys.insert(PhysicalKey::BracketLeft, KeyMapping::new("[", "{"));
    keys.insert(PhysicalKey::BracketRight, KeyMapping::new("]", "}"));
    keys.insert(PhysicalKey::Backslash, KeyMapping::new("\\", "|"));

    // Row C
    keys.insert(PhysicalKey::KeyA, KeyMapping::new("ৃ", "ঋ"));
    keys.insert(PhysicalKey::KeyS, KeyMapping::new("ু", "ূ"));
    keys.insert(PhysicalKey::KeyD, KeyMapping::new("ি", "ী"));
    keys.insert(PhysicalKey::KeyF, KeyMapping::new("া", "অ"));
    keys.insert(PhysicalKey::KeyG, KeyMapping::new("্", "ঽ"));
    keys.insert(PhysicalKey::KeyH, KeyMapping::new("ব", "ভ"));
    keys.insert(PhysicalKey::KeyJ, KeyMapping::new("ক", "খ"));
    keys.insert(PhysicalKey::KeyK, KeyMapping::new("ত", "থ"));
    keys.insert(PhysicalKey::KeyL, KeyMapping::new("দ", "ধ"));
    keys.insert(PhysicalKey::Semicolon, KeyMapping::new("ম", "ণ"));
    keys.insert(PhysicalKey::Quote, KeyMapping::new("ন", "ষ"));

    // Row B
    keys.insert(PhysicalKey::KeyZ, KeyMapping::new("্র", "্য"));
    keys.insert(PhysicalKey::KeyX, KeyMapping::new("ো", "ৌ"));
    keys.insert(PhysicalKey::KeyC, KeyMapping::new("ে", "ৈ"));
    keys.insert(PhysicalKey::KeyV, KeyMapping::new("র", "ল"));
    keys.insert(PhysicalKey::KeyB, KeyMapping::new("ব", "ভ"));
    keys.insert(PhysicalKey::KeyN, KeyMapping::new("স", "ষ"));
    keys.insert(PhysicalKey::KeyM, KeyMapping::new("শ", "স"));
    keys.insert(PhysicalKey::Comma, KeyMapping::new(",", "ৎ"));
    keys.insert(PhysicalKey::Period, KeyMapping::new(".", "।"));
    keys.insert(PhysicalKey::Slash, KeyMapping::new("?", "ঃ"));

    // Row A
    keys.insert(PhysicalKey::Space, KeyMapping::new(" ", " "));

    LayoutDefinition {
        schema_version: "1.0.0".to_string(),
        metadata: LayoutMetadata {
            id: "bijoy".to_string(),
            name: "Bijoy-Compatible".to_string(),
            version: "1.0.0".to_string(),
            standard: Some("OmicronLab Avro Keyboard Bijoy-compatible Unicode baseline".to_string()),
            author: "Ananda Computers (Geometry) / OmicronLab (Unicode Mapping)".to_string(),
            description: "Traditional Bijoy geometry adapted for standard Unicode code point emission".to_string(),
            supported_layers: vec![ModifierLayer::Normal, ModifierLayer::Shift],
        },
        rules: LayoutRules {
            link_key: Some(PhysicalKey::KeyG),
            supports_link_vowel_composition: true,
            link_vowel_map: standard_link_vowel_map(),
        },
        keys,
    }
}

static NATIONAL_DEF: OnceLock<LayoutDefinition> = OnceLock::new();
static PROBHAT_DEF: OnceLock<LayoutDefinition> = OnceLock::new();
static BIJOY_DEF: OnceLock<LayoutDefinition> = OnceLock::new();

/// Return static reference to National layout definition.
pub fn get_national_layout() -> &'static LayoutDefinition {
    NATIONAL_DEF.get_or_init(|| {
        let def = build_national_layout();
        def.validate().expect("National layout failed built-in validation");
        def
    })
}

/// Return static reference to Probhat layout definition.
pub fn get_probhat_layout() -> &'static LayoutDefinition {
    PROBHAT_DEF.get_or_init(|| {
        let def = build_probhat_layout();
        def.validate().expect("Probhat layout failed built-in validation");
        def
    })
}

/// Return static reference to Bijoy-Compatible layout definition.
pub fn get_bijoy_layout() -> &'static LayoutDefinition {
    BIJOY_DEF.get_or_init(|| {
        let def = build_bijoy_layout();
        def.validate().expect("Bijoy layout failed built-in validation");
        def
    })
}

/// Retrieve built-in layout by identifier.
pub fn get_layout_by_id(id: FixedLayoutId) -> &'static LayoutDefinition {
    match id {
        FixedLayoutId::National => get_national_layout(),
        FixedLayoutId::Probhat => get_probhat_layout(),
        FixedLayoutId::Bijoy => get_bijoy_layout(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_layouts_valid() {
        let national = build_national_layout();
        assert!(national.validate().is_ok());

        let probhat = build_probhat_layout();
        assert!(probhat.validate().is_ok());

        let bijoy = build_bijoy_layout();
        assert!(bijoy.validate().is_ok());
    }

    #[test]
    fn test_export_fixtures() {
        let fixtures_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures");
        std::fs::create_dir_all(&fixtures_dir).unwrap();

        let national_json = serde_json::to_string_pretty(&build_national_layout()).unwrap();
        std::fs::write(fixtures_dir.join("national_reference.json"), national_json).unwrap();

        let probhat_json = serde_json::to_string_pretty(&build_probhat_layout()).unwrap();
        std::fs::write(fixtures_dir.join("probhat_reference.json"), probhat_json).unwrap();

        let bijoy_json = serde_json::to_string_pretty(&build_bijoy_layout()).unwrap();
        std::fs::write(fixtures_dir.join("bijoy_reference.json"), bijoy_json).unwrap();
    }
}
