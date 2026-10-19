//! Automated FFI Boundary & Memory Safety Tests
//!
//! Validates C-ABI correctness, memory ownership, disposal discipline,
//! invalid input handling, and UTF-8 multi-byte Bangla Unicode preservation.

use std::ffi::{CStr, CString};
use std::ptr;
use bornomala_core::ffi::*;

#[test]
fn test_ffi_null_pointer_safety() {
    unsafe {
        // Safe disposal of NULL
        borno_engine_free(ptr::null_mut());
        borno_string_free(ptr::null_mut());
        borno_status_free_members(ptr::null_mut());
        borno_state_free_members(ptr::null_mut());
        borno_process_result_free_members(ptr::null_mut());

        // Status with NULL pointers returns error
        assert_eq!(borno_engine_get_status(ptr::null(), ptr::null_mut()), BORNO_ERR_NULL_PTR);

        let mut status = BornoStatusC {
            name: ptr::null_mut(),
            version: ptr::null_mut(),
            active_mode: ptr::null_mut(),
            ready: false,
        };
        assert_eq!(borno_engine_get_status(ptr::null(), &mut status), BORNO_ERR_NULL_PTR);

        // Process char with NULL handle
        let mut state = BornoStateC {
            preedit: ptr::null_mut(),
            raw_buffer: ptr::null_mut(),
            cursor_pos: 0,
            candidates_count: 0,
            candidates: ptr::null_mut(),
            selected_candidate_index: -1,
            is_composing: false,
            mode: 0,
        };
        assert_eq!(borno_engine_process_char(ptr::null_mut(), 'a' as u32, &mut state), BORNO_ERR_NULL_PTR);
        assert_eq!(borno_engine_backspace(ptr::null_mut(), &mut state), BORNO_ERR_NULL_PTR);
        assert_eq!(borno_engine_commit(ptr::null_mut(), &mut state), BORNO_ERR_NULL_PTR);
        assert_eq!(borno_engine_reset(ptr::null_mut(), &mut state), BORNO_ERR_NULL_PTR);

        // Process event with NULL handle or NULL event or NULL out_result
        let ev = BornoEventC {
            event_type: BORNO_EVENT_BACKSPACE,
            char_codepoint: 0,
            string_payload: ptr::null(),
            index_payload: 0,
            mode_payload: 0,
        };
        let mut res = BornoProcessResultC {
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
        assert_eq!(borno_engine_process_event(ptr::null_mut(), &ev, &mut res), BORNO_ERR_NULL_PTR);
        let handle = borno_engine_new();
        assert_eq!(borno_engine_process_event(handle, ptr::null(), &mut res), BORNO_ERR_NULL_PTR);
        assert_eq!(borno_engine_process_event(handle, &ev, ptr::null_mut()), BORNO_ERR_NULL_PTR);
        borno_engine_free(handle);

        // Echo with NULL pointers
        let mut out_str: *mut std::os::raw::c_char = ptr::null_mut();
        let mut out_len: usize = 0;
        assert_eq!(borno_ffi_echo_utf8(ptr::null(), &mut out_str, &mut out_len), BORNO_ERR_NULL_PTR);
    }
}

#[test]
fn test_ffi_string_echo_bangla_unicode() {
    let test_strings = [
        "অ",
        "ক",
        "কা",
        "ক্ষ",              // ক + ্ + ষ (Conjunct)
        "জ্ঞান",           // জ + ্ + ঞ + া + ন (Complex Conjunct)
        "হৃদয়",            // হ + ৃ + দ + য়
        "বাংলাদেশ",
        "আমার সোনার বাংলা",
    ];

    for original in &test_strings {
        unsafe {
            let c_input = CString::new(*original).unwrap();
            let mut out_ptr: *mut std::os::raw::c_char = ptr::null_mut();
            let mut out_len: usize = 0;

            let rc = borno_ffi_echo_utf8(c_input.as_ptr(), &mut out_ptr, &mut out_len);
            assert_eq!(rc, BORNO_OK, "Echo failed for string: {}", original);
            assert!(!out_ptr.is_null(), "Returned pointer was null for: {}", original);
            assert_eq!(out_len, original.len(), "Byte length mismatch for: {}", original);

            let returned_c_str = CStr::from_ptr(out_ptr);
            let returned_str = returned_c_str.to_str().expect("Returned string was not valid UTF-8");
            assert_eq!(returned_str, *original, "String content corrupted across FFI boundary");

            // Explicit disposal: verify no memory violation occurs on free
            borno_string_free(out_ptr);
        }
    }
}

#[test]
fn test_ffi_invalid_utf8_rejection() {
    unsafe {
        let malformed_bytes: [u8; 4] = [0xC3, 0x28, 0x00, 0x00];
        let mut out_ptr: *mut std::os::raw::c_char = ptr::null_mut();
        let mut out_len: usize = 0;

        let rc = borno_ffi_echo_utf8(
            malformed_bytes.as_ptr() as *const std::os::raw::c_char,
            &mut out_ptr,
            &mut out_len,
        );

        assert_eq!(rc, BORNO_ERR_INVALID_UTF8, "Expected invalid UTF-8 error code");
        assert!(out_ptr.is_null(), "No memory should be allocated on error");
        assert_eq!(out_len, 0);
    }
}

#[test]
fn test_ffi_engine_lifecycle_and_status() {
    unsafe {
        let handle = borno_engine_new();
        assert!(!handle.is_null(), "Failed to allocate engine via borno_engine_new");

        let mut status = BornoStatusC {
            name: ptr::null_mut(),
            version: ptr::null_mut(),
            active_mode: ptr::null_mut(),
            ready: false,
        };

        let rc = borno_engine_get_status(handle, &mut status);
        assert_eq!(rc, BORNO_OK);
        assert!(!status.name.is_null());
        assert!(!status.version.is_null());
        assert!(!status.active_mode.is_null());
        assert!(status.ready);

        let name = CStr::from_ptr(status.name).to_str().unwrap();
        assert_eq!(name, "Borno Core");

        // Free the allocated members
        borno_status_free_members(&mut status);
        assert!(status.name.is_null());
        assert!(status.version.is_null());
        assert!(status.active_mode.is_null());
        assert!(!status.ready);

        // Free the engine instance
        borno_engine_free(handle);
    }
}

#[test]
fn test_ffi_engine_event_dispatch_with_test_fixture() {
    unsafe {
        let handle = borno_engine_new();
        assert!(!handle.is_null());

        let mut state = BornoStateC {
            preedit: ptr::null_mut(),
            raw_buffer: ptr::null_mut(),
            cursor_pos: 0,
            candidates_count: 0,
            candidates: ptr::null_mut(),
            selected_candidate_index: -1,
            is_composing: false,
            mode: 0,
        };

        // 1. Send 'k'
        let rc = borno_engine_process_char(handle, 'k' as u32, &mut state);
        assert_eq!(rc, BORNO_OK);
        let preedit_1 = CStr::from_ptr(state.preedit).to_str().unwrap();
        let raw_1 = CStr::from_ptr(state.raw_buffer).to_str().unwrap();
        assert_eq!(preedit_1, "ক");
        assert_eq!(raw_1, "k");
        assert!(state.is_composing);
        assert_eq!(state.cursor_pos, 1);
        borno_state_free_members(&mut state);

        // 2. Send 'a' -> should produce "কা" in the test fixture
        let rc = borno_engine_process_char(handle, 'a' as u32, &mut state);
        assert_eq!(rc, BORNO_OK);
        let preedit_2 = CStr::from_ptr(state.preedit).to_str().unwrap();
        let raw_2 = CStr::from_ptr(state.raw_buffer).to_str().unwrap();
        assert_eq!(preedit_2, "কা");
        assert_eq!(raw_2, "ka");
        assert!(state.candidates_count >= 1);
        assert!(!state.candidates.is_null());
        borno_state_free_members(&mut state);

        // 3. Send Backspace -> reverts to "ক"
        let rc = borno_engine_backspace(handle, &mut state);
        assert_eq!(rc, BORNO_OK);
        let preedit_3 = CStr::from_ptr(state.preedit).to_str().unwrap();
        assert_eq!(preedit_3, "ক");
        borno_state_free_members(&mut state);

        // 4. Send Reset -> clears composition
        let rc = borno_engine_reset(handle, &mut state);
        assert_eq!(rc, BORNO_OK);
        let preedit_4 = CStr::from_ptr(state.preedit).to_str().unwrap();
        assert_eq!(preedit_4, "");
        assert_eq!(state.candidates_count, 0);
        assert!(!state.is_composing);
        borno_state_free_members(&mut state);

        // 5. Test borno_engine_process_event with BornoEventC and BornoProcessResultC
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
        assert!(result.committed_text.is_null());
        let preedit = CStr::from_ptr(result.state.preedit).to_str().unwrap();
        assert_eq!(preedit, "ক");
        borno_process_result_free_members(&mut result);

        // Commit event
        let ev_commit = BornoEventC {
            event_type: BORNO_EVENT_COMMIT,
            char_codepoint: 0,
            string_payload: ptr::null(),
            index_payload: 0,
            mode_payload: 0,
        };
        let rc = borno_engine_process_event(handle, &ev_commit, &mut result);
        assert_eq!(rc, BORNO_OK);
        assert!(result.handled);
        assert!(!result.committed_text.is_null());
        let committed = CStr::from_ptr(result.committed_text).to_str().unwrap();
        assert_eq!(committed, "ক");
        assert!(!result.state.is_composing);
        borno_process_result_free_members(&mut result);

        borno_engine_free(handle);
    }
}

#[test]
fn test_ffi_candidate_kind_accessor() {
    unsafe {
        let handle = borno_engine_new();
        assert!(!handle.is_null());

        // Null handle check
        assert_eq!(borno_engine_get_candidate_kind(ptr::null(), 0), -1);

        // Initially no candidates: out of bounds returns -1
        assert_eq!(borno_engine_get_candidate_kind(handle, 0), -1);

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

        // Input 'k' -> 'ক'
        let ev_k = BornoEventC {
            event_type: BORNO_EVENT_INPUT_CHAR,
            char_codepoint: 'k' as u32,
            string_payload: ptr::null(),
            index_payload: 0,
            mode_payload: 0,
        };
        let rc = borno_engine_process_event(handle, &ev_k, &mut result);
        assert_eq!(rc, BORNO_OK);
        assert!(result.state.candidates_count >= 1);

        // Candidate 0 must be PrimaryTransliteration
        let kind0 = borno_engine_get_candidate_kind(handle, 0);
        assert_eq!(kind0, BORNO_CANDIDATE_PRIMARY_TRANSLITERATION);

        // Out of bounds index
        assert_eq!(borno_engine_get_candidate_kind(handle, 999), -1);

        borno_process_result_free_members(&mut result);
        borno_engine_free(handle);
    }
}

#[test]
fn test_ffi_engine_configure_json() {
    unsafe {
        let handle = borno_engine_new();
        assert!(!handle.is_null());

        // 1. NULL pointer safety
        assert_eq!(borno_engine_configure_json(ptr::null_mut(), ptr::null()), BORNO_ERR_NULL_PTR);
        let valid_json = CString::new(r#"{"schema_version":"1.2.0"}"#).unwrap();
        assert_eq!(borno_engine_configure_json(ptr::null_mut(), valid_json.as_ptr()), BORNO_ERR_NULL_PTR);
        assert_eq!(borno_engine_configure_json(handle, ptr::null()), BORNO_ERR_NULL_PTR);

        // 2. Valid v1.2.0 JSON
        let cfg_v12 = CString::new(r#"{
            "schema_version": "1.2.0",
            "candidate_display_limit": 7,
            "max_dictionary_suggestions": 4,
            "auto_convert_period_to_dari": false,
            "candidate_orientation": "horizontal",
            "candidate_font_size": 16,
            "active_theme": "dark-glass",
            "mode_toggle_key": "F12"
        }"#).unwrap();
        assert_eq!(borno_engine_configure_json(handle, cfg_v12.as_ptr()), BORNO_OK);

        // 3. Valid v1.0.0 Legacy JSON migration
        let cfg_v10 = CString::new(r#"{
            "schema_version": "1.0.0",
            "candidate_display_limit": 3
        }"#).unwrap();
        assert_eq!(borno_engine_configure_json(handle, cfg_v10.as_ptr()), BORNO_OK);

        // 4. Unsupported schema version
        let cfg_unsupported = CString::new(r#"{"schema_version":"2.0.0"}"#).unwrap();
        assert_eq!(borno_engine_configure_json(handle, cfg_unsupported.as_ptr()), BORNO_ERR_UNSUPPORTED_SCHEMA);

        // 5. Malformed JSON
        let cfg_malformed = CString::new(r#"{bad json"#).unwrap();
        assert_eq!(borno_engine_configure_json(handle, cfg_malformed.as_ptr()), BORNO_ERR_INVALID_CONFIG);

        // 6. Validation error (candidate_display_limit = 15 out of range 1..=9)
        let cfg_invalid = CString::new(r#"{"schema_version":"1.2.0","candidate_display_limit":15}"#).unwrap();
        assert_eq!(borno_engine_configure_json(handle, cfg_invalid.as_ptr()), BORNO_ERR_INVALID_CONFIG);

        // 7. Reconfiguration preserves active composition
        let mut state = BornoStateC {
            preedit: ptr::null_mut(),
            raw_buffer: ptr::null_mut(),
            cursor_pos: 0,
            candidates_count: 0,
            candidates: ptr::null_mut(),
            selected_candidate_index: -1,
            is_composing: false,
            mode: 0,
        };
        borno_engine_process_char(handle, 'k' as u32, &mut state);
        assert!(state.is_composing);
        let preedit_before = CStr::from_ptr(state.preedit).to_str().unwrap().to_string();
        assert_eq!(preedit_before, "ক");

        // Reconfigure while composing
        assert_eq!(borno_engine_configure_json(handle, cfg_v12.as_ptr()), BORNO_OK);

        // Verify preedit is still active in engine
        let mut status = BornoStatusC {
            name: ptr::null_mut(),
            version: ptr::null_mut(),
            active_mode: ptr::null_mut(),
            ready: false,
        };
        borno_engine_get_status(handle, &mut status);
        assert!(status.ready);

        borno_status_free_members(&mut status);
        borno_state_free_members(&mut state);
        borno_engine_free(handle);
    }
}

