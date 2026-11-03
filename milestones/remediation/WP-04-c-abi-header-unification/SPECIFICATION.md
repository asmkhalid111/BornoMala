# Work Package WP-04: C-ABI Header Unification (`borno.h` vs `bornomala.h`) — Specification

> **Work Package ID:** `WP-04`  
> **Status:** Draft — Pending User Review & Revision  
> **Date:** 2026-10-27  
> **Target Scope:** Core C-ABI Public Headers (`crates/bornomala-core/include/bornomala.h`, `crates/bornomala-core/include/borno.h`), Contract Documentation (`docs/engine-contract.md`), and Multi-Level C/C++ Header Parity Verification  
> **Source Documents:**  
> - `milestones/remediation/FINAL-IMPLEMENTATION-READINESS-REVIEW.md` (Finding 5)  
> - `milestones/remediation/INDEX.md` (WP-04 entry)  
> - `milestones/AUDIT-2026-10-23.md` (Finding C-FFI-4 & WP8)  
> - `milestones/revisions/RV-02-contract-and-ffi.md` (§2, §4.6, AC-3)  
> - `AGENTS.md` (Persistent Rules for AI Coding Agents)

---

## 1. Executive Summary & Problem Statement

### 1.1 The Header Fork Discrepancy

During the evolution of the BornoMala codebase between milestones M0.1 and M9, the public C-ABI headers diverged into two separate files residing in the same directory (`crates/bornomala-core/include/`):

1. **`crates/bornomala-core/include/bornomala.h`** (329 lines, 12,427 bytes):
   - Created to match the project and crate naming (`bornomala-core`).
   - Extended in Milestone M9 with error codes `BORNO_ERR_INVALID_CONFIG` (`-5`) and `BORNO_ERR_UNSUPPORTED_SCHEMA` (`-6`), along with the configuration injection function `borno_engine_configure_json()`.
   - Actively consumed by all native platform adapters in the repository:
     - `native/borno-tsf/borno_tsf.cpp:13`
     - `native/borno-tsf/key_mapper.h:12`
     - `native/borno-tsf/test_harness.cpp:12`
     - `native/bornomala-fcitx5/engine.h:15`
     - `native/bornomala-fcitx5/key_mapper.h:7`
     - `native/bornomala-fcitx5/tests/test_c_linkage.cpp:4`
     - `native/bornomala-fcitx5/tests/test_engine_adapter.cpp:6`

2. **`crates/bornomala-core/include/borno.h`** (314 lines, 11,684 bytes):
   - The original C-ABI header established in Milestone M0.1 and updated through M6.
   - **Lacks** `BORNO_ERR_INVALID_CONFIG` (`-5`).
   - **Lacks** `BORNO_ERR_UNSUPPORTED_SCHEMA` (`-6`).
   - **Lacks** `borno_engine_configure_json()` declaration.
   - Cited in `ROADMAP.md` (lines 280, 301), `README.md` (line 112), and `docs/engine-contract.md` (line 222).
   - Not directly included by any active C/C++ source file in the repository.

### 1.2 The Header Guard Collision & Silent Shadowing

Both headers currently declare the **exact same preprocessor macro guard**:

```c
/* crates/bornomala-core/include/borno.h:11-12 */
#ifndef BORNO_H
#define BORNO_H

/* crates/bornomala-core/include/bornomala.h:11-12 */
#ifndef BORNO_H
#define BORNO_H
```

Furthermore, line 2 of `bornomala.h` still contains the un-renamed comment:
```c
/**
 * @file borno.h
```

**Critical Failure Mode:**  
If a downstream build, test, or external consumer includes `borno.h` before `bornomala.h` (either directly or via transitive includes), `borno.h` defines `BORNO_H`. When `bornomala.h` is subsequently reached, the preprocessor evaluates `#ifndef BORNO_H` to false and **silently skips the entire header**.  
Consequently, `borno_engine_configure_json`, `BORNO_ERR_INVALID_CONFIG`, and `BORNO_ERR_UNSUPPORTED_SCHEMA` become unavailable, causing unexpected compilation failures in code that calls the configuration APIs.

### 1.3 Exact Header Differences on Disk vs. Architectural Scope

A raw textual diff (`git diff --no-index borno.h bornomala.h`) shows that the files on disk currently differ in two content blocks:

```diff
--- a/crates/bornomala-core/include/borno.h
+++ b/crates/bornomala-core/include/bornomala.h
@@ -27,2 +27,4 @@ extern "C" {
 #define BORNO_ERR_INVALID_ARGUMENT  -4
+#define BORNO_ERR_INVALID_CONFIG    -5
+#define BORNO_ERR_UNSUPPORTED_SCHEMA -6
 
@@ -209,2 +211,15 @@ BornoEngineHandle* borno_engine_new(void);
 void borno_engine_free(BornoEngineHandle* handle);
 
+/**
+ * @brief Reconfigure the engine in-place using a JSON configuration string.
+ *
+ * Supports schemas v1.0.0, v1.1.0, and v1.2.0. In-place reconfiguration updates
+ * settings and layout preferences without mutating active composition or preedit text.
+ *
+ * @param handle Valid engine handle.
+ * @param json_utf8 Null-terminated UTF-8 JSON configuration string.
+ * @return BORNO_OK on success, BORNO_ERR_INVALID_CONFIG (-5) on malformed JSON or validation
+ *         failure, BORNO_ERR_UNSUPPORTED_SCHEMA (-6) on unsupported schema, or standard FFI error.
+ */
+int32_t borno_engine_configure_json(BornoEngineHandle* handle, const char* json_utf8);
```

However, from an architectural and specification standpoint, the full discrepancy encompasses **four distinct dimensions**:
1. **API Content Disparity:** Missing error codes `-5`/`-6` and `borno_engine_configure_json` in `borno.h`.
2. **Preprocessor Guard Collision:** Both files defining `BORNO_H`, causing transitive inclusion masking.
3. **Metadata & Doxygen Drift:** `bornomala.h:2` claiming to be `@file borno.h`.
4. **Header Architecture & Redundancy:** Maintaining two duplicate 330-line files instead of an authoritative canonical master and a forwarding compatibility wrapper.

---

## 2. Invariants & Scope Boundaries

In accordance with `AGENTS.md` and repository standards:

1. **One Milestone at a Time (AGENTS.md §1.1):** WP-04 is confined strictly to C-ABI header unification, header guard resolution, contract documentation consistency, and multi-level verification. It does not implement WP-05 (desktop app identifier migration), touch candidate/cursor logic, or modify Rust engine internals.
2. **Preserve Existing Code & Consumer Compatibility (AGENTS.md §1.4):**
   - Native adapters (`native/borno-tsf` and `native/bornomala-fcitx5`) already include `bornomala.h` and must continue to build cleanly without code modifications.
   - Any third-party consumer or legacy script including `borno.h` must continue to compile and function without breaking changes or missing symbols.
   - Both include paths (`<borno.h>` and `<bornomala.h>`) remain valid in `crates/bornomala-core/include/`.
3. **Strict Truth in Status & Evidence-Based Verification (AGENTS.md §2.1):**
   - Header correctness, guard independence, macro coverage, and symbol declarations must be proven through reproducible compiler execution with both C (`gcc`) and C++ (`g++`) compilers.
   - Distinct verification levels must be maintained: compile-only tests (`-c`) must never be conflated with link-time or runtime tests.
4. **Offline & Privacy by Default (AGENTS.md §3.4):** No external network calls, downloads, or un-versioned dependencies.
5. **Continuous Regression Discipline & Working-Tree Preservation (AGENTS.md §4.3):**
   - All 10 existing Rust core test targets (251 baseline tests) must continue to pass with 0 regressions.
   - Running the test suite executes `test_export_fixtures` in `builtin.rs`, which touches reference JSON fixtures (`bijoy_reference.json`, `national_reference.json`, `probhat_reference.json`) due to line-ending serialization differences on Windows. In adherence to `AGENTS.md` §1.4, these working-tree fixture changes must be preserved and must not be automatically reverted with destructive git commands.

---

## 3. Architecture & Strategic Evaluation

Three architectural strategies were evaluated for resolving this discrepancy:

| Criterion | Option A: Canonical Master + Forwarding Header (Recommended) | Option B: Dual Synchronized Headers | Option C: Delete `borno.h` |
|---|---|---|---|
| **Description** | `bornomala.h` is the canonical master header. `borno.h` becomes a thin forwarding header: `#include "bornomala.h"`. | Both `bornomala.h` and `borno.h` contain duplicated definitions with independent guards (`BORNOMALA_H` vs `BORNO_H`). | Delete `borno.h` completely; update all documentation to point to `bornomala.h`. |
| **Single Source of Truth** | **Yes.** Edits to the C-ABI occur in exactly one file (`bornomala.h`). | **No.** Two duplicate 330-line files must be kept in lockstep manually. | **Yes.** Only one file exists. |
| **Future Drift Risk** | **Eliminates duplicated API-definition maintenance.** `borno.h` forwards directly to `bornomala.h`, preventing separate declaration divergence. | **High.** Future additions to `ffi.rs` risk updating one and forgetting the other (re-introducing WP-04 defect). | **Zero.** |
| **Backward Compatibility** | **High (Preserves ordinary source-level compatibility).** Code including either documented path (`<borno.h>` or `<bornomala.h>`) receives all symbols. Deliberately changes behavior only for external code that manually predefines `BORNO_H` to suppress inclusion. | **100%.** | **Breaks** any external consumer, build script, or un-migrated code referencing `borno.h`. |
| **Guard Collision Resolution** | Cleanly resolved with distinct forwarding guard. | Resolved if guards are manually made distinct, but dual declaration collisions occur if both are included. | N/A |
| **Compliance with Prior Audits** | Fully recommended by `RV-02-contract-and-ffi.md` (§4.6, AC-3) and `AUDIT-2026-10-23.md` (Finding C-FFI-4). | Rejected by `RV-02` ("No keeping both for compatibility without forwarding"). | Rejected due to unnecessary breaking change. |

### 3.1 Strategic Decision: Option A (Canonical Master + Forwarding Header)

Option A is the established industry pattern for header renaming and deprecation:
- `bornomala.h` is the authoritative canonical header.
- `borno.h` becomes a thin forwarding wrapper with clear deprecation guidance.
- Guard collision is eliminated while preserving legacy macro checks.
- Eliminates duplicated API-definition maintenance across separate header files.
- Preserves ordinary source-level compatibility for consumers including either documented path, while acknowledging the deliberately altered behavior of non-standard code that manually predefines `BORNO_H` to suppress inclusion.

### 3.2 Evaluation of the `BORNO_H` Backward-Compatibility Alias

The canonical header `bornomala.h` will introduce its own authoritative guard `BORNOMALA_H` while defining `BORNO_H` as an internal alias:

```c
#ifndef BORNOMALA_H
#define BORNOMALA_H

/* Backward-compatibility guard alias for legacy code expecting BORNO_H */
#ifndef BORNO_H
#define BORNO_H
#endif
```

#### Evaluation Against Repository Evidence:
1. **Actual Usage:** A repository-wide regex search confirmed that no active source file in `crates/` or `native/` currently queries `BORNO_H` as a feature-test macro.
2. **Compatibility Benefit:** External consumers or third-party wrappers written against milestones M0.1–M6 may contain preprocessor checks such as `#ifdef BORNO_H` to detect whether Borno C-ABI definitions are present. Defining `BORNO_H` inside `bornomala.h` guarantees that ordinary legacy feature checks evaluate to true regardless of whether the consumer included `borno.h` or `bornomala.h`.
3. **Explicit Boundary of the Guarantee:** This alias preserves *ordinary inclusion-detection* macro checks. It does *not* preserve deliberate macro-suppression patterns (e.g., if an external consumer intentionally executed `#define BORNO_H` *prior* to `#include "bornomala.h"` in an attempt to block the header from declaring symbols). Under the revised contract, `bornomala.h` is controlled by `BORNOMALA_H` and will not be suppressed by an external `BORNO_H` definition. This behavior is deliberate and necessary to prevent the very shadowing defect that WP-04 is tasked with resolving.

---

## 4. Detailed Implementation Specification

### 4.1 Canonical Header Specification (`crates/bornomala-core/include/bornomala.h`)

1. **Header Metadata:**
   - Correct `@file` tag from `borno.h` to `bornomala.h`.
   - Update brief comment to describe BornoMala Core C-API.
2. **Preprocessor Guards:**
   - Primary guard: `BORNOMALA_H`.
   - Backward-compatibility alias guard: `#ifndef BORNO_H \n #define BORNO_H \n #endif`.
3. **Content Integrity:**
   - Retain all current declarations, constants, error codes (`-1` through `-6`), mode constants (`0` through `4`), event types (`0` through `14`), candidate kinds (`0` through `4`), physical keys (`0` through `47`), and function signatures.
   - Include verified documentation on out-parameter initialization mechanics, reuse obligations, and memory disposal rules.

#### Proposed Target Code (`crates/bornomala-core/include/bornomala.h` lines 1–45):
```c
/**
 * @file bornomala.h
 * @brief C-API declarations for BornoMala Core Bangla Input Platform.
 *
 * Memory Ownership & Deallocation Rules:
 * - Preferred Deallocation Method: Callers SHOULD always release output structures
 *   using the corresponding structure cleanup helper (borno_status_free_members,
 *   borno_state_free_members, borno_process_result_free_members). These helpers safely
 *   free all nested allocations, null dangling pointers, and reset status flags.
 * - Individual String Deallocation: Direct standalone strings (e.g. borno_ffi_echo_utf8)
 *   or independently allocated structure string members (BornoStatusC::name, version,
 *   active_mode; BornoStateC::preedit, raw_buffer; BornoProcessResultC::committed_text)
 *   may be freed individually using borno_string_free().
 * - Double-Free Prevention: If a structure string member is freed individually via
 *   borno_string_free(), the caller MUST immediately assign that pointer field to NULL.
 *   Callers must NEVER free a member individually and subsequently invoke the structure
 *   cleanup helper while that member still contains its dangling pointer.
 * - Candidate Array Lifecycle: Candidate inner strings and the outer pointer array
 *   (BornoStateC::candidates, BornoProcessResultC::state.candidates) MUST be managed
 *   exclusively using borno_state_free_members() or borno_process_result_free_members().
 *   borno_string_free() must NEVER be used to deallocate the candidate outer array (char**),
 *   as doing so corrupts the allocator heap.
 * - Standard C free() or C++ delete must NEVER be used on any pointer returned by Rust.
 *
 * Output Structure Lifecycle & Reuse:
 * - On success (BORNO_OK), output-writing functions completely overwrite all structure
 *   members with fresh allocations. The implementation does NOT free previously held
 *   pointers. Callers MUST call the appropriate cleanup helper before reusing an
 *   already-populated structure to avoid leaking memory.
 * - On early error returns (e.g., BORNO_ERR_NULL_PTR, BORNO_ERR_INVALID_ARGUMENT,
 *   BORNO_ERR_INVALID_UTF8), output structures are left completely untouched.
 * - On panic (BORNO_ERR_PANIC), output structures may be left untouched or partially
 *   written; cleanup after a panic cannot be guaranteed safe by the implementation.
 * - Zero-initialization before first call (e.g. memset or = {0}) is a strongly recommended
 *   caller safety practice to ensure calling cleanup helpers on error paths is a safe
 *   no-op, but it is not an engine runtime prerequisite on successful calls.
 * - Cleanup helpers are safe for NULL pointers, safe for zero-initialized structures,
 *   and proven idempotent across repeated calls under their documented preconditions.
 */

#ifndef BORNOMALA_H
#define BORNOMALA_H

/* Backward compatibility guard alias for legacy code expecting BORNO_H */
#ifndef BORNO_H
#define BORNO_H
#endif

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif
```

And closing lines (lines 325–331):
```c
#ifdef __cplusplus
}
#endif

#endif /* BORNOMALA_H */
```

### 4.2 Forwarding Header Specification (`crates/bornomala-core/include/borno.h`)

Replace the duplicated 314-line body with a standardized forwarding wrapper:

```c
/**
 * @file borno.h
 * @brief Legacy compatibility forwarding header for bornomala.h.
 *
 * @deprecated This header is maintained for backward compatibility.
 *             Please include "bornomala.h" directly in new code.
 */

#ifndef BORNO_LEGACY_FORWARDING_H
#define BORNO_LEGACY_FORWARDING_H

#include "bornomala.h"

#endif /* BORNO_LEGACY_FORWARDING_H */
```

### 4.3 Mutual Inclusion & Precedence Proof

Under this architecture, all inclusion combinations are completely safe, idempotent, and order-independent:

- **Scenario 1: Consumer includes only `bornomala.h`:**
  - `BORNOMALA_H` is defined. `BORNO_H` is defined.
  - All symbols, constants, and functions are declared.
- **Scenario 2: Consumer includes only `borno.h`:**
  - `BORNO_LEGACY_FORWARDING_H` is defined.
  - `borno.h` includes `bornomala.h`.
  - `BORNOMALA_H` and `BORNO_H` are defined.
  - All symbols (including `borno_engine_configure_json` and error codes `-5`/`-6`) are declared.
- **Scenario 3: Consumer includes `borno.h` then `bornomala.h`:**
  - `borno.h` defines `BORNO_LEGACY_FORWARDING_H` and includes `bornomala.h`.
  - `bornomala.h` defines `BORNOMALA_H` and `BORNO_H` and declares all symbols.
  - Later `#include "bornomala.h"` evaluates `#ifndef BORNOMALA_H` to false and is safely skipped.
  - **Result:** No silent exclusion! No missing configuration symbols! No duplicate definition errors!
- **Scenario 4: Consumer includes `bornomala.h` then `borno.h`:**
  - `bornomala.h` defines `BORNOMALA_H` and `BORNO_H` and declares all symbols.
  - Later `#include "borno.h"` defines `BORNO_LEGACY_FORWARDING_H` and includes `bornomala.h`.
  - `bornomala.h` is safely skipped via `#ifndef BORNOMALA_H`.
- **Scenario 5: Repeated inclusions (`#include "bornomala.h"` or `#include "borno.h"` multiple times):**
  - Completely idempotent.

---

## 5. Grounded FFI Contract & Documentation Specification

Every guarantee specified below is grounded directly in repository source code and verified tests.

### 5.1 Documented Error Codes Table

Update the status codes table in `docs/engine-contract.md` (§7):

| Constant | Value | Origin in `ffi.rs` | Grounded Trigger Condition & Evidence |
|---|:---:|:---:|---|
| `BORNO_OK` | `0` | Line 24 | Successful operation completion (`ffi.rs:256, 285, 350`). |
| `BORNO_ERR_NULL_PTR` | `-1` | Line 25 | A required pointer argument is `NULL` (`ffi.rs:244, 278, 299, 364, 389, 409, 429, 451, 478, 506`). Tested in `ffi_tests.rs:11-81, 338-341`. |
| `BORNO_ERR_INVALID_UTF8` | `-2` | Line 26 | Input string payload fails UTF-8 decoding (`ffi.rs:250, 318, 513`). |
| `BORNO_ERR_PANIC` | `-3` | Line 27 | Rust panic caught by `catch_unwind` at the FFI boundary (`ffi.rs:258, 288, 353, 379, 399, 419, 439, 468`). |
| `BORNO_ERR_INVALID_ARGUMENT` | `-4` | Line 28 | Invalid event type, invalid codepoint, or invalid layout key code (`ffi.rs:307, 339, 343, 369, 455, 522`). |
| `BORNO_ERR_INVALID_CONFIG` | `-5` | Line 29 | JSON string is malformed syntax, or semantic validation failed (`candidate_display_limit` not in 1..=9, `max_dictionary_suggestions` not in 1..=5, `candidate_font_size` not in 10..=24). Grounded in `config.rs:232-246, 262` and `ffi.rs:262`. Tested in `ffi_tests.rs:368-374`. |
| `BORNO_ERR_UNSUPPORTED_SCHEMA` | `-6` | Line 30 | JSON `schema_version` is present but is not one of `"1.0.0"`, `"1.1.0"`, or `"1.2.0"`. Grounded in `config.rs:227-231, 294` and `ffi.rs:260`. Tested in `ffi_tests.rs:364-365`. |

### 5.2 Verified FFI Struct & Function Contract Analysis

#### 5.2.1 Output Structure Lifecycle, Error-Path Behavior & Memory Ownership
Inspection of `crates/bornomala-core/src/ffi.rs` establishes the exact lifecycle rules per API:

1. **`BornoStatusC` (`borno_engine_get_status`):**
   - **Fields Written on Success:** Completely overwrites `out_status->name`, `version`, `active_mode` (fresh heap-allocated C strings from Rust), and `ready` (`bool`) (`ffi.rs:544-549`).
   - **Reuse / Overwrite Rule:** Callers **must** call `borno_status_free_members(out_status)` before reusing an already-populated `BornoStatusC` structure. The engine unconditionally overwrites existing pointer fields without freeing them; failing to release prior members leaks memory.
   - **Early Error Paths:** If `handle` or `out_status` is `NULL`, returns `BORNO_ERR_NULL_PTR` (`-1`) without modifying `out_status` (`ffi.rs:277-279`). If `out_status` is non-null but `handle` is null, `*out_status` remains completely untouched.
   - **Panic Behavior:** If a panic occurs, returns `BORNO_ERR_PANIC` (`-3`). If the panic occurs in engine logic before `fill_status_c`, `out_status` is untouched. If a panic occurs during string allocation inside `fill_status_c`, fields may be left partially written. Because the implementation provides no transactional rollback on panic, cleanup safety after a panic is **not guaranteed** by the source.
   - **Cleanup Helper & Double-Free Prevention:** The preferred deallocation method is `borno_status_free_members(out_status)`. It safely accepts `NULL` and zero-initialized structs, frees all string members, sets pointers to `NULL`, and resets `ready` to `false` (`ffi.rs:139-151`). While individual string members (`name`, `version`, `active_mode`) may be freed with `borno_string_free()`, the caller MUST immediately set that field to `NULL`. Callers must never free a member individually and subsequently invoke `borno_status_free_members()` while that field retains its dangling pointer (preventing double-free UB). Repeated calls to the cleanup helper are proven idempotent under its documented preconditions.

2. **`BornoStateC` (`borno_engine_process_char`, `backspace`, `commit`, `reset`):**
   - **Fields Written on Success:** Completely overwrites `preedit`, `raw_buffer` (fresh heap-allocated C strings), `cursor_pos` (`usize`), `candidates_count` (`usize`), `candidates` (`*mut *mut c_char` array or `NULL` if count == 0), `selected_candidate_index` (`i32`), `is_composing` (`bool`), and `mode` (`i32`) (`ffi.rs:551-578`).
   - **Reuse / Overwrite Rule:** Callers **must** call `borno_state_free_members(out_state)` before reusing an already-populated structure. Existing heap pointers (`preedit`, `raw_buffer`, `candidates`) are overwritten without being freed.
   - **Early Error Paths:** If `handle` or `out_state` is `NULL`, returns `BORNO_ERR_NULL_PTR` (`-1`). In `borno_engine_process_char`, if `codepoint` is an invalid Unicode scalar value (`char::from_u32(codepoint)` is `None`), returns `BORNO_ERR_INVALID_ARGUMENT` (`-4`). On all early error paths, `out_state` is left completely untouched (`ffi.rs:363-370`).
   - **Panic Behavior:** Returns `BORNO_ERR_PANIC` (`-3`). If a panic occurs in engine event processing, `out_state` is untouched. If a panic occurs during string or candidate array allocation inside `fill_state_c`, `out_state` may be left partially written. Cleanup after a panic is **not guaranteed safe** by the implementation.
   - **Candidate Array Ownership & Double-Free Prevention:** The preferred deallocation method is `borno_state_free_members(out_state)`. Standalone string members (`preedit`, `raw_buffer`) may be freed with `borno_string_free()` provided the caller immediately assigns that field to `NULL`. Candidate inner strings and the outer pointer array (`candidates`) MUST be managed exclusively using `borno_state_free_members()`. `borno_string_free()` must NEVER be used on the outer `candidates` pointer array (`char**`), as doing so deallocates a slice of pointers via `CString::from_raw()`, corrupting the allocator heap. Callers must never free a member individually and subsequently invoke `borno_state_free_members()` without first nulling the freed pointer.
   - **Cleanup Helper (`borno_state_free_members`):** Safely accepts `NULL` and zero-initialized structs. Reconstructs and drops the candidate array vector, frees each candidate string, frees `preedit` and `raw_buffer`, zeroes scalar fields, and resets pointers to `NULL` (`ffi.rs:156-181`). Repeated cleanup under documented preconditions is **proven idempotent**.

3. **`BornoProcessResultC` (`borno_engine_process_event`, `borno_engine_process_key`):**
   - **Fields Written on Success:** Overwrites `state` (all 8 fields detailed above), `committed_text` (fresh heap-allocated C string or `NULL`), `cancelled` (`bool`), and `handled` (`bool`) (`ffi.rs:580-588`).
   - **Reuse / Overwrite Rule:** Callers **must** call `borno_process_result_free_members(out_result)` before reusing an already-populated structure. Existing heap pointers in `state` and `committed_text` are overwritten without being freed.
   - **Early Error Paths:** If `handle`, `event`, or `out_result` is `NULL`, returns `BORNO_ERR_NULL_PTR` (`-1`). If `string_payload` is invalid UTF-8, returns `BORNO_ERR_INVALID_UTF8` (`-2`). If event type, codepoint, or key code is invalid, returns `BORNO_ERR_INVALID_ARGUMENT` (`-4`). On all early error returns, `out_result` is left completely untouched (`ffi.rs:298-343, 450-456`).
   - **Panic Behavior:** Returns `BORNO_ERR_PANIC` (`-3`). Outputs may be untouched or partially written; cleanup after a panic is **not guaranteed safe**.
   - **Cleanup Helper & Double-Free Prevention:** The preferred method is `borno_process_result_free_members(out_result)`. It invokes `borno_state_free_members(&result->state)`, frees `committed_text`, resets flags to `false`, and nulls pointers (`ffi.rs:185-196`). If `committed_text` is freed individually with `borno_string_free()`, the caller MUST immediately assign `result->committed_text = NULL` before invoking `borno_process_result_free_members()` to avoid a double-free. Repeated cleanup under documented preconditions is **proven idempotent**.

4. **Direct String Allocations (`borno_ffi_echo_utf8`):**
   - On success (`BORNO_OK`), allocates a new C string via `CString::into_raw()` and writes it to `*out_str`, and writes character length to `*out_len`.
   - On early error (`-1`, `-2`, `-4`), `*out_str` and `*out_len` are untouched.
   - Caller owns `*out_str` and MUST free it via `borno_string_free(*out_str)`.

5. **Recommended Initialization Practice vs. Actual API Requirement:**
   - **API Requirement on Success Path:** Zero-initialization is **not a runtime requirement** for the engine on successful calls. On success, `fill_*` unconditionally overwrites all structure fields with valid Rust allocations and default values.
   - **Caller Safety Rule for Error Paths:** Because early error paths leave output structures completely untouched, any caller implementing defensive cleanup patterns (e.g. calling `free_members` in an error or exit block) will pass uninitialized stack memory to `free_members`. Since `borno_string_free` and `borno_state_free_members` check `!ptr.is_null()`, uninitialized stack garbage will be treated as valid heap pointers, leading to immediate memory corruption or process crash.
   - **Conclusion:** Pre-call zero-initialization (`{}` in C++ / `memset` in C) is a **strongly recommended caller safety practice**, but must be clearly distinguished from a mandatory engine precondition.

6. **Memory Ownership & Disposal Matrix:**

| Pointer / Struct Member | Exact C-ABI Type | Allocator | Preferred Deallocation Method | Individual Deallocation via `borno_string_free()` Permitted? | Safety Rule to Prevent Double-Free |
|---|---|---|---|:---:|---|
| `BornoStatusC.name` | `char*` | Rust `CString` | `borno_status_free_members()` | **Yes** | Caller must assign `name = NULL` immediately after `borno_string_free(name)` if struct helper is later called. |
| `BornoStatusC.version` | `char*` | Rust `CString` | `borno_status_free_members()` | **Yes** | Caller must assign `version = NULL` immediately after `borno_string_free(version)`. |
| `BornoStatusC.active_mode` | `char*` | Rust `CString` | `borno_status_free_members()` | **Yes** | Caller must assign `active_mode = NULL` immediately after `borno_string_free(active_mode)`. |
| `BornoStateC.preedit` | `char*` | Rust `CString` | `borno_state_free_members()` | **Yes** | Caller must assign `preedit = NULL` immediately after `borno_string_free(preedit)`. |
| `BornoStateC.raw_buffer` | `char*` | Rust `CString` | `borno_state_free_members()` | **Yes** | Caller must assign `raw_buffer = NULL` immediately after `borno_string_free(raw_buffer)`. |
| `BornoStateC.candidates` | `char**` (outer array) | Rust `Vec<*mut c_char>` | `borno_state_free_members()` | **NO (Fatal UB)** | Outer array pointer MUST be freed via `borno_state_free_members()`; `borno_string_free` corrupts heap. |
| `BornoStateC.candidates[i]` | `char*` (inner strings) | Rust `CString` | `borno_state_free_members()` | **NO** | Must be managed collectively via struct cleanup helper alongside outer array to avoid dangling pointers. |
| `BornoProcessResultC.committed_text` | `char*` (or NULL) | Rust `CString` | `borno_process_result_free_members()` | **Yes** | Caller must assign `committed_text = NULL` immediately after `borno_string_free(committed_text)`. |
| `borno_ffi_echo_utf8` `*out_str` | `char*` | Rust `CString` | `borno_string_free()` | **Yes** | Primary deallocation method for direct standalone string return. |
| `borno_engine_new()` handle | `BornoEngineHandle*` | Rust `Box` | `borno_engine_free()` | **NO (Fatal UB)** | Opaque engine handle MUST be released via `borno_engine_free()`. |


#### 5.2.2 `borno_engine_configure_json` Specifications
Document the exact function contract in `docs/engine-contract.md` (§7):

```c
int32_t borno_engine_configure_json(BornoEngineHandle* handle, const char* json_utf8);
```

1. **Parameter Null Safety:** If `handle` or `json_utf8` is `NULL`, returns `BORNO_ERR_NULL_PTR` (`-1`) immediately without dereferencing (`ffi.rs:243-245`).
2. **Borrowed String Ownership:** Caller retains full ownership of `json_utf8`. Rust borrows it as a `&str` for parsing (`ffi.rs:246-250`). Rust neither frees it nor stores a pointer to it after returning.
3. **Supported Schemas & In-Memory Migration:**
   - Schema `"1.2.0"`: Authoritative current schema. Accepted without migration (`config.rs:290-292`).
   - Schemas `"1.0.0"` and `"1.1.0"`: Migrated in-memory to `"1.2.0"` defaults during parsing (`config.rs:265-289`).
   - Any other schema version returns `BORNO_ERR_UNSUPPORTED_SCHEMA` (`-6`) (`config.rs:294`, `ffi.rs:260`).
4. **Configuration Failure Isolation:** Parsing and semantic validation occur in `BornoConfig::from_json_str(json_str)` *before* mutating the engine (`ffi.rs:251-264`). If parsing or validation fails, `engine.configure(&cfg)` is never called; prior engine configuration remains 100% untouched.
5. **Active State Preservation Invariants:**
   - In-place reconfiguration (whether successful or failed) preserves `state.preedit`, `state.raw_buffer`, `state.cursor_pos`, `self.source_cursor`, `state.is_composing`, and `state.mode` intact without disruption (`lib.rs:310-323`, `ffi_tests.rs:375-395`).
   - **Candidate Truncation Qualification:** If `candidate_display_limit` is lowered below the current candidate count, existing candidates and candidate details are truncated to the new limit (`lib.rs:319-322`). Candidates are not re-ranked or cleared, but they are truncated if the display budget was reduced.

---

## 6. Multi-Level Verification & Test Suite

To satisfy `AGENTS.md` §2 ("Evidence-Based Development"), verification is organized into three distinct levels with explicit separation between compile-time, link-time, and platform-adapter evidence:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Level 1: C99 & C++17 Header Syntax & Inclusion Order Tests             │
│ (Compile-only via gcc / g++ -c; proves syntax, guards, and visibility) │
└────────────────────────────────────┬───────────────────────────────────┘
                                     │
                                     ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Level 2: Target-Aware Linkage & ABI Layout Parity Against Rust Core     │
│ (Links against libbornomala_core.a; compares C vs Rust sizeof/offsetof)│
└────────────────────────────────────┬───────────────────────────────────┘
                                     │
                                     ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Level 3: Native Platform Adapter Integration & Regression Gates       │
│ (Windows TSF safe compile/link check; bornomala-core cargo test suite) │
└────────────────────────────────────────────────────────────────────────┘
```

### 6.1 Level 1: C99 and C++17 Header Syntax & Inclusion-Order Tests (Compile-Only)

Level 1 verifies that both headers compile under strict C99 and C++17 standards without warnings or errors. Every command in this level compiles an object file (`-c`) with an explicit output path into a dynamically resolved output directory (`<output_dir>`, e.g. `<target_dir>/c_abi_tests`). It makes no claim about link-time symbol resolution.

**Dedicated Verification Directory:**  
The test files specified below will be created in `crates/bornomala-core/tests/c_abi/`:

#### C99 Strict Compliance Tests
Flags: `-std=c99 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c`

1. **Translation Unit 1 (`tu_c99_borno_then_bornomala.c`):**
   ```c
   #include "borno.h"
   #include "bornomala.h"

   /* Explicit function pointer assignment to verify declaration visibility */
   int32_t (*fn_configure)(BornoEngineHandle*, const char*) = &borno_engine_configure_json;
   ```
   Command:
   ```bash
   gcc -std=c99 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c crates/bornomala-core/tests/c_abi/tu_c99_borno_then_bornomala.c -o <output_dir>/tu_c99_1.o
   ```
2. **Translation Unit 2 (`tu_c99_bornomala_then_borno.c`):**
   ```c
   #include "bornomala.h"
   #include "borno.h"

   int32_t (*fn_configure)(BornoEngineHandle*, const char*) = &borno_engine_configure_json;
   ```
   Command:
   ```bash
   gcc -std=c99 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c crates/bornomala-core/tests/c_abi/tu_c99_bornomala_then_borno.c -o <output_dir>/tu_c99_2.o
   ```
3. **Translation Unit 3 (`tu_c99_repeat_inclusion.c`):**
   ```c
   #include "bornomala.h"
   #include "bornomala.h"
   #include "borno.h"
   #include "borno.h"
   ```
   Command:
   ```bash
   gcc -std=c99 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c crates/bornomala-core/tests/c_abi/tu_c99_repeat_inclusion.c -o <output_dir>/tu_c99_3.o
   ```
4. **Translation Unit 4 (`tu_c99_constants_and_types.c`):**
   Validates constants, enums, and C99-compatible compile-time assertions:
   ```c
   #include "bornomala.h"

   /* Standard C99 compile-time assertion macro (no C11 _Static_assert dependency) */
   #define C99_STATIC_ASSERT(cond, name) typedef char static_assertion_##name[(cond) ? 1 : -1]

   C99_STATIC_ASSERT(BORNO_OK == 0, check_ok);
   C99_STATIC_ASSERT(BORNO_ERR_NULL_PTR == -1, check_null_ptr);
   C99_STATIC_ASSERT(BORNO_ERR_INVALID_UTF8 == -2, check_invalid_utf8);
   C99_STATIC_ASSERT(BORNO_ERR_PANIC == -3, check_panic);
   C99_STATIC_ASSERT(BORNO_ERR_INVALID_ARGUMENT == -4, check_invalid_argument);
   C99_STATIC_ASSERT(BORNO_ERR_INVALID_CONFIG == -5, check_invalid_config);
   C99_STATIC_ASSERT(BORNO_ERR_UNSUPPORTED_SCHEMA == -6, check_unsupported_schema);

   C99_STATIC_ASSERT(BORNO_MODE_PHONETIC == 0, check_mode_phonetic);
   C99_STATIC_ASSERT(BORNO_MODE_FIXED_BIJOY == 4, check_mode_bijoy);
   C99_STATIC_ASSERT(BORNO_CANDIDATE_PRIMARY_TRANSLITERATION == 0, check_cand_primary);
   C99_STATIC_ASSERT(BORNO_CANDIDATE_SPELLING_CORRECTION == 4, check_cand_spelling);

   C99_STATIC_ASSERT(BORNO_KEY_BACKQUOTE == 0, check_key_0);
   C99_STATIC_ASSERT(BORNO_KEY_SPACE == 47, check_key_space);
   C99_STATIC_ASSERT(BORNO_KEY_COUNT == 48, check_key_count);
   ```
   Command:
   ```bash
   gcc -std=c99 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c crates/bornomala-core/tests/c_abi/tu_c99_constants_and_types.c -o <output_dir>/tu_c99_4.o
   ```

#### C++17 Strict Compliance & Declaration Visibility Tests
Flags: `-std=c++17 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c`

1. **Translation Unit 1 (`tu_cpp17_borno_then_bornomala.cpp`):**
   ```cpp
   #include "borno.h"
   #include "bornomala.h"

   /* Explicit function pointer assignment verifying declaration visibility in C++ */
   int32_t (*fn_configure)(BornoEngineHandle*, const char*) = &borno_engine_configure_json;

   static_assert(BORNO_ERR_INVALID_CONFIG == -5, "BORNO_ERR_INVALID_CONFIG mismatch");
   static_assert(BORNO_ERR_UNSUPPORTED_SCHEMA == -6, "BORNO_ERR_UNSUPPORTED_SCHEMA mismatch");
   ```
   Command:
   ```bash
   g++ -std=c++17 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c crates/bornomala-core/tests/c_abi/tu_cpp17_borno_then_bornomala.cpp -o <output_dir>/tu_cpp17_1.o
   ```
2. **Translation Unit 2 (`tu_cpp17_bornomala_then_borno.cpp`):**
   ```cpp
   #include "bornomala.h"
   #include "borno.h"

   int32_t (*fn_configure)(BornoEngineHandle*, const char*) = &borno_engine_configure_json;

   static_assert(BORNO_ERR_INVALID_CONFIG == -5, "BORNO_ERR_INVALID_CONFIG mismatch");
   ```
   Command:
   ```bash
   g++ -std=c++17 -Wall -Wextra -Werror -pedantic -Icrates/bornomala-core/include -c crates/bornomala-core/tests/c_abi/tu_cpp17_bornomala_then_borno.cpp -o <output_dir>/tu_cpp17_2.o
   ```

---

### 6.2 Level 2: Target-Aware Linkage & ABI Layout Parity Against Rust Core

Level 2 tests link against the compiled static library `libbornomala_core.a`.

#### 6.2.1 Target-Aware Archive Path & Prerequisite Resolution
The static library archive `libbornomala_core.a` is an explicit prerequisite for Level 2 linkage and cannot be substituted with compile-only tests. Its path must be dynamically derived and verified:

1. **Explicit Build Parameters, Profile Mapping & Cargo Invocation:**
   Cargo build parameters must be defined explicitly and reused identically across compilation, archive resolution, and runtime library inspection, preventing silent artifact path drift:
   - **Profile Mapping & Standard Verification Profile:**
     - **`release` Profile (Authoritative Standard for WP-04 Linkage Verification):**
       - Cargo build invocation: `cargo build --release -p bornomala-core [--target <triple>]`
       - Artifact directory: `<target_dir>/<triple>/release/libbornomala_core.a` (or `<target_dir>/release/libbornomala_core.a` if omitting `--target`)
       - Native static library inspection: `cargo rustc --release -p bornomala-core [--target <triple>] -- --print native-static-libs`
     - **`debug` Profile (Cargo Default Development Profile, permitted for debugging):**
       - Cargo build invocation: `cargo build -p bornomala-core [--target <triple>]` (omits `--release`)
       - Artifact directory: `<target_dir>/<triple>/debug/libbornomala_core.a` (or `<target_dir>/debug/libbornomala_core.a` if omitting `--target`)
       - Native static library inspection: `cargo rustc -p bornomala-core [--target <triple>] -- --print native-static-libs`
     - **Authoritative Rule:** For standard WP-04 Level 2 and Level 3 verification, `$profile = "release"` is the primary, verified profile. All build commands, archive path derivations, and native library inspection commands MUST use the identical selected profile without ambiguity.
   - **Target Triple (`$targetTriple` / `TARGET_TRIPLE`):** Explicit target triple representing the actual Rust compilation target (e.g. `"x86_64-pc-windows-gnu"` on Windows MinGW, `"x86_64-unknown-linux-gnu"` on Linux).
   - **Explicit Build Invocations:**
     - For explicit Windows GNU release build:
       ```bash
       cargo build --release -p bornomala-core --target x86_64-pc-windows-gnu
       ```
     - For Linux shared module release build (Fcitx 5 addon):
       ```bash
       RUSTFLAGS="-C relocation-model=pic" cargo build --release -p bornomala-core --target x86_64-unknown-linux-gnu
       ```
     - For host-default target release build (when `--target` is deliberately omitted):
       ```bash
       cargo build --release -p bornomala-core
       ```
     - **Recommendation:** Explicitly passing `--target <triple>` is strongly preferred so that Cargo configuration defaults cannot silently redirect the output directory or alter target identity.

2. **Resolution of `CARGO_TARGET_DIR`, Archive Path, and Output Directory:**
   The static library archive location must be resolved from the exact build parameters used:
   - PowerShell:
     ```powershell
     # 1. Explicit build parameters (Authoritative for WP-04: "release")
     $profile = "release" # "release" or "debug"
     $targetTriple = "x86_64-pc-windows-gnu" # Explicit target triple; set to $null if cargo build omitted --target

     # 2. Derive cargo command flags matching the selected profile:
     $profileFlag = if ($profile -eq "release") { "--release" } else { "" }
     $targetFlag = if ($targetTriple) { "--target $targetTriple" } else { "" }

     # Cargo build invocation matching the profile:
     # cargo build $profileFlag -p bornomala-core $targetFlag

     # 3. Query authoritative Cargo target directory
     $targetDir = (cargo metadata --format-version 1 --no-deps | ConvertFrom-Json).target_directory
     if ($env:CARGO_TARGET_DIR) {
         if ([System.IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR)) {
             $targetDir = $env:CARGO_TARGET_DIR
         } else {
             $targetDir = Join-Path $PWD $env:CARGO_TARGET_DIR
         }
     }

     # 4. Derive archive path from the exact same profile and target parameters:
     if ($targetTriple) {
         $staticLib = Join-Path $targetDir "$targetTriple\$profile\libbornomala_core.a"
     } else {
         $staticLib = Join-Path $targetDir "$profile\libbornomala_core.a"
     }

     $outputDir = Join-Path $targetDir "c_abi_tests"
     New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
     ```
   - Bash:
     ```bash
     # 1. Explicit build parameters (Authoritative for WP-04: "release")
     PROFILE="release" # "release" or "debug"
     TARGET_TRIPLE="x86_64-unknown-linux-gnu" # Explicit target triple; empty if cargo build omitted --target

     # 2. Derive cargo command flags matching the selected profile:
     PROFILE_FLAG=""
     [ "$PROFILE" = "release" ] && PROFILE_FLAG="--release"
     TARGET_FLAG=""
     [ -n "$TARGET_TRIPLE" ] && TARGET_FLAG="--target $TARGET_TRIPLE"

     # Cargo build invocation matching the profile:
     # cargo build $PROFILE_FLAG -p bornomala-core $TARGET_FLAG

     # 3. Query authoritative Cargo target directory
     TARGET_DIR=$(cargo metadata --format-version 1 --no-deps | jq -r '.target_directory')
     if [ -n "$CARGO_TARGET_DIR" ]; then
         if [[ "$CARGO_TARGET_DIR" = /* ]]; then
             TARGET_DIR="$CARGO_TARGET_DIR"
         else
             TARGET_DIR="$(pwd)/$CARGO_TARGET_DIR"
         fi
     fi

     # 4. Derive archive path from the exact same profile and target parameters:
     if [ -n "$TARGET_TRIPLE" ]; then
         STATIC_LIB="$TARGET_DIR/$TARGET_TRIPLE/$PROFILE/libbornomala_core.a"
     else
         STATIC_LIB="$TARGET_DIR/$PROFILE/libbornomala_core.a"
     fi

     OUTPUT_DIR="$TARGET_DIR/c_abi_tests"
     mkdir -p "$OUTPUT_DIR"
     ```
   - In all subsequent commands, `<resolved_archive_path>` (or `$staticLib` / `$STATIC_LIB`) represents the dynamically resolved static archive path, and `<output_dir>` (or `$outputDir` / `$OUTPUT_DIR`) represents the dynamically resolved test output directory.

3. **Impact of Target Triples and Profiles on Archive Path:**
   - Host-default target (when `--target` is omitted): `<target_dir>/<profile>/libbornomala_core.a`.
   - Explicit target (when `--target <triple>` is passed): `<target_dir>/<triple>/<profile>/libbornomala_core.a`.
   - Where `<profile>` is `release` (using `--release`) or `debug` (omitting `--release`), matching the Cargo build invocation.
   - If the actual build parameters (`$profile`, `$targetTriple`, `$targetDir`) cannot be established unambiguously, archive-dependent checks MUST be recorded as `Blocked / Not run (Cannot establish build parameters or static archive path)` rather than guessing.

4. **Target-Mismatched Linkage Prevention:**
   Before invoking any C/C++ linker, the test runner must verify:
   - The static archive `libbornomala_core.a` exists at the exact resolved path `<resolved_archive_path>` for the active target and profile.
   - The compilation target of the static archive matches the target machine reported by the C compiler (`$CC -dumpmachine`).
   - If the archive is missing or target-mismatched, Level 2 linkage MUST NOT attempt linkage against an incompatible archive; it MUST record: `Blocked / Not run (Missing prerequisite: static archive not built for target <triple> and profile <profile>)`. A Windows archive must never be linked with Linux toolchains, nor vice versa.

5. **Determination of Required Native System Libraries:**
   - When Rust compiles a `staticlib`, rustc emits the target-specific native system libraries required by the Rust runtime. These must be inspected using the profile matching the static archive build:
     - For release profile:
       ```bash
       cargo rustc --release -p bornomala-core -- --print native-static-libs
       ```
     - For debug profile:
       ```bash
       cargo rustc -p bornomala-core -- --print native-static-libs
       ```
     - (When targeting an explicit triple, append `--target <triple>` to either command).
   - **Candidate Windows Link Flags (`x86_64-pc-windows-gnu` / MinGW-w64 GCC/G++):**
     `-luserenv -lws2_32 -lbcrypt -lntdll`  
     *(Grounded in `native/borno-tsf/build.ps1:58, 73`).*
   - **Candidate Linux Link Flags (`x86_64-unknown-linux-gnu` / GCC/G++):**
     `-lpthread -ldl -lm`  
     *(Grounded in `native/bornomala-fcitx5/CMakeLists.txt:109-112`).*
   - **Candidate Status Qualification:** These linker commands are designated as candidate configurations verified against existing repository native adapter build scripts. They are not claimed to be universally sufficient for all possible C toolchains (e.g. MSVC `cl.exe` uses `.lib` format and `/defaultlib` directives).
   - **Preservation of Verification Distinctions:**
     - Level 1: Compile-only checks (`-c`, syntax and declaration visibility only, no linking).
     - Level 2 compile & link: Compiles test translation unit and links against `libbornomala_core.a`.
     - Level 2 execution: Executes linked binary and verifies exit code 0.

#### 6.2.2 Dedicated C Linkage Test (`test_c_linkage.c`)

A dedicated new C test file (`crates/bornomala-core/tests/c_abi/test_c_linkage.c`) with explicit error handling and cleanup:

```c
#include "bornomala.h"
#include <stdio.h>

int main(void) {
    BornoEngineHandle* engine = borno_engine_new();
    if (engine == NULL) {
        fprintf(stderr, "FAIL: borno_engine_new returned NULL\n");
        return 1;
    }

    /* Valid minimal configuration conforming to schema 1.2.0 */
    const char* json_cfg = "{\"schema_version\":\"1.2.0\"}";
    int32_t rc = borno_engine_configure_json(engine, json_cfg);
    if (rc != BORNO_OK) {
        fprintf(stderr, "FAIL: borno_engine_configure_json returned error %d\n", rc);
        borno_engine_free(engine);
        return 1;
    }

    borno_engine_free(engine);
    printf("Level 2 C linkage test passed.\n");
    return 0;
}
```

- **Candidate Windows GNU Link Command (`x86_64-pc-windows-gnu` / MinGW-w64 GCC):**
  ```bash
  gcc -std=c99 -Wall -Wextra -Werror -pedantic \
      -Icrates/bornomala-core/include \
      crates/bornomala-core/tests/c_abi/test_c_linkage.c \
      <resolved_archive_path> \
      -luserenv -lws2_32 -lbcrypt -lntdll \
      -o <output_dir>/test_c_linkage.exe
  ```
- **Candidate Linux Link Command (`x86_64-unknown-linux-gnu` / GCC):**
  ```bash
  gcc -std=c99 -Wall -Wextra -Werror -pedantic \
      -Icrates/bornomala-core/include \
      crates/bornomala-core/tests/c_abi/test_c_linkage.c \
      <resolved_archive_path> \
      -lpthread -ldl -lm \
      -o <output_dir>/test_c_linkage
  ```

#### 6.2.3 Dedicated C++ Linkage Test (`test_cpp_linkage.cpp`)

A dedicated new C++ test file (`crates/bornomala-core/tests/c_abi/test_cpp_linkage.cpp`) proving `extern "C"` unmangled symbol resolution with explicit error handling:

```cpp
#include "bornomala.h"
#include <iostream>

int main() {
    BornoEngineHandle* engine = borno_engine_new();
    if (!engine) {
        std::cerr << "FAIL: borno_engine_new returned NULL\n";
        return 1;
    }

    const char* json_cfg = "{\"schema_version\":\"1.2.0\"}";
    int32_t rc = borno_engine_configure_json(engine, json_cfg);
    if (rc != BORNO_OK) {
        std::cerr << "FAIL: borno_engine_configure_json returned error " << rc << "\n";
        borno_engine_free(engine);
        return 1;
    }

    borno_engine_free(engine);
    std::cout << "Level 2 C++ linkage test passed.\n";
    return 0;
}
```

- **Candidate Windows GNU Link Command (`x86_64-pc-windows-gnu` / MinGW-w64 G++):**
  ```bash
  g++ -std=c++17 -Wall -Wextra -Werror -pedantic \
      -Icrates/bornomala-core/include \
      crates/bornomala-core/tests/c_abi/test_cpp_linkage.cpp \
      <resolved_archive_path> \
      -luserenv -lws2_32 -lbcrypt -lntdll \
      -o <output_dir>/test_cpp_linkage.exe
  ```
- **Candidate Linux Link Command (`x86_64-unknown-linux-gnu` / G++):**
  ```bash
  g++ -std=c++17 -Wall -Wextra -Werror -pedantic \
      -Icrates/bornomala-core/include \
      crates/bornomala-core/tests/c_abi/test_cpp_linkage.cpp \
      <resolved_archive_path> \
      -lpthread -ldl -lm \
      -o <output_dir>/test_cpp_linkage
  ```

#### 6.2.4 Target-Aware ABI Layout Parity Mechanism

Universal hardcoded struct sizes across architectures do not exist. To establish binary ABI parity legitimately without guesswork:

1. **Rust Compile-Time Target Attribute Inspection:**
   The Rust test (`crates/bornomala-core/tests/c_abi_layout_tests.rs`) identifies its compilation target using built-in Rust compile-time target attributes (`cfg!`) and standard library constants without depending on `option_env!("TARGET")` (which is not available in ordinary crate test sources without a custom build script) and without introducing new external dependencies:
   - **Target Architecture (`target_arch`):** `cfg!(target_arch = "x86_64")`, `cfg!(target_arch = "aarch64")`, `cfg!(target_arch = "x86")`, or `std::env::consts::ARCH`.
   - **Target Vendor (`target_vendor`):** `cfg!(target_vendor = "pc")`, `cfg!(target_vendor = "unknown")`, `cfg!(target_vendor = "apple")`, etc.
   - **Target OS (`target_os`):** `cfg!(target_os = "windows")`, `cfg!(target_os = "linux")`, `cfg!(target_os = "macos")`, or `std::env::consts::OS`.
   - **Target ABI / Environment (`target_env`):** `cfg!(target_env = "gnu")`, `cfg!(target_env = "msvc")`, `cfg!(target_env = "musl")`, or empty string for targets without an explicit environment tag.
   - **Pointer Width:** `std::mem::size_of::<*const ()>() * 8` (64 on 64-bit targets, 32 on 32-bit targets).
   - **Reliably Recognized Target Combinations:**
     The test explicitly recognizes only target combinations that can be unambiguously identified from these compile-time attributes and compared against the C compiler's target machine:
     - `x86_64-pc-windows-gnu` (`target_arch = "x86_64"`, `target_os = "windows"`, `target_env = "gnu"`, pointer width = 64)
     - `x86_64-unknown-linux-gnu` (`target_arch = "x86_64"`, `target_os = "linux"`, `target_env = "gnu"`, pointer width = 64)
     - `x86_64-pc-windows-msvc` (`target_arch = "x86_64"`, `target_os = "windows"`, `target_env = "msvc"`, pointer width = 64)
     - `aarch64-unknown-linux-gnu` (`target_arch = "aarch64"`, `target_os = "linux"`, `target_env = "gnu"`, pointer width = 64)
     - `aarch64-apple-darwin` (`target_arch = "aarch64"`, `target_os = "macos"`, pointer width = 64)
   - **Ambiguity & Mismatch Handling:**
     If an exact or sufficiently reliable target identity cannot be established from these compile-time attributes, the test MUST NOT guess or infer an arbitrary target triple; it MUST report:  
     `Blocked / Skipped (Cannot determine reliable Rust target identity from compile-time attributes)`  
     and exit cleanly without asserting parity. Parity is never reported as passed under ambiguous target identity.

2. **C Compiler Selection & Target Matching:**
   - The test selects a C compiler by inspecting the `CC` or `TARGET_CC` environment variable.
   - If unset, the test probes for standard compilers in PATH (`gcc`, then `clang`).
   - If no C compiler is available or executable, the test marks the layout check as:
     `Blocked / Skipped (Missing C compiler: CC not set and gcc/clang not found on PATH)`.
     Parity is **never reported as passed**.
   - **Target Machine Verification:** The test invokes `$CC -dumpmachine` to inspect the C compiler's target machine string (e.g. `x86_64-w64-mingw32` for MinGW-w64 GCC on Windows, `x86_64-linux-gnu` on Linux).
   - **Target Compatibility Validation:**
     - Architecture check: Both must match (e.g. `x86_64` vs `x86_64`).
     - OS family & object format check: Both must match (`windows` / `mingw` vs `windows` PE/COFF, or `linux` vs `linux` ELF).
     - Pointer width check: Both must have identical pointer width. The C reporter explicitly outputs `POINTER_WIDTH:<N>`, and the Rust test asserts `POINTER_WIDTH == std::mem::size_of::<*const ()>() * 8`.
     - ABI environment check: Both must match the toolchain environment where applicable (`gnu` / `mingw` vs GNU, `msvc` vs MSVC, `musl` vs musl). A GNU Rust target (`cfg!(target_env = "gnu")`) must not be paired with an MSVC compiler, nor an MSVC Rust target with a MinGW compiler.
   - **Mismatch Enforcement:** If the C compiler's target differs from the Rust target in architecture, pointer width, OS/object format, or ABI environment (e.g. cross-compiling a 32-bit Rust binary on a 64-bit host with host 64-bit GCC, cross-compiling for Linux from Windows, or pairing a GNU Rust build with an MSVC C compiler):
     - The test **must not silently fall back to an arbitrary host compiler**.
     - The test MUST abort the parity check and record:
       `Blocked / Skipped (Target mismatch: Rust target '<rust_target>' vs C compiler target '<dumpmachine_target>')`.
     - Parity is **never reported as passed** under a target mismatch.

3. **Grounded MSRV & Toolchain Verification:**
   - Inspection of `crates/bornomala-core/Cargo.toml` confirmed `edition = "2021"` with no explicit `rust-version` constraint.
   - Inspection of the active repository toolchain confirmed `rustc 1.99.1 (58cf8f9c3 2026-10-13) (x86_64-pc-windows-gnu)`.
   - The standard library macro `std::mem::offset_of!` was stabilized in Rust 1.77.0. Because the active toolchain is 1.99.1 (well above 1.77.0), `std::mem::offset_of!` is natively supported without requiring external dependencies (such as `memoffset`).
   - The test specification establishes Rust $\ge$ 1.77.0 as a hard prerequisite for compiling `c_abi_layout_tests.rs`.

4. **C Layout Reporter (`report_c_layout.c`):**
   A dedicated C file (`crates/bornomala-core/tests/c_abi/report_c_layout.c`) using standard `sizeof` and `<stddef.h>`'s `offsetof`:
   ```c
   #include "bornomala.h"
   #include <stdio.h>
   #include <stddef.h>

   int main(void) {
       printf("POINTER_WIDTH:%zu\n", sizeof(void*) * 8);

       printf("SIZE:BornoEventC=%zu\n", sizeof(BornoEventC));
       printf("OFFSET:BornoEventC.event_type=%zu\n", offsetof(BornoEventC, event_type));
       printf("OFFSET:BornoEventC.char_codepoint=%zu\n", offsetof(BornoEventC, char_codepoint));
       printf("OFFSET:BornoEventC.string_payload=%zu\n", offsetof(BornoEventC, string_payload));
       printf("OFFSET:BornoEventC.index_payload=%zu\n", offsetof(BornoEventC, index_payload));
       printf("OFFSET:BornoEventC.mode_payload=%zu\n", offsetof(BornoEventC, mode_payload));

       printf("SIZE:BornoStatusC=%zu\n", sizeof(BornoStatusC));
       printf("OFFSET:BornoStatusC.name=%zu\n", offsetof(BornoStatusC, name));
       printf("OFFSET:BornoStatusC.version=%zu\n", offsetof(BornoStatusC, version));
       printf("OFFSET:BornoStatusC.active_mode=%zu\n", offsetof(BornoStatusC, active_mode));
       printf("OFFSET:BornoStatusC.ready=%zu\n", offsetof(BornoStatusC, ready));

       printf("SIZE:BornoStateC=%zu\n", sizeof(BornoStateC));
       printf("OFFSET:BornoStateC.preedit=%zu\n", offsetof(BornoStateC, preedit));
       printf("OFFSET:BornoStateC.raw_buffer=%zu\n", offsetof(BornoStateC, raw_buffer));
       printf("OFFSET:BornoStateC.cursor_pos=%zu\n", offsetof(BornoStateC, cursor_pos));
       printf("OFFSET:BornoStateC.candidates_count=%zu\n", offsetof(BornoStateC, candidates_count));
       printf("OFFSET:BornoStateC.candidates=%zu\n", offsetof(BornoStateC, candidates));
       printf("OFFSET:BornoStateC.selected_candidate_index=%zu\n", offsetof(BornoStateC, selected_candidate_index));
       printf("OFFSET:BornoStateC.is_composing=%zu\n", offsetof(BornoStateC, is_composing));
       printf("OFFSET:BornoStateC.mode=%zu\n", offsetof(BornoStateC, mode));

       printf("SIZE:BornoProcessResultC=%zu\n", sizeof(BornoProcessResultC));
       printf("OFFSET:BornoProcessResultC.state=%zu\n", offsetof(BornoProcessResultC, state));
       printf("OFFSET:BornoProcessResultC.committed_text=%zu\n", offsetof(BornoProcessResultC, committed_text));
       printf("OFFSET:BornoProcessResultC.cancelled=%zu\n", offsetof(BornoProcessResultC, cancelled));
       printf("OFFSET:BornoProcessResultC.handled=%zu\n", offsetof(BornoProcessResultC, handled));
       return 0;
   }
   ```

5. **Host Execution Confirmation & Output Comparison:**
   - Locates `report_c_layout.c` via `std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/c_abi/report_c_layout.c")`.
   - Compiles `report_c_layout.c` to a temporary executable using the verified target C compiler. If compilation fails, the test records: `FAILED: report_c_layout.c failed to compile with <compiler>`.
   - Attempts execution of the compiled binary on the host:
     - If host execution fails to start (e.g. cross-compiled binary cannot run on host, returning OS error 193 or `ENOEXEC`):
       - The test records: `Blocked / Skipped (Host cannot execute cross-target binary)`.
       - Parity is **never reported as passed** when execution was blocked.
     - If execution succeeds (exit status 0), the test parses stdout lines.
   - Generates expected values dynamically using Rust's `size_of::<T>()` and `offset_of!(T, field)` for each struct member.
   - Compares lines. Any mismatch fails the test with an exact diff.
   - Parity is **never reported as passed** if the comparison did not execute.

---

### 6.3 Level 3: Native Platform Adapter Integration & Regression Gates

#### 6.3.1 Rust Core Regression Suite & Test-Count Baseline
```bash
cargo test -p bornomala-core
```
- **Pre-Existing Baseline (WP-03):** Verified at **10 test targets and 251 tests passed**.
- **Expansion & Regression Reporting Requirements:**
  1. **Baseline Proof:** The implementation report must provide executable evidence that all 251 pre-existing baseline tests across the 10 pre-existing targets continue to pass with 0 regressions.
  2. **Observed Totals Accounting:** After adding `c_abi_layout_tests.rs`, the test suite expands to an 11th test target. The completion report must document the exact, actual test-target count and total test count observed during test execution.
  3. **No Speculative Pre-Implementation Counts:** The test runner must not hardcode or anticipate an expected new-test count before running the tests.
  4. **Separate Reporting Breakdown:** The report must separately enumerate:
     - The pre-existing 251 passing tests across 10 targets.
     - Any new tests executed within `c_abi_layout_tests.rs` with the exact count observed.
     - Any blocked or skipped checks (e.g. if the target C compiler is unavailable or cross-target execution cannot run on host).
     - Any working-tree fixture side effects (preserving `test_export_fixtures` LF/CRLF JSON file modifications).
- **Fixture Side-Effect Preservation:** Running `cargo test -p bornomala-core` executes `test_export_fixtures` in `builtin.rs`, touching reference JSON fixtures (`bijoy_reference.json`, `national_reference.json`, `probhat_reference.json`) with Windows line-ending differences. In accordance with `AGENTS.md` §1.4, these working-tree changes must be preserved and must not be reverted.

#### 6.3.2 Windows TSF Consumer Compatibility (Safe Compile/Link Regression)

Verification of Windows TSF consumer compatibility confirms that the existing C++ adapter sources (`borno_tsf.cpp`, `candidate_window.cpp`, `lang_bar_item.cpp`, `key_mapper.h`) compile and link cleanly against the updated canonical header `bornomala.h` and the dynamically resolved Rust static library `libbornomala_core.a`.

**Safety Hazard Identification & Execution Restriction:**
- Inspection of `native/borno-tsf/build.ps1` and `native/borno-tsf/test_harness.cpp` confirms that `build.ps1` executes `test_harness.exe` unconditionally upon completion of compilation.
- In `test_harness.cpp` (lines 254, 273), Level 4 invokes `DllRegisterServer()` and subsequently `DllUnregisterServer()`.
- `DllUnregisterServer()` deletes both `HKLM` and `HKCU` COM registration keys and unregisters the `CLSID_BornoTextService` profile from the active operating system.
- Because the existing installed BornoMala IME shares this exact CLSID, executing `build.ps1` as a routine verification command silently deletes the developer's installed system IME registration (Audit Finding P0-8).
- Additionally, `build.ps1` writes directly to `native/borno-tsf/bornomala_tsf.dll`, which encounters fatal file-locking errors if the active Windows Text Services Framework or an open host application has loaded the DLL.
- **Strict Execution Restriction:** `native/borno-tsf/build.ps1` and `test_harness.exe` **must NOT be executed** during WP-04 verification. Furthermore, `build.ps1` must not be modified as part of WP-04. Full runtime integration and elevated registration testing remain strictly decoupled from this header-unification work package.

**Safe Two-Stage Verification Procedure (Isolated Compilation & Static Linkage):**
To verify consumer compatibility safely without modifying existing build scripts, risking file locks, or touching system registry entries, verification is partitioned into two distinct stages preceded by a strict compiler target compatibility gate, grounded in the exact flags from `native/borno-tsf/build.ps1:51-58`:

1. **Explicit Build Parameter Derivation & Isolated Directory:**
   Establish the exact build parameters matching the Cargo build invocation, derive the static archive location, and create the isolated test directory:
   ```powershell
   # 1. Explicit build parameters (must match the cargo build invocation)
   $profile = "release" # Must match Cargo profile ("release" or "debug")
   $targetTriple = "x86_64-pc-windows-gnu" # Actual Rust target triple used for the build

   if (-not $targetTriple -or -not $profile) {
       Write-Warning "Build parameters cannot be established."
       # Mark Stage A and Stage B as:
       # Blocked / Not run (Cannot establish build parameters or target triple)
       return
   }

   # 2. Query authoritative Cargo target directory
   $targetDir = (cargo metadata --format-version 1 --no-deps | ConvertFrom-Json).target_directory
   if ($env:CARGO_TARGET_DIR) {
       if ([System.IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR)) {
           $targetDir = $env:CARGO_TARGET_DIR
       } else {
           $targetDir = Join-Path $PWD $env:CARGO_TARGET_DIR
       }
   }

   # 3. Derive archive path from identical parameters
   $staticLib = Join-Path $targetDir "$targetTriple\$profile\libbornomala_core.a"
   $isolatedDir = Join-Path $targetDir "tsf_compat_test"
   New-Item -ItemType Directory -Force -Path $isolatedDir | Out-Null
   ```

2. **Strict Windows TSF Target Compatibility Gate (Pre-Stage Check):**
   Before invoking G++ for either Stage A or Stage B, verify compiler availability and query its reported target machine via `g++ -dumpmachine`:
   ```powershell
   # 1. Probe for G++ compiler on PATH
   $gppCmd = Get-Command "g++.exe" -ErrorAction SilentlyContinue
   if (-not $gppCmd) {
       # Mark Stage A and Stage B as:
       # Blocked / Not run (Missing prerequisite: g++.exe not found on PATH)
       Write-Warning "MinGW-w64 G++ compiler not found on PATH."
       return
   }

   # 2. Strict Windows GNU Rust target requirement:
   # Reject Linux, MSVC, Darwin, and any non-Windows-GNU targets
   if ($targetTriple -notmatch "^[a-zA-Z0-9_]+-(pc|w64)-windows-gnu$") {
       # Mark Stage A and Stage B as:
       # Blocked / Not run (Unsupported Rust target for Windows TSF: '$targetTriple'; requires supported Windows GNU target such as x86_64-pc-windows-gnu)
       Write-Warning "Unsupported target '$targetTriple': Windows TSF requires a Windows GNU target (e.g. x86_64-pc-windows-gnu)."
       return
   }

   # 3. Query G++ compiler target machine string
   $gppMachine = (& g++.exe -dumpmachine).Trim()

   # 4. Strict multi-property target matching:
   # Must match Rust target architecture, Windows PE/COFF OS format, and GNU/MinGW ABI environment
   $rustArch = ($targetTriple -split "-")[0] # e.g. "x86_64"

   # Architecture parity: Must match exact architecture (architecture alone is NOT sufficient)
   $isArchMatch = ($gppMachine -match "^$rustArch-")
   # OS/Object format & ABI parity: Must target MinGW Windows (PE/COFF format with GNU runtime)
   $isWindowsGnuMatch = ($gppMachine -match "-(w64|pc)-mingw32") -or ($gppMachine -match "-mingw32")

   if (-not ($isArchMatch -and $isWindowsGnuMatch)) {
       # Incompatible toolchain target:
       # MUST NOT silently fall back to an arbitrary host compiler or link an archive built for a different target
       # Mark Stage A and Stage B as:
       # Blocked / Not run (Target mismatch: G++ target '$gppMachine' incompatible with Rust target '$targetTriple')
       Write-Warning "Target mismatch: G++ target '$gppMachine' is incompatible with Rust target '$targetTriple'."
       return
   }
   ```

3. **Stage A — Isolated C++ Translation Unit Compilation (`-c`):**
   Separately compile each TSF C++ source file into an isolated object file (`.o`) with strict warnings, verifying the exit code of every command. Note: `-Wno-unused-parameter` and `-Wno-missing-field-initializers` are applied because standard Win32 COM callback method overrides (`ITfTextInputProcessorEx`, `ITfThreadMgrEventSink`, `ITfKeyEventSink`) and C structs (`MONITORINFO`) in the existing adapter source declare interface parameters and fields unused by this implementation:
   ```powershell
   $stageAPassed = $true
   $tsfSources = @(
       @{ Src = "native/borno-tsf/borno_tsf.cpp"; Obj = "$isolatedDir/borno_tsf.o" },
       @{ Src = "native/borno-tsf/candidate_window.cpp"; Obj = "$isolatedDir/candidate_window.o" },
       @{ Src = "native/borno-tsf/lang_bar_item.cpp"; Obj = "$isolatedDir/lang_bar_item.o" }
   )

   foreach ($tu in $tsfSources) {
       & g++ -c -std=c++17 -Wall -Wextra -Werror -pedantic `
           -Wno-unused-parameter -Wno-missing-field-initializers `
           -Icrates/bornomala-core/include -Inative/borno-tsf `
           $tu.Src -o $tu.Obj

       if ($LASTEXITCODE -ne 0) {
           Write-Error "Stage A compilation failed on $($tu.Src) with exit code $LASTEXITCODE."
           $stageAPassed = $false
           break
       }
   }

   if (-not $stageAPassed) {
       # Stage A FAILED; Stage B MUST NOT RUN
       Write-Warning "Stage B static linkage skipped because Stage A compilation failed."
       # Record:
       # Stage A: FAILED (Compilation error)
       # Stage B: Blocked / Skipped (Prerequisite failed: Stage A compilation did not pass)
       return
   }
   # Record Stage A as: PASSED
   ```

4. **Stage B — Isolated Static Linkage (`test_bornomala_tsf.dll`):**
   Execute static linkage ONLY when Stage A has passed, verifying archive existence and checking linker exit code:
   ```powershell
   # Prerequisite: Verify static archive exists before linking
   if (-not (Test-Path $staticLib)) {
       # Mark Stage B as:
       # Blocked / Not run (Missing prerequisite: static archive not found at '$staticLib')
       return
   }

   & g++ -shared -std=c++17 -static -static-libgcc -static-libstdc++ `
       "$isolatedDir/borno_tsf.o" `
       "$isolatedDir/candidate_window.o" `
       "$isolatedDir/lang_bar_item.o" `
       native/borno-tsf/borno_tsf.def `
       "$staticLib" `
       -o "$isolatedDir/test_bornomala_tsf.dll" `
       -lole32 -loleaut32 -luuid -ladvapi32 -luser32 -lgdi32 -luserenv -lws2_32 -lntdll -lbcrypt

   if ($LASTEXITCODE -ne 0) {
       # Record Stage B as: FAILED (Link error, exit code $LASTEXITCODE)
   } else {
       # Record Stage B as: PASSED (Clean static linkage)
   }
   ```

5. **Verification Invariants & Independent Reporting:**
   - **Target Parity Enforcement:** Verifies G++ compiler target against the full Rust target triple; rejects Linux, MSVC, or incompatible architectures without silent fallback.
   - **Strict Stage Dependencies:** Stage A checks the exit code of every compilation command; Stage B strictly does not run if Stage A failed.
   - **Separate Reporting:** Compilation (Stage A) and static linkage (Stage B) MUST be reported as separate verification results.
   - **No Execution:** Does **not** execute `test_harness.exe` or the compiled DLL.
   - **No Script Invocation:** Does **not** execute `native/borno-tsf/build.ps1`.
   - **No In-Tree DLL Overwrite:** Does **not** overwrite `native/borno-tsf/bornomala_tsf.dll` or `borno_tsf.dll`.
   - **Zero Registry Impact:** Does **not** invoke `DllRegisterServer()`, `DllUnregisterServer()`, or any COM/registry APIs.
   - **No IME Modification:** Does **not** register, unregister, install, remove, or replace the active system IME.
6. **Environmental Limitations:**
   If MinGW-w64 G++ is unavailable on PATH, `$staticLib` is missing/incompatible, compiler target machine mismatches the Rust target, or a safe build cannot be performed without modifying `build.ps1` or risking the active IME, Stage A and/or Stage B MUST be reported as:
   `Blocked / Not run (Missing prerequisite: G++ toolchain unavailable, target mismatch, or safe build without build.ps1 modification blocked)`. Under no circumstances should Stage A and Stage B be combined, approximated, or reported as passed when blocked.

---

## 7. Risk Assessment & Mitigations

| Risk | Likelihood | Impact | Mitigation Strategy |
|---|---|---|---|
| Break external consumers including `borno.h` | Low | High | Forwarding header retains `borno.h` at identical filesystem path with full API exposure. |
| Preprocessor guard recursion or loop | Negligible | Low | Distinct guard names (`BORNO_LEGACY_FORWARDING_H` vs `BORNOMALA_H`) prevent recursion. |
| Incompatible include path in existing build scripts | Low | Medium | Both headers remain in `crates/bornomala-core/include/`. Include paths like `-Iinclude` continue to resolve both names. |
| Struct layout discrepancy across compilers | Low | High | Parity verification compares `sizeof`/`offsetof` directly between GCC and Rust `#[repr(C)]` for the active target using normalized output. |
| Accidental git discarding of fixture files | Medium | Low | Explicit rule in WP-04 specification warning about `test_export_fixtures` LF/CRLF serialization side effects. |
| Missing C/C++ compiler or locked DLL misclassified as header bug | Low | Medium | Strict separation between Level 1/2 header verification and Level 3 platform-adapter verification. |

---

## 8. Acceptance Criteria (Definition of Done)

The final completion report must record separate outcomes for each criterion. A blocked environment-dependent check must not be reported as passed:

- [ ] **AC-1 (Authoritative Canonical Header Structure):** `crates/bornomala-core/include/bornomala.h` has `@file bornomala.h`, `#ifndef BORNOMALA_H`, backward-compatibility `#define BORNO_H`, documented out-parameter lifecycle notes, and contains the full C-ABI surface (including `-5`, `-6`, and `borno_engine_configure_json`).
- [ ] **AC-2 (Forwarding Legacy Header Structure):** `crates/bornomala-core/include/borno.h` is a clean, documented forwarding header including `bornomala.h` under `#ifndef BORNO_LEGACY_FORWARDING_H`.
- [ ] **AC-3 (C99 Inclusion Order & Declarations — Level 1):** Compiling C translation units with both inclusion orders (`borno.h` $\to$ `bornomala.h` and `bornomala.h` $\to$ `borno.h`) and repeated inclusions succeeds with zero errors and zero warnings under `gcc -std=c99 -Wall -Wextra -Werror -pedantic -c`.
- [ ] **AC-4 (C++17 Declaration Visibility — Level 1):** Compiling C++ translation units with both inclusion orders and typed function-pointer assignment to `borno_engine_configure_json` succeeds with zero errors and zero warnings under `g++ -std=c++17 -Wall -Wextra -Werror -pedantic -c`.
- [ ] **AC-5 (Target-Aware C Linkage Against Static Library — Level 2):** Linking `test_c_linkage.c` against the dynamically resolved `libbornomala_core.a` (<resolved_archive_path>, distinguishing host-default `<target_dir>/<profile>/libbornomala_core.a` from explicit target `<target_dir>/<triple>/<profile>/libbornomala_core.a`) succeeds on the active target, resolving `borno_engine_configure_json`, and runs to clean exit (0). If the static library is missing or target-mismatched, it must be explicitly reported as `Blocked / Not run (Missing prerequisite: static archive not built for target <triple> and profile <profile>)`, never as passed.
- [ ] **AC-6 (Target-Aware C++ Linkage & `extern "C"` Symbol Resolution — Level 2):** Linking `test_cpp_linkage.cpp` against the dynamically resolved `libbornomala_core.a` (<resolved_archive_path>) succeeds, confirming unmangled C++ symbol resolution, and runs to clean exit (0). (Or explicitly reported as `Blocked / Not run`).
- [ ] **AC-7 (Target-Aware C/Rust Struct Layout Parity — Level 2):** `c_abi_layout_tests.rs` executes `report_c_layout.c` and proves byte-for-byte size and member offset parity against Rust `#[repr(C)]` types for the active target architecture. If the target C compiler is unavailable, targets mismatch, or host cannot execute the target binary, it must be explicitly reported as `Blocked / Skipped`, never as passed.
- [ ] **AC-8 (Documentation Alignment):** `docs/engine-contract.md` (§7) accurately documents `bornomala.h`, `borno.h` forwarding, error codes `-5`/`-6`, string borrowing, schema migration, state preservation, and `borno_engine_configure_json()`.
- [ ] **AC-9 (Rust Core Baseline Regression Gate & Expansion Reporting):** All 10 pre-existing test targets pass (executable evidence that all 251 baseline tests continue to pass with 0 regressions). The actual test-target count and total test count observed after adding `c_abi_layout_tests.rs` are recorded without speculating or hardcoding new-test counts in advance. New tests, any blocked checks, and tracked fixture side effects (`test_export_fixtures` LF/CRLF JSON files) are reported separately.
- [ ] **AC-10 (Windows TSF Consumer Compatibility — Safe Compile/Link Regression):** The existing Windows TSF C++ adapter sources (`borno_tsf.cpp`, `candidate_window.cpp`, `lang_bar_item.cpp`) compile and link cleanly against the canonical header `bornomala.h` and the dynamically resolved static library `libbornomala_core.a` (<resolved_archive_path>) using MinGW-w64 G++ without errors or warnings. Before compiling or linking, the test runner establishes the actual Rust target triple (requiring a supported Windows GNU target, such as `x86_64-pc-windows-gnu`) and compares it against `g++ -dumpmachine`; if the compiler target does not match the Rust target's architecture, OS/object format, and ABI environment, or if target identity cannot be established unambiguously, the check is marked `Blocked / Not run` without silent fallback to another compiler or linking cross-target archives. Verification enforces strict stage dependencies: **Stage A (C++ Compilation)** compiles each translation unit into an object file (`.o`) with `-std=c++17 -Wall -Wextra -Werror -pedantic -Wno-unused-parameter -Wno-missing-field-initializers -Icrates/bornomala-core/include -Inative/borno-tsf -c` and checks every command exit code; Stage A passes only if all units compile successfully. **Stage B (Static Linkage)** executes ONLY if Stage A passed, linking object files, `borno_tsf.def`, and `$staticLib` into `<isolated_dir>/test_bornomala_tsf.dll` with system libraries, passing only if the linker exits successfully. To protect the developer's installed IME and avoid file-locking conflicts, verification never executes `test_harness.exe` or `build.ps1`, never overwrites in-tree or active DLLs, and executes zero registry or COM registration operations (`DllRegisterServer`/`DllUnregisterServer`). Stage A and Stage B outcomes are reported separately. (If the toolchain is unavailable or a safe build cannot be performed without risking the active IME, explicitly reported as `Blocked / Not run`).
