# RV-07 — Desktop Architecture & Scaling

> **Revision ID:** RV-07
> **Status:** Proposed — awaiting developer approval
> **Priority:** High (Phase 1; unblocks all UI growth)
> **Estimated effort:** 6–9 days
> **Dependencies:** RV-02 (generated contract), RV-06 (test harness)
> **Blocks:** RV-08, RV-10, RV-11

---

## 1. Why this revision exists

The product cannot scale past its current feature set while the engine is one 1,565-line file and the desktop UI is two ~1,400-line components with no state layer.

**Measured scaling debt (audit):**

| Item | Current |
|---|---|
| `crates/bornomala-core/src/lib.rs` | 1,565 lines; 3 duplicated commit paths; 4 duplicated hasanta exceptions |
| `src/components/SandboxTypingArea.tsx` | 1,361 lines; 19 props; 147 inline style objects |
| `src/components/SettingsDrawer.tsx` | 1,454 lines; 9 props; 123 inline styles; 4 duplicated toggle blocks |
| `src/App.tsx` | 13 `useState`; 22 `useCallback`; 44 props across 6 children |
| State management | none (no reducer/context/store/hooks directory) |
| Duplicated key maps | 3 (`types.ts`, `SandboxTypingArea`, `VirtualKeyboard`) |
| Duplicated palette | 2 (`index.css`, `SettingsDrawer`) |
| Frontend tests | 0 |
| Desktop backend tests | 0 (`persistence_suite` is a bin) |

**Rule of thumb for this revision:** a component or module that a single developer cannot hold in their head will produce defects. Target: no UI component over 400 lines, no core module over 600.

---

## 2. Current-state inspection

Read:

- `crates/bornomala-core/src/lib.rs` and `docs/design/core-structure-baseline.md` (from RV-01.5)
- `src/App.tsx`, `src/components/*.tsx`, `src/types.ts`, `src/index.css`
- `src-tauri/src/lib.rs`, `config_store.rs`, `persistence_suite.rs`
- `docs/engine-contract.md`, `docs/design-system-and-usability.md`

Baseline:

```powershell
cargo test --workspace
cargo run -p bornomala-desktop --bin persistence_suite
npm run build
```

Record line counts and hook/prop counts before changing anything.

---

## 3. Scope

### Goals

1. **Engine modularisation** that preserves the public API and every test.
2. **Generated contract types** replacing hand copies (depends on RV-02.5).
3. **A typed, queued IPC client** and a single state store.
4. **Backend hardening**: typed errors, poison recovery, save ordering, log directory, dead-code removal.
5. **Component decomposition** into feature folders with extracted hooks and static data modules.
6. **Tests for every layer** that did not have them.

### Non-goals

- No visual redesign (RV-08).
- No new features.
- No ABI change (RV-02 already did it).
- No performance optimisation beyond removing accidental O(n²) render work.

### Explicit no-go

- **No** “big bang” rewrite. Each package must leave the app runnable.
- **No** public API rename during modularisation.
- **No** new state library (Redux/Zustand/Jotai) unless the reducer + context approach demonstrably fails; justify any dependency.
- **No** component split that duplicates candidate rendering.
- **No** deleting the sandbox deterministic test functionality; move it to a throwaway engine.

---

## 4. Work packages

### WP-7.1 — Engine modularisation (behaviour-preserving)

Target layout:

```
crates/bornomala-core/src/
  lib.rs                 // re-exports + BornoEngine facade
  contract.rs            // EngineEvent, EngineState, ProcessResult, InputMode, CandidateKind
  engine/
    mod.rs
    phonetic.rs
    fixed.rs
    passthrough.rs
    commit.rs            // one commit_selection + auto-Dari + NFC
  config.rs
  unicode.rs
  ...
```

Rules:

1. Move code; do not rewrite logic in the same commit.
2. After each move, run `cargo test --workspace`. It must stay green with zero test edits.
3. Collapse the duplicated commit paths **only after** the moves are green, in a separate commit, with a test proving equivalence.
4. Keep `pub` items re-exported from `lib.rs` so consumers do not change.

Gate: `lib.rs` under ~400 lines; all 232 tests pass; no test file changed.

### WP-7.2 — Generated contract

Use the RV-02 codegen output. Delete hand-written `EngineState`/`EngineEvent` types from `src/types.ts` and re-export generated ones. Keep UI-only types (view models) separate.

Gate: `git diff` shows no hand-maintained contract fields; CI drift check green.

### WP-7.3 — Typed IPC client + event queue

Create `src/ipc/client.ts`:

```ts
type Command = "get_engine_status" | "send_engine_event" | ...
interface Result<T> { ok: true; value: T } | { ok: false; error: AppError }
```

Requirements:

1. Every command has a typed request and response.
2. Errors are a discriminated union, not `String`.
3. `send_engine_event` calls are serialised through a queue with a monotonically increasing sequence number; a stale response is dropped.
4. The UI never sends an event while a reset is in flight.
5. A failed event surfaces a retryable banner and does not desynchronise state.

Gate: unit tests for queue ordering, stale-drop, and error mapping.

### WP-7.4 — State store

Create `src/state/`:

- `engineStore.ts` — reducer over engine state + pending events.
- `configStore.ts` — effective config + persistence status + dirty form state.
- `useEngine()` / `useConfig()` hooks.

Remove the 13 `useState`/22 `useCallback` prop drilling in `App.tsx`. Components read from hooks; `App` becomes a layout shell.

Gate: `App.tsx` under ~200 lines; components take ≤6 props.

### WP-7.5 — Component decomposition

Split `SandboxTypingArea` into:

- `useEngineInput` (key handling, later IME composition — RV-08.1)
- `CandidateList`
- `CommittedOutput`
- `PhoneticGuide` (static data module)
- `ShapingPanel`
- `ModeToolbar`

Split `SettingsDrawer` into:

- `useConfigForm` (dirty state, save, discard, reset)
- `Toggle`, `Slider`, `ChoiceCard`, `Tabs`, `DialogShell`
- `AppearanceTab`, `TypingTab`, `ShortcutsTab`, `BackupTab`

Move static data (guide rows, shaping samples, 48-key maps, palettes) into `src/data/*.ts` — one source per map.

Gate: no component >400 lines; one key map; one palette source.

### WP-7.6 — Backend hardening

1. Typed error enum (`AppError`) instead of `Result<T, String>`.
2. Poison recovery: `lock().unwrap_or_else(|e| e.into_inner())` for pure state, or recreate the engine on poison and report it.
3. `save_config` order: acquire → validate → update memory/engine → write → confirm.
4. Remove `get_config_path`, merge `reset_config_to_defaults` into `reset_config`.
5. `run_deterministic_test` uses a throwaway `BornoEngine`, never the live one; it switches to Phonetic first.
6. Panic hook writes to `app_log_dir()` with a timestamp and chains the default hook; no CWD writes.
7. File I/O for `sync_all` off the async runtime (`spawn_blocking`).
8. Config import: strip BOM, try JSON then path; typed error.

Gate: `src-tauri` has real `#[test]`s using `tempfile` for config_store and command handlers.

### WP-7.7 — Tests

1. `src-tauri`: config_store (already covered by the bin — convert to `#[test]`), commands, migration, read-only failure.
2. Frontend: Vitest + Testing Library for the store, queue, and form state.
3. Convert `persistence_suite` into `#[test]`s or keep the bin and run it in CI (both acceptable; document).

Gate: `cargo test -p bornomala-desktop` is non-zero tests; `npm test` exists and passes.

---

## 5. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | `lib.rs` < 400 lines; all tests unchanged and green | diff + output |
| AC-2 | One commit path (`commit_selection`) | grep |
| AC-3 | Generated contract used by UI | diff |
| AC-4 | IPC queue serialises and drops stale responses | unit test |
| AC-5 | `App.tsx` < 200 lines; children ≤ 6 props | count |
| AC-6 | No component > 400 lines | count |
| AC-7 | One key map, one palette source | grep |
| AC-8 | Backend typed errors + poison recovery | code + tests |
| AC-9 | Desktop tests > 0; frontend tests > 0 | CI |
| AC-10 | App still launches and types | manual |

---

## 6. Testing plan

```powershell
cargo test --workspace
cargo test -p bornomala-desktop
npm test
npm run build
```

Manual: full sandbox flow (type, candidates, modes, settings, export/import, reset).

Negative checks:

- Break the IPC queue (return out of order) → test fails.
- Poison a mutex in a test → engine recovers or reports, no permanent brick.

---

## 7. Runnable checkpoint

Every work package must leave `npm run tauri dev` working. If a split breaks the build, fix before the next package.

Rollback: each package is one commit; the modularisation commits are pure moves and can be reverted independently.

---

## 8. Evidence and reporting

`docs/evidence/RV-07/<date>-architecture.md`: before/after line counts, test counts, bundle size, and the equivalence proof for the commit consolidation.

---

## 9. Roadmap update

- Record the new module layout in `docs/architecture.md` (which is currently 43 lines and stale).
- Mark the desktop UI dimension above 50 % only after RV-08.

---

## 10. Stop conditions

- A module split changes behaviour (test diff) → revert and redo as a pure move.
- The generated contract cannot express the UI’s view models → keep view models separate, do not force generation.
- A split requires changing the visual design → stop; that is RV-08.

---

## 11. Handoff

RV-08 now has small components and a state layer to build on. RV-11 can package a maintainable app.
