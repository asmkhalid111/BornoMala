# Work Package WP-01: Linux Fcitx 5 Privacy & Build Integrity — Completion Report

> **Work Package ID:** `WP-01`  
> **Status:** Implemented & Verified  
> **Date:** 2026-10-26  
> **Target Scope:** Linux Fcitx 5 Addon (`native/bornomala-fcitx5/`)

---

## 1. Summary of Changes

### A. Privacy Log Redaction in `native/bornomala-fcitx5/bornomala.cpp`
- **Lines 183–185:** Removed `FCITX_INFO()` logging of raw keysyms (`sym=0x...`) in `BornoEngine::keyEvent`.
- **Lines 404–407:** Redacted preedit text strings (`preedit='...'`) and committed user text (`committed='...'`) from `BornoEngine::syncContextAndUI`. Retained non-sensitive operational metrics (`handled`, `isComposing`, `candidates_count`).
- **Diff:**
  ```diff
  --- a/native/bornomala-fcitx5/bornomala.cpp
  +++ b/native/bornomala-fcitx5/bornomala.cpp
  @@ -181,9 +181,7 @@ void BornoEngine::keyEvent(const fcitx::InputMethodEntry& entry, fcitx::KeyEvent
       }
   
       auto sym = keyEvent.key().sym();
  -    FCITX_INFO() << "[Borno] keyEvent: sym=0x" << std::hex << sym << std::dec 
  -                 << ", mode=" << prop->mode() 
  -                 << ", isComposing=" << prop->isComposing();
  +    // Raw keysym logging omitted to preserve user privacy (zero keystroke retention)
   
       // Step 4: Mode switch via unmodified F12 (or F4 for laptop keyboards with Airplane Mode on F12)
       if (sym == FcitxKey_F12 || sym == FcitxKey_F4) {
  @@ -404,8 +402,7 @@ void BornoEngine::syncContextAndUI(fcitx::InputContext* ic, BornoContextProperty
   void BornoEngine::syncContextAndUI(fcitx::InputContext* ic, BornoContextProperty* prop, const BornoProcessResultC& res) {
       FCITX_INFO() << "[Borno] syncContextAndUI: handled=" << res.handled
                    << ", isComposing=" << res.state.is_composing
  -                 << ", preedit='" << (res.state.preedit ? res.state.preedit : "") << "'"
  -                 << ", committed='" << (res.committed_text ? res.committed_text : "") << "'";
  +                 << ", candidates=" << res.state.candidates_count;
       if (res.committed_text && std::strlen(res.committed_text) > 0) {
           ic->commitString(res.committed_text);
       }
  ```

### B. Multi-Tier Static Library Discovery in `native/bornomala-fcitx5/CMakeLists.txt`
- Eliminated `/home/asmkhalid/...` hardcoded path.
- Added hierarchical multi-tier resolution:
  1. `-DBORNOMALA_CORE_STATIC` override.
  2. Cargo workspace root target: `${CMAKE_CURRENT_SOURCE_DIR}/../../target/release/libbornomala_core.a`.
  3. Sub-crate target fallback: `${CMAKE_CURRENT_SOURCE_DIR}/../../crates/bornomala-core/target/release/libbornomala_core.a`.
  4. Environment variable `$ENV{CARGO_TARGET_DIR}/release/libbornomala_core.a`.
- **Diff:**
  ```diff
  --- a/native/bornomala-fcitx5/CMakeLists.txt
  +++ b/native/bornomala-fcitx5/CMakeLists.txt
  @@ -11,14 +11,35 @@ find_package(Fcitx5Utils REQUIRED)
   
   include(GNUInstallDirs)
   
  -# Locate libbornomala_core.a
  -set(BORNOMALA_CORE_STATIC "/home/asmkhalid/borno-linux-target/release/libbornomala_core.a")
  -if(NOT EXISTS ${BORNOMALA_CORE_STATIC})
  -    set(BORNOMALA_CORE_STATIC "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/bornomala-core/target/release/libbornomala_core.a")
  -endif()
  +# Locate libbornomala_core.a static library
  +if(NOT DEFINED BORNOMALA_CORE_STATIC OR "${BORNOMALA_CORE_STATIC}" STREQUAL "")
  +    # 1. Standard Cargo workspace target (repository root)
  +    set(WORKSPACE_TARGET "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release/libbornomala_core.a")
  +    # 2. Sub-crate target directory fallback
  +    set(CRATE_TARGET "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/bornomala-core/target/release/libbornomala_core.a")
  +    # 3. Environment variable CARGO_TARGET_DIR if set
  +    if(DEFINED ENV{CARGO_TARGET_DIR})
  +        set(ENV_TARGET "$ENV{CARGO_TARGET_DIR}/release/libbornomala_core.a")
  +    else()
  +        set(ENV_TARGET "")
  +    endif()
   
  -if(NOT EXISTS ${BORNOMALA_CORE_STATIC})
  -    message(FATAL_ERROR "Could not find libbornomala_core.a. Please build bornomala-core in release profile first.")
  +    if(EXISTS "${WORKSPACE_TARGET}")
  +        set(BORNOMALA_CORE_STATIC "${WORKSPACE_TARGET}")
  +    elseif(EXISTS "${CRATE_TARGET}")
  +        set(BORNOMALA_CORE_STATIC "${CRATE_TARGET}")
  +    elseif(EXISTS "${ENV_TARGET}")
  +        set(BORNOMALA_CORE_STATIC "${ENV_TARGET}")
  +    else()
  +        message(FATAL_ERROR 
  +            "Could not locate libbornomala_core.a.\n"
  +            "Checked locations:\n"
  +            "  1. ${WORKSPACE_TARGET}\n"
  +            "  2. ${CRATE_TARGET}\n"
  +            "Please build bornomala-core using: cargo build --release -p bornomala-core\n"
  +            "Or specify the path directly via: cmake -DBORNOMALA_CORE_STATIC=/path/to/libbornomala_core.a"
  +        )
  +    endif()
   endif()
  ```

### C. Target Alignment in `native/bornomala-fcitx5/scripts/install.sh`
- Aligned `TARGET_DIR` to `${PROJECT_ROOT}/target`.
- Added `-DBORNOMALA_CORE_STATIC="${TARGET_DIR}/release/libbornomala_core.a"` to the `cmake` invocation.
- **Diff:**
  ```diff
  --- a/native/bornomala-fcitx5/scripts/install.sh
  +++ b/native/bornomala-fcitx5/scripts/install.sh
  @@ -5,7 +5,7 @@ SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
   PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
   FCITX_SRC_DIR="$PROJECT_ROOT/native/bornomala-fcitx5"
   BUILD_DIR="${HOME}/bornomala-fcitx5-build"
  -TARGET_DIR="${HOME}/borno-linux-target"
  +TARGET_DIR="${PROJECT_ROOT}/target"
   
   echo "=== BornoMala Fcitx 5 User-Mode Installer ==="
   
  @@ -17,7 +17,7 @@ mkdir -p "$TARGET_DIR"
   # 2. Build Fcitx 5 addon using CMake
   echo "[2/5] Building Fcitx 5 addon..."
   mkdir -p "$BUILD_DIR"
  -(cd "$BUILD_DIR" && cmake -DCMAKE_BUILD_TYPE=Release "$FCITX_SRC_DIR" && make -j"$(nproc)")
  +(cd "$BUILD_DIR" && cmake -DCMAKE_BUILD_TYPE=Release -DBORNOMALA_CORE_STATIC="${TARGET_DIR}/release/libbornomala_core.a" "$FCITX_SRC_DIR" && make -j"$(nproc)")
  ```

### D. Automated Regression Invariant in `crates/bornomala-core/tests/ffi_tests.rs`
- Appended `test_linux_adapter_privacy_and_portability_invariants` to assert that no privacy or portability regressions occur during future development.

---

## 2. Verification Evidence

### A. Dedicated Test Command
```powershell
cargo test --test ffi_tests
```
**Output:**
```
running 8 tests
test test_ffi_invalid_utf8_rejection ... ok
test test_ffi_string_echo_bangla_unicode ... ok
test test_ffi_null_pointer_safety ... ok
test test_ffi_engine_lifecycle_and_status ... ok
test test_linux_adapter_privacy_and_portability_invariants ... ok
test test_ffi_engine_configure_json ... ok
test test_ffi_engine_event_dispatch_with_test_fixture ... ok
test test_ffi_candidate_kind_accessor ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

### B. Full Workspace Regression Suite
```powershell
cargo test --workspace
```
**Output:**
- All 230 tests passed with zero failures.

---

## 3. Residual Verification

- Full runtime verification of Fcitx 5 (`bornomala.so` loading into daemon, GTK/Qt typing) requires access to a Linux environment with `fcitx5-devel` installed.
