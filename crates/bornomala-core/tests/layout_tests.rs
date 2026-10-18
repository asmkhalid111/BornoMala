//! # Fixed Keyboard Layouts Integration Test Suite (Milestone M4)
//!
//! Comprehensive layered test suite covering:
//! - Layer 1: Schema Validation & Error Rejection (9 tests)
//! - Layer 2: National / Jatiya Layout Suite (10 tests)
//! - Layer 3: Probhat Layout Suite (10 tests)
//! - Layer 4: Bijoy-Compatible Layout Suite (8 tests)
//! - Layer 5: In-Composition Editing & Edge-Case Semantics (10 tests)
//! - Layer 6: Dual-Mode Switching & C-ABI Compatibility (6 tests)
//! - Layer 7: Quantitative Keystroke Latency Benchmark (4 tests)
//! Total: Exactly 57 automated tests.

use std::time::Instant;

use bornomala_core::ffi::*;
use bornomala_core::layout::{
    build_national_layout, build_probhat_layout, get_bijoy_layout, get_national_layout,
    get_probhat_layout, FixedLayoutId, KeyMapping, LayoutDefinition, LayoutError, ModifierLayer,
    Modifiers, PhysicalKey,
};
use bornomala_core::{BornoEngine, EngineEvent, InputMode};

// ============================================================================
// Layer 1: Schema Validation & Error Rejection (9 tests)
// ============================================================================

#[test]
fn test_layer1_valid_layout_deserialization_and_validation() {
    let fixtures_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    let national_raw = std::fs::read_to_string(fixtures_dir.join("national_reference.json"))
        .expect("national_reference.json missing");
    let national_def: LayoutDefinition =
        serde_json::from_str(&national_raw).expect("Deserialization of national_reference failed");
    assert!(national_def.validate().is_ok());

    let probhat_raw = std::fs::read_to_string(fixtures_dir.join("probhat_reference.json"))
        .expect("probhat_reference.json missing");
    let probhat_def: LayoutDefinition =
        serde_json::from_str(&probhat_raw).expect("Deserialization of probhat_reference failed");
    assert!(probhat_def.validate().is_ok());

    let bijoy_raw = std::fs::read_to_string(fixtures_dir.join("bijoy_reference.json"))
        .expect("bijoy_reference.json missing");
    let bijoy_def: LayoutDefinition =
        serde_json::from_str(&bijoy_raw).expect("Deserialization of bijoy_reference failed");
    assert!(bijoy_def.validate().is_ok());
}

#[test]
fn test_layer1_rejection_missing_shifted_mapping() {
    let mut def = build_probhat_layout();
    // Replace KeyA with empty shifted mapping
    def.keys.insert(
        PhysicalKey::KeyA,
        KeyMapping {
            unshifted: "া".to_string(),
            shifted: "".to_string(),
            altgr: None,
        },
    );

    let err = def.validate().expect_err("Should reject empty shifted mapping");
    assert_eq!(err, LayoutError::EmptyMapping(PhysicalKey::KeyA, "shifted".to_string()));
}

#[test]
fn test_layer1_rejection_missing_physical_key() {
    let mut def = build_probhat_layout();
    // Remove Space key entirely (only 47 keys present)
    def.keys.remove(&PhysicalKey::Space);

    let err = def.validate().expect_err("Should reject missing physical key");
    assert_eq!(err, LayoutError::MissingKeyMapping(PhysicalKey::Space));
}

#[test]
fn test_layer1_rejection_unexpected_altgr_unsupported_layout() {
    let mut def = build_probhat_layout();
    // Probhat does not declare AltGr in supported_layers
    def.keys.insert(
        PhysicalKey::KeyA,
        KeyMapping {
            unshifted: "া".to_string(),
            shifted: "অ".to_string(),
            altgr: Some("ক".to_string()),
        },
    );

    let err = def.validate().expect_err("Should reject AltGr on unsupported layout");
    assert_eq!(
        err,
        LayoutError::UnexpectedLayerMapping {
            key: PhysicalKey::KeyA,
            layer: ModifierLayer::AltGr
        }
    );
}

#[test]
fn test_layer1_rejection_undeclared_altgr_in_supported_layout() {
    let mut def = build_national_layout();
    // National declares AltGr, but only on Digit4, Digit7, Equal.
    // Adding AltGr to KeyA must fail.
    def.keys.insert(
        PhysicalKey::KeyA,
        KeyMapping {
            unshifted: "ৃ".to_string(),
            shifted: "ঋ".to_string(),
            altgr: Some("ক".to_string()),
        },
    );

    let err = def.validate().expect_err("Should reject undeclared AltGr mapping");
    assert_eq!(
        err,
        LayoutError::UnexpectedLayerMapping {
            key: PhysicalKey::KeyA,
            layer: ModifierLayer::AltGr
        }
    );
}

#[test]
fn test_layer1_rejection_unsupported_schema_version() {
    let mut def = build_national_layout();
    def.schema_version = "2.0.0".to_string();

    let err = def.validate().expect_err("Should reject invalid schema version");
    assert_eq!(
        err,
        LayoutError::UnsupportedSchemaVersion {
            expected: "1.0.0".to_string(),
            found: "2.0.0".to_string()
        }
    );
}

#[test]
fn test_layer1_rejection_invalid_metadata_id() {
    let mut def = build_national_layout();
    def.metadata.id = "National Layout!".to_string();

    let err = def.validate().expect_err("Should reject invalid metadata id");
    assert_eq!(err, LayoutError::InvalidMetadataId("National Layout!".to_string()));
}

#[test]
fn test_layer1_rejection_disallowed_codepoint() {
    let mut def = build_national_layout();
    // Bell control character \u{0007} is disallowed
    def.keys.insert(
        PhysicalKey::KeyQ,
        KeyMapping::new("\u{0007}", "ং"),
    );

    let err = def.validate().expect_err("Should reject control character");
    assert_eq!(
        err,
        LayoutError::DisallowedCodePoint {
            key: PhysicalKey::KeyQ,
            layer: "unshifted".to_string(),
            ch: '\u{0007}',
            codepoint: 7
        }
    );
}

#[test]
fn test_layer1_rejection_shift_altgr_level4_event() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Shift + AltGr (Level 4) is explicitly excluded across all layouts
    let res = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::new(true, true, false, false),
    });

    assert!(!res.handled, "Shift+AltGr must be rejected by fixed layout engine");
    assert_eq!(res.state.preedit, "");
}

// ============================================================================
// Layer 2: National / Jatiya Layout Suite (10 tests)
// ============================================================================

#[test]
fn test_layer2_national_all_48_keys_unshifted_parity() {
    let fixtures_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let ref_raw = std::fs::read_to_string(fixtures_dir.join("national_reference.json")).unwrap();
    let ref_def: LayoutDefinition = serde_json::from_str(&ref_raw).unwrap();
    let national = get_national_layout();

    for key in PhysicalKey::ALL {
        let expected = &ref_def.keys.get(&key).unwrap().unshifted;
        let actual = &national.keys.get(&key).unwrap().unshifted;
        assert_eq!(
            actual, expected,
            "National unshifted mismatch on key {:?}",
            key
        );
    }
}

#[test]
fn test_layer2_national_all_48_keys_shifted_parity() {
    let fixtures_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let ref_raw = std::fs::read_to_string(fixtures_dir.join("national_reference.json")).unwrap();
    let ref_def: LayoutDefinition = serde_json::from_str(&ref_raw).unwrap();
    let national = get_national_layout();

    for key in PhysicalKey::ALL {
        let expected = &ref_def.keys.get(&key).unwrap().shifted;
        let actual = &national.keys.get(&key).unwrap().shifted;
        assert_eq!(
            actual, expected,
            "National shifted mismatch on key {:?}",
            key
        );
    }
}

#[test]
fn test_layer2_national_altgr_layer() {
    let national = get_national_layout();

    let d4 = national.keys.get(&PhysicalKey::Digit4).unwrap();
    assert_eq!(d4.altgr.as_deref(), Some("$"));

    let d7 = national.keys.get(&PhysicalKey::Digit7).unwrap();
    assert_eq!(d7.altgr.as_deref(), Some("&"));

    let eq = national.keys.get(&PhysicalKey::Equal).unwrap();
    assert_eq!(eq.altgr.as_deref(), Some("॥"));

    // Check runtime engine dispatch
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Digit4,
        modifiers: Modifiers::new(false, true, false, false),
    });
    assert_eq!(r.state.preedit, "$");
}

#[test]
fn test_layer2_national_conjunct_kka() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // j + g + j -> ক + ্ + ক -> ক্ক
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    assert_eq!(engine.state().preedit, "ক");

    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    assert_eq!(engine.state().preedit, "ক্");

    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "ক্ক");
}

#[test]
fn test_layer2_national_conjunct_ksha() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // j + g + Shift+X -> ক + ্ + ষ -> ক্ষ
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyX,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(r.state.preedit, "ক্ষ");
}

#[test]
fn test_layer2_national_conjunct_jnya() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // u + g + Shift+I -> জ + ্ + ঞ -> জ্ঞ
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyU,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyI,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(r.state.preedit, "জ্ঞ");
}

#[test]
fn test_layer2_national_initial_independent_vowels() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // g + f (া) -> আ
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyF,
        modifiers: Modifiers::default(),
    });
    // Wait, g was omitted in above line: let's do g then f:
    engine.reset();
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyF,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "আ");

    engine.reset();

    // g + d (ি) -> ই
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    let r2 = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyD,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r2.state.preedit, "ই");
}

#[test]
fn test_layer2_national_medial_independent_vowels() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // h (ব) + g (্) + d (ি) -> বই
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyH,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyD,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "বই");
}

#[test]
fn test_layer2_national_direct_dependent_vowels() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // h (ব) + d (ি) -> বি (Direct dependent vowel sign without link key)
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyH,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyD,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "বি");
}

#[test]
fn test_layer2_national_reph_formation() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // v (র) + g (্) + j (ক) -> র্ক (logical codepoints: র + ্ + ক)
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyV,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "র্ক");
    assert_eq!(r.state.preedit, "\u{09B0}\u{09CD}\u{0995}");
}

// ============================================================================
// Layer 3: Probhat Layout Suite (10 tests)
// ============================================================================

#[test]
fn test_layer3_probhat_all_48_keys_unshifted_parity() {
    let fixtures_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let ref_raw = std::fs::read_to_string(fixtures_dir.join("probhat_reference.json")).unwrap();
    let ref_def: LayoutDefinition = serde_json::from_str(&ref_raw).unwrap();
    let probhat = get_probhat_layout();

    for key in PhysicalKey::ALL {
        let expected = &ref_def.keys.get(&key).unwrap().unshifted;
        let actual = &probhat.keys.get(&key).unwrap().unshifted;
        assert_eq!(
            actual, expected,
            "Probhat unshifted mismatch on key {:?}",
            key
        );
    }
}

#[test]
fn test_layer3_probhat_all_48_keys_shifted_parity() {
    let fixtures_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let ref_raw = std::fs::read_to_string(fixtures_dir.join("probhat_reference.json")).unwrap();
    let ref_def: LayoutDefinition = serde_json::from_str(&ref_raw).unwrap();
    let probhat = get_probhat_layout();

    for key in PhysicalKey::ALL {
        let expected = &ref_def.keys.get(&key).unwrap().shifted;
        let actual = &probhat.keys.get(&key).unwrap().shifted;
        assert_eq!(
            actual, expected,
            "Probhat shifted mismatch on key {:?}",
            key
        );
    }
}

#[test]
fn test_layer3_probhat_conjunct_ksha() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Probhat)));

    // k (ক) + / (্) + Shift+S (ষ) -> ক্ষ
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyK,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Slash,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyS,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(r.state.preedit, "ক্ষ");
}

#[test]
fn test_layer3_probhat_conjunct_jnya() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Probhat)));

    // j (জ) + / (্) + Shift+Digit7 (ঞ) -> জ্ঞ
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Slash,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Digit7,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(r.state.preedit, "জ্ঞ");
}

#[test]
fn test_layer3_probhat_direct_independent_vowels() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Probhat)));

    // Shift+KeyA -> অ
    let r1 = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyA,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(r1.state.preedit, "অ");

    engine.reset();

    // KeyV -> আ
    let r2 = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyV,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r2.state.preedit, "আ");

    engine.reset();

    // KeyY -> এ
    let r3 = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyY,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r3.state.preedit, "এ");
}

#[test]
fn test_layer3_probhat_kar_replacement_under_link() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Probhat)));

    // k (ক) + / (্) + a (া) -> কা (Probhat replaces virama with vowel sign)
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyK,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Slash,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyA,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "কা");
}

#[test]
fn test_layer3_probhat_reph_formation() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Probhat)));

    // r (র) + / (্) + k (ক) -> র্ক
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyR,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Slash,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyK,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "র্ক");
}

#[test]
fn test_layer3_probhat_zwnj_emission() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Probhat)));

    // Backslash -> ZWNJ (\u{200C})
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Backslash,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "\u{200C}");
}

#[test]
fn test_layer3_probhat_zwj_emission() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Probhat)));

    // Backquote -> ZWJ (\u{200D})
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Backquote,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "\u{200D}");
}

#[test]
fn test_layer3_probhat_double_danda_shift_backslash() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Probhat)));

    // Shift + Backslash -> Double Danda (॥ U+0965)
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Backslash,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(r.state.preedit, "॥");
}

// ============================================================================
// Layer 4: Bijoy-Compatible Layout Suite (8 tests)
// ============================================================================

#[test]
fn test_layer4_bijoy_all_48_keys_unshifted_parity() {
    let fixtures_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let ref_raw = std::fs::read_to_string(fixtures_dir.join("bijoy_reference.json")).unwrap();
    let ref_def: LayoutDefinition = serde_json::from_str(&ref_raw).unwrap();
    let bijoy = get_bijoy_layout();

    for key in PhysicalKey::ALL {
        let expected = &ref_def.keys.get(&key).unwrap().unshifted;
        let actual = &bijoy.keys.get(&key).unwrap().unshifted;
        assert_eq!(
            actual, expected,
            "Bijoy unshifted mismatch on key {:?}",
            key
        );
    }
}

#[test]
fn test_layer4_bijoy_all_48_keys_shifted_parity() {
    let fixtures_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let ref_raw = std::fs::read_to_string(fixtures_dir.join("bijoy_reference.json")).unwrap();
    let ref_def: LayoutDefinition = serde_json::from_str(&ref_raw).unwrap();
    let bijoy = get_bijoy_layout();

    for key in PhysicalKey::ALL {
        let expected = &ref_def.keys.get(&key).unwrap().shifted;
        let actual = &bijoy.keys.get(&key).unwrap().shifted;
        assert_eq!(
            actual, expected,
            "Bijoy shifted mismatch on key {:?}",
            key
        );
    }
}

#[test]
fn test_layer4_bijoy_conjunct_formation() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Bijoy)));

    // j (ক) + g (্) + Shift+N (ষ) -> ক্ষ
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyN,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(r.state.preedit, "ক্ষ");
}

#[test]
fn test_layer4_bijoy_independent_vowel_generation() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Bijoy)));

    // g + f (া) -> আ
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyF,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "আ");
}

#[test]
fn test_layer4_bijoy_medial_independent_vowel() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Bijoy)));

    // h (ব) + g (্) + d (ি) -> বই
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyH,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyD,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "বই");
}

#[test]
fn test_layer4_bijoy_unicode_nfc_emission() {
    let bijoy = get_bijoy_layout();

    for (_key, mapping) in &bijoy.keys {
        // Assert unshifted code points are strictly standard Unicode (Bengali or ASCII)
        for ch in mapping.unshifted.chars() {
            let u = ch as u32;
            let is_bangla = (0x0980..=0x09FF).contains(&u);
            let is_indic_punct = ch == '।' || ch == '॥';
            let is_ascii = (0x0020..=0x007E).contains(&u);
            assert!(
                is_bangla || is_indic_punct || is_ascii,
                "Disallowed character in Bijoy unshifted: {:?} (U+{:04X})",
                ch,
                u
            );
        }
        // Assert shifted code points are strictly standard Unicode (Bengali or ASCII)
        for ch in mapping.shifted.chars() {
            let u = ch as u32;
            let is_bangla = (0x0980..=0x09FF).contains(&u);
            let is_indic_punct = ch == '।' || ch == '॥';
            let is_ascii = (0x0020..=0x007E).contains(&u);
            assert!(
                is_bangla || is_indic_punct || is_ascii,
                "Disallowed character in Bijoy shifted: {:?} (U+{:04X})",
                ch,
                u
            );
        }
    }
}

#[test]
fn test_layer4_bijoy_semicolon_and_quote_mappings() {
    let bijoy = get_bijoy_layout();

    let semi = bijoy.keys.get(&PhysicalKey::Semicolon).unwrap();
    assert_eq!(semi.unshifted, "ম");
    assert_eq!(semi.shifted, "ণ");

    let quote = bijoy.keys.get(&PhysicalKey::Quote).unwrap();
    assert_eq!(quote.unshifted, "ন");
    assert_eq!(quote.shifted, "ষ");
}

#[test]
fn test_layer4_bijoy_comma_and_slash() {
    let bijoy = get_bijoy_layout();

    let comma = bijoy.keys.get(&PhysicalKey::Comma).unwrap();
    assert_eq!(comma.unshifted, ",");
    assert_eq!(comma.shifted, "ৎ");

    let slash = bijoy.keys.get(&PhysicalKey::Slash).unwrap();
    assert_eq!(slash.unshifted, "?");
    assert_eq!(slash.shifted, "ঃ");
}

// ============================================================================
// Layer 5: In-Composition Editing & Edge-Case Semantics (10 tests)
// ============================================================================

#[test]
fn test_layer5_leading_virama_discard_on_consonant() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Link key at text start -> preedit = "্"
    let r1 = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r1.state.preedit, "্");

    // Consonant KeyJ (ক) typed -> leading virama discarded to prevent dotted circle ◌্ক
    let r2 = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r2.state.preedit, "ক");
    assert_eq!(r2.state.cursor_pos, 1);
}

#[test]
fn test_layer5_leading_virama_with_vowel() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Link key at text start -> preedit = "্"
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });

    // Vowel KeyF (া) -> generates independent vowel "আ"
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyF,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "আ");
    assert_eq!(r.state.cursor_pos, 1);
}

#[test]
fn test_layer5_dangling_virama_space_cleanup() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Link key at text start
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });

    // Space key: commits " " and virama is cleaned up
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Space,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.committed_text, Some(" ".to_string()));
    assert_eq!(r.state.preedit, "");
    assert_eq!(r.state.cursor_pos, 0);
}

#[test]
fn test_layer5_dangling_virama_backspace_cleanup() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Link key at text start
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    assert_eq!(engine.state().preedit, "্");

    // Backspace: removes virama and cleans up buffer
    let r = engine.process_event(EngineEvent::Backspace);
    assert_eq!(r.state.preedit, "");
    assert_eq!(r.state.cursor_pos, 0);
}

#[test]
fn test_layer5_backspace_in_link_pending() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // ক + ্ -> "ক্"
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    assert_eq!(engine.state().preedit, "ক্");

    // Backspace: deletes Hasanta '্', keeping base consonant 'ক' intact
    let r = engine.process_event(EngineEvent::Backspace);
    assert_eq!(r.state.preedit, "ক");
    assert_eq!(r.state.cursor_pos, 1);
}

#[test]
fn test_layer5_backspace_deconstruction_reverts_to_link_pending() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // ক + ্ + ষ -> "ক্ষ"
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyX,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(engine.state().preedit, "ক্ষ");

    // Backspace: deletes trailing 'ষ', leaves "ক্", and automatically transitions back to LinkPending
    let r1 = engine.process_event(EngineEvent::Backspace);
    assert_eq!(r1.state.preedit, "ক্");

    // Typing 'ক' immediately forms conjunct "ক্ক" without needing to retype link key!
    let r2 = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r2.state.preedit, "ক্ক");
}

#[test]
fn test_layer5_delete_forward_in_preedit() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Type "কখ"
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(engine.state().preedit, "কখ");

    // Move cursor to start
    engine.process_event(EngineEvent::SetCursor(0));
    assert_eq!(engine.state().cursor_pos, 0);

    // Delete forward: deletes 'ক', leaving 'খ'
    let r = engine.process_event(EngineEvent::Delete);
    assert_eq!(r.state.preedit, "খ");
    assert_eq!(r.state.cursor_pos, 0);
}

#[test]
fn test_layer5_interior_cursor_typing() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Type "কখ"
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::new(true, false, false, false),
    });

    // Move cursor left to between 'ক' and 'খ' (index 1)
    engine.process_event(EngineEvent::MoveCursorLeft);
    assert_eq!(engine.state().cursor_pos, 1);

    // Type 'গ' (KeyO)
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyO,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "কগখ");
    assert_eq!(r.state.cursor_pos, 2);
}

#[test]
fn test_layer5_repeated_link_key_collapse() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Type 'ক' + '্'
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    assert_eq!(engine.state().preedit, "ক্");

    // Second consecutive link key: collapses into single Hasanta
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "ক্");
}

#[test]
fn test_layer5_post_punctuation_virama_drop() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Type Dari '।' (Shift + Period)
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::Period,
        modifiers: Modifiers::new(true, false, false, false),
    });
    assert_eq!(engine.state().preedit, "।");

    // Type link key '্'
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyG,
        modifiers: Modifiers::default(),
    });
    assert_eq!(engine.state().preedit, "।্");

    // Type consonant 'ক' (KeyJ): virama is dropped after punctuation
    let r = engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    assert_eq!(r.state.preedit, "।ক");
}

// ============================================================================
// Layer 6: Dual-Mode Switching & C-ABI Compatibility (6 tests)
// ============================================================================

#[test]
fn test_layer6_c_abi_struct_layout_and_offsets() {
    use std::mem::{align_of, size_of};

    // Assert C-ABI struct alignments and sizes are non-zero
    assert!(size_of::<BornoEventC>() >= 32);
    assert_eq!(align_of::<BornoEventC>(), align_of::<usize>());

    assert!(size_of::<BornoStatusC>() >= 24);
    assert_eq!(align_of::<BornoStatusC>(), align_of::<usize>());

    assert!(size_of::<BornoStateC>() >= 48);
    assert_eq!(align_of::<BornoStateC>(), align_of::<usize>());

    assert!(size_of::<BornoProcessResultC>() >= size_of::<BornoStateC>());
}

#[test]
fn test_layer6_c_abi_mode_mapping_roundtrip() {
    // 0: Phonetic, 1: Passthrough, 2: Fixed(National), 3: Fixed(Probhat), 4: Fixed(Bijoy)
    let modes = [
        (BORNO_MODE_PHONETIC, InputMode::Phonetic),
        (BORNO_MODE_PASSTHROUGH, InputMode::EnglishPassthrough),
        (BORNO_MODE_FIXED_NATIONAL, InputMode::Fixed(FixedLayoutId::National)),
        (BORNO_MODE_FIXED_PROBHAT, InputMode::Fixed(FixedLayoutId::Probhat)),
        (BORNO_MODE_FIXED_BIJOY, InputMode::Fixed(FixedLayoutId::Bijoy)),
    ];

    for (c_mode, rust_mode) in modes {
        let mut engine = BornoEngine::new();
        let r = engine.process_event(EngineEvent::SwitchMode(rust_mode));
        assert_eq!(r.state.mode, rust_mode);
        assert_eq!(c_mode, match rust_mode {
            InputMode::Phonetic => 0,
            InputMode::EnglishPassthrough => 1,
            InputMode::Fixed(FixedLayoutId::National) => 2,
            InputMode::Fixed(FixedLayoutId::Probhat) => 3,
            InputMode::Fixed(FixedLayoutId::Bijoy) => 4,
        });
    }
}

#[test]
fn test_layer6_c_abi_process_key() {
    unsafe {
        let handle = borno_engine_new();
        assert!(!handle.is_null());

        // Switch to National mode via C event
        let mode_event = BornoEventC {
            event_type: BORNO_EVENT_SWITCH_MODE,
            char_codepoint: 0,
            string_payload: std::ptr::null(),
            index_payload: 0,
            mode_payload: BORNO_MODE_FIXED_NATIONAL,
        };
        let mut mode_result: BornoProcessResultC = std::mem::zeroed();
        let r_switch = borno_engine_process_event(handle, &mode_event, &mut mode_result);
        assert_eq!(r_switch, BORNO_OK);
        assert_eq!(mode_result.state.mode, BORNO_MODE_FIXED_NATIONAL);
        borno_process_result_free_members(&mut mode_result);

        // Process PhysicalKey::KeyJ (code 32)
        let mut key_result: BornoProcessResultC = std::mem::zeroed();
        let r_key = borno_engine_process_key(
            handle,
            PhysicalKey::KeyJ as u32,
            0, // no modifiers
            &mut key_result,
        );
        assert_eq!(r_key, BORNO_OK);
        assert!(key_result.handled);

        let preedit_str = std::ffi::CStr::from_ptr(key_result.state.preedit).to_str().unwrap();
        assert_eq!(preedit_str, "ক");

        borno_process_result_free_members(&mut key_result);
        borno_engine_free(handle);
    }
}

#[test]
fn test_layer6_legacy_process_event_parity() {
    unsafe {
        let handle = borno_engine_new();
        assert!(!handle.is_null());

        // InputChar 'k' in Phonetic mode
        let char_event = BornoEventC {
            event_type: BORNO_EVENT_INPUT_CHAR,
            char_codepoint: 'k' as u32,
            string_payload: std::ptr::null(),
            index_payload: 0,
            mode_payload: 0,
        };
        let mut res: BornoProcessResultC = std::mem::zeroed();
        assert_eq!(borno_engine_process_event(handle, &char_event, &mut res), BORNO_OK);
        assert_eq!(std::ffi::CStr::from_ptr(res.state.preedit).to_str().unwrap(), "ক");
        borno_process_result_free_members(&mut res);

        borno_engine_free(handle);
    }
}

#[test]
fn test_layer6_mode_switch_auto_commits_preedit() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Type "ক" in National mode
    engine.process_event(EngineEvent::InputKey {
        key: PhysicalKey::KeyJ,
        modifiers: Modifiers::default(),
    });
    assert_eq!(engine.state().preedit, "ক");

    // Switching mode to Phonetic must auto-commit "ক"
    let r = engine.process_event(EngineEvent::SwitchMode(InputMode::Phonetic));
    assert_eq!(r.committed_text, Some("ক".to_string()));
    assert_eq!(r.state.preedit, "");
    assert_eq!(r.state.cursor_pos, 0);
    assert_eq!(r.state.mode, InputMode::Phonetic);
}

#[test]
fn test_layer6_legacy_json_backward_compatibility() {
    // Verify that legacy JSON serialized events still deserialize properly
    let json_char = r#"{"type":"InputChar","payload":"a"}"#;
    let event: EngineEvent = serde_json::from_str(json_char).unwrap();
    assert_eq!(event, EngineEvent::InputChar('a'));

    let json_backspace = r#"{"type":"Backspace"}"#;
    let event2: EngineEvent = serde_json::from_str(json_backspace).unwrap();
    assert_eq!(event2, EngineEvent::Backspace);

    let json_key = r#"{"type":"InputKey","payload":{"key":"KeyJ","modifiers":{"shift":false,"altgr":false,"ctrl":false,"alt":false}}}"#;
    let event3: EngineEvent = serde_json::from_str(json_key).unwrap();
    assert_eq!(
        event3,
        EngineEvent::InputKey {
            key: PhysicalKey::KeyJ,
            modifiers: Modifiers::default(),
        }
    );
}

// ============================================================================
// Layer 7: Quantitative Keystroke Latency Benchmark (4 tests)
// ============================================================================

#[test]
fn test_benchmark_national_keystroke_latency() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    let keys = [
        PhysicalKey::KeyJ, // ক
        PhysicalKey::KeyG, // ্
        PhysicalKey::KeyJ, // ক -> ক্ক
        PhysicalKey::KeyF, // া -> ক্কা
        PhysicalKey::Space,// Commit
    ];

    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let key = keys[i % keys.len()];
        engine.process_event(EngineEvent::InputKey {
            key,
            modifiers: Modifiers::default(),
        });
    }

    let elapsed = start.elapsed();
    let mean_micros = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "\n[Benchmark National Fixed Layout] Mean latency: {:.3} µs / keystroke (Total: {:?})",
        mean_micros, elapsed
    );

    assert!(
        mean_micros < 100.0,
        "National keystroke latency must be < 100 µs provisional budget; got {:.3} µs",
        mean_micros
    );
}

#[test]
fn test_benchmark_probhat_keystroke_latency() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Probhat)));

    let keys = [
        PhysicalKey::KeyK,  // ক
        PhysicalKey::Slash, // ্
        PhysicalKey::KeyS,  // ষ (shifted) -> ক্ষ
        PhysicalKey::KeyA,  // া -> ক্ষা
        PhysicalKey::Space, // Commit
    ];

    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let key = keys[i % keys.len()];
        let shift = (i % keys.len()) == 2;
        engine.process_event(EngineEvent::InputKey {
            key,
            modifiers: Modifiers::new(shift, false, false, false),
        });
    }

    let elapsed = start.elapsed();
    let mean_micros = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "\n[Benchmark Probhat Fixed Layout] Mean latency: {:.3} µs / keystroke (Total: {:?})",
        mean_micros, elapsed
    );

    assert!(
        mean_micros < 100.0,
        "Probhat keystroke latency must be < 100 µs provisional budget; got {:.3} µs",
        mean_micros
    );
}

#[test]
fn test_benchmark_bijoy_keystroke_latency() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::Bijoy)));

    let keys = [
        PhysicalKey::KeyJ, // ক
        PhysicalKey::KeyG, // ্
        PhysicalKey::KeyN, // ষ (shifted) -> ক্ষ
        PhysicalKey::KeyF, // া -> ক্ষা
        PhysicalKey::Space,// Commit
    ];

    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let key = keys[i % keys.len()];
        let shift = (i % keys.len()) == 2;
        engine.process_event(EngineEvent::InputKey {
            key,
            modifiers: Modifiers::new(shift, false, false, false),
        });
    }

    let elapsed = start.elapsed();
    let mean_micros = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "\n[Benchmark Bijoy Fixed Layout] Mean latency: {:.3} µs / keystroke (Total: {:?})",
        mean_micros, elapsed
    );

    assert!(
        mean_micros < 100.0,
        "Bijoy keystroke latency must be < 100 µs provisional budget; got {:.3} µs",
        mean_micros
    );
}

#[test]
fn test_benchmark_phonetic_keystroke_latency() {
    let mut engine = BornoEngine::new();
    let chars = ['a', 'm', 'i', ' '];
    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        engine.process_event(EngineEvent::InputChar(chars[i % chars.len()]));
    }

    let elapsed = start.elapsed();
    let mean_micros = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "\n[Benchmark Phonetic Transliteration] Mean latency: {:.3} µs / keystroke (Total: {:?})",
        mean_micros, elapsed
    );

    assert!(
        mean_micros < 100.0,
        "Phonetic keystroke latency must be < 100 µs provisional budget; got {:.3} µs",
        mean_micros
    );
}
