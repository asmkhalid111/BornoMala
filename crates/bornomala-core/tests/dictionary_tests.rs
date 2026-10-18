use bornomala_core::{BornoEngine, CandidateKind, EngineEvent, InputMode};
use bornomala_core::layout::FixedLayoutId;

#[test]
fn test_layer3_phonetic_ambiguity_khoma_to_k_shoma() {
    let mut engine = BornoEngine::new();
    for ch in "khoma".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }

    let state = engine.state();
    assert_eq!(state.preedit, "খমা");
    assert!(state.candidates.len() >= 2);

    // Slot 0 Invariant: User raw transliteration
    assert_eq!(state.candidates[0], "খমা");
    assert_eq!(state.candidate_details[0].kind, CandidateKind::PrimaryTransliteration);

    // Slot 1: Lexicon-validated ambiguity match
    assert_eq!(state.candidates[1], "ক্ষমা");
    assert_eq!(state.candidate_details[1].kind, CandidateKind::ExactLexiconMatch);
    assert!(state.candidate_details[1].score > 0);

    // Lockstep invariant
    assert_eq!(state.candidates.len(), state.candidate_details.len());
    for (i, cand) in state.candidates.iter().enumerate() {
        assert_eq!(cand, &state.candidate_details[i].text);
    }
}

#[test]
fn test_layer3_phonetic_ambiguity_biggan_and_bhasha() {
    let mut engine = BornoEngine::new();

    // biggan -> "বিজ্ঞান"
    for ch in "biggan".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let state = engine.state();
    assert_eq!(state.candidates[0], "বিগ্গান");
    assert_eq!(state.candidates[1], "বিজ্ঞান");
    assert_eq!(state.candidate_details[1].kind, CandidateKind::ExactLexiconMatch);

    engine.reset();

    // bhasha -> "ভাষা"
    for ch in "bhasha".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let state2 = engine.state();
    assert_eq!(state2.candidates[0], "ভাশা");
    assert_eq!(state2.candidates[1], "ভাষা");
    assert_eq!(state2.candidate_details[1].kind, CandidateKind::ExactLexiconMatch);

    engine.reset();

    // opekkha -> "অপেক্ষা"
    for ch in "opekkha".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let state3 = engine.state();
    assert_eq!(state3.candidates[0], "অপেক্কহা");
    assert_eq!(state3.candidates[1], "অপেক্ষা");
    assert_eq!(state3.candidate_details[1].kind, CandidateKind::ExactLexiconMatch);
}

#[test]
fn test_layer3_prefix_completion_and_single_consonant_protection() {
    let mut engine = BornoEngine::new();

    // Single consonant: "b" -> "ব"
    engine.process_event(EngineEvent::InputChar('b'));
    let state_b = engine.state();
    assert_eq!(state_b.preedit, "ব");
    assert_eq!(state_b.candidates[0], "ব");
    // Single character raw buffer should NOT flood with completions
    // Only primary transliteration should be present
    assert_eq!(state_b.candidates.len(), 1);

    engine.reset();

    // Multi-character prefix: "bang" -> "বাং"
    for ch in "bang".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let state_bang = engine.state();
    assert_eq!(state_bang.preedit, "বাং");
    assert_eq!(state_bang.candidates[0], "বাং");
    assert!(state_bang.candidates.len() >= 2);

    // Should contain high-frequency completions like "বাংলাদেশ" or "বাংলা"
    assert!(state_bang.candidates.contains(&"বাংলাদেশ".to_string()));
    assert_eq!(state_bang.candidate_details[1].kind, CandidateKind::PrefixCompletion);
}

#[test]
fn test_layer4_ranking_and_display_limit_slicing() {
    let mut engine = BornoEngine::new();
    engine.set_candidate_display_limit(2);

    for ch in "bang".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let state = engine.state();
    assert_eq!(state.candidates.len(), 2);
    assert_eq!(state.candidate_details.len(), 2);

    // Display limit K = 1: only Slot 0
    engine.set_candidate_display_limit(1);
    let state_k1 = engine.state();
    assert_eq!(state_k1.candidates.len(), 1);
    assert_eq!(state_k1.candidate_details.len(), 1);
    assert_eq!(state_k1.candidates[0], "বাং");
    assert_eq!(state_k1.candidate_details[0].kind, CandidateKind::PrimaryTransliteration);
}

#[test]
fn test_layer5_settings_toggles_and_isolation() {
    let mut engine = BornoEngine::new();

    // Toggle off word completion: ambiguity still works, completion disabled
    engine.set_enable_word_completion(false);
    for ch in "bang".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let state_no_comp = engine.state();
    assert_eq!(state_no_comp.candidates.len(), 1); // No completions generated

    engine.reset();

    // Ambiguity still works when completion disabled
    for ch in "khoma".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let state_amb = engine.state();
    assert_eq!(state_amb.candidates[1], "ক্ষমা");

    engine.reset();

    // Toggle off dictionary assistance completely
    engine.set_enable_dictionary_assistance(false);
    for ch in "khoma".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let state_no_dict = engine.state();
    assert_eq!(state_no_dict.candidates.len(), 1); // No dictionary candidates at all
    assert_eq!(state_no_dict.candidates[0], "খমা");
}

#[test]
fn test_layer5_fixed_layout_mode_isolation() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::Fixed(FixedLayoutId::National)));

    // Physical key for 'k' (National: D)
    engine.process_event(EngineEvent::InputKey {
        key: bornomala_core::layout::PhysicalKey::KeyD,
        modifiers: bornomala_core::layout::Modifiers::new(false, false, false, false),
    });

    let state = engine.state();
    // In Fixed mode, candidates and candidate_details must be strictly empty!
    assert!(state.candidates.is_empty());
    assert!(state.candidate_details.is_empty());
}

#[test]
fn test_layer5_english_passthrough_mode_isolation() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::SwitchMode(InputMode::EnglishPassthrough));

    let res = engine.process_event(EngineEvent::InputChar('a'));
    assert_eq!(res.committed_text, Some("a".to_string()));
    assert!(res.state.candidates.is_empty());
    assert!(res.state.candidate_details.is_empty());
}

#[test]
fn test_layer5_typo_correction_fixture_brristi() {
    let mut engine = BornoEngine::new();

    // Input "brristi" -> Primary transliteration is "বৃস্তি" (typo with dental s)
    for ch in "brristi".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }

    let state = engine.state();
    assert_eq!(state.candidates[0], "বৃস্তি");

    // Spelling correction discovers "বৃষ্টি"
    assert!(state.candidates.contains(&"বৃষ্টি".to_string()));
    let bristi_idx = state.candidates.iter().position(|c| c == "বৃষ্টি").unwrap();
    assert_eq!(state.candidate_details[bristi_idx].kind, CandidateKind::SpellingCorrection);
}

#[test]
fn test_manual_checklist_khoma_commit_distinction() {
    let mut engine = BornoEngine::new();

    // 1. Space commit: commits Candidate 0 ("খমা ")
    for ch in "khoma".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let res_space = engine.process_event(EngineEvent::InputChar(' '));
    assert_eq!(res_space.committed_text, Some("খমা ".to_string()));

    engine.reset();

    // 2. Candidate 1 selection: commits Candidate 1 ("ক্ষমা")
    for ch in "khoma".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }
    let res_select = engine.process_event(EngineEvent::SelectCandidate(1));
    assert_eq!(res_select.committed_text, Some("ক্ষমা".to_string()));
}

#[test]
fn test_manual_checklist_manus_ambiguity_match() {
    let mut engine = BornoEngine::new();

    for ch in "manus".chars() {
        engine.process_event(EngineEvent::InputChar(ch));
    }

    let state = engine.state();
    assert_eq!(state.candidates[0], "মানুস");
    assert_eq!(state.candidate_details[0].kind, CandidateKind::PrimaryTransliteration);

    // Homophone ambiguity rule discovers "মানুষ"
    assert!(state.candidates.contains(&"মানুষ".to_string()));
    let manus_idx = state.candidates.iter().position(|c| c == "মানুষ").unwrap();
    assert_eq!(state.candidate_details[manus_idx].kind, CandidateKind::ExactLexiconMatch);
}

