# RV-02 — Contract & C ABI Hardening

> **Revision ID:** RV-02
> **Status:** Proposed — awaiting developer approval
> **Priority:** P0 (blocking)
> **Estimated effort:** 3–4 days
> **Dependencies:** RV-01 (engine semantics fixed), RV-01.2 (cursor units known)
> **Blocks:** RV-03, RV-04, RV-05, RV-07

---

## 1. Why this revision exists

Three hand-maintained copies of the engine contract already diverge, the C ABI has an allocator-contract bug and no thread/panic contract, and the public header is forked:

| ID | Defect | Evidence |
|---|---|---|
| P0/High | `Vec<*mut c_char>` leaked with only `len`; freed with `Vec::from_raw_parts(ptr, len, len)` — capacity not guaranteed | [`ffi.rs:573-576`](../../crates/bornomala-core/src/ffi.rs#L573-L576), [`ffi.rs:173`](../../crates/bornomala-core/src/ffi.rs#L173) |
| High | No documented or enforced thread-safety for `BornoEngineHandle` | [`ffi.rs:63-66`](../../crates/bornomala-core/src/ffi.rs#L63-L66) |
| High | Panic containment returns an error but leaves half-updated state; panic text discarded | [`ffi.rs:207`–`461`](../../crates/bornomala-core/src/ffi.rs#L207) |
| High | `include/borno.h` shares `BORNO_H` guard with `bornomala.h`, lacks `configure_json` and `-5/-6` | [`borno.h:11-12`](../../crates/bornomala-core/include/borno.h#L11-L12) |
| Medium | Candidate kinds fetched by a second live-handle call → can race the snapshot | [`ffi.rs:473-492`](../../crates/bornomala-core/src/ffi.rs#L473-L492) |
| Medium | Out-params have undocumented initialisation/free requirements | [`bornomala.h:236-249`](../../crates/bornomala-core/include/bornomala.h#L236-L249) |
| Medium | Interior NUL silently becomes an empty string | [`ffi.rs:540-542`](../../crates/bornomala-core/src/ffi.rs#L540-L542) |
| Medium | Test-only helpers exported in the production ABI | [`ffi.rs:498-524`](../../crates/bornomala-core/src/ffi.rs#L498-L524) |

The ABI is consumed by two native adapters and the Rust desktop backend. Every change here must land in all three consumers in the same revision.

---

## 2. Current-state inspection

Read:

- `crates/bornomala-core/src/ffi.rs` (all)
- `crates/bornomala-core/include/bornomala.h` and `include/borno.h`
- `crates/bornomala-core/tests/ffi_tests.rs`
- `native/borno-tsf/key_mapper.h`, `borno_tsf.cpp` (FFI usage)
- `native/bornomala-fcitx5/engine.h`, `ut8_utils.h` (FFI usage)
- `docs/engine-contract.md`

Inventory every exported symbol (`Select-String -Path ... -Pattern 'pub unsafe extern "C" fn|no_mangle'`) and every consumer call site. Produce a call-site map before changing a signature.

Baseline:

```powershell
cargo test -p bornomala-core --test ffi_tests
cargo test --workspace
```

---

## 3. Scope

### Goals

1. **ABI v2** with correct allocation, self-contained snapshots, explicit thread and panic contracts, and a documented initialisation protocol.
2. **One header**, generated or drift-checked, with an explicit ABI version.
3. **One contract source** for the event/state types; C and TypeScript outputs derived from it.
4. Both native adapters and the desktop backend migrated in the same revision.

### Non-goals

- No new engine features.
- No change to `EngineEvent` semantics (only representation/codegen).
- No UI redesign.

### Explicit no-go

- **No** silent ABI break. `BORNO_ABI_VERSION` must be bumped and every consumer recompiled.
- **No** keeping both `borno.h` and `bornomala.h` “for compatibility”.
- **No** `Vec::from_raw_parts` with an assumed capacity.
- **No** C++ exceptions crossing the ABI boundary.
- **No** new dependency for codegen without a written rationale (cbindgen/ts-rs are acceptable; record licence and maintenance).

---

## 4. Design decisions

### 4.1 Allocation model (choose one)

**Option A — caller-owned buffers (recommended for an IME):**
`borno_engine_process_event` fills a caller-provided `BornoProcessResultC` whose candidate array is allocated by the caller from `candidates_count` returned by a sizing call. Two-call protocol: `borno_state_measure` → allocate → `borno_state_fill`.

**Option B — library-owned with exact metadata:**
`BornoStateC` gains `candidates_capacity: usize`; free uses it. Simpler diff, still an ABI break.

Pick one, document it in the header, and add a test that allocates/frees 10,000 times.

### 4.2 Thread contract

- Document: “A `BornoEngineHandle` must not be used concurrently. One handle per input context/thread.”
- Add an `AtomicBool` busy guard that returns `BORNO_ERR_BUSY` on concurrent entry (cheap, prevents UB-by-misuse).
- Add a two-thread stress test that asserts no UB under the guard (either serialised by the caller or rejected).

### 4.3 Panic contract

- Build the new composition into locals; commit to `self` only on success.
- Store `last_error: Option<String>`; expose `borno_engine_last_error()`.
- Keep `catch_unwind`, but after a panic reset the engine to a clean idle state rather than leaving mismatched buffers.

### 4.4 Snapshot completeness

Add `candidate_kinds: *mut i32` (or a packed struct) to the same snapshot as `candidates`, so kinds cannot race. Remove the second-call accessor or mark it deprecated.

### 4.5 Initialisation and free

- Document “zero-initialise before first use”.
- Provide `borno_state_init(BornoStateC*)`, `borno_process_result_init(...)`.
- Free functions must be safe on a zeroed struct and idempotent.

### 4.6 ABI version

```c
#define BORNO_ABI_VERSION 2
const uint32_t borno_abi_version(void);
```

Adapters assert the version at load and refuse to run (with a clear log) on mismatch.

### 4.7 Codegen

- C header from the Rust contract with `cbindgen` (build-time, dev-dependency only).
- TypeScript from the same contract with `ts-rs` (dev-dependency), emitted to `src/generated/contract.ts`.
- CI check: regenerate and `git diff --exit-code`.

If codegen is rejected, the fallback is a **contract test** that parses the header and the TS file and asserts every field/variant exists with the same name and shape. This is weaker but acceptable.

---

## 5. Work packages

### WP-2.1 — Contract inventory and call-site map

Produce `docs/design/abi-call-sites.md` listing every export, every consumer, and the exact line. No code changes.

### WP-2.2 — ABI v2 types and functions

Implement §4.1–4.6 in `ffi.rs` and the header. Delete `borno.h` (or turn it into `#include "bornomala.h"` with `#ifndef BORNO_LEGACY_H`). Remove `borno_ffi_echo_utf8` and `IsResultNull` from the public surface (feature-gate the echo helper for tests if needed).

### WP-2.3 — Migrate consumers

- `native/borno-tsf`: update every FFI call; add version check; free results on every path (including the early returns at `:690`/`:696`).
- `native/bornomala-fcitx5`: same; update `engine.h`.
- `src-tauri`: the desktop uses the Rust API, but `config_store` and any FFI tests must be checked.

### WP-2.4 — FFI safety tests

- null/zeroed/invalid inputs for every entry point,
- double free and free-after-error,
- interior NUL rejection,
- concurrent entry returns `BORNO_ERR_BUSY`,
- panic path leaves idle state and sets `last_error`,
- 10,000 alloc/free iterations.

### WP-2.5 — Codegen (or drift test)

Implement §4.7. Commit generated files and the CI check.

---

## 6. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | No `Vec::from_raw_parts` with assumed capacity remains | grep |
| AC-2 | Header has `BORNO_ABI_VERSION` and every consumer checks it | grep + test |
| AC-3 | One header only; `borno.h` removed or forwarding | tree |
| AC-4 | Concurrent use returns an error, never corrupts | stress test |
| AC-5 | Panic leaves a clean idle state and exposes `last_error` | test |
| AC-6 | Snapshot contains candidates and kinds atomically | test |
| AC-7 | Generated C/TS files match the contract in CI | `git diff --exit-code` |
| AC-8 | Both adapters compile and their smoke tests pass | build logs |
| AC-9 | `cargo test --workspace` green | output |

---

## 7. Testing plan

```powershell
cargo test -p bornomala-core --test ffi_tests
cargo test --workspace
cargo test -p bornomala-core --features test-abi
pwsh -File native/borno-tsf/build.ps1 -NoRun   # build only, no registration
```

Windows adapter smoke: build the DLL, run `test_harness.exe` **without** the registration step (see RV-04). Linux: build via the RV-05 fixed CMake path.

If Miri/ASAN is available on the toolchain, run the FFI unit tests under it; otherwise document the limitation and rely on the stress test. Do not claim sanitizer coverage without running it.

---

## 8. Runnable checkpoint

- Both adapters build.
- Sandbox launches.
- FFI tests green.
- Old symbols removed; no deprecated duplicate paths.

Rollback: the ABI bump is one commit; revert it and revert the adapters together.

---

## 9. Evidence and reporting

`docs/evidence/RV-02/<date>-abi-v2.md`:

- call-site map,
- before/after struct layouts with `offsetof`/`sizeof` assertions,
- stress and concurrency results,
- generated-file diff check output,
- adapter build logs.

---

## 10. Roadmap update

- Mark M1 (engine contract) as re-verified against ABI v2.
- Update `docs/engine-contract.md` to v2.0.0 with the full C section and the ownership rules.
- Record the ABI version in `README.md`.

---

## 11. Stop conditions

- ABI v2 forces a UI change beyond the generated types → stop and split.
- A consumer cannot be migrated without behaviour change → stop and document.
- Codegen adds unacceptable build complexity → fall back to the drift test and record the decision.

---

## 12. Handoff

The ABI is now a stable, versioned contract. RV-03 can rely on `borno_engine_configure_json`; RV-07 can rely on generated types instead of hand copies.
