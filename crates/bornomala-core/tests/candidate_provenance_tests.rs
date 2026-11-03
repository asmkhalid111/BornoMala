//! Automated Candidate Provenance & Anti-Fabrication Test Suite
//!
//! Validates:
//! 1. Complete eradication of candidate test fixtures ("k" -> ["খ"], "ka" -> ["ক্যা", "খা"]).
//! 2. Complete eradication of ad-hoc literal checks (e.g. raw_buffer == "tumi").
//! 3. Behavioral provenance of Rule ALT-1 (word-final high vowel alternation) with positive & negative controls.
//! 4. Behavioral provenance of Rule ALT-2 (sibilant alternation: ALT-2a initial and ALT-2b medial) with positive & negative controls.
//! 5. Exact candidate metadata lockstep and CandidateKind classification across rule-based and dictionary pipelines.
//! 6. Slot 0 transliteration invariant, bounded 5-candidate capacity, and deduplication.
//! 7. Narrow static source guard tripwire preventing future introduction of hardcoded literal matches in candidate generation.

use bornomala_core::{BornoEngine, CandidateKind, EngineEvent};
use bornomala_core::phonetic::PhoneticEngine;

// ============================================================================
// 1. Candidate Test Fixtures Eradication (AC-2, AC-3)
// ============================================================================

#[test]
fn test_candidate_fixtures_purged() {
    let mut engine = BornoEngine::new();
    engine.set_enable_dictionary_assistance(false);

    // "k" must produce strictly ["ক"] — no fabricated "খ"
    let r_k = engine.process_event(EngineEvent::InputString("k".to_string()));
    assert_eq!(r_k.state.candidates, vec!["ক"]);
    assert!(!r_k.state.candidates.contains(&"খ".to_string()));

    engine.reset();

    // "ka" must produce strictly ["কা"] — no fabricated "ক্যা" or "খা"
    let r_ka = engine.process_event(EngineEvent::InputString("ka".to_string()));
    assert_eq!(r_ka.state.candidates, vec!["কা"]);
    assert!(!r_ka.state.candidates.contains(&"ক্যা".to_string()));
    assert!(!r_ka.state.candidates.contains(&"খা".to_string()));
}

// ============================================================================
// 2. Rule ALT-1: Word-Final High Vowel Provenance & Controls (AC-4)
// ============================================================================

#[test]
fn test_candidate_alt1_word_final_provenance_and_controls() {
    let mut engine = BornoEngine::new();
    engine.set_enable_dictionary_assistance(false);

    // Positive controls: trailing 'i' alternates ি -> ী
    let r_ami = engine.process_event(EngineEvent::InputString("ami".to_string()));
    assert_eq!(r_ami.state.candidates, vec!["আমি", "আমী"]);

    engine.reset();
    let r_tumi = engine.process_event(EngineEvent::InputString("tumi".to_string()));
    // Genuine ALT-1 produces ["তুমি", "তুমী"]. Must NOT contain fabricated initial vowel alternate "তূমি".
    assert_eq!(r_tumi.state.candidates, vec!["তুমি", "তুমী"]);
    assert!(!r_tumi.state.candidates.contains(&"তূমি".to_string()));

    engine.reset();
    let r_pakhi = engine.process_event(EngineEvent::InputString("pakhi".to_string()));
    assert_eq!(r_pakhi.state.candidates, vec!["পাখি", "পাখী"]);

    // Positive controls: trailing 'u' alternates ু -> ূ
    engine.reset();
    let r_babu = engine.process_event(EngineEvent::InputString("babu".to_string()));
    assert_eq!(r_babu.state.candidates, vec!["বাবু", "বাবূ"]);

    // Negative controls: words NOT ending in 'i' or 'u' must NOT trigger ALT-1
    engine.reset();
    let r_amra = engine.process_event(EngineEvent::InputString("amra".to_string()));
    assert_eq!(r_amra.state.candidates, vec!["\u{0986}\u{09AE}\u{09CD}\u{09B0}\u{09BE}"]); // "আমরা" (with conjunct ম্র)

    engine.reset();
    let r_kolom = engine.process_event(EngineEvent::InputString("kolom".to_string()));
    assert_eq!(r_kolom.state.candidates, vec!["কলম"]);
}

// ============================================================================
// 3. Rule ALT-2: Sibilant Alternation Provenance & Controls
// ============================================================================

#[test]
fn test_candidate_alt2_sibilant_provenance_and_controls() {
    let mut engine = BornoEngine::new();
    engine.set_enable_dictionary_assistance(false);

    // ALT-2a: Initial Sibilant Homophone Alternation ("shan" / "shant" / "shand")
    let r_shanto = engine.process_event(EngineEvent::InputString("shanto".to_string()));
    assert_eq!(r_shanto.state.candidates, vec!["শান্ত", "সান্ত"]);

    engine.reset();
    // "shanti" triggers both ALT-1 (trailing 'i') and ALT-2a (initial "shant")
    let r_shanti = engine.process_event(EngineEvent::InputString("shanti".to_string()));
    assert_eq!(r_shanti.state.candidates, vec!["শান্তি", "শান্তী", "সান্তি"]);

    engine.reset();
    let r_shandho = engine.process_event(EngineEvent::InputString("shandho".to_string()));
    assert_eq!(r_shandho.state.candidates, vec!["শান্ধ", "সান্ধ"]);

    // ALT-2a Negative Controls: words not starting with "shan"
    // "shob" transliterates to "শব"; does not match "shan", so no ALT-2a alternation to "সব"
    engine.reset();
    let r_shob = engine.process_event(EngineEvent::InputString("shob".to_string()));
    assert_eq!(r_shob.state.candidates, vec!["শব"]);
    assert!(!r_shob.state.candidates.contains(&"সব".to_string()));

    engine.reset();
    let r_sundor = engine.process_event(EngineEvent::InputString("sundor".to_string()));
    assert_eq!(r_sundor.state.candidates, vec!["সুন্দর"]);

    // ALT-2b: Medial Sibilant Cluster Alternation (contains "sT" / "sTh")
    engine.reset();
    let r_kosto = engine.process_event(EngineEvent::InputString("kosTo".to_string()));
    assert_eq!(r_kosto.state.candidates, vec!["কস্ট", "কষ্ট"]);

    // ALT-2b Negative Control: dental "st" does NOT trigger retroflex alternation
    engine.reset();
    let r_dental_st = engine.process_event(EngineEvent::InputString("kosto".to_string()));
    assert_eq!(r_dental_st.state.candidates, vec!["কস্ত"]);
    assert!(!r_dental_st.state.candidates.contains(&"কষ্ট".to_string()));
}

// ============================================================================
// 4. Candidate Metadata Lockstep & CandidateKind Classification (AC-7)
// ============================================================================

#[test]
fn test_candidate_metadata_lockstep_without_dictionary() {
    let mut engine = BornoEngine::new();
    engine.set_enable_dictionary_assistance(false);

    engine.process_event(EngineEvent::InputString("shanti".to_string()));
    let state = engine.state();

    assert_eq!(state.candidates.len(), 3);
    assert_eq!(state.candidate_details.len(), 3);

    // 1:1 Lockstep Text Invariant
    for (i, cand) in state.candidates.iter().enumerate() {
        assert_eq!(&state.candidate_details[i].text, cand);
    }

    // Slot 0 Invariant: Must be PrimaryTransliteration
    assert_eq!(state.candidate_details[0].kind, CandidateKind::PrimaryTransliteration);
    assert_eq!(state.candidate_details[0].text, "শান্তি");

    // Orthographic Alternates: Must be PhoneticAlternative
    assert_eq!(state.candidate_details[1].kind, CandidateKind::PhoneticAlternative);
    assert_eq!(state.candidate_details[1].text, "শান্তী");

    assert_eq!(state.candidate_details[2].kind, CandidateKind::PhoneticAlternative);
    assert_eq!(state.candidate_details[2].text, "সান্তি");
}

#[test]
fn test_candidate_metadata_lockstep_with_dictionary() {
    let mut engine = BornoEngine::new();
    engine.set_enable_dictionary_assistance(true);

    engine.process_event(EngineEvent::InputString("k".to_string()));
    let state = engine.state();

    assert!(state.candidates.len() >= 1);
    assert_eq!(state.candidates.len(), state.candidate_details.len());

    // Slot 0 Invariant: Always PrimaryTransliteration, matches preedit
    assert_eq!(state.candidate_details[0].kind, CandidateKind::PrimaryTransliteration);
    assert_eq!(state.candidate_details[0].text, state.preedit);
    assert_eq!(state.candidates[0], "ক");

    // 1:1 Lockstep text mapping across all slots
    for (i, cand) in state.candidates.iter().enumerate() {
        assert_eq!(&state.candidate_details[i].text, cand);
    }
}

// ============================================================================
// 5. Invariants: Bounded Capacity & Deduplication
// ============================================================================

#[test]
fn test_candidate_invariants_bounded_cap_and_deduplication() {
    let mut engine = BornoEngine::new();

    let test_inputs = ["k", "ka", "ami", "tumi", "shanti", "kosTo", "bangla", "borno", "desh"];

    for input in test_inputs {
        engine.reset();
        engine.process_event(EngineEvent::InputString(input.to_string()));
        let state = engine.state();

        // 1. Strict Cap Invariant: <= 5 candidates
        assert!(state.candidates.len() <= 5, "Candidate cap exceeded for input '{}'", input);
        assert_eq!(state.candidates.len(), state.candidate_details.len());

        // 2. Slot 0 Invariant: Slot 0 matches preedit
        if !state.candidates.is_empty() {
            assert_eq!(state.candidates[0], state.preedit);
            assert_eq!(state.candidate_details[0].kind, CandidateKind::PrimaryTransliteration);
        }

        // 3. Deduplication Invariant: All candidate strings must be strictly unique
        for i in 0..state.candidates.len() {
            for j in (i + 1)..state.candidates.len() {
                assert_ne!(
                    state.candidates[i], state.candidates[j],
                    "Duplicate candidate found for input '{}': '{}'",
                    input, state.candidates[i]
                );
            }
        }
    }
}

// ============================================================================
// 6. Direct PhoneticEngine API Candidate Checks
// ============================================================================

#[test]
fn test_phonetic_engine_direct_candidate_generation() {
    let (_, _, c_k) = PhoneticEngine::transliterate("k");
    assert_eq!(c_k, vec!["ক"]);

    let (_, _, c_ka) = PhoneticEngine::transliterate("ka");
    assert_eq!(c_ka, vec!["কা"]);

    let (_, _, c_tumi) = PhoneticEngine::transliterate("tumi");
    assert_eq!(c_tumi, vec!["তুমি", "তুমী"]);

    let (_, _, c_shanti) = PhoneticEngine::transliterate("shanti");
    assert_eq!(c_shanti, vec!["শান্তি", "শান্তী", "সান্তি"]);
}

// ============================================================================
// 7. Narrow Static Source Guard Tripwire (AC-1)
// ============================================================================

/// Narrowly-scoped static source check to detect accidental reintroduction of
/// literal match branches in `candidates.rs`.
///
/// NOTE: This static check is an automated regression tripwire only and does not
/// replace the comprehensive behavioral test suite above.
#[test]
fn test_static_source_guard_anti_fabrication() {
    let source_content = include_str!("../src/phonetic/candidates.rs");

    // Must not compare raw_buffer to literal strings
    assert!(
        !source_content.contains("raw_buffer =="),
        "Violation: `candidates.rs` contains literal comparison with `raw_buffer ==`"
    );
    assert!(
        !source_content.contains("raw_buffer.eq("),
        "Violation: `candidates.rs` contains literal equality check `raw_buffer.eq(`"
    );

    // Must not contain backward compatibility fixtures block
    assert!(
        !source_content.contains("Backward Compatibility Fixtures"),
        "Violation: `candidates.rs` re-introduced backward compatibility fixtures"
    );
}
