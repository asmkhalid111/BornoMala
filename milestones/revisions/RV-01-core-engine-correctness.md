# RV-01 — Core Engine Correctness

> **Revision ID:** RV-01
> **Status:** Proposed — awaiting developer approval
> **Priority:** P0 (blocking)
> **Estimated effort:** 2–3 days (excluding sub-files)
> **Dependencies:** none
> **Sub-instructions:** [`RV-01.1-candidate-integrity.md`](RV-01.1-candidate-integrity.md), [`RV-01.2-cursor-editing-safety.md`](RV-01.2-cursor-editing-safety.md)
> **Blocks:** RV-02, RV-06, RV-07

---

## 1. Why this revision exists

The core engine is architecturally sound but ships four correctness defects that make downstream verification meaningless:

| ID | Defect | Audit ref |
|---|---|---|
| P0-1 | Hard-coded candidate answers in production code | §5.1 P0-1 |
| P0-2 | Byte/char cursor conflation | §5.1 P0-2 |
| C-DICT-1 | Dictionary loaded (and `expect`-panicked) on first keystroke even when disabled | §5.2 |
| C-API-1 | `configure()` silently ignores presentation fields and clamps instead of rejecting | §5.2 |

It also inherits two structural problems that must be *measured* now and fixed in RV-07: a 1,565-line `lib.rs` and three duplicated commit paths.

This revision fixes the four defects and records the structural baseline. It does **not** refactor the engine.

---

## 2. Current-state inspection

Read:

- `crates/bornomala-core/src/lib.rs` (structure map: list every `fn` with its line range before editing)
- `crates/bornomala-core/src/config.rs`
- `crates/bornomala-core/src/dictionary/mod.rs`
- `crates/bornomala-core/src/phonetic/candidates.rs`
- `crates/bornomala-core/tests/engine_config_tests.rs`
- `crates/bornomala-core/tests/dictionary_tests.rs`

Baseline:

```powershell
cargo test --workspace
cargo run -p bornomala-desktop --bin persistence_suite
```

Record exact counts. Expected at audit commit: 232 + 8.

---

## 3. Scope

### Goals

1. Execute RV-01.1 and RV-01.2 to completion.
2. Make dictionary loading lazy **and** conditional; never panic on corrupt lexicon data.
3. Make `configure()` validate, return a `Result`, and expose the presentation fields it receives.
4. Record a structural baseline (function map, duplication list, test inventory) for RV-07.

### Non-goals

- No module split, no rename, no new engine features.
- No ABI change (RV-02).
- No UI change.
- No new rules or dictionary entries.

### Explicit no-go

- **No** “fix the fixture by adding a rule” (see RV-01.1).
- **No** changing `EngineState` fields; additive fields require RV-02 coordination.
- **No** performance optimisation hidden inside a correctness fix.

---

## 4. Work packages

### WP-1.1 — Candidate integrity

Execute [`RV-01.1-candidate-integrity.md`](RV-01.1-candidate-integrity.md) in full. Gate: its AC-1…AC-7.

### WP-1.2 — Cursor and editing safety

Execute [`RV-01.2-cursor-editing-safety.md`](RV-01.2-cursor-editing-safety.md) in full. Gate: its AC-1…AC-7.

### WP-1.3 — Conditional, non-panicking dictionary load

**Current behaviour:** `update_composition()` calls `DictionaryEngine::global()` unconditionally ([`lib.rs:1379`](../../crates/bornomala-core/src/lib.rs#L1379)); the `OnceLock` builds a trie from a 1.1 MB embedded blob and `expect`s on failure ([`dictionary/mod.rs:18-37`](../../crates/bornomala-core/src/dictionary/mod.rs#L18-L37)).

**Required behaviour:**

1. If `enable_dictionary_assistance == false`, do not touch `DictionaryEngine::global()` at all in `update_composition` (verify with a test that a disabled engine never initialises the global — use an `AtomicBool` test hook or a `OnceLock` inspection helper behind `#[cfg(test)]`; do not add production state for this).
2. `DictionaryEngine::try_global() -> Result<&'static DictionaryEngine, DictionaryError>` replaces the panicking accessor. The engine stores the error and returns an empty candidate set rather than panicking.
3. Add `BornoEngine::warm_up_dictionary()` for the host to call at activation, so the first keystroke does not pay the cost.
4. Add a cold-start benchmark (separate test, `#[ignore]` by default) measuring the first query after construction. Record it in the evidence file.

**Acceptance:**

- A test with a deliberately corrupted lexicon (test-only loader input) returns `Err`, not a panic.
- A test proves disabled assistance never initialises the global.
- The first-keystroke latency is measured before/after warm-up.

### WP-1.4 — Configuration semantics

**Current behaviour:** `configure()` clamps (`candidate_display_limit = 0 → 1`) and ignores `candidate_font_size`, `candidate_orientation`, `active_theme`, `mode_toggle_key`, `default_startup_mode` ([`lib.rs:310-323`](../../crates/bornomala-core/src/lib.rs#L310-L323)).

**Required behaviour:**

1. `pub fn configure(&mut self, config: &BornoConfig) -> Result<(), ConfigError>`.
2. Validate first (`config.validate()`), return the existing `ConfigValidationError` on failure, and leave the engine untouched on error.
3. Apply every field the engine owns: startup/mode policy, candidate limit, dictionary flags, ambiguity budgets, auto-Dari, auto-commit.
4. For fields the engine does **not** own (font size, orientation, theme, toggle key), either:
   - store them in a `PresentationConfig` exposed by a getter, or
   - document explicitly that they are adapter-owned and must be read from the config file.
   Choose one; do not silently drop them.
5. Truncation on a smaller limit stays, but add a test that raising the limit re-queries candidates on the next composition event (documented behaviour, not silent loss).

**Acceptance:**

- `configure()` with an invalid config returns `Err` and leaves state identical (test).
- Every `BornoConfig` field is either applied or explicitly declared adapter-owned in `docs/engine-contract.md`.
- Call sites updated: `src-tauri/src/lib.rs`, both native adapters (they use the C ABI, which still calls the infallible path until RV-02; add a temporary `configure_unchecked` only if needed, documented as deprecated).

### WP-1.5 — Structural baseline (no code change)

Produce `docs/design/core-structure-baseline.md`:

- function map of `lib.rs` with line ranges and responsibilities,
- every duplicated commit/space/virama block with line numbers,
- every place that mutates `raw_buffer`,
- the test inventory by file and count,
- the top five refactor candidates with risk notes.

This becomes the input to RV-07. It is documentation only.

---

## 5. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | RV-01.1 and RV-01.2 verified | their evidence files |
| AC-2 | Disabled dictionary never initialises the global | test |
| AC-3 | Corrupt lexicon returns an error, never panics | test |
| AC-4 | `configure()` is transactional | test |
| AC-5 | Every config field has an owner | doc table |
| AC-6 | All previous tests pass | `cargo test --workspace` |
| AC-7 | Sandbox still launches and types | manual |
| AC-8 | Structural baseline exists | doc |

---

## 6. Testing plan

```powershell
cargo test -p bornomala-core
cargo test --workspace
cargo run -p bornomala-desktop --bin persistence_suite
npm run build
```

Manual:

1. Launch the sandbox, type `ka`, `ami`, `tumi`, `brristi`.
2. Open settings, change candidate limit, theme, orientation, toggle key; confirm the sandbox reflects each.
3. Confirm the native adapter is **not** expected to reflect them yet (RV-03 owns that) and state it in the evidence.

Negative checks:

- Corrupt the embedded lexicon in a test fixture copy; assert `Err`.
- Pass an invalid config; assert the engine state hash is unchanged.

---

## 7. Runnable checkpoint

- Workspace green; sandbox launchable; no ABI change.
- Rollback: each work package is a separate commit.

---

## 8. Evidence and reporting

`docs/evidence/RV-01/<date>-core-correctness.md`, plus the two sub-evidence files. Include the structural baseline link.

---

## 9. Roadmap update

- RV-01 `Verified` only after both sub-revisions and WP-1.3…1.5.
- Update M3 and M5 status honestly if their “Verified” claim depended on the old tests.
- Do not change M7/M8 statuses here (RV-04/RV-05).

---

## 10. Stop conditions

- A config field turns out to be used by the UI in a way that requires an ABI change → stop, route to RV-02.
- The dictionary corruption test cannot be written without production test hooks → propose a `#[cfg(test)]` loader and get approval.
- Any fix requires changing the public `EngineState` shape → stop.

---

## 11. Handoff

The engine is now trustworthy enough for ABI work (RV-02) and for the verification rewrite (RV-06). Structural debt is documented but untouched.
