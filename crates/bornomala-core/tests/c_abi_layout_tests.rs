//! C-ABI Layout Parity Tests
//!
//! Validates byte-for-byte size and member offset parity between C structs
//! (from `bornomala.h`) and Rust `#[repr(C)]` structs in `bornomala_core::ffi`.

use bornomala_core::ffi::{BornoEventC, BornoProcessResultC, BornoStateC, BornoStatusC};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_c_abi_layout_parity() {
    // 1. Rust Compile-Time Target Attribute Inspection
    let target_arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        std::env::consts::ARCH
    };

    let target_os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        std::env::consts::OS
    };

    let target_env = if cfg!(target_env = "gnu") {
        "gnu"
    } else if cfg!(target_env = "msvc") {
        "msvc"
    } else if cfg!(target_env = "musl") {
        "musl"
    } else {
        ""
    };

    let pointer_width = std::mem::size_of::<*const ()>() * 8;

    // Recognize reliable target combinations
    let rust_target = if target_arch == "x86_64" && target_os == "windows" && target_env == "gnu" {
        "x86_64-pc-windows-gnu"
    } else if target_arch == "x86_64" && target_os == "linux" && target_env == "gnu" {
        "x86_64-unknown-linux-gnu"
    } else if target_arch == "x86_64" && target_os == "windows" && target_env == "msvc" {
        "x86_64-pc-windows-msvc"
    } else if target_arch == "aarch64" && target_os == "linux" && target_env == "gnu" {
        "aarch64-unknown-linux-gnu"
    } else if target_arch == "aarch64" && target_os == "macos" {
        "aarch64-apple-darwin"
    } else {
        eprintln!(
            "Blocked / Skipped (Cannot determine reliable Rust target identity from compile-time attributes: arch={}, os={}, env={}, ptr_width={})",
            target_arch, target_os, target_env, pointer_width
        );
        return;
    };

    // 2. Select C compiler
    let cc = if let Ok(c) = std::env::var("CC") {
        c
    } else if let Ok(tc) = std::env::var("TARGET_CC") {
        tc
    } else {
        // Probe for standard compilers
        if Command::new("gcc").arg("--version").output().is_ok() {
            "gcc".to_string()
        } else if Command::new("clang").arg("--version").output().is_ok() {
            "clang".to_string()
        } else {
            eprintln!("Blocked / Skipped (Missing C compiler: CC not set and gcc/clang not found on PATH)");
            return;
        }
    };

    // 3. Inspect C compiler target via $CC -dumpmachine
    let dumpmachine_output = match Command::new(&cc).arg("-dumpmachine").output() {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => {
            eprintln!("Blocked / Skipped (C compiler '{}' failed to run -dumpmachine)", cc);
            return;
        }
    };

    // 4. Validate target compatibility
    let is_compatible = match rust_target {
        "x86_64-pc-windows-gnu" => {
            (dumpmachine_output.starts_with("x86_64-")
                && (dumpmachine_output.contains("mingw32") || dumpmachine_output.contains("windows")))
                && pointer_width == 64
        }
        "x86_64-unknown-linux-gnu" => {
            (dumpmachine_output.starts_with("x86_64-")
                && dumpmachine_output.contains("linux"))
                && pointer_width == 64
        }
        "aarch64-unknown-linux-gnu" => {
            (dumpmachine_output.starts_with("aarch64-")
                && dumpmachine_output.contains("linux"))
                && pointer_width == 64
        }
        "aarch64-apple-darwin" => {
            (dumpmachine_output.starts_with("arm64-") || dumpmachine_output.starts_with("aarch64-"))
                && dumpmachine_output.contains("darwin")
                && pointer_width == 64
        }
        _ => false,
    };

    if !is_compatible {
        eprintln!(
            "Blocked / Skipped (Target mismatch: Rust target '{}' vs C compiler target '{}')",
            rust_target, dumpmachine_output
        );
        return;
    }

    // 5. Compile report_c_layout.c
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let c_source = manifest_dir.join("tests/c_abi/report_c_layout.c");
    let include_dir = manifest_dir.join("include");

    let out_dir = std::env::temp_dir().join("bornomala_layout_test");
    let _ = std::fs::create_dir_all(&out_dir);
    let exe_name = if cfg!(target_os = "windows") {
        "report_c_layout.exe"
    } else {
        "report_c_layout"
    };
    let out_exe = out_dir.join(exe_name);

    let compile_status = Command::new(&cc)
        .arg("-std=c99")
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-Werror")
        .arg("-pedantic")
        .arg("-I")
        .arg(&include_dir)
        .arg(&c_source)
        .arg("-o")
        .arg(&out_exe)
        .output();

    let compile_res = match compile_status {
        Ok(out) if out.status.success() => out,
        Ok(out) => {
            panic!(
                "FAILED: report_c_layout.c failed to compile with {}: {}",
                cc,
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Err(e) => {
            panic!("FAILED: Could not invoke C compiler '{}': {}", cc, e);
        }
    };
    let _ = compile_res;

    // 6. Execute compiled binary
    let exec_output = match Command::new(&out_exe).output() {
        Ok(out) if out.status.success() => out,
        Ok(out) => {
            panic!(
                "FAILED: report_c_layout executable exited with error: status={:?}, stderr={}",
                out.status,
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Err(e) => {
            eprintln!("Blocked / Skipped (Host cannot execute cross-target binary: {})", e);
            return;
        }
    };

    // Clean up temporary executable
    let _ = std::fs::remove_file(&out_exe);

    // 7. Parse C output
    let stdout_str = String::from_utf8_lossy(&exec_output.stdout);
    let mut c_map: HashMap<String, usize> = HashMap::new();
    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            if let Some((tag, val_str)) = v.split_once('=') {
                let full_key = format!("{}:{}", k, tag);
                if let Ok(val) = val_str.parse::<usize>() {
                    c_map.insert(full_key, val);
                }
            } else if k == "POINTER_WIDTH" {
                if let Ok(val) = v.parse::<usize>() {
                    c_map.insert("POINTER_WIDTH".to_string(), val);
                }
            }
        }
    }

    // 8. Generate Rust expectations and compare
    assert_eq!(
        c_map.get("POINTER_WIDTH").copied(),
        Some(pointer_width),
        "Pointer width mismatch between C and Rust"
    );

    // BornoEventC
    assert_eq!(
        c_map.get("SIZE:BornoEventC").copied(),
        Some(std::mem::size_of::<BornoEventC>()),
        "Size mismatch for BornoEventC"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoEventC.event_type").copied(),
        Some(std::mem::offset_of!(BornoEventC, event_type)),
        "Offset mismatch for BornoEventC.event_type"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoEventC.char_codepoint").copied(),
        Some(std::mem::offset_of!(BornoEventC, char_codepoint)),
        "Offset mismatch for BornoEventC.char_codepoint"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoEventC.string_payload").copied(),
        Some(std::mem::offset_of!(BornoEventC, string_payload)),
        "Offset mismatch for BornoEventC.string_payload"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoEventC.index_payload").copied(),
        Some(std::mem::offset_of!(BornoEventC, index_payload)),
        "Offset mismatch for BornoEventC.index_payload"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoEventC.mode_payload").copied(),
        Some(std::mem::offset_of!(BornoEventC, mode_payload)),
        "Offset mismatch for BornoEventC.mode_payload"
    );

    // BornoStatusC
    assert_eq!(
        c_map.get("SIZE:BornoStatusC").copied(),
        Some(std::mem::size_of::<BornoStatusC>()),
        "Size mismatch for BornoStatusC"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStatusC.name").copied(),
        Some(std::mem::offset_of!(BornoStatusC, name)),
        "Offset mismatch for BornoStatusC.name"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStatusC.version").copied(),
        Some(std::mem::offset_of!(BornoStatusC, version)),
        "Offset mismatch for BornoStatusC.version"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStatusC.active_mode").copied(),
        Some(std::mem::offset_of!(BornoStatusC, active_mode)),
        "Offset mismatch for BornoStatusC.active_mode"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStatusC.ready").copied(),
        Some(std::mem::offset_of!(BornoStatusC, ready)),
        "Offset mismatch for BornoStatusC.ready"
    );

    // BornoStateC
    assert_eq!(
        c_map.get("SIZE:BornoStateC").copied(),
        Some(std::mem::size_of::<BornoStateC>()),
        "Size mismatch for BornoStateC"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStateC.preedit").copied(),
        Some(std::mem::offset_of!(BornoStateC, preedit)),
        "Offset mismatch for BornoStateC.preedit"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStateC.raw_buffer").copied(),
        Some(std::mem::offset_of!(BornoStateC, raw_buffer)),
        "Offset mismatch for BornoStateC.raw_buffer"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStateC.cursor_pos").copied(),
        Some(std::mem::offset_of!(BornoStateC, cursor_pos)),
        "Offset mismatch for BornoStateC.cursor_pos"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStateC.candidates_count").copied(),
        Some(std::mem::offset_of!(BornoStateC, candidates_count)),
        "Offset mismatch for BornoStateC.candidates_count"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStateC.candidates").copied(),
        Some(std::mem::offset_of!(BornoStateC, candidates)),
        "Offset mismatch for BornoStateC.candidates"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStateC.selected_candidate_index").copied(),
        Some(std::mem::offset_of!(BornoStateC, selected_candidate_index)),
        "Offset mismatch for BornoStateC.selected_candidate_index"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStateC.is_composing").copied(),
        Some(std::mem::offset_of!(BornoStateC, is_composing)),
        "Offset mismatch for BornoStateC.is_composing"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoStateC.mode").copied(),
        Some(std::mem::offset_of!(BornoStateC, mode)),
        "Offset mismatch for BornoStateC.mode"
    );

    // BornoProcessResultC
    assert_eq!(
        c_map.get("SIZE:BornoProcessResultC").copied(),
        Some(std::mem::size_of::<BornoProcessResultC>()),
        "Size mismatch for BornoProcessResultC"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoProcessResultC.state").copied(),
        Some(std::mem::offset_of!(BornoProcessResultC, state)),
        "Offset mismatch for BornoProcessResultC.state"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoProcessResultC.committed_text").copied(),
        Some(std::mem::offset_of!(BornoProcessResultC, committed_text)),
        "Offset mismatch for BornoProcessResultC.committed_text"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoProcessResultC.cancelled").copied(),
        Some(std::mem::offset_of!(BornoProcessResultC, cancelled)),
        "Offset mismatch for BornoProcessResultC.cancelled"
    );
    assert_eq!(
        c_map.get("OFFSET:BornoProcessResultC.handled").copied(),
        Some(std::mem::offset_of!(BornoProcessResultC, handled)),
        "Offset mismatch for BornoProcessResultC.handled"
    );

    println!(
        "C-ABI layout parity verified successfully: C vs Rust struct layout identical for target '{}'.",
        rust_target
    );
}

