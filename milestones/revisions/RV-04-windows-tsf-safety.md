# RV-04 — Windows TSF Safety & Verification

> **Revision ID:** RV-04
> **Status:** Proposed — awaiting developer approval
> **Priority:** P0 (blocking)
> **Estimated effort:** 4–5 days
> **Dependencies:** RV-01, RV-02, RV-03
> **Sub-instruction:** [`RV-04.1-composition-lifecycle.md`](RV-04.1-composition-lifecycle.md)
> **Blocks:** RV-11, M13

---

## 1. Why this revision exists

The Windows TSF adapter is a serious COM skeleton — class factory, refcounts, `TF_ES_READWRITE | TF_ES_SYNC` edit sessions and the `handled`/`pfEaten` contract are correct — but it is not safe to install:

| ID | Defect | Evidence |
|---|---|---|
| P0-5 | Compositions are released, not ended, on deactivate/focus change | [`borno_tsf.cpp:331-336`](../../native/borno-tsf/borno_tsf.cpp#L331-L336), `:415-431` |
| P0-6 | `F12`/`F4`/`Alt+F4` eaten without modifier checks; `Shift+1..9` commits candidates | [`borno_tsf.cpp:452-455`](../../native/borno-tsf/borno_tsf.cpp#L452-L455), `:516-521`, [`key_mapper.h:117-127`](../../native/borno-tsf/key_mapper.h#L117-L127) |
| P0-8 | `build.ps1` registers and unregisters the system IME via the harness | [`build.ps1:75-79`](../../native/borno-tsf/build.ps1#L75-L79) |
| High | `ActivateEx` ignores every HRESULT | [`borno_tsf.cpp:276-323`](../../native/borno-tsf/borno_tsf.cpp#L276-L323) |
| High | Config read on the key path with unbounded allocation | [`borno_tsf.cpp:92-120`](../../native/borno-tsf/borno_tsf.cpp#L92-L120) |
| High | Candidate popup at screen origin on first use | [`candidate_window.cpp:103`](../../native/borno-tsf/candidate_window.cpp#L103) |
| High | Wrong `TF_LBI_TEXT` constant | [`lang_bar_item.h:39-41`](../../native/borno-tsf/lang_bar_item.h#L39-L41) |
| High | No DPI manifest; `GetDeviceCaps` only | [`candidate_window.cpp:107-113`](../../native/borno-tsf/candidate_window.cpp#L107-L113) |
| Contradicted | “Preedit underline” claimed but no display-attribute provider exists | [`M7_COMPLETION_REPORT.md:16,44-46`](../M7_COMPLETION_REPORT.md#L16) |
| Contradicted | M7 marked Verified while Levels 5–6 are pending | [`M7_COMPLETION_REPORT.md:132-133`](../M7_COMPLETION_REPORT.md#L132-L133) |

---

## 2. Current-state inspection

Read all of `native/borno-tsf/`:

`borno_tsf.cpp`, `candidate_window.cpp/.h`, `lang_bar_item.cpp/.h`, `edit_session.h`, `key_mapper.h`, `test_harness.cpp`, `build.ps1`, `scripts/register.bat`, `scripts/unregister.bat`, `borno_tsf.def`.

Read the audit sections §5.1 P0-5/P0-6/P0-8 and §5.2 “Native adapters”.

Baseline:

```powershell
powershell -ExecutionPolicy Bypass -File native/borno-tsf/build.ps1
```

**Warning:** run this only after WP-4.1 removes the harness registration, or it will unregister your installed IME. Until then, inspect only.

---

## 3. Scope

### Goals

1. Safe composition lifetime (delegated to RV-04.1).
2. A key policy that never hijacks OS or application shortcuts.
3. No machine-wide side effects from a build.
4. A truthful preedit presentation: implement the display attribute or remove the claim.
5. Correct DPI behaviour and candidate positioning.
6. Real HRESULT handling and error paths.
7. A recorded end-to-end matrix that justifies a future `Verified`.

### Non-goals

- No installer (RV-11).
- No new candidate UI design.
- No changes to the engine or ABI.
- No Linux work (RV-05).

### Explicit no-go

- **No** `F12`/`F4` as an unconditional global toggle.
- **No** `Alt+F4` interception under any setting.
- **No** registration or unregistration from `build.ps1`, tests, or CI.
- **No** “Verified” until the E2E matrix in §7 exists.
- **No** claiming the underline works until a display-attribute provider is registered.

---

## 4. Work packages

### WP-4.1 — Composition lifecycle

Execute [`RV-04.1-composition-lifecycle.md`](RV-04.1-composition-lifecycle.md). Do not proceed until it is verified.

### WP-4.2 — Key policy

Required rules:

| Key | Condition | Action |
|---|---|---|
| Toggle key (`F12` or `F4` per config) | no Ctrl, no Alt, no Shift, no Win | toggle mode, eat |
| Toggle key with any modifier | any | pass through |
| `Alt+F4` | always | pass through (never eat) |
| `1`..`9` | composing, candidates visible, **no Shift/Ctrl/Alt** | select candidate |
| `Ctrl+C/V/X/A/Z` | any | pass through |
| Other printable | composing, no Ctrl/Alt/Win | engine |

Implementation:

1. Add `IsShortcutModifier(lParam)` and `GetModifiers(lParam)` helpers; use them in **both** `OnTestKeyDown` and `OnKeyDown`.
2. Change `IsCandidateSelectionDigit(wParam, modifiers, count, &idx)` to reject any modifier.
3. Add a Windows setting for the toggle key that defaults to `F4` (less destructive than `F12`); keep `F12` selectable but warn in the UI.

Acceptance: a manual matrix in §7 shows `F12`, `Alt+F12`, `Alt+F4`, `Shift+1`, `Ctrl+C` behaving correctly while composing and while idle.

### WP-4.3 — Build and registration hygiene

1. Remove the `test_harness.exe` invocation from `build.ps1`; replace it with `-RunHarness` (off by default) and `-RegisterTest` (requires elevation, never default).
2. Move registration to `scripts/register.ps1` / `scripts/unregister.ps1`; keep the `.bat` wrappers for convenience but fix `unregister.bat:28-30`.
3. The harness must not call `DllRegisterServer`/`DllUnregisterServer` unless `--register-test` is passed; default mode checks exports only.
4. Add a registry before/after diff helper for evidence.

Acceptance: running `build.ps1` leaves `HKLM\SOFTWARE\Classes\CLSID\{8A57F89C-…}` unchanged (diff recorded).

### WP-4.4 — Preedit display attributes

Choose one:

- **Implement:** register a display-attribute GUID, implement `ITfDisplayAttributeProvider`, expose it via `ITfCategoryMgr`/`GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER`, and set `GUID_PROP_ATTRIBUTE` to the provider’s atom. Verify the underline appears in Notepad and Word.
- **Remove:** delete the claim from M7/M9/README, and document “preedit is shown as inline text without underline”.

Do not leave the current half-state (an atom from `GUID_PROP_ATTRIBUTE` with no provider).

### WP-4.5 — DPI, positioning, and resources

1. Add a `.rc` file with: version resource, per-monitor-v2 DPI manifest.
2. Use `GetDpiForWindow(m_hwnd)` and handle `WM_DPICHANGED`; fall back to `GetDpiForMonitor` for the caret monitor.
3. Fix candidate positioning: fill candidates before positioning (`SetCandidates` then `UpdatePosition`), size from the actual count.
4. Release GDI objects (fonts, brushes, DC) on every path; add a leak check via GDI handle count over 100 show/hide cycles.
5. Fix `TF_LBI_TEXT` to `0x00000002`, and the menu `cch` off-by-one.
6. Fix `m_isComposing = true` when `StartComposition` failed.

Acceptance: popup appears at the caret on first use; no GDI handle growth; language bar text updates on mode change.

### WP-4.6 — Error handling

1. Check every HRESULT in `ActivateEx`; unwind advised sinks on failure; return the failing HRESULT.
2. Check `SetText`, `SetSelection`, `StartComposition` results; on failure reset the engine and pass the key through (or drop safely), without leaving a ghost composition.
3. Free `BornoProcessResultC` on every return path (including `:690`, `:696`).
4. On any document mutation failure, log a non-content diagnostic and reset to idle.
5. Implement the “reset on mutation failure” behaviour that M7 claimed, or remove the claim.

Acceptance: injected failure tests (e.g. a read-only context) leave no ghost preedit and no leak.

### WP-4.7 — Diagnostic logging (privacy)

Add a debug-only log that records state transitions, HRESULTs, and composition events — **never** typed characters or preedit text. Gate behind a registry/env flag.

---

## 5. End-to-end verification matrix (required for `Verified`)

Run on a recorded Windows build, with the IME registered from a **stable path** (not the build tree; for this revision a temporary copy is acceptable, documented):

| # | Application | Scenario | Expected | Result |
|---|---|---|---|---|
| 1 | Notepad | type `ami ` in Phonetic | `আমি ` committed | |
| 2 | Notepad | switch to National, type a conjunct | correct Unicode | |
| 3 | Word | type, select candidate with `1` | correct commit | |
| 4 | Chrome (textarea) | type, press Escape | composition cancelled, no text | |
| 5 | VS Code | type, press Backspace ×3 | one char removed per press | |
| 6 | Explorer (rename) | type, click elsewhere | composition ends cleanly | |
| 7 | Notepad | change focus to another window mid-composition | no stuck underline, no crash | |
| 8 | Notepad | `Alt+F4` while composing | window closes, no mode toggle | |
| 9 | Notepad | `F12`/`F4` idle | toggles per config | |
| 10 | Notepad | `Shift+1` while composing | `!` typed, no candidate commit | |
| 11 | Word | 50 focus switches with composition | no crash, no leak | |
| 12 | Any | change candidate limit in sandbox, refocus IME | new limit applies | |

Evidence: application version, Windows build, screen capture or log excerpt per row.

---

## 6. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | No composition is released without being ended | code review + RV-04.1 tests |
| AC-2 | `Alt+F4`, `F12`, `F4`, `Shift+digit` behave per matrix | manual matrix |
| AC-3 | `build.ps1` leaves registration unchanged | registry diff |
| AC-4 | Display attribute implemented or claim removed | code/doc |
| AC-5 | Popup at caret on first use | manual |
| AC-6 | No GDI/COM leaks over 100 cycles | handle counts |
| AC-7 | ActivateEx failure returns an error | test/log |
| AC-8 | Matrix rows 1–12 recorded | evidence file |
| AC-9 | No typed content in logs | log inspection |

---

## 7. Testing plan

```powershell
# Build only (no registration)
pwsh -File native/borno-tsf/build.ps1 -NoRunHarness

# Harness without registration
native\borno-tsf\test_harness.exe --no-register

# Registry diff before/after a build
reg query "HKLM\SOFTWARE\Classes\CLSID\{8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}" /s > before.txt
pwsh -File native/borno-tsf/build.ps1 -NoRunHarness
reg query ... > after.txt
Compare-Object (Get-Content before.txt) (Get-Content after.txt)
```

Manual: the matrix in §5.

---

## 8. Runnable checkpoint

- DLL builds; sandbox still launches; IME still registered if it was before.
- No build step changes machine state.

Rollback: revert the adapter commits; the IME continues to work with the previous DLL.

---

## 9. Evidence and reporting

`docs/evidence/RV-04/<date>-tsf-safety.md` plus an E2E matrix file with screenshots.

Update M7 to `Implemented — not verified` until the matrix exists; then `Verified` with the evidence link.

---

## 10. Roadmap update

- M7 status corrected; preedit claim corrected.
- Add the E2E matrix requirement to the M7 acceptance criteria permanently.
- Record the DLL’s final path and registration model as an M12 input.

---

## 11. Stop conditions

- Composition lifecycle cannot be made safe without a context-lifetime redesign → stop and document (RV-04.1 has the decision point).
- Display attribute provider requires a build change that breaks MinGW → stop and remove the claim instead.
- Any change requires the installer before it is testable → defer to RV-11, keep the status unverified.

---

## 12. Handoff

RV-11 may package the DLL only after this revision and the matrix are verified.
