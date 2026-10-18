//! Integration tests for BornoEngine configuration behaviors.
//!
//! Validates candidate presentation slicing, auto-commit on mode switch,
//! preferred layout preference storage, and startup mode initialization.

use bornomala_core::{
    layout::FixedLayoutId, BornoConfig, BornoEngine, EngineEvent, InputMode, StartupMode,
};

#[test]
fn test_engine_candidate_display_limit_presentation_slicing() {
    let mut engine = BornoEngine::new();
    engine.set_enable_dictionary_assistance(false);

    // Input "ka" generates 3 candidates in Borno: ["কা", "ক্যা", "খা"]
    engine.set_candidate_display_limit(2);
    let res = engine.process_event(EngineEvent::InputString("ka".to_string()));

    assert_eq!(res.state.candidates.len(), 2);
    assert_eq!(res.state.candidates, vec!["কা", "ক্যা"]);

    // Updating limit dynamically re-slices or expands on next composition event
    engine.set_candidate_display_limit(5);
    let res2 = engine.process_event(EngineEvent::InputChar('a')); // "kaa" -> "কাআ"
    assert!(res2.state.candidates.len() <= 5);

    // Re-test with clean "ka" under limit 5
    engine.reset();
    let res3 = engine.process_event(EngineEvent::InputString("ka".to_string()));
    assert_eq!(res3.state.candidates.len(), 3);
    assert_eq!(res3.state.candidates, vec!["কা", "ক্যা", "খা"]);
}

#[test]
fn test_engine_candidate_display_limit_with_fewer_candidates() {
    let mut engine = BornoEngine::new();
    engine.set_candidate_display_limit(5);

    // Input "b" produces exactly 1 candidate: ["ব"]
    let res = engine.process_event(EngineEvent::InputString("b".to_string()));
    assert_eq!(res.state.candidates.len(), 1);
    assert_eq!(res.state.candidates, vec!["ব"]);
}

#[test]
fn test_engine_auto_commit_on_mode_switch_true() {
    let mut engine = BornoEngine::new();
    assert!(engine.auto_commit_on_mode_switch());

    // Compose "ka" -> preedit "কা"
    let r1 = engine.process_event(EngineEvent::InputString("ka".to_string()));
    assert_eq!(r1.state.preedit, "কা");
    assert!(r1.state.is_composing);

    // Switch mode to EnglishPassthrough with auto-commit enabled
    let r2 = engine.process_event(EngineEvent::SwitchMode(InputMode::EnglishPassthrough));
    assert_eq!(r2.committed_text, Some("কা".to_string()));
    assert_eq!(r2.state.preedit, "");
    assert!(!r2.state.is_composing);
    assert_eq!(r2.state.mode, InputMode::EnglishPassthrough);
}

#[test]
fn test_engine_auto_commit_on_mode_switch_false() {
    let mut engine = BornoEngine::new();
    engine.set_auto_commit_on_mode_switch(false);
    assert!(!engine.auto_commit_on_mode_switch());

    // Compose "ka" -> preedit "কা"
    let r1 = engine.process_event(EngineEvent::InputString("ka".to_string()));
    assert_eq!(r1.state.preedit, "কা");
    assert!(r1.state.is_composing);

    // Switch mode to EnglishPassthrough with auto-commit disabled
    let r2 = engine.process_event(EngineEvent::SwitchMode(InputMode::EnglishPassthrough));
    assert_eq!(r2.committed_text, None);
    assert_eq!(r2.state.preedit, "");
    assert!(!r2.state.is_composing);
    assert_eq!(r2.state.mode, InputMode::EnglishPassthrough);
}

#[test]
fn test_engine_preferred_fixed_layout_preference_does_not_mutate_active_composition() {
    let mut engine = BornoEngine::new();
    // Switch to Fixed mode (National by default)
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Type a key in Fixed mode: KeyJ is 'ক' in National
    let r1 = engine.process_event(EngineEvent::InputKey {
        key: bornomala_core::layout::PhysicalKey::KeyJ,
        modifiers: bornomala_core::layout::Modifiers::default(),
    });
    assert_eq!(r1.state.preedit, "ক");
    assert!(r1.state.is_composing);

    // Update preferred fixed layout preference to Probhat
    engine.set_preferred_fixed_layout(FixedLayoutId::Probhat);
    assert_eq!(engine.preferred_fixed_layout(), FixedLayoutId::Probhat);

    // Invariant: active composition remains intact, active mode remains National
    assert_eq!(engine.state().preedit, "ক");
    assert!(engine.state().is_composing);
    assert_eq!(engine.state().mode, InputMode::Fixed(FixedLayoutId::National));
}

#[test]
fn test_engine_startup_mode_initialization() {
    // 1. StartupMode::Fixed with Probhat
    let mut config = BornoConfig::default();
    config.default_startup_mode = StartupMode::Fixed;
    config.preferred_fixed_layout = FixedLayoutId::Probhat;
    config.candidate_display_limit = 3;
    config.auto_commit_on_mode_switch = false;

    let engine = BornoEngine::with_config(&config);
    assert_eq!(engine.state().mode, InputMode::Fixed(FixedLayoutId::Probhat));
    assert_eq!(engine.candidate_display_limit(), 3);
    assert!(!engine.auto_commit_on_mode_switch());
    assert_eq!(engine.preferred_fixed_layout(), FixedLayoutId::Probhat);

    // 2. StartupMode::EnglishPassthrough
    config.default_startup_mode = StartupMode::EnglishPassthrough;
    let engine_en = BornoEngine::with_config(&config);
    assert_eq!(engine_en.state().mode, InputMode::EnglishPassthrough);

    // 3. StartupMode::Phonetic
    config.default_startup_mode = StartupMode::Phonetic;
    let engine_ph = BornoEngine::with_config(&config);
    assert_eq!(engine_ph.state().mode, InputMode::Phonetic);
}
