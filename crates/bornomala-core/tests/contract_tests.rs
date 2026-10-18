//! Automated Contract Test Suite for Borno Engine (Milestone M1)
//!
//! Validates:
//! 1. State Invariants (persistent state vs ephemeral output effects)
//! 2. Composition Lifecycle Transitions (Idle -> Composing -> Committed/Cancelled -> Idle)
//! 3. Candidate Navigation & Direct Selection Commit
//! 4. In-Composition Cursor Semantics & Interior Backspace/Delete
//! 5. Input Modes (Phonetic vs English Passthrough) and Mode Switching Auto-Commit
//! 6. C-ABI Behavioral Parity & Memory Cleanup

use bornomala_core::{BornoEngine, EngineEvent, InputMode};
use bornomala_core::ffi::*;
use std::ffi::CStr;
use std::ptr;

// ============================================================================
// 1. State Invariant Tests
// ============================================================================

#[test]
fn test_state_invariants_during_composition() {
    let mut engine = BornoEngine::new();

    // Initial state: Idle
    assert!(!engine.state().is_composing);
    assert_eq!(engine.state().cursor_pos, 0);
    assert_eq!(engine.state().selected_candidate_index, None);
    assert!(engine.state().preedit.is_empty());
    assert!(engine.state().raw_buffer.is_empty());

    // Type 'k' -> Composing
    let res = engine.process_event(EngineEvent::InputChar('k'));
    assert!(res.state.is_composing);
    assert_eq!(res.state.cursor_pos, res.state.preedit.chars().count());
    assert_eq!(res.committed_text, None);
    assert!(!res.cancelled);
    assert!(res.handled);

    // Invariant: Persistent state does NOT contain committed_text
    assert_eq!(engine.state().preedit, "ক");
    assert_eq!(engine.state().raw_buffer, "k");
    assert!(engine.state().is_composing);

    // Type 'a' -> "কা"
    let res = engine.process_event(EngineEvent::InputChar('a'));
    assert!(res.state.is_composing);
    assert_eq!(res.state.cursor_pos, res.state.preedit.chars().count());
    assert!(res.state.cursor_pos <= res.state.preedit.chars().count());

    // Invariant: candidate index is None or valid within candidates.len()
    if let Some(idx) = res.state.selected_candidate_index {
        assert!(idx < res.state.candidates.len());
    }

    // Commit -> Idle
    let res = engine.process_event(EngineEvent::Commit);
    assert_eq!(res.committed_text, Some("কা".to_string()));
    assert!(!res.state.is_composing);
    assert_eq!(res.state.preedit, "");
    assert_eq!(res.state.raw_buffer, "");
    assert_eq!(res.state.cursor_pos, 0);
    assert_eq!(res.state.selected_candidate_index, None);

    // Invariant: Next event receives committed_text = None (no stale text!)
    let res2 = engine.process_event(EngineEvent::InputChar('k'));
    assert_eq!(res2.committed_text, None);
    assert_eq!(res2.state.preedit, "ক");
}

// ============================================================================
// 2. Lifecycle Transition Tests
// ============================================================================

#[test]
fn test_lifecycle_commit_transition() {
    let mut engine = BornoEngine::new();

    // Idle -> InputChar -> Composing
    let r1 = engine.process_event(EngineEvent::InputChar('a'));
    assert!(r1.state.is_composing);
    assert_eq!(r1.committed_text, None);

    // Composing -> Commit -> Idle (emits committed_text)
    let r2 = engine.process_event(EngineEvent::Commit);
    assert_eq!(r2.committed_text, Some("আ".to_string()));
    assert!(!r2.state.is_composing);
    assert_eq!(r2.state.preedit, "");
    assert!(!r2.cancelled);
    assert!(r2.handled);

    // Idle -> Commit -> no-op (handled = false)
    let r3 = engine.process_event(EngineEvent::Commit);
    assert_eq!(r3.committed_text, None);
    assert!(!r3.handled);
}

#[test]
fn test_lifecycle_cancel_transition() {
    let mut engine = BornoEngine::new();

    // Idle -> InputChar -> Composing
    engine.process_event(EngineEvent::InputChar('k'));
    engine.process_event(EngineEvent::InputChar('a'));
    assert_eq!(engine.state().preedit, "কা");
    assert!(engine.state().is_composing);

    // Composing -> Cancel -> Idle (cancelled = true, committed_text = None)
    let res = engine.process_event(EngineEvent::Cancel);
    assert!(res.cancelled);
    assert_eq!(res.committed_text, None);
    assert!(!res.state.is_composing);
    assert_eq!(res.state.preedit, "");
    assert_eq!(res.state.raw_buffer, "");
    assert_eq!(res.state.cursor_pos, 0);
    assert!(res.handled);

    // Idle -> Cancel -> no-op (handled = false)
    let res2 = engine.process_event(EngineEvent::Cancel);
    assert!(!res2.cancelled);
    assert!(!res2.handled);
}

#[test]
fn test_lifecycle_reset_transition() {
    let mut engine = BornoEngine::new();

    engine.process_event(EngineEvent::InputChar('k'));
    assert!(engine.state().is_composing);

    // Reset silently restores idle without emitting cancelled = true
    let res = engine.process_event(EngineEvent::Reset);
    assert!(!res.cancelled);
    assert_eq!(res.committed_text, None);
    assert!(!res.state.is_composing);
    assert_eq!(res.state.preedit, "");
    assert!(res.handled);
}

// ============================================================================
// 3. Candidate Navigation & Direct Selection Tests
// ============================================================================

#[test]
fn test_candidate_navigation_wrapping_and_selection() {
    let mut engine = BornoEngine::new();
    engine.set_enable_dictionary_assistance(false);

    // "ka" fixture generates: ["কা", "ক্যা", "খা"]
    let res = engine.process_event(EngineEvent::InputString("ka".to_string()));
    assert_eq!(res.state.candidates, vec!["কা", "ক্যা", "খা"]);
    assert_eq!(res.state.selected_candidate_index, None);

    // NextCandidate from None -> 0 ("কা")
    let r1 = engine.process_event(EngineEvent::NextCandidate);
    assert_eq!(r1.state.selected_candidate_index, Some(0));
    assert!(r1.state.is_composing); // Still composing! Not committed!
    assert_eq!(r1.committed_text, None);

    // NextCandidate -> 1 ("ক্যা")
    let r2 = engine.process_event(EngineEvent::NextCandidate);
    assert_eq!(r2.state.selected_candidate_index, Some(1));
    assert_eq!(r2.committed_text, None);

    // NextCandidate -> 2 ("খা")
    let r3 = engine.process_event(EngineEvent::NextCandidate);
    assert_eq!(r3.state.selected_candidate_index, Some(2));
    assert_eq!(r3.committed_text, None);

    // NextCandidate -> wrap around to 0 ("কা")
    let r4 = engine.process_event(EngineEvent::NextCandidate);
    assert_eq!(r4.state.selected_candidate_index, Some(0));

    // PrevCandidate -> wrap back to 2 ("খা")
    let r5 = engine.process_event(EngineEvent::PrevCandidate);
    assert_eq!(r5.state.selected_candidate_index, Some(2));

    // PrevCandidate -> 1 ("ক্যা")
    let r6 = engine.process_event(EngineEvent::PrevCandidate);
    assert_eq!(r6.state.selected_candidate_index, Some(1));

    // Commit while index 1 is highlighted -> commits "ক্যা"
    let r_commit = engine.process_event(EngineEvent::Commit);
    assert_eq!(r_commit.committed_text, Some("ক্যা".to_string()));
    assert!(!r_commit.state.is_composing);
    assert_eq!(r_commit.state.preedit, "");
}

#[test]
fn test_candidate_direct_selection_and_invalid_index() {
    let mut engine = BornoEngine::new();
    engine.set_enable_dictionary_assistance(false);

    engine.process_event(EngineEvent::InputString("ka".to_string()));
    assert_eq!(engine.state().candidates, vec!["কা", "ক্যা", "খা"]);

    // Out-of-bounds selection (index 99) -> rejected, state preserved
    let r_err = engine.process_event(EngineEvent::SelectCandidate(99));
    assert!(!r_err.handled);
    assert_eq!(r_err.committed_text, None);
    assert_eq!(engine.state().preedit, "কা");
    assert!(engine.state().is_composing);

    // Direct SelectCandidate(2) -> immediately commits "খা" and transitions to Idle
    let r_select = engine.process_event(EngineEvent::SelectCandidate(2));
    assert!(r_select.handled);
    assert_eq!(r_select.committed_text, Some("খা".to_string()));
    assert!(!r_select.state.is_composing);
    assert_eq!(r_select.state.preedit, "");
}

#[test]
fn test_candidate_empty_behavior() {
    let mut engine = BornoEngine::new();

    // In idle (no candidates):
    let r_next = engine.process_event(EngineEvent::NextCandidate);
    assert!(!r_next.handled);
    assert_eq!(r_next.state.selected_candidate_index, None);

    let r_prev = engine.process_event(EngineEvent::PrevCandidate);
    assert!(!r_prev.handled);

    let r_sel = engine.process_event(EngineEvent::SelectCandidate(0));
    assert!(!r_sel.handled);
    assert_eq!(r_sel.committed_text, None);
}

// ============================================================================
// 4. Composition Cursor & In-Preedit Editing Tests
// ============================================================================

#[test]
fn test_cursor_movement_and_bounds_clamping() {
    let mut engine = BornoEngine::new();

    // Type "ami" -> "আমি" (3 codepoints: 'আ', 'ম', 'ি')
    let res = engine.process_event(EngineEvent::InputString("ami".to_string()));
    assert_eq!(res.state.preedit, "আমি");
    assert_eq!(res.state.cursor_pos, 3);

    // MoveCursorLeft -> cursor_pos = 2
    let r1 = engine.process_event(EngineEvent::MoveCursorLeft);
    assert_eq!(r1.state.cursor_pos, 2);

    // MoveCursorLeft -> cursor_pos = 1
    let r2 = engine.process_event(EngineEvent::MoveCursorLeft);
    assert_eq!(r2.state.cursor_pos, 1);

    // MoveCursorLeft -> cursor_pos = 0
    let r3 = engine.process_event(EngineEvent::MoveCursorLeft);
    assert_eq!(r3.state.cursor_pos, 0);

    // MoveCursorLeft at 0 -> clamps at 0
    let r4 = engine.process_event(EngineEvent::MoveCursorLeft);
    assert_eq!(r4.state.cursor_pos, 0);

    // MoveCursorRight -> cursor_pos = 1
    let r5 = engine.process_event(EngineEvent::MoveCursorRight);
    assert_eq!(r5.state.cursor_pos, 1);

    // SetCursor(100) -> clamps to 3 (max codepoint count)
    let r6 = engine.process_event(EngineEvent::SetCursor(100));
    assert_eq!(r6.state.cursor_pos, 3);
}

#[test]
fn test_cursor_interior_backspace_and_delete() {
    let mut engine = BornoEngine::new();

    // Type "kor" -> "কর" (2 codepoints: 'ক', 'র')
    engine.process_event(EngineEvent::InputString("kor".to_string()));
    assert_eq!(engine.state().preedit, "কর");
    assert_eq!(engine.state().cursor_pos, 2);

    // Move cursor to middle (pos = 1, between 'ক' and 'র')
    engine.process_event(EngineEvent::MoveCursorLeft);
    assert_eq!(engine.state().cursor_pos, 1);

    // Backspace at pos 1 deletes character before cursor ('k' removed from "kor" -> "or" -> "অর")
    let res_bs = engine.process_event(EngineEvent::Backspace);
    assert!(res_bs.handled);
    assert_eq!(res_bs.state.cursor_pos, 0);
    assert_eq!(res_bs.state.preedit, "অর");

    // Delete when cursor is at 0 deletes character after cursor ('o' removed from "or" -> "r" -> "র")
    let res_del = engine.process_event(EngineEvent::Delete);
    assert!(res_del.handled);
    assert_eq!(res_del.state.preedit, "র");

    // Another delete removes 'r' -> empty
    let res_del2 = engine.process_event(EngineEvent::Delete);
    assert!(res_del2.handled);
    assert_eq!(res_del2.state.preedit, "");
    assert!(!res_del2.state.is_composing);
}

#[test]
fn test_idle_backspace_and_delete_passthrough() {
    let mut engine = BornoEngine::new();

    // When idle, Backspace and Delete must return handled = false for host passthrough
    let r_bs = engine.process_event(EngineEvent::Backspace);
    assert!(!r_bs.handled);
    assert_eq!(r_bs.committed_text, None);

    let r_del = engine.process_event(EngineEvent::Delete);
    assert!(!r_del.handled);
    assert_eq!(r_del.committed_text, None);
}

// ============================================================================
// 5. Input Modes & Mode Switching Tests
// ============================================================================

#[test]
fn test_english_passthrough_mode() {
    let mut engine = BornoEngine::new();

    // Switch to English Passthrough
    let r_mode = engine.process_event(EngineEvent::SwitchMode(InputMode::EnglishPassthrough));
    assert!(r_mode.handled);
    assert_eq!(r_mode.state.mode, InputMode::EnglishPassthrough);
    assert_eq!(r_mode.state.status.active_mode, "English Passthrough");

    // Typing a character in passthrough immediately emits committed_text
    let r_char = engine.process_event(EngineEvent::InputChar('h'));
    assert!(r_char.handled);
    assert_eq!(r_char.committed_text, Some("h".to_string()));
    assert!(!r_char.state.is_composing);
    assert_eq!(r_char.state.preedit, "");

    // String input in passthrough immediately emits committed_text
    let r_str = engine.process_event(EngineEvent::InputString("ello".to_string()));
    assert!(r_str.handled);
    assert_eq!(r_str.committed_text, Some("ello".to_string()));
    assert!(!r_str.state.is_composing);
}

#[test]
fn test_mode_switch_during_composition_auto_commits() {
    let mut engine = BornoEngine::new();

    // Composing in phonetic mode
    engine.process_event(EngineEvent::InputString("ami".to_string()));
    assert_eq!(engine.state().preedit, "আমি");
    assert!(engine.state().is_composing);

    // Switch mode while composing -> auto-commits active preedit
    let res = engine.process_event(EngineEvent::SwitchMode(InputMode::EnglishPassthrough));
    assert!(res.handled);
    assert_eq!(res.committed_text, Some("আমি".to_string()));
    assert!(!res.state.is_composing);
    assert_eq!(res.state.preedit, "");
    assert_eq!(res.state.mode, InputMode::EnglishPassthrough);
}

// ============================================================================
// 6. C-ABI Behavioral Parity & Memory Safety Tests
// ============================================================================

#[test]
fn test_c_abi_process_event_parity_and_cleanup() {
    unsafe {
        let handle = borno_engine_new();
        assert!(!handle.is_null());

        let mut result = BornoProcessResultC {
            state: BornoStateC {
                preedit: ptr::null_mut(),
                raw_buffer: ptr::null_mut(),
                cursor_pos: 0,
                candidates_count: 0,
                candidates: ptr::null_mut(),
                selected_candidate_index: -1,
                is_composing: false,
                mode: 0,
            },
            committed_text: ptr::null_mut(),
            cancelled: false,
            handled: false,
        };

        // 1. InputChar('k')
        let ev_k = BornoEventC {
            event_type: BORNO_EVENT_INPUT_CHAR,
            char_codepoint: 'k' as u32,
            string_payload: ptr::null(),
            index_payload: 0,
            mode_payload: 0,
        };
        let rc = borno_engine_process_event(handle, &ev_k, &mut result);
        assert_eq!(rc, BORNO_OK);
        assert!(result.handled);
        assert!(result.state.is_composing);
        let preedit = CStr::from_ptr(result.state.preedit).to_str().unwrap();
        assert_eq!(preedit, "ক");
        assert!(result.committed_text.is_null());
        borno_process_result_free_members(&mut result);

        // 2. InputChar('a')
        let ev_a = BornoEventC {
            event_type: BORNO_EVENT_INPUT_CHAR,
            char_codepoint: 'a' as u32,
            string_payload: ptr::null(),
            index_payload: 0,
            mode_payload: 0,
        };
        let rc = borno_engine_process_event(handle, &ev_a, &mut result);
        assert_eq!(rc, BORNO_OK);
        let preedit2 = CStr::from_ptr(result.state.preedit).to_str().unwrap();
        assert_eq!(preedit2, "কা");
        assert!(result.state.candidates_count >= 1);
        borno_process_result_free_members(&mut result);

        // 3. NextCandidate
        let ev_next = BornoEventC {
            event_type: BORNO_EVENT_NEXT_CANDIDATE,
            char_codepoint: 0,
            string_payload: ptr::null(),
            index_payload: 0,
            mode_payload: 0,
        };
        let rc = borno_engine_process_event(handle, &ev_next, &mut result);
        assert_eq!(rc, BORNO_OK);
        assert_eq!(result.state.selected_candidate_index, 0);
        assert!(result.committed_text.is_null());
        borno_process_result_free_members(&mut result);

        // 4. SelectCandidate(1) -> commits index 1 ("ক্যা")
        let ev_sel = BornoEventC {
            event_type: BORNO_EVENT_SELECT_CANDIDATE,
            char_codepoint: 0,
            string_payload: ptr::null(),
            index_payload: 1,
            mode_payload: 0,
        };
        let rc = borno_engine_process_event(handle, &ev_sel, &mut result);
        assert_eq!(rc, BORNO_OK);
        assert!(result.handled);
        assert!(!result.committed_text.is_null());
        let committed = CStr::from_ptr(result.committed_text).to_str().unwrap();
        assert_eq!(committed, "ক্যা");
        assert!(!result.state.is_composing);
        borno_process_result_free_members(&mut result);

        borno_engine_free(handle);
    }
}
