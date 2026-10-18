//! Comprehensive Layered Evaluation Suite for Borno Core Phonetic Engine (Milestone M3)
//!
//! Validates:
//! 1. Vowels (independent vs. dependent kars across all cases, inherent vowel bridge 'o')
//! 2. Consonants, retroflex vs dental stops, sibilants, flapped rhotics
//! 3. Curated Core Conjunct Matrix (30 high-frequency verified conjuncts)
//! 4. Geminates (identical consonant doubling)
//! 5. Digraph `sT` precedence and codepoint sequence verification
//! 6. Phalas and special modifiers (Reph, Ra-phala, Ya-phala, Ba-phala, Candrabindu, Khanda-Ta)
//! 7. Token boundary breaks (backtick) and escape sequences
//! 8. Bounded candidate generation (ALT-1, ALT-2a, ALT-2b, max 5 cap, single-fallback)
//! 9. Strict exclusion of hidden word-specific dictionary aliases (khoma -> খমা, biggan -> বিগ্গান)
//! 10. Cursor mapping Scenarios C-1 through C-5 (including normalization-induced scalar collapse)
//! 11. Quantitative release-mode latency benchmark (<1.0 ms budget)

use bornomala_core::phonetic::{self, PhoneticEngine};
use bornomala_core::{BornoEngine, EngineEvent};
use std::time::Instant;

// ============================================================================
// Layer 1: Vowels (Independent & Dependent Kars)
// ============================================================================

#[test]
fn test_vowel_independent_a() {
    assert_eq!(phonetic::transliterate("o"), "অ");
    assert_eq!(phonetic::transliterate("a"), "আ");
    assert_eq!(phonetic::transliterate("A"), "আ");
    assert_eq!(phonetic::transliterate("aa"), "আ");
}

#[test]
fn test_vowel_independent_i_and_u() {
    assert_eq!(phonetic::transliterate("i"), "ই");
    assert_eq!(phonetic::transliterate("I"), "ঈ");
    assert_eq!(phonetic::transliterate("ee"), "ঈ");
    assert_eq!(phonetic::transliterate("u"), "উ");
    assert_eq!(phonetic::transliterate("U"), "ঊ");
    assert_eq!(phonetic::transliterate("oo"), "ঊ");
}

#[test]
fn test_vowel_independent_e_and_ae() {
    assert_eq!(phonetic::transliterate("e"), "এ");
    assert_eq!(phonetic::transliterate("E"), "অ্যা");
    assert_eq!(phonetic::transliterate("ae"), "অ্যা");
}

#[test]
fn test_vowel_independent_diphthongs_and_rri() {
    assert_eq!(phonetic::transliterate("oi"), "ঐ");
    assert_eq!(phonetic::transliterate("OI"), "ঐ");
    assert_eq!(phonetic::transliterate("O"), "ও");
    assert_eq!(phonetic::transliterate("ou"), "ঔ");
    assert_eq!(phonetic::transliterate("OU"), "ঔ");
    assert_eq!(phonetic::transliterate("rri"), "ঋ");
}

#[test]
fn test_vowel_kar_aa_and_ii() {
    assert_eq!(phonetic::transliterate("ka"), "কা");
    assert_eq!(phonetic::transliterate("kA"), "কা");
    assert_eq!(phonetic::transliterate("kaa"), "কা");
    assert_eq!(phonetic::transliterate("ki"), "কি");
    assert_eq!(phonetic::transliterate("kI"), "কী");
    assert_eq!(phonetic::transliterate("kee"), "কী");
}

#[test]
fn test_vowel_kar_uu_and_ee() {
    assert_eq!(phonetic::transliterate("ku"), "কু");
    assert_eq!(phonetic::transliterate("kU"), "কূ");
    assert_eq!(phonetic::transliterate("koo"), "কূ");
    assert_eq!(phonetic::transliterate("ke"), "কে");
    assert_eq!(phonetic::transliterate("kE"), "ক্যা");
}

#[test]
fn test_vowel_kar_o_and_diphthongs() {
    assert_eq!(phonetic::transliterate("kO"), "কো");
    assert_eq!(phonetic::transliterate("koi"), "কৈ");
    assert_eq!(phonetic::transliterate("kOI"), "কৈ");
    assert_eq!(phonetic::transliterate("kou"), "কৌ");
    assert_eq!(phonetic::transliterate("kOU"), "কৌ");
    assert_eq!(phonetic::transliterate("krri"), "কৃ");
}

#[test]
fn test_inherent_vowel_bridge() {
    assert_eq!(phonetic::transliterate("kor"), "কর");
    assert_eq!(phonetic::transliterate("kol"), "কল");
    assert_eq!(phonetic::transliterate("omuk"), "অমুক");
    assert_eq!(phonetic::transliterate("kOnO"), "কোনো");
}

// ============================================================================
// Layer 2: Consonants & Phonological Distinctions
// ============================================================================

#[test]
fn test_consonants_velar() {
    assert_eq!(phonetic::transliterate("k"), "ক");
    assert_eq!(phonetic::transliterate("kh"), "খ");
    assert_eq!(phonetic::transliterate("g"), "গ");
    assert_eq!(phonetic::transliterate("gh"), "ঘ");
    assert_eq!(phonetic::transliterate("Ng"), "ঙ");
    assert_eq!(phonetic::transliterate("ng"), "ং");
}

#[test]
fn test_consonants_palatal() {
    assert_eq!(phonetic::transliterate("c"), "চ");
    assert_eq!(phonetic::transliterate("ch"), "ছ");
    assert_eq!(phonetic::transliterate("Ch"), "ছ");
    assert_eq!(phonetic::transliterate("j"), "জ");
    assert_eq!(phonetic::transliterate("jh"), "ঝ");
    assert_eq!(phonetic::transliterate("NG"), "ঞ");
}

#[test]
fn test_consonants_retroflex_stops() {
    assert_eq!(phonetic::transliterate("T"), "ট");
    assert_eq!(phonetic::transliterate("Th"), "ঠ");
    assert_eq!(phonetic::transliterate("D"), "ড");
    assert_eq!(phonetic::transliterate("Dh"), "ঢ");
    assert_eq!(phonetic::transliterate("N"), "ণ");
}

#[test]
fn test_consonants_dental_stops() {
    assert_eq!(phonetic::transliterate("t"), "ত");
    assert_eq!(phonetic::transliterate("th"), "থ");
    assert_eq!(phonetic::transliterate("d"), "দ");
    assert_eq!(phonetic::transliterate("dh"), "ধ");
    assert_eq!(phonetic::transliterate("n"), "ন");
}

#[test]
fn test_consonants_labial() {
    assert_eq!(phonetic::transliterate("p"), "প");
    assert_eq!(phonetic::transliterate("ph"), "ফ");
    assert_eq!(phonetic::transliterate("f"), "ফ");
    assert_eq!(phonetic::transliterate("b"), "ব");
    assert_eq!(phonetic::transliterate("bh"), "ভ");
    assert_eq!(phonetic::transliterate("v"), "ভ");
    assert_eq!(phonetic::transliterate("m"), "ম");
}

#[test]
fn test_consonants_sibilants() {
    assert_eq!(phonetic::transliterate("sh"), "শ");
    assert_eq!(phonetic::transliterate("S"), "শ");
    assert_eq!(phonetic::transliterate("Sh"), "ষ");
    assert_eq!(phonetic::transliterate("s"), "স");
}

#[test]
fn test_consonants_semivowels_and_liquids() {
    assert_eq!(phonetic::transliterate("z"), "য");
    assert_eq!(phonetic::transliterate("r"), "র");
    assert_eq!(phonetic::transliterate("l"), "ল");
    assert_eq!(phonetic::transliterate("h"), "হ");
}

#[test]
fn test_consonants_flapped_rhotics_nfc() {
    assert_eq!(phonetic::transliterate("R"), bornomala_core::unicode::normalize_nfc("ড়"));
    assert_eq!(phonetic::transliterate("Rh"), bornomala_core::unicode::normalize_nfc("ঢ়"));
    assert_eq!(phonetic::transliterate("y"), bornomala_core::unicode::normalize_nfc("য়"));
    assert_eq!(phonetic::transliterate("Y"), bornomala_core::unicode::normalize_nfc("য়"));
}

// ============================================================================
// Layer 3: Curated Core Conjunct Matrix (30 High-Frequency Conjuncts)
// ============================================================================

#[test]
fn test_conjunct_ksh_and_aliases() {
    assert_eq!(phonetic::transliterate("kSh"), "ক্ষ");
    assert_eq!(phonetic::transliterate("x"), "ক্ষ");
    assert_eq!(phonetic::transliterate("kShm"), "ক্ষ্ম");
    assert_eq!(phonetic::transliterate("xm"), "ক্ষ্ম");
    assert_eq!(phonetic::transliterate("kShN"), "ক্ষ্ণ");
    assert_eq!(phonetic::transliterate("xN"), "ক্ষ্ণ");
}

#[test]
fn test_conjunct_jng_and_cch() {
    assert_eq!(phonetic::transliterate("jNG"), "জ্ঞ");
    assert_eq!(phonetic::transliterate("jgy"), "জ্ঞ");
    assert_eq!(phonetic::transliterate("cch"), "চ্ছ");
}

#[test]
fn test_conjunct_palatal_nasals() {
    assert_eq!(phonetic::transliterate("NGc"), "ঞ্চ");
    assert_eq!(phonetic::transliterate("Nc"), "ঞ্চ");
    assert_eq!(phonetic::transliterate("NGch"), "ঞ্ছ");
    assert_eq!(phonetic::transliterate("Nch"), "ঞ্ছ");
    assert_eq!(phonetic::transliterate("NGj"), "ঞ্জ");
    assert_eq!(phonetic::transliterate("nj"), "ঞ্জ");
}

#[test]
fn test_conjunct_velar_nasals() {
    assert_eq!(phonetic::transliterate("Ngk"), "ঙ্ক");
    assert_eq!(phonetic::transliterate("Ngkh"), "ঙ্খ");
    assert_eq!(phonetic::transliterate("Ngg"), "ঙ্গ");
    assert_eq!(phonetic::transliterate("Nggh"), "ঙ্ঘ");
}

#[test]
fn test_conjunct_triple_clusters() {
    assert_eq!(phonetic::transliterate("skr"), "স্ক্র");
    assert_eq!(phonetic::transliterate("str"), "স্ত্র");
}

#[test]
fn test_conjunct_retroflex_clusters() {
    assert_eq!(phonetic::transliterate("sT"), "স্ট");
    assert_eq!(phonetic::transliterate("ST"), "ষ্ট");
    assert_eq!(phonetic::transliterate("ShT"), "ষ্ট");
    assert_eq!(phonetic::transliterate("shT"), "ষ্ট");
    assert_eq!(phonetic::transliterate("STh"), "ষ্ঠ");
    assert_eq!(phonetic::transliterate("ShTh"), "ষ্ঠ");
    assert_eq!(phonetic::transliterate("SN"), "ষ্ণ");
    assert_eq!(phonetic::transliterate("ShN"), "ষ্ণ");
}

#[test]
fn test_conjunct_dental_clusters() {
    assert_eq!(phonetic::transliterate("nt"), "ন্ত");
    assert_eq!(phonetic::transliterate("nth"), "ন্থ");
    assert_eq!(phonetic::transliterate("nd"), "ন্দ");
    assert_eq!(phonetic::transliterate("ndh"), "ন্ধ");
}

#[test]
fn test_conjunct_labial_clusters() {
    assert_eq!(phonetic::transliterate("mp"), "ম্প");
    assert_eq!(phonetic::transliterate("mb"), "ম্ব");
    assert_eq!(phonetic::transliterate("mbh"), "ম্ভ");
}

#[test]
fn test_conjunct_ha_complex_clusters() {
    assert_eq!(phonetic::transliterate("hm"), "হ্ম");
    assert_eq!(phonetic::transliterate("hn"), "হ্ন");
    assert_eq!(phonetic::transliterate("hN"), "হ্ণ");
    assert_eq!(phonetic::transliterate("hl"), "হ্ল");
    assert_eq!(phonetic::transliterate("hr"), "হ্র");
}

#[test]
fn test_conjunct_dental_sibilant_clusters() {
    assert_eq!(phonetic::transliterate("st"), "স্ত");
    assert_eq!(phonetic::transliterate("St"), "শ্ত");
    assert_eq!(phonetic::transliterate("sht"), "শ্ত");
}

// ============================================================================
// Layer 4: Geminates & Digraph Precedence
// ============================================================================

#[test]
fn test_geminates_stops() {
    assert_eq!(phonetic::transliterate("kk"), "ক্ক");
    assert_eq!(phonetic::transliterate("gg"), "গ্গ");
    assert_eq!(phonetic::transliterate("cc"), "চ্চ");
    assert_eq!(phonetic::transliterate("jj"), "জ্জ");
    assert_eq!(phonetic::transliterate("TT"), "ট্ট");
    assert_eq!(phonetic::transliterate("DD"), "ড্ড");
    assert_eq!(phonetic::transliterate("tt"), "ত্ত");
    assert_eq!(phonetic::transliterate("dd"), "দ্দ");
    assert_eq!(phonetic::transliterate("pp"), "প্প");
    assert_eq!(phonetic::transliterate("bb"), "ব্ব");
}

#[test]
fn test_geminates_nasals_and_liquids() {
    assert_eq!(phonetic::transliterate("mm"), "ম্ম");
    assert_eq!(phonetic::transliterate("nn"), "ন্ন");
    assert_eq!(phonetic::transliterate("ll"), "ল্ল");
    assert_eq!(phonetic::transliterate("ss"), "স্স");
}

#[test]
fn test_digraph_st_precedence() {
    assert_eq!(phonetic::transliterate("sT"), "স্ট");
    let chars: Vec<char> = phonetic::transliterate("sT").chars().collect();
    assert_eq!(chars, vec!['\u{09B8}', '\u{09CD}', '\u{099F}']);
    assert_eq!(phonetic::transliterate("sTeshon"), "স্টেশন");
}

#[test]
fn test_digraph_st_distinction_tatsama_and_dental() {
    assert_eq!(phonetic::transliterate("ST"), "ষ্ট");
    assert_eq!(phonetic::transliterate("st"), "স্ত");
}

// ============================================================================
// Layer 5: Phalas & Modifiers
// ============================================================================

#[test]
fn test_phala_reph() {
    assert_eq!(phonetic::transliterate("rk"), "র্ক");
    assert_eq!(phonetic::transliterate("rb"), "র্ব");
    assert_eq!(phonetic::transliterate("rm"), "র্ম");
    assert_eq!(phonetic::transliterate("rSh"), "র্ষ");
    assert_eq!(phonetic::transliterate("borNo"), "বর্ণ");
}

#[test]
fn test_phala_ra() {
    assert_eq!(phonetic::transliterate("kr"), "ক্র");
    assert_eq!(phonetic::transliterate("gr"), "গ্র");
    assert_eq!(phonetic::transliterate("pr"), "প্র");
    assert_eq!(phonetic::transliterate("tr"), "ত্র");
    assert_eq!(phonetic::transliterate("br"), "ব্র");
}

#[test]
fn test_phala_ya() {
    assert_eq!(phonetic::transliterate("ky"), "ক্য");
    assert_eq!(phonetic::transliterate("kZ"), "ক্য");
    assert_eq!(phonetic::transliterate("by"), "ব্য");
    assert_eq!(phonetic::transliterate("ny"), "ন্য");
    assert_eq!(phonetic::transliterate("ty"), "ত্য");
    assert_eq!(phonetic::transliterate("bakyo"), "বাক্য");
}

#[test]
fn test_phala_ba() {
    assert_eq!(phonetic::transliterate("sw"), "স্ব");
    assert_eq!(phonetic::transliterate("sv"), "স্ব");
    assert_eq!(phonetic::transliterate("dw"), "দ্ব");
    assert_eq!(phonetic::transliterate("kw"), "ক্ব");
    assert_eq!(phonetic::transliterate("swadhin"), "স্বাধিন");
    assert_eq!(phonetic::transliterate("swadheen"), "স্বাধীন");
}

#[test]
fn test_modifier_candrabindu_reordering() {
    assert_eq!(phonetic::transliterate("c^ad"), "চাঁদ");
    assert_eq!(phonetic::transliterate("b^aTi"), "বাঁটি");
}

#[test]
fn test_modifier_khanda_ta() {
    assert_eq!(phonetic::transliterate("ut`sob"), "উৎসব");
    assert_eq!(phonetic::transliterate("hoThat`"), "হঠাৎ");
    assert_eq!(phonetic::transliterate("u\\tsob"), "উৎসব");
}

#[test]
fn test_modifier_virama_and_token_breaks() {
    assert_eq!(phonetic::transliterate("k,,"), "ক্");
    assert_eq!(phonetic::transliterate("k``"), "ক্");
    assert_eq!(phonetic::transliterate("k`"), "ক্");
    assert_eq!(phonetic::transliterate("ek`i"), "একই");
    assert_eq!(phonetic::transliterate("pOt`h"), "পোতহ");
}

// ============================================================================
// Layer 6: Bounded Candidate Generation & Scope Isolation
// ============================================================================

#[test]
fn test_candidate_alt1_ami() {
    let (_, _, c) = PhoneticEngine::transliterate("ami");
    assert_eq!(c, vec!["আমি", "আমী"]);
}

#[test]
fn test_candidate_alt1_tumi() {
    let (_, _, c) = PhoneticEngine::transliterate("tumi");
    assert_eq!(c, vec!["তুমি", "তূমি"]);
}

#[test]
fn test_candidate_alt1_bari_and_pakhi() {
    let (_, _, c_bari) = PhoneticEngine::transliterate("baRi");
    assert_eq!(c_bari, vec![bornomala_core::unicode::normalize_nfc("বাড়ি"), bornomala_core::unicode::normalize_nfc("বাড়ী")]);

    let (_, _, c_pakhi) = PhoneticEngine::transliterate("pakhi");
    assert_eq!(c_pakhi, vec!["পাখি", "পাখী"]);
}

#[test]
fn test_candidate_alt2a_initial_sibilants() {
    let (_, _, c) = PhoneticEngine::transliterate("shanto");
    assert_eq!(c, vec!["শান্ত", "সান্ত"]);
}

#[test]
fn test_candidate_alt2b_medial_st() {
    let (_, _, c) = PhoneticEngine::transliterate("kosTo");
    assert_eq!(c, vec!["কস্ট", "কষ্ট"]);
}

#[test]
fn test_candidate_alt2b_dental_non_alternation() {
    let (_, _, c) = PhoneticEngine::transliterate("kosto");
    assert_eq!(c, vec!["কস্ত"]);
}

#[test]
fn test_candidate_single_fallback() {
    let (_, _, c) = PhoneticEngine::transliterate("jol");
    assert_eq!(c, vec!["জল"]);
}

#[test]
fn test_candidate_strict_cap_5() {
    let (_, _, c) = PhoneticEngine::transliterate("shantikosToami");
    assert!(c.len() <= 5);
}

#[test]
fn test_no_dictionary_overrides_khoma() {
    assert_eq!(phonetic::transliterate("khoma"), "খমা");
    assert_eq!(phonetic::transliterate("kShoma"), "ক্ষমা");
    assert_eq!(phonetic::transliterate("xoma"), "ক্ষমা");
}

#[test]
fn test_no_dictionary_overrides_biggan() {
    assert_eq!(phonetic::transliterate("biggan"), "বিগ্গান");
    assert_eq!(phonetic::transliterate("bijNGan"), "বিজ্ঞান");
    assert_eq!(phonetic::transliterate("bijgyan"), "বিজ্ঞান");
}

#[test]
fn test_verified_example_words_brristi_and_shreshtho() {
    assert_eq!(phonetic::transliterate("brriSTi"), "বৃষ্টি");
    assert_eq!(phonetic::transliterate("brriShTi"), "বৃষ্টি");
    assert_eq!(phonetic::transliterate("shreshTho"), "শ্রেষ্ঠ");
}

// ============================================================================
// Layer 7: Cursor Mapping Scenarios C-1 Through C-5
// ============================================================================

#[test]
fn test_scenario_c1_ksh_interior_backspace() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::InputString("kSh".to_string()));
    assert_eq!(engine.state().preedit, "ক্ষ");
    assert_eq!(engine.state().cursor_pos, 3);

    engine.process_event(EngineEvent::MoveCursorLeft);
    assert_eq!(engine.state().cursor_pos, 2);

    let r_bs = engine.process_event(EngineEvent::Backspace);
    assert_eq!(r_bs.state.raw_buffer, "kh");
    assert_eq!(r_bs.state.preedit, "খ");
    assert_eq!(r_bs.state.cursor_pos, 0);

    let r_right = engine.process_event(EngineEvent::MoveCursorRight);
    assert_eq!(r_right.state.cursor_pos, 1);
}

#[test]
fn test_scenario_c1_ksh_end_backspace_contrast() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::InputString("kSh".to_string()));
    let r_bs_end = engine.process_event(EngineEvent::Backspace);
    assert_eq!(r_bs_end.state.raw_buffer, "kS");
    assert_eq!(r_bs_end.state.preedit, "কশ");
    assert_eq!(r_bs_end.state.cursor_pos, 2);
}

#[test]
fn test_scenario_c2_jang_lowercase() {
    let (preedit, spans) = phonetic::Tokenizer::tokenize("jaNg");
    assert_eq!(preedit, "জাঙ");
    assert_eq!(spans.len(), 3);
    assert_eq!(phonetic::Tokenizer::map_cursor_to_target(2, "jaNg", &spans, &preedit), 2);
    assert_eq!(phonetic::Tokenizer::map_cursor_to_target(3, "jaNg", &spans, &preedit), 2);
    assert_eq!(phonetic::Tokenizer::map_cursor_to_target(4, "jaNg", &spans, &preedit), 3);
}

#[test]
fn test_scenario_c2_jang_uppercase() {
    let (preedit, spans) = phonetic::Tokenizer::tokenize("jaNG");
    assert_eq!(preedit, "জাঞ");
    assert_eq!(phonetic::Tokenizer::map_cursor_to_target(0, "jaNG", &spans, &preedit), 0);
    assert_eq!(phonetic::Tokenizer::map_cursor_to_target(2, "jaNG", &spans, &preedit), 2);
    assert_eq!(phonetic::Tokenizer::map_cursor_to_target(3, "jaNG", &spans, &preedit), 2);
    assert_eq!(phonetic::Tokenizer::map_cursor_to_target(4, "jaNG", &spans, &preedit), 3);
}

#[test]
fn test_scenario_c3_vowel_delete() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::InputString("ee".to_string()));
    assert_eq!(engine.state().preedit, "ঈ");
    assert_eq!(engine.state().cursor_pos, 1);

    engine.process_event(EngineEvent::MoveCursorLeft);
    assert_eq!(engine.state().cursor_pos, 0);

    let r_del = engine.process_event(EngineEvent::Delete);
    assert_eq!(r_del.state.raw_buffer, "e");
    assert_eq!(r_del.state.preedit, "এ");
    assert_eq!(r_del.state.cursor_pos, 0);
}

#[test]
fn test_scenario_c4_boundary_backspace() {
    let mut engine = BornoEngine::new();
    engine.process_event(EngineEvent::InputString("ka".to_string()));
    assert_eq!(engine.state().preedit, "কা");
    assert_eq!(engine.state().cursor_pos, 2);

    let res = engine.process_event(EngineEvent::Backspace);
    assert_eq!(res.state.raw_buffer, "k");
    assert_eq!(res.state.preedit, "ক");
    assert_eq!(res.state.cursor_pos, 1);
}

#[test]
fn test_scenario_c5_scalar_collapse_and_spans() {
    let (raw_preedit, spans) = phonetic::Tokenizer::tokenize("kO");
    assert_eq!(raw_preedit, "ক\u{09CB}");
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].source_range, 0..1);
    assert_eq!(spans[1].source_range, 1..2);

    let (normalized, realigned_spans, _) = PhoneticEngine::transliterate("kO");
    assert_eq!(normalized, "কো");
    assert_eq!(normalized.chars().count(), 2);
    assert_eq!(realigned_spans[0].target_range, 0..1);
    assert_eq!(realigned_spans[1].target_range, 1..2);
}

#[test]
fn test_scenario_c5_cursor_positions_and_clamping() {
    let (normalized, realigned_spans, _) = PhoneticEngine::transliterate("kO");
    assert_eq!(PhoneticEngine::map_cursor_to_target(0, "kO", &realigned_spans, &normalized), 0);
    assert_eq!(PhoneticEngine::map_cursor_to_target(1, "kO", &realigned_spans, &normalized), 1);
    assert_eq!(PhoneticEngine::map_cursor_to_target(2, "kO", &realigned_spans, &normalized), 2);
    assert_eq!(PhoneticEngine::map_cursor_to_target(999, "kO", &realigned_spans, &normalized), 2);
}

// ============================================================================
// Layer 8: Quantitative Keystroke Latency Benchmark (<1.0 ms gate)
// ============================================================================

#[test]
fn test_benchmark_keystroke_latency() {
    let mut engine = BornoEngine::new();

    let benchmark_tokens = [
        "k", "ka", "ami", "kor", "boi",
        "bangladesh", "bijNGan", "kShoma", "brriSTi",
        "amarbharotborsho", "amarshonarbangla",
    ];

    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let token = benchmark_tokens[i % benchmark_tokens.len()];
        for ch in token.chars() {
            engine.process_event(EngineEvent::InputChar(ch));
        }
        engine.process_event(EngineEvent::Commit);
    }

    let elapsed = start.elapsed();
    let total_keystrokes: usize = (0..iterations)
        .map(|i| benchmark_tokens[i % benchmark_tokens.len()].len() + 1)
        .sum();

    let mean_micros = elapsed.as_micros() as f64 / total_keystrokes as f64;
    let mean_millis = mean_micros / 1000.0;

    println!(
        "\n--- Borno Core Phonetic Benchmark ---\n\
         Total keystrokes: {}\n\
         Total elapsed: {:.2?}\n\
         Mean latency per keystroke: {:.4} ms ({:.2} µs)\n",
        total_keystrokes, elapsed, mean_millis, mean_micros
    );

    #[cfg(debug_assertions)]
    let max_budget_ms = 2.5;
    #[cfg(not(debug_assertions))]
    let max_budget_ms = 1.0;

    assert!(
        mean_millis < max_budget_ms,
        "Mean latency per keystroke exceeded budget: {:.4} ms (budget: {:.1} ms)",
        mean_millis,
        max_budget_ms
    );
}
