# Borno Engine Behavioral Contract

> **Document Version:** 1.0.0  
> **Status:** Authoritative Specification (Milestone M1)  
> **Target Audience:** Core Engine Developers, Platform Adapter Implementers (Windows TSF, Linux Fcitx/IBus, macOS InputMethodKit), and Desktop Sandbox Integrators.

---

## 1. Purpose and Architectural Role

The Borno Engine is the centralized, platform-agnostic core responsible for all Bangla input logic. To guarantee uniform behavior across operating systems and client applications, every platform adapter must consume the engine through a single, standardized behavioral contract.

### Architectural Principles
1. **Single Source of Truth:** Platform adapters never implement transliteration, cursor management, or candidate ranking heuristics. All composition rules reside in `borno-core`.
2. **Separation of Persistent State from Output Effects:** State represents ongoing composition (preedit, cursor, candidates). Output effects represent step-level side effects (committed text, cancellation, handled status) and are strictly ephemeral.
3. **Decoupled Event Boundary:** Physical hardware keystrokes and OS messages (`WM_KEYDOWN`, virtual keys, scan codes) are translated at the adapter boundary into normalized, platform-independent `EngineEvent` instances.
4. **Memory Safety Across C-ABI:** All allocations across the C-ABI boundary have explicit, documented ownership rules. Pointers allocated by Rust are freed exclusively by Rust destructor helpers.

---

## 2. Event Model: `EngineEvent`

Platform adapters capture platform-specific input signals and map them to normalized `EngineEvent` values:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum EngineEvent {
    /// Printable Unicode character entered (e.g. 'k', 'a', ' ')
    InputChar(char),
    /// Raw string input (e.g. paste, simulated text, or multi-char sequence)
    InputString(String),
    /// Backspace request at current composition cursor
    Backspace,
    /// Forward delete request at current composition cursor
    Delete,
    /// Explicit request to commit the active composition
    Commit,
    /// Explicit cancellation of current composition (e.g. Escape)
    Cancel,
    /// Administrative reset of engine state (e.g. focus loss, window close)
    Reset,
    /// Move candidate highlight to the next item (with wrap-around)
    NextCandidate,
    /// Move candidate highlight to the previous item (with wrap-around)
    PrevCandidate,
    /// Set candidate highlight to a specific 0-based index without committing
    HighlightCandidate(usize),
    /// Select and immediately commit the candidate at the given 0-based index
    SelectCandidate(usize),
    /// Move composition cursor one codepoint to the left
    MoveCursorLeft,
    /// Move composition cursor one codepoint to the right
    MoveCursorRight,
    /// Set composition cursor to an explicit codepoint index
    SetCursor(usize),
    /// Switch input mode (e.g. Phonetic vs English Passthrough)
    SwitchMode(InputMode),
}
```

### Event Payload Constraints & Semantics
- `InputChar(c)`:
  - In `Phonetic` mode:
    - If `c == '\n'` or `c == '\r'`: triggers `Commit` of the active preedit.
    - If `c == ' '`: commits the active preedit followed by a space (or emits `" "` if idle).
    - Otherwise: appends `c` to `raw_buffer`, recalculates `preedit`, updates candidate list, and advances `cursor_pos` to the end of `preedit`.
  - In `EnglishPassthrough` mode: immediately emits `committed_text = Some(c.to_string())` with `is_composing = false`.
- `InputString(s)`:
  - In `Phonetic` mode: appends `s` into `raw_buffer`, updates composition, and positions cursor at the end.
  - In `EnglishPassthrough` mode: immediately emits `committed_text = Some(s)` with `is_composing = false`.
- `Backspace` / `Delete`:
  - Modifies active composition if `is_composing == true`.
  - If `is_composing == false`: returns `handled = false`, signaling the host OS or application to perform normal document editing.
- `Commit`:
  - Commits the currently highlighted candidate. If no candidate is highlighted, commits index 0; if candidates are empty, commits `preedit`. Transitions engine to `Idle`.
- `Cancel`:
  - Discards provisional state, clears buffers, and sets `cancelled = true` on `ProcessResult`.
- `Reset`:
  - Silently restores the engine to default `Idle` state without emitting cancellation effects (`cancelled = false, committed_text = None`).
- `SwitchMode(mode)`:
  - If `is_composing == true`: auto-commits active `preedit` before switching modes.

---

## 3. Persistent State vs Ephemeral Output Effects

### Persistent State: `EngineState`
The `EngineState` represents the ongoing internal state of the engine. It contains NO ephemeral step effects:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineState {
    /// Active provisional Bangla Unicode string displayed in preedit/composition window
    pub preedit: String,
    /// Raw Latin keystroke sequence driving the current composition
    pub raw_buffer: String,
    /// Composition cursor position measured in Unicode codepoints (0 <= cursor_pos <= preedit.chars().count())
    pub cursor_pos: usize,
    /// Ranked candidate strings for the current composition
    pub candidates: Vec<String>,
    /// Currently highlighted candidate index (None if no candidate is highlighted)
    pub selected_candidate_index: Option<usize>,
    /// True if an active composition is currently in progress
    pub is_composing: bool,
    /// Active input mode
    pub mode: InputMode,
    /// Operational metadata
    pub status: EngineStatus,
}
```

### Ephemeral Output Effect: `ProcessResult`
Every invocation of `process_event` produces a `ProcessResult` containing the updated state snapshot along with step-level output effects:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessResult {
    /// Snapshot of the persistent engine state after processing the event
    pub state: EngineState,
    /// Text to be inserted into the host application document during this step (None if no text committed)
    pub committed_text: Option<String>,
    /// True if an active composition was aborted/cancelled during this step
    pub cancelled: bool,
    /// True if the engine handled/consumed the event; false if the event should pass through to the host OS
    pub handled: bool,
}
```

### Elimination of Stale Committed Text
In prior iterations (M0), `committed_text` was stored directly in `EngineState`. This caused stale committed text to persist across subsequent queries. Under this contract:
1. `EngineState` never stores committed text.
2. `committed_text` exists only inside `ProcessResult` for the exact step that finalized the text.
3. Subsequent event processing results in `committed_text = None` unless a new commitment occurs.

---

## 4. Composition & Cursor Semantics

### Indexing Unit & Bounds Invariant
- **Unit:** `cursor_pos` is measured strictly in **Unicode scalar values (codepoints)**. It is NOT a byte offset and NOT a UTF-16 code unit index.
- **Invariant:** `0 <= cursor_pos <= preedit.chars().count()`.
- **Bounds Clamping:** All operations (`SetCursor`, `MoveCursorLeft`, `MoveCursorRight`) clamp `cursor_pos` strictly within `[0, preedit.chars().count()]`.

### In-Composition Editing (`Backspace` and `Delete`)
When `is_composing == true`:
- `Backspace`:
  - If `cursor_pos > 0`: removes the codepoint immediately preceding `cursor_pos`. Updates `raw_buffer`, recomposes `preedit`, decrements `cursor_pos` by 1, and refreshes candidates. If `raw_buffer` becomes empty, sets `is_composing = false`.
  - If `cursor_pos == 0`: no-op within composition; returns `handled = false`.
- `Delete`:
  - If `cursor_pos < preedit.chars().count()`: removes the codepoint immediately following `cursor_pos`. Updates `raw_buffer`, recomposes `preedit`, preserves `cursor_pos` (clamped to new length), and refreshes candidates. If `raw_buffer` becomes empty, sets `is_composing = false`.
  - If `cursor_pos == preedit.chars().count()`: no-op within composition; returns `handled = false`.

### Active Composition vs Host Document Editing
- The engine's cursor and editing logic apply **exclusively** while `is_composing == true`.
- Once text is committed (`is_composing == false`), keystrokes like `Backspace`, `Delete`, and arrow keys return `handled = false`. Platform adapters must let the host application handle caret movement and deletion in the target document.

### Complex Grapheme Cluster Boundary Disclaimer
In M1, editing is defined at the **Unicode codepoint** boundary. Full linguistic grapheme cluster normalization (such as preventing the splitting of multi-codepoint conjuncts like `ক + ্ + ষ` or consonant + vowel sign sequences like `ক + া`) is explicitly deferred to **Milestone M2 (Bangla Unicode & Phonetic Core)**.

---

## 5. Candidate Navigation and Selection Contract

### Highlight vs Selection Commit
- **Highlighting (`HighlightCandidate(idx)`, `NextCandidate`, `PrevCandidate`):**
  - Mutates `selected_candidate_index`.
  - Does **NOT** commit text.
  - Composition remains active (`is_composing = true`).
  - `NextCandidate` wraps around from index `candidates.len() - 1` to `0`.
  - `PrevCandidate` wraps around from index `0` to `candidates.len() - 1`.
  - If `selected_candidate_index` was `None`, `NextCandidate` highlights `0` and `PrevCandidate` highlights `candidates.len() - 1`.
- **Selecting (`SelectCandidate(idx)`):**
  - Selects candidate at `idx` and **commits it immediately**.
  - Emits `committed_text = Some(candidates[idx])`.
  - Clears `preedit`, `raw_buffer`, and `candidates`.
  - Sets `is_composing = false` and returns engine to `Idle`.
- **Committing (`Commit`):**
  - If `selected_candidate_index` is valid, commits `candidates[selected_candidate_index]`.
  - Else if `!candidates.is_empty()`, commits `candidates[0]`.
  - Else if `!preedit.is_empty()`, commits `preedit`.
  - Else, emits `committed_text = None`.
  - Returns engine to `Idle`.

### Fallbacks & Boundary Protection
- If `candidates` is empty:
  - `NextCandidate` / `PrevCandidate`: no-op (`selected_candidate_index = None`).
  - `SelectCandidate(idx)`: returns `handled = false` with unchanged state.
  - `Commit`: falls back to committing `preedit`.
- If `idx >= candidates.len()`:
  - `SelectCandidate(idx)`: returns `handled = false`, preserves state, emits `committed_text = None`.
  - `HighlightCandidate(idx)`: returns `handled = false`, preserves state.

---

## 6. Lifecycle State Machine

```
              ┌───────────────┐
              │     Idle      │◄──────────────────────────┐
              └───────┬───────┘                           │
                      │                                   │
             InputChar/InputString                        │
                      │                                   │
                      ▼                                   │
              ┌───────────────┐      Commit /             │
              │   Composing   │─── SelectCandidate ───────┤
              └───────┬───────┘    (committed_text != None)
                      │                                   │
                      │ Cancel (Esc)                      │
                      │ (cancelled = true)                │
                      └───────────────────────────────────┘
```

1. **`Idle`:** `preedit.is_empty()`, `raw_buffer.is_empty()`, `is_composing == false`. Unhandled non-composing keys pass through.
2. **`Composing`:** `!raw_buffer.is_empty()`, `is_composing == true`. Keys modify composition or navigate candidates.
3. **`Committed`:** Preedit is cleared, `committed_text` is emitted on `ProcessResult`, engine returns immediately to `Idle`.
4. **`Cancelled`:** Preedit is cleared, `cancelled = true` is emitted on `ProcessResult`, engine returns immediately to `Idle`.

---

## 7. C-ABI Interface (`bornomala.h` / `borno.h` Forwarding / `ffi.rs`)

To ensure robust interoperability with native Windows TSF, Linux Fcitx, or macOS IMK adapters, the C-ABI is exposed through a canonical header and legacy forwarding wrapper:
- **`crates/bornomala-core/include/bornomala.h`:** Authoritative canonical master header containing the full C-ABI surface, preprocessor guard `BORNOMALA_H`, and backward-compatibility alias `BORNO_H`.
- **`crates/bornomala-core/include/borno.h`:** Thin legacy forwarding header (`#include "bornomala.h"`) under guard `BORNO_LEGACY_FORWARDING_H` with deprecation guidance.

### C Types Definition
```c
/* Status and Error Codes */
#define BORNO_OK                      0
#define BORNO_ERR_NULL_PTR           -1
#define BORNO_ERR_INVALID_UTF8       -2
#define BORNO_ERR_PANIC              -3
#define BORNO_ERR_INVALID_ARGUMENT   -4
#define BORNO_ERR_INVALID_CONFIG     -5
#define BORNO_ERR_UNSUPPORTED_SCHEMA -6

/* Input Modes */
#define BORNO_MODE_PHONETIC          0
#define BORNO_MODE_PASSTHROUGH       1
#define BORNO_MODE_FIXED_NATIONAL    2
#define BORNO_MODE_FIXED_PROBHAT     3
#define BORNO_MODE_FIXED_BIJOY       4

/* Candidate Kinds */
#define BORNO_CANDIDATE_PRIMARY_TRANSLITERATION  0
#define BORNO_CANDIDATE_PHONETIC_ALTERNATIVE    1
#define BORNO_CANDIDATE_EXACT_LEXICON_MATCH     2
#define BORNO_CANDIDATE_PREFIX_COMPLETION       3
#define BORNO_CANDIDATE_SPELLING_CORRECTION     4

/* Event Types */
#define BORNO_EVENT_INPUT_CHAR           0
#define BORNO_EVENT_INPUT_STRING         1
#define BORNO_EVENT_BACKSPACE            2
#define BORNO_EVENT_DELETE               3
#define BORNO_EVENT_COMMIT               4
#define BORNO_EVENT_CANCEL               5
#define BORNO_EVENT_RESET                6
#define BORNO_EVENT_NEXT_CANDIDATE       7
#define BORNO_EVENT_PREV_CANDIDATE       8
#define BORNO_EVENT_HIGHLIGHT_CANDIDATE  9
#define BORNO_EVENT_SELECT_CANDIDATE     10
#define BORNO_EVENT_SET_CURSOR           11
#define BORNO_EVENT_MOVE_CURSOR_LEFT     12
#define BORNO_EVENT_MOVE_CURSOR_RIGHT    13
#define BORNO_EVENT_SWITCH_MODE          14

typedef struct BornoEventC {
    int32_t     event_type;
    uint32_t    char_codepoint;     /* Used for INPUT_CHAR */
    const char* string_payload;     /* Used for INPUT_STRING (borrowed UTF-8) */
    size_t      index_payload;      /* Used for HIGHLIGHT/SELECT/SET_CURSOR */
    int32_t     mode_payload;       /* Used for SWITCH_MODE */
} BornoEventC;

typedef struct BornoStatusC {
    char* name;
    char* version;
    char* active_mode;
    bool  ready;
} BornoStatusC;

typedef struct BornoStateC {
    char*   preedit;                  /* Null-terminated UTF-8 */
    char*   raw_buffer;               /* Null-terminated UTF-8 */
    size_t  cursor_pos;               /* Measured in Unicode codepoints */
    size_t  candidates_count;         /* Number of items in candidates array */
    char**  candidates;               /* Array of null-terminated UTF-8 strings */
    int32_t selected_candidate_index; /* -1 if None, otherwise 0 <= idx < count */
    bool    is_composing;
    int32_t mode;                     /* 0: Phonetic, 1: EnglishPassthrough */
} BornoStateC;

typedef struct BornoProcessResultC {
    BornoStateC state;                /* Persistent state snapshot */
    char*       committed_text;       /* Null-terminated UTF-8 or NULL */
    bool        cancelled;
    bool        handled;
} BornoProcessResultC;
```

### In-Place Configuration (`borno_engine_configure_json`)
```c
int32_t borno_engine_configure_json(BornoEngineHandle* handle, const char* json_utf8);
```
1. **Borrowed String Ownership:** Caller retains full ownership of `json_utf8`. Rust borrows it as a `&str` for parsing, never frees it, and stores no pointers to it.
2. **Supported Schemas & In-Memory Migration:**
   - Schema `"1.2.0"`: Current schema, accepted without migration.
   - Schema `"1.1.0"`: Migrated in-memory to v1.2.0. Missing `dictionary` fields default to empty values.
   - Schema `"1.0.0"`: Migrated in-memory to v1.2.0. Missing fields default safely.
   - Unsupported / Future Schemas: Returns `BORNO_ERR_UNSUPPORTED_SCHEMA` (`-6`) without mutating active configuration.
   - Malformed JSON / Field Type Errors: Returns `BORNO_ERR_INVALID_CONFIG` (`-5`) without mutating active configuration.
3. **State Preservation:** In-place reconfiguration preserves active composition state (`preedit`, `raw_buffer`, `cursor_pos`, `is_composing`, `mode`).
4. **Candidate Truncation Qualification:** If `candidate_display_limit` is lowered below the current candidate count, existing candidates and candidate details are truncated to the new limit without clearing or re-ranking.

### Memory Ownership & Lifecycle Rules
1. **Preferred Deallocation:** Callers SHOULD always release output structures using the corresponding structure cleanup helper (`borno_status_free_members`, `borno_state_free_members`, `borno_process_result_free_members`). These helpers safely free all nested allocations, null dangling pointers, and reset status flags.
2. **Individual Deallocation & Double-Free Prevention:** Standalone string members (`name`, `version`, `active_mode`, `preedit`, `raw_buffer`, `committed_text`) may be freed individually using `borno_string_free()`, provided the caller **immediately assigns that pointer field to `NULL`**. Callers must NEVER free a member individually and subsequently invoke the structure cleanup helper while that pointer remains dangling (preventing double-free UB).
3. **Candidate Array Ownership:** Candidate inner strings and the outer pointer array (`candidates`, `char**`) MUST be managed exclusively using `borno_state_free_members()` or `borno_process_result_free_members()`. `borno_string_free()` must NEVER be used on the outer array (`char**`), as doing so corrupts the allocator heap.
4. **No Direct C `free()`:** Standard C `free()` or C++ `delete` must **never** be invoked on pointers allocated by Rust.
5. **Output Structure Overwrite & Reuse:** On success (`BORNO_OK`), output-writing functions overwrite all structure members with fresh Rust allocations. The engine does NOT free previously held pointers. Callers MUST call the appropriate cleanup helper before reusing an already-populated structure to avoid memory leaks.
6. **Error Paths & Zero-Initialization:** Early error returns (`-1`, `-2`, `-4`, `-5`, `-6`) leave output structures completely untouched. Zero-initialization before first call (`memset` or `= {0}`) is a strongly recommended caller safety practice so that invoking cleanup helpers on error paths is a safe no-op.
7. **Unwind Safety:** All FFI entry points wrap inner logic in `std::panic::catch_unwind` and return negative error codes upon panic (`BORNO_ERR_PANIC`, `-3`). Cleanup safety after a panic is not guaranteed by the implementation.

---

## 8. Division of Responsibilities

| Responsibility | Shared Core (`borno-core`) | Platform Adapter (TSF / Fcitx / IMK / Sandbox) |
|---|---|---|
| Keystroke normalization | ❌ Receives `EngineEvent` | ✅ Maps OS scan codes / virtual keys to `EngineEvent` |
| Transliteration logic | ✅ Pure Rust implementation | ❌ Never parses phonetic keystrokes |
| Composition cursor position | ✅ Computes codepoint offset & bounds | ✅ Renders caret at measured position |
| Candidate list generation & wrapping | ✅ Generates, ranks, wraps indices | ✅ Displays UI window or popover |
| Text insertion into client document | ❌ Emits `committed_text` | ✅ Writes string to active text store (`ITfRange`, etc.) |
| Cancellation & cleanup | ✅ Resets internal state | ✅ Closes composition window & cancels text range |
| Memory management across boundary | ✅ Provides explicit destructor helpers | ✅ Invokes destructor helpers on received results |
