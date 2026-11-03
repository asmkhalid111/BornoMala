# RV-03 — Configuration Unification & Native Sync

> **Revision ID:** RV-03
> **Status:** Proposed — awaiting developer approval
> **Priority:** P0 (blocking)
> **Estimated effort:** 2–3 days
> **Dependencies:** RV-01 (config semantics), RV-02 (typed config delivery)
> **Blocks:** RV-04, RV-05, RV-11

---

## 1. Why this revision exists

The settings feature is half-real. The desktop writes one path; both native IMEs read another:

| Side | Path | Evidence |
|---|---|---|
| Desktop writer | `%APPDATA%\com.bornomala.sandbox\config.json` | [`tauri.conf.json:5`](../../src-tauri/tauri.conf.json#L5), [`lib.rs:225-229`](../../src-tauri/src/lib.rs#L225-L229) |
| Windows adapter | `%APPDATA%\com.bornomala.desktop\config.json` | [`borno_tsf.cpp:83-86`](../../native/borno-tsf/borno_tsf.cpp#L83-L86) |
| Linux adapter | `~/.config/com.bornomala.desktop/config.json` | [`bornomala.cpp:15`](../../native/bornomala-fcitx5/bornomala.cpp#L15) |

The audit verified that only `com.bornomala.sandbox` exists on the development machine. Every “live reload” claim in M9 is therefore inert.

Secondary problems:

- Configuration migration is implemented twice: once correctly in `BornoConfig::from_json_str`, once bespoke in `config_store.rs` and again in each adapter.
- The Windows adapter parses JSON by hand ([`borno_tsf.cpp:150-185`](../../native/borno-tsf/borno_tsf.cpp#L150-L185)).
- The adapter reads the config **on the key path** with an unbounded allocation ([`borno_tsf.cpp:120`](../../native/borno-tsf/borno_tsf.cpp#L120)).
- `save_config` writes to disk before updating engine/state ([`src-tauri/src/lib.rs:90-103`](../../src-tauri/src/lib.rs#L90-L103)).

---

## 2. Current-state inspection

Read:

- `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`, `src-tauri/src/config_store.rs`, `src-tauri/src/bin/persistence_suite.rs`
- `crates/bornomala-core/src/config.rs`
- `native/borno-tsf/borno_tsf.cpp` (config sections), `native/bornomala-fcitx5/bornomala.cpp` (config sections)
- `docs/design-system-and-usability.md` §2 (lifecycle contract)
- `docs/usability-and-linguistic-backlog.md` BACKLOG-07

Record the existing config file and its schema before changing anything:

```powershell
Get-Content "$env:APPDATA\com.bornomala.sandbox\config.json"
Get-ChildItem "$env:APPDATA" -Directory -Filter "*bornomala*"
```

---

## 3. Scope

### Goals

1. **One canonical identifier and path: `com.bornomala.desktop`.**
2. Migrate an existing `com.bornomala.sandbox` configuration safely, once, with a backup.
3. One parser (`BornoConfig::from_json_str`) for every consumer that reads JSON.
4. Adapter reload off the key path, bounded, exception-safe, applied at a safe boundary.
5. Transactional save: memory/engine update and disk write cannot diverge silently.

### Non-goals

- No new settings fields.
- No settings UI redesign (RV-08).
- No installer (RV-11 creates the installed path).

### Explicit no-go

- **No** dual-path compatibility layer. One path, one migration, then the old path is inert.
- **No** adapter-side schema migration. Adapters call the core or read a validated file.
- **No** config I/O in `OnTestKeyDown`/`OnKeyDown`.
- **No** unbounded `resize()` on a file size field.

---

## 4. Design

### 4.1 Canonical path

| Platform | Path |
|---|---|
| Windows | `%APPDATA%\com.bornomala.desktop\config.json` |
| Linux | `${XDG_CONFIG_HOME:-$HOME/.config}/com.bornomala.desktop/config.json` |
| macOS (future) | `~/Library/Application Support/com.bornomala.desktop/config.json` |

Rule: the desktop app is the **single writer**; adapters are **read-only consumers**. Document this in `docs/design-system-and-usability.md` §2.1.

### 4.2 Migration

On desktop startup:

1. Resolve the new path.
2. If it exists → load.
3. If it does not exist and the legacy `com.bornomala.sandbox/config.json` exists:
   - read the legacy bytes,
   - validate through `BornoConfig::from_json_str` (which already migrates 1.0.0/1.1.0 → 1.2.0),
   - write the new file atomically,
   - copy (do not move) the legacy file to `config.json.legacy-backup`,
   - log one line without content.
4. If the legacy file is invalid → leave it untouched, start with defaults, record a persistence status that the UI can display.

Migration must be idempotent and tested with: missing new + valid legacy, missing new + corrupt legacy, both present (new wins), unsupported future schema (untouched).

### 4.3 Adapter reload

- A dedicated watcher thread per adapter process (or a 1–2 s throttle), never the key callback.
- Cap file size (64 KB), read inside `try/catch`, parse through the core via `borno_engine_configure_json`.
- Apply at a commit boundary: if `is_composing`, defer until the next commit/cancel.
- On parse failure, keep the last good config and log once (no typed content).
- Linux: use `StandardPaths` so `XDG_CONFIG_HOME` and Flatpak remaps work.

### 4.4 Save ordering

`save_config` must:

1. Acquire config lock.
2. Validate.
3. Update the in-memory effective config and the engine (both under their locks).
4. Write atomically.
5. Only then report success.

If the write fails, roll the in-memory state back to the previous value and return a structured error. Add a test that simulates a write failure (read-only directory or injected failure) and asserts memory/disk consistency.

### 4.5 Removed duplication

- `config_store::load_config` delegates parsing/migration to `BornoConfig::from_json_str`; it keeps file I/O, backups, and status only.
- The Windows hand JSON parser is deleted; the adapter passes file bytes to `borno_engine_configure_json` and reads presentation fields through the new ABI getters (RV-02).
- The Linux adapter does the same.

---

## 5. Work packages

### WP-3.1 — Identifier and path

Change `tauri.conf.json` `identifier` to `com.bornomala.desktop`. Confirm `app_config_dir()` resolves to the new path on Windows. Update docs.

Gate: launching the app creates the new directory.

### WP-3.2 — Migration

Implement §4.2 in `config_store.rs` with unit tests using `tempfile`. Add the backup. Do not delete the legacy file.

Gate: migration tests pass for all four cases.

### WP-3.3 — Single parser

Delete the bespoke migration in `config_store.rs`; call the core parser. Update `persistence_suite` accordingly.

Gate: persistence suite green; no duplicated version strings remain (`grep "1.0.0"` in `config_store.rs` should only appear in tests, if at all).

### WP-3.4 — Adapter reload

Implement §4.3 in both adapters. Remove the key-path checks.

Gate: with a debugger or a log line, confirm the file is read on a timer, not on keydown.

### WP-3.5 — Transactional save

Implement §4.4. Add the failure test.

Gate: simulated failure leaves memory and disk consistent.

### WP-3.6 — End-to-end verification

1. Launch the sandbox, change theme/orientation/font size/mode toggle/candidate limit.
2. Confirm the file at `com.bornomala.desktop` changed.
3. Activate the TSF IME (or the Fcitx addon) and confirm the changed setting takes effect (candidate limit, font size, toggle key).
4. Record both sides.

Gate: the setting visibly affects the native IME, not just the sandbox.

---

## 6. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | Exactly one config path exists for the app | filesystem + doc |
| AC-2 | Legacy config migrates with backup, idempotently | tests |
| AC-3 | Corrupt/unsupported legacy is preserved untouched | test |
| AC-4 | One parser for load/import/migration | grep + tests |
| AC-5 | Adapter does no file I/O in key callbacks | code review |
| AC-6 | Config parse failure keeps last good config | test/manual |
| AC-7 | Save failure leaves memory and disk consistent | test |
| AC-8 | A setting change is observed in the native IME | manual evidence |
| AC-9 | `docs/design-system-and-usability.md` §2 matches the code | doc review |

---

## 7. Testing plan

```powershell
cargo test -p bornomala-core config
cargo test -p bornomala-desktop
cargo run -p bornomala-desktop --bin persistence_suite
npm run build
pwsh -File native/borno-tsf/build.ps1 -NoRun
```

Manual (Windows): the six steps in WP-3.6.
Manual (Linux, if available): same with the Fcitx addon; otherwise mark `Implemented — not verified` for the Linux side and record it.

Negative checks:

- Write an invalid JSON file at the canonical path → app starts with defaults, file untouched, status shown.
- Make the directory read-only → save returns an error and memory stays unchanged.

---

## 8. Runnable checkpoint

- App launches and writes the new path.
- Old file preserved as backup.
- Native IME reflects at least the candidate limit and toggle key.

Rollback: revert the identifier change and the migration commit together; the old path becomes canonical again.

---

## 9. Evidence and reporting

`docs/evidence/RV-03/<date>-config-unification.md`:

- directory listings before/after,
- migration test output,
- the native-IME observation (screenshot/log with no typed content),
- the read-only failure result.

---

## 10. Roadmap update

- Close BACKLOG-07.
- Update the M5 acceptance row: “UI changes affect actual engine behaviour” now includes native adapters.
- Update `docs/design-system-and-usability.md` §2 with the canonical path and migration lifecycle.

---

## 11. Stop conditions

- Changing the identifier breaks the installed TSF registration path (it should not; registration is CLSID-based, but verify).
- Migration would require deleting the legacy file to work → stop and redesign.
- The adapter cannot read the new path due to sandboxing → document and choose a different canonical location **before** implementing.

---

## 12. Handoff

RV-04 and RV-05 may now rely on a working config path. RV-11 can build the installer around `com.bornomala.desktop` without a second migration.
