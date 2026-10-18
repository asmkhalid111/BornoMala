//! # C-Compatible FFI Interface for Borno Core
//!
//! Provides a stable, C-ABI (`extern "C"`) boundary for native platform adapters
//! (Windows TSF, Linux IBus/Fcitx, macOS InputMethodKit).
//!
//! ## Memory Ownership Rules:
//! - Pointers passed into functions as `*const c_char` or `*const BornoEventC` are borrowed;
//!   the caller retains ownership.
//! - Pointers returned as `*mut c_char` or within structs are allocated by the Rust allocator.
//! - The caller MUST release returned structures using the respective struct destructor helpers
//!   ([`borno_status_free_members`], [`borno_state_free_members`], [`borno_process_result_free_members`])
//!   or [`borno_string_free`].
//! - The caller must NEVER use standard C `free()` or C++ `delete` on pointers allocated by Rust.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use crate::{BornoEngine, EngineEvent, EngineState, EngineStatus, InputMode, ProcessResult};

// Status and error codes
pub const BORNO_OK: i32 = 0;
pub const BORNO_ERR_NULL_PTR: i32 = -1;
pub const BORNO_ERR_INVALID_UTF8: i32 = -2;
pub const BORNO_ERR_PANIC: i32 = -3;
pub const BORNO_ERR_INVALID_ARGUMENT: i32 = -4;

// Input Mode constants
pub const BORNO_MODE_PHONETIC: i32 = 0;
pub const BORNO_MODE_PASSTHROUGH: i32 = 1;
pub const BORNO_MODE_FIXED_NATIONAL: i32 = 2;
pub const BORNO_MODE_FIXED_PROBHAT: i32 = 3;
pub const BORNO_MODE_FIXED_BIJOY: i32 = 4;

// Event type constants
pub const BORNO_EVENT_INPUT_CHAR: i32 = 0;
pub const BORNO_EVENT_INPUT_STRING: i32 = 1;
pub const BORNO_EVENT_BACKSPACE: i32 = 2;
pub const BORNO_EVENT_DELETE: i32 = 3;
pub const BORNO_EVENT_COMMIT: i32 = 4;
pub const BORNO_EVENT_CANCEL: i32 = 5;
pub const BORNO_EVENT_RESET: i32 = 6;
pub const BORNO_EVENT_NEXT_CANDIDATE: i32 = 7;
pub const BORNO_EVENT_PREV_CANDIDATE: i32 = 8;
pub const BORNO_EVENT_HIGHLIGHT_CANDIDATE: i32 = 9;
pub const BORNO_EVENT_SELECT_CANDIDATE: i32 = 10;
pub const BORNO_EVENT_SET_CURSOR: i32 = 11;
pub const BORNO_EVENT_MOVE_CURSOR_LEFT: i32 = 12;
pub const BORNO_EVENT_MOVE_CURSOR_RIGHT: i32 = 13;
pub const BORNO_EVENT_SWITCH_MODE: i32 = 14;

// Candidate Kind constants
pub const BORNO_CANDIDATE_PRIMARY_TRANSLITERATION: i32 = 0;
pub const BORNO_CANDIDATE_PHONETIC_ALTERNATIVE: i32 = 1;
pub const BORNO_CANDIDATE_EXACT_LEXICON_MATCH: i32 = 2;
pub const BORNO_CANDIDATE_PREFIX_COMPLETION: i32 = 3;
pub const BORNO_CANDIDATE_SPELLING_CORRECTION: i32 = 4;

/// Opaque wrapper around [`BornoEngine`].
pub struct BornoEngineHandle {
    inner: BornoEngine,
}

/// Normalized event representation for C-ABI.
#[repr(C)]
pub struct BornoEventC {
    pub event_type: i32,
    pub char_codepoint: u32,
    pub string_payload: *const c_char,
    pub index_payload: usize,
    pub mode_payload: i32,
}

/// C-compatible representation of [`EngineStatus`].
#[repr(C)]
pub struct BornoStatusC {
    pub name: *mut c_char,
    pub version: *mut c_char,
    pub active_mode: *mut c_char,
    pub ready: bool,
}

/// C-compatible representation of persistent [`EngineState`].
#[repr(C)]
pub struct BornoStateC {
    /// Active preedit / composition string in Bangla (null-terminated UTF-8)
    pub preedit: *mut c_char,
    /// Raw uncommitted Latin keystroke buffer (null-terminated UTF-8)
    pub raw_buffer: *mut c_char,
    /// Cursor position in Unicode codepoints
    pub cursor_pos: usize,
    /// Number of candidates in the candidates array
    pub candidates_count: usize,
    /// Array of pointers to null-terminated UTF-8 strings
    pub candidates: *mut *mut c_char,
    /// Currently highlighted candidate index (-1 if None)
    pub selected_candidate_index: i32,
    /// True if active composition is in progress
    pub is_composing: bool,
    /// Active input mode (0: Phonetic, 1: EnglishPassthrough)
    pub mode: i32,
}

/// C-compatible representation of [`ProcessResult`].
#[repr(C)]
pub struct BornoProcessResultC {
    /// Persistent state snapshot after event processing
    pub state: BornoStateC,
    /// Text committed during this step (null-terminated UTF-8, or NULL)
    pub committed_text: *mut c_char,
    /// True if composition was cancelled during this step
    pub cancelled: bool,
    /// True if engine consumed the event
    pub handled: bool,
}

// ============================================================================
// Memory Management Helpers
// ============================================================================

/// Free a string allocated by Borno Core.
///
/// Safety: `ptr` must either be NULL (no-op) or a valid pointer previously returned
/// by this library.
#[no_mangle]
pub unsafe extern "C" fn borno_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Free the string members inside a [`BornoStatusC`] struct.
/// Does not free the struct container itself.
#[no_mangle]
pub unsafe extern "C" fn borno_status_free_members(status: *mut BornoStatusC) {
    if status.is_null() {
        return;
    }
    let s = &mut *status;
    borno_string_free(s.name);
    s.name = ptr::null_mut();
    borno_string_free(s.version);
    s.version = ptr::null_mut();
    borno_string_free(s.active_mode);
    s.active_mode = ptr::null_mut();
    s.ready = false;
}

/// Free the heap-allocated members inside a [`BornoStateC`] struct.
/// Does not free the struct container itself.
#[no_mangle]
pub unsafe extern "C" fn borno_state_free_members(state: *mut BornoStateC) {
    if state.is_null() {
        return;
    }
    let s = &mut *state;
    borno_string_free(s.preedit);
    s.preedit = ptr::null_mut();
    borno_string_free(s.raw_buffer);
    s.raw_buffer = ptr::null_mut();

    if !s.candidates.is_null() {
        let slice = std::slice::from_raw_parts_mut(s.candidates, s.candidates_count);
        for item in slice.iter_mut() {
            borno_string_free(*item);
            *item = ptr::null_mut();
        }
        // Reconstruct and drop the outer Vec
        drop(Vec::from_raw_parts(s.candidates, s.candidates_count, s.candidates_count));
        s.candidates = ptr::null_mut();
        s.candidates_count = 0;
    }
    s.cursor_pos = 0;
    s.selected_candidate_index = -1;
    s.is_composing = false;
    s.mode = 0;
}

/// Free the heap-allocated members inside a [`BornoProcessResultC`] struct.
/// Does not free the struct container itself.
#[no_mangle]
pub unsafe extern "C" fn borno_process_result_free_members(result: *mut BornoProcessResultC) {
    if result.is_null() {
        return;
    }
    let r = &mut *result;
    borno_state_free_members(&mut r.state);
    borno_string_free(r.committed_text);
    r.committed_text = ptr::null_mut();
    r.cancelled = false;
    r.handled = false;
}

// ============================================================================
// Engine Lifecycle
// ============================================================================

/// Allocate and initialize a new [`BornoEngine`] instance.
///
/// Returns NULL on allocation failure.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_new() -> *mut BornoEngineHandle {
    let result = std::panic::catch_unwind(|| {
        Box::into_raw(Box::new(BornoEngineHandle {
            inner: BornoEngine::new(),
        }))
    });
    result.unwrap_or(ptr::null_mut())
}

/// Destroy an engine instance previously created by [`borno_engine_new`].
///
/// Passing NULL is safe and performs no action.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_free(handle: *mut BornoEngineHandle) {
    if !handle.is_null() {
        let _ = std::panic::catch_unwind(|| {
            drop(Box::from_raw(handle));
        });
    }
}

// ============================================================================
// Engine State & Events
// ============================================================================

/// Retrieve the engine's current operational status.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_get_status(
    handle: *const BornoEngineHandle,
    out_status: *mut BornoStatusC,
) -> i32 {
    if handle.is_null() || out_status.is_null() {
        return BORNO_ERR_NULL_PTR;
    }

    let result = std::panic::catch_unwind(|| {
        let engine = &(*handle).inner;
        let status = engine.status();
        fill_status_c(&status, &mut *out_status);
        BORNO_OK
    });

    result.unwrap_or(BORNO_ERR_PANIC)
}

/// Process a normalized engine event and retrieve full process result.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_process_event(
    handle: *mut BornoEngineHandle,
    event: *const BornoEventC,
    out_result: *mut BornoProcessResultC,
) -> i32 {
    if handle.is_null() || event.is_null() || out_result.is_result_null() {
        return BORNO_ERR_NULL_PTR;
    }

    let ev = &*event;
    let rust_event = match ev.event_type {
        BORNO_EVENT_INPUT_CHAR => {
            match char::from_u32(ev.char_codepoint) {
                Some(c) => EngineEvent::InputChar(c),
                None => return BORNO_ERR_INVALID_ARGUMENT,
            }
        }
        BORNO_EVENT_INPUT_STRING => {
            if ev.string_payload.is_null() {
                return BORNO_ERR_NULL_PTR;
            }
            let c_str = CStr::from_ptr(ev.string_payload);
            match c_str.to_str() {
                Ok(s) => EngineEvent::InputString(s.to_string()),
                Err(_) => return BORNO_ERR_INVALID_UTF8,
            }
        }
        BORNO_EVENT_BACKSPACE => EngineEvent::Backspace,
        BORNO_EVENT_DELETE => EngineEvent::Delete,
        BORNO_EVENT_COMMIT => EngineEvent::Commit,
        BORNO_EVENT_CANCEL => EngineEvent::Cancel,
        BORNO_EVENT_RESET => EngineEvent::Reset,
        BORNO_EVENT_NEXT_CANDIDATE => EngineEvent::NextCandidate,
        BORNO_EVENT_PREV_CANDIDATE => EngineEvent::PrevCandidate,
        BORNO_EVENT_HIGHLIGHT_CANDIDATE => EngineEvent::HighlightCandidate(ev.index_payload),
        BORNO_EVENT_SELECT_CANDIDATE => EngineEvent::SelectCandidate(ev.index_payload),
        BORNO_EVENT_SET_CURSOR => EngineEvent::SetCursor(ev.index_payload),
        BORNO_EVENT_MOVE_CURSOR_LEFT => EngineEvent::MoveCursorLeft,
        BORNO_EVENT_MOVE_CURSOR_RIGHT => EngineEvent::MoveCursorRight,
        BORNO_EVENT_SWITCH_MODE => {
            let mode = match ev.mode_payload {
                BORNO_MODE_PHONETIC => InputMode::Phonetic,
                BORNO_MODE_PASSTHROUGH => InputMode::EnglishPassthrough,
                BORNO_MODE_FIXED_NATIONAL => InputMode::Fixed(crate::layout::FixedLayoutId::National),
                BORNO_MODE_FIXED_PROBHAT => InputMode::Fixed(crate::layout::FixedLayoutId::Probhat),
                BORNO_MODE_FIXED_BIJOY => InputMode::Fixed(crate::layout::FixedLayoutId::Bijoy),
                _ => return BORNO_ERR_INVALID_ARGUMENT,
            };
            EngineEvent::SwitchMode(mode)
        }
        _ => return BORNO_ERR_INVALID_ARGUMENT,
    };

    let result = std::panic::catch_unwind(|| {
        let engine = &mut (*handle).inner;
        let proc_res = engine.process_event(rust_event);
        fill_process_result_c(&proc_res, &mut *out_result);
        BORNO_OK
    });

    result.unwrap_or(BORNO_ERR_PANIC)
}

/// Convenience helper: Process a single Unicode character input (UTF-32 codepoint).
#[no_mangle]
pub unsafe extern "C" fn borno_engine_process_char(
    handle: *mut BornoEngineHandle,
    codepoint: u32,
    out_state: *mut BornoStateC,
) -> i32 {
    if handle.is_null() || out_state.is_null() {
        return BORNO_ERR_NULL_PTR;
    }

    let ch = match char::from_u32(codepoint) {
        Some(c) => c,
        None => return BORNO_ERR_INVALID_ARGUMENT,
    };

    let result = std::panic::catch_unwind(|| {
        let engine = &mut (*handle).inner;
        let proc_res = engine.process_event(EngineEvent::InputChar(ch));
        fill_state_c(&proc_res.state, &mut *out_state);
        BORNO_OK
    });

    result.unwrap_or(BORNO_ERR_PANIC)
}

/// Convenience helper: Process a Backspace event.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_backspace(
    handle: *mut BornoEngineHandle,
    out_state: *mut BornoStateC,
) -> i32 {
    if handle.is_null() || out_state.is_null() {
        return BORNO_ERR_NULL_PTR;
    }

    let result = std::panic::catch_unwind(|| {
        let engine = &mut (*handle).inner;
        let proc_res = engine.process_event(EngineEvent::Backspace);
        fill_state_c(&proc_res.state, &mut *out_state);
        BORNO_OK
    });

    result.unwrap_or(BORNO_ERR_PANIC)
}

/// Convenience helper: Process an explicit Commit event.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_commit(
    handle: *mut BornoEngineHandle,
    out_state: *mut BornoStateC,
) -> i32 {
    if handle.is_null() || out_state.is_null() {
        return BORNO_ERR_NULL_PTR;
    }

    let result = std::panic::catch_unwind(|| {
        let engine = &mut (*handle).inner;
        let proc_res = engine.process_event(EngineEvent::Commit);
        fill_state_c(&proc_res.state, &mut *out_state);
        BORNO_OK
    });

    result.unwrap_or(BORNO_ERR_PANIC)
}

/// Convenience helper: Reset the engine composition state.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_reset(
    handle: *mut BornoEngineHandle,
    out_state: *mut BornoStateC,
) -> i32 {
    if handle.is_null() || out_state.is_null() {
        return BORNO_ERR_NULL_PTR;
    }

    let result = std::panic::catch_unwind(|| {
        let engine = &mut (*handle).inner;
        let proc_res = engine.reset();
        fill_state_c(&proc_res.state, &mut *out_state);
        BORNO_OK
    });

    result.unwrap_or(BORNO_ERR_PANIC)
}

/// Convenience helper: Process a physical hardware key event directly via C-ABI.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_process_key(
    handle: *mut BornoEngineHandle,
    key_code: u32,
    modifiers: u32,
    out_result: *mut BornoProcessResultC,
) -> i32 {
    if handle.is_null() || out_result.is_null() {
        return BORNO_ERR_NULL_PTR;
    }

    let Some(key) = crate::layout::PhysicalKey::from_u32(key_code) else {
        return BORNO_ERR_INVALID_ARGUMENT;
    };

    let mods = crate::layout::Modifiers::from_bits(modifiers);
    let event = EngineEvent::InputKey { key, modifiers: mods };

    let result = std::panic::catch_unwind(|| {
        let engine = &mut (*handle).inner;
        let proc_res = engine.process_event(event);
        fill_process_result_c(&proc_res, &mut *out_result);
        BORNO_OK
    });

    result.unwrap_or(BORNO_ERR_PANIC)
}

/// Query the candidate origin kind at a specific index from the active engine state.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_get_candidate_kind(
    handle: *const BornoEngineHandle,
    index: usize,
) -> i32 {
    if handle.is_null() {
        return -1;
    }
    let engine = &(*handle).inner;
    let state = engine.state();
    if index >= state.candidate_details.len() {
        return -1;
    }
    match state.candidate_details[index].kind {
        crate::CandidateKind::PrimaryTransliteration => BORNO_CANDIDATE_PRIMARY_TRANSLITERATION,
        crate::CandidateKind::PhoneticAlternative => BORNO_CANDIDATE_PHONETIC_ALTERNATIVE,
        crate::CandidateKind::ExactLexiconMatch => BORNO_CANDIDATE_EXACT_LEXICON_MATCH,
        crate::CandidateKind::PrefixCompletion => BORNO_CANDIDATE_PREFIX_COMPLETION,
        crate::CandidateKind::SpellingCorrection => BORNO_CANDIDATE_SPELLING_CORRECTION,
    }
}

// ============================================================================
// Direct UTF-8 Unicode FFI Verification Utility
// ============================================================================

/// Direct UTF-8 round-trip verification function.
#[no_mangle]
pub unsafe extern "C" fn borno_ffi_echo_utf8(
    input: *const c_char,
    out_str: *mut *mut c_char,
    out_len: *mut usize,
) -> i32 {
    if input.is_null() || out_str.is_null() || out_len.is_null() {
        return BORNO_ERR_NULL_PTR;
    }

    let c_str = CStr::from_ptr(input);
    let rust_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return BORNO_ERR_INVALID_UTF8,
    };

    let len = rust_str.len();
    match CString::new(rust_str) {
        Ok(cs) => {
            *out_str = cs.into_raw();
            *out_len = len;
            BORNO_OK
        }
        Err(_) => BORNO_ERR_INVALID_ARGUMENT,
    }
}

// ============================================================================
// Internal Conversion Helpers
// ============================================================================

trait IsResultNull {
    fn is_result_null(&self) -> bool;
}

impl IsResultNull for *mut BornoProcessResultC {
    fn is_result_null(&self) -> bool {
        self.is_null()
    }
}

fn string_to_c(s: &str) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

fn fill_status_c(status: &EngineStatus, out: &mut BornoStatusC) {
    out.name = string_to_c(&status.name);
    out.version = string_to_c(&status.version);
    out.active_mode = string_to_c(&status.active_mode);
    out.ready = status.ready;
}

fn fill_state_c(state: &EngineState, out: &mut BornoStateC) {
    out.preedit = string_to_c(&state.preedit);
    out.raw_buffer = string_to_c(&state.raw_buffer);
    out.cursor_pos = state.cursor_pos;
    out.selected_candidate_index = match state.selected_candidate_index {
        Some(idx) => idx as i32,
        None => -1,
    };
    out.is_composing = state.is_composing;
    out.mode = match state.mode {
        InputMode::Phonetic => BORNO_MODE_PHONETIC,
        InputMode::EnglishPassthrough => BORNO_MODE_PASSTHROUGH,
        InputMode::Fixed(crate::layout::FixedLayoutId::National) => BORNO_MODE_FIXED_NATIONAL,
        InputMode::Fixed(crate::layout::FixedLayoutId::Probhat) => BORNO_MODE_FIXED_PROBHAT,
        InputMode::Fixed(crate::layout::FixedLayoutId::Bijoy) => BORNO_MODE_FIXED_BIJOY,
    };

    let count = state.candidates.len();
    if count == 0 {
        out.candidates_count = 0;
        out.candidates = ptr::null_mut();
    } else {
        let mut ptrs: Vec<*mut c_char> = state.candidates.iter().map(|s| string_to_c(s)).collect();
        out.candidates_count = count;
        out.candidates = ptrs.as_mut_ptr();
        std::mem::forget(ptrs);
    }
}

fn fill_process_result_c(res: &ProcessResult, out: &mut BornoProcessResultC) {
    fill_state_c(&res.state, &mut out.state);
    out.committed_text = match &res.committed_text {
        Some(c) => string_to_c(c),
        None => ptr::null_mut(),
    };
    out.cancelled = res.cancelled;
    out.handled = res.handled;
}
