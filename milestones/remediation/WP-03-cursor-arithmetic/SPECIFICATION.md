# Work Package WP-03: Unicode In-Composition Cursor Arithmetic & UTF-8 Dimension Synchronization — Specification

> **Work Package ID:** `WP-03`  
> **Status:** Specified — Ready for Technical Review  
> **Date:** 2026-10-26  
> **Target Scope:** Core Engine In-Composition Cursor Arithmetic (`crates/bornomala-core/src/lib.rs`) & Dependent Contract Integration Tests (`crates/bornomala-core/tests/contract_tests.rs`)  
> **Source Documents:** `milestones/remediation/FINAL-IMPLEMENTATION-READINESS-REVIEW.md` (Finding 3), `milestones/AUDIT-2026-10-23.md` (Finding 4.2), WP-03 Specification Audit  

---

## 1. Objectives

1. **Establish and Enforce the Strict Cursor Invariant:**
   Formally establish that `source_cursor` is an internal **UTF-8 byte offset** into `state.raw_buffer`, and `state.cursor_pos` is an observable **Unicode scalar value (codepoint) offset** in `state.preedit`. Enforce that `source_cursor` satisfies:
   $$\text{source\_cursor} \le \text{raw\_buffer.len()} \quad \land \quad \text{raw\_buffer.is\_char\_boundary(source\_cursor)}$$
   at all stable engine execution points.
2. **Correct Source-Buffer Mutations:**
   Replace scalar-indexing and dimension-mixing in `insert_char_at_cursor`, `delete_before_cursor`, and `delete_after_cursor` with UTF-8 byte-safe operations that operate directly on a valid cursor and preserve the invariant **by construction**, without mutation-time boundary flooring.
3. **Formalize Source-to-Target and Target-to-Source Mapping Semantics:**
   Document exact mapping rules across buffer limits, token boundaries, interior positions, empty target spans, and multi-byte/multi-codepoint tokens. Account for left affinity, span-order precedence, and document where exact round-trip bijections hold and where directional collapse occurs. Explicitly distinguish proportional interpolation from guaranteed semantic cursor mapping.
4. **Clarify External Ingress vs. Internal Coordinate Conversion:**
   Specify that external cursor ingress arrives exclusively as target codepoint positions via `EngineEvent::SetCursor(pos)`. The engine clamps this position to valid preedit bounds and converts it to a valid source byte offset via `map_target_to_source`.
5. **Partition and Ground the Test Suite:**
   - Establish **internal unit tests in `crates/bornomala-core/src/lib.rs`** to verify private cursor invariants, exact byte positions, mutation arithmetic, and proportional interpolation without exposing private fields.
   - Establish **contract integration tests in `crates/bornomala-core/tests/contract_tests.rs`** to verify observable engine behavior, preedit composition, target cursor mapping, and event handling.
   - Test external ingress via public `EngineEvent::SetCursor` rather than artificial private-field corruption.
6. **Reconcile Verification Targets and Toolchain Portability:**
   Align verification commands with the 10 actual test targets in `bornomala-core` (purging references to non-existent test binaries), while implementing portable boundary helpers that do not require unpinned compiler features.

---

## 2. The Cursor Invariant Contract & Ingress Boundaries

### 2.1 Coordinate Space Definitions

The engine maintains two distinct, decoupled cursor coordinate systems:

1. **Source Coordinate (`self.source_cursor`):**
   - **Type & Visibility:** Private `usize` field on `BornoEngine` (`lib.rs:237`).
   - **Domain:** UTF-8 byte offset in `state.raw_buffer` ($0 \le \text{source\_cursor} \le \text{raw\_buffer.len()}$).
   - **Contract Invariant:** Must **always** point to a valid character boundary (`raw_buffer.is_char_boundary(source_cursor)`).
   - **Consumers:**
     - `Tokenizer::map_cursor_to_target` (compares `source_cursor` against `span.source_range: Range<usize>`).
     - Buffer slicing and mutation operations in `lib.rs`.

2. **Target Coordinate (`self.state.cursor_pos`):**
   - **Type & Visibility:** Public `usize` field on `EngineState` (`lib.rs:136-137`) and exposed via C ABI `BornoStateC.cursor_pos`.
   - **Domain:** Unicode scalar value (codepoint) index in `state.preedit` ($0 \le \text{cursor\_pos} \le \text{preedit.chars().count()}$).
   - **Consumers:** Host IME text input frameworks (Windows TSF, Linux Fcitx 5, macOS InputMethodKit) for caret positioning.

### 2.2 Invariant Preservation by Construction

All internal cursor calculations must preserve the invariant through exact arithmetic:

- **Initialization:** `source_cursor` is set to `0`. For an empty string `""`, $0 \le 0$ and `"".is_char_boundary(0)` is true.
- **Insertion at $S$:** Inserting a character $c$ at a valid character boundary $S$ produces a string where $S + \text{len\_utf8}(c)$ is a valid character boundary by construction.
- **Backspace before $S$:** Slicing `raw_buffer[..S]` and removing the preceding Unicode scalar value of length $L$ sets $S_{new} = S - L$. Since $S$ was a character boundary, $S - L$ is the character's starting boundary by construction.
- **Forward Delete after $S$:** Slicing `raw_buffer[S..]` and removing the following Unicode scalar value of length $L$ preserves $S_{new} = S$. Since the trailing characters shift left by exactly $L$ bytes, $S$ remains a character boundary by construction.

### 2.3 Strict Internal Invariant Policy vs. External Target Ingress

The engine strictly separates internal correctness from external ingress validation:

1. **Internal Invariant Policy (No Mutation-Time Masking):**
   Internal mutation methods (`insert_char_at_cursor`, `delete_before_cursor`, `delete_after_cursor`) assume that `self.source_cursor` is already valid. They operate directly on `self.source_cursor` without calling `floor_char_boundary`.
   - In debug builds, all mutation methods assert the invariant unconditionally at the very beginning of the method, before any conditional guards:
     ```rust
     debug_assert!(self.source_cursor <= self.state.raw_buffer.len());
     debug_assert!(self.state.raw_buffer.is_char_boundary(self.source_cursor));
     ```
   - Placing assertions unconditionally before conditional guards ensures that an invalid `source_cursor` (such as pointing to a UTF-8 continuation byte or exceeding buffer length) cannot quietly bypass debug verification merely because the method's deletion condition evaluates to false (e.g. `source_cursor == 0` or `source_cursor >= raw_buffer.len()`).
   - An invalid internal `source_cursor` represents an internal programming defect and will immediately fail debug tests. Boundary flooring must **never** be used to mask broken internal mutation arithmetic.

2. **External Ingress Boundary (`EngineEvent::SetCursor`):**
   External callers (Windows TSF, Linux Fcitx 5, Tauri frontend) can only provide a target codepoint offset via `EngineEvent::SetCursor(pos)`. No external event or caller can directly inspect or set the private `source_cursor`.
   - In `process_phonetic_event` (`lib.rs:840-845`):
     ```rust
     EngineEvent::SetCursor(pos) => {
         if self.state.is_composing {
             let max = self.state.preedit.chars().count();
             let clamped = pos.min(max);
             self.state.cursor_pos = clamped;
             self.source_cursor = self.map_target_to_source(clamped);
             ...
     ```
   - Target position `pos` is clamped to valid preedit bounds `0..=max`.
   - `map_target_to_source(clamped)` is an internal coordinate-conversion function that translates the valid target codepoint index into a valid source byte boundary. The output of `map_target_to_source` is required to satisfy the source cursor invariant.

### 2.4 Audit Scope of Source Cursor Writers & Buffer Mutations in `src/lib.rs`

The source-cursor invariant contract ($0 \le \text{source\_cursor} \le \text{raw\_buffer.len()} \land \text{raw\_buffer.is\_char\_boundary(source\_cursor)}$) must hold at **all stable engine execution points**, not merely within the three core mutation methods.

During implementation, the developer must audit and verify every write to `self.source_cursor` and every mutation of `self.state.raw_buffer` in `crates/bornomala-core/src/lib.rs`:

1. **Direct Buffer Mutators (Primary Remediation Focus):**
   - `insert_char_at_cursor(c)` (`lib.rs:1310-1323`): Inserts character $c$ and advances `self.source_cursor += c.len_utf8()`.
   - `delete_before_cursor()` (`lib.rs:1326-1343`): Drains preceding character and updates `self.source_cursor = prev_start`.
   - `delete_after_cursor()` (`lib.rs:1346-1362`): Drains following character and preserves `self.source_cursor` at current valid boundary.
2. **Composition Reset & Constructors:**
   - `BornoEngine::new()` (`lib.rs:255`): Initializes `source_cursor = 0` on empty `raw_buffer = String::new()`.
   - `BornoEngine::with_options(...)` (`lib.rs:292`): Initializes `source_cursor = 0` on empty `raw_buffer = String::new()`.
   - `clear_composition()` (`lib.rs:1274-1284`): Clears `raw_buffer.clear()` and explicitly resets `self.source_cursor = 0;`.
3. **Cursor Navigation & External Ingress Event Handlers:**
   - `EngineEvent::MoveCursorLeft` (`lib.rs:798-808`): Decrements `state.cursor_pos` and writes `self.source_cursor = self.map_target_to_source(self.state.cursor_pos);`.
   - `EngineEvent::MoveCursorRight` (`lib.rs:819-830`): Increments `state.cursor_pos` and writes `self.source_cursor = self.map_target_to_source(self.state.cursor_pos);`.
   - `EngineEvent::SetCursor(pos)` (`lib.rs:840-845`): Clamps `pos` to `max` and writes `self.source_cursor = self.map_target_to_source(clamped);`.
4. **High-Level Event Dispatch Writers:**
   - `EngineEvent::InputString(s)` (`lib.rs:627-631`): Funnels each character through `insert_char_at_cursor`.
   - `EngineEvent::InputChar(c)`: Funnels through `insert_char_at_cursor`.
   - `EngineEvent::Backspace`: Funnels through `delete_before_cursor()`.
   - `EngineEvent::Delete`: Funnels through `delete_after_cursor()`.
   - `EngineEvent::Commit` / `EngineEvent::Reset`: Funnel through `clear_composition()`.

**Verification Requirement:** While source inspection demonstrates that navigation events funnel exclusively through `map_target_to_source` and composition lifecycle methods reset `source_cursor` to 0, implementation cannot assume correctness across all execution paths. Every existing writer and buffer mutation in `src/lib.rs` must be checked and confirmed for invariant preservation during milestone implementation. The authorized file scope remains strictly limited to `crates/bornomala-core/src/lib.rs` and `crates/bornomala-core/tests/contract_tests.rs`.

---

## 3. Source-to-Target and Target-to-Source Mapping Semantics

### 3.1 TokenSpan Architecture

Each `TokenSpan` (`tokenizer.rs:11-17`) represents a bidirectional mapping between:
- `source_range: Range<usize>`: UTF-8 byte slice $[S_{start}, S_{end})$ in `state.raw_buffer`. Source span boundaries must themselves always be valid UTF-8 character boundaries (guaranteed by `Tokenizer` by construction).
- `target_range: Range<usize>`: Unicode scalar value range $[T_{start}, T_{end})$ in normalized `state.preedit`.

### 3.2 Formal Forward Mapping Contract (`PhoneticEngine::map_cursor_to_target`)

The forward mapping ($S \rightarrow T$) converts an internal byte offset into a public codepoint offset:

1. **Buffer Boundaries:**
   - If $S = 0$ or `raw_buffer.is_empty()`, returns $T = 0$.
   - If $S \ge \text{raw\_buffer.len()}$, returns $T = \text{preedit.chars().count()}$.
2. **Token Boundary Matches:**
   - If $S == \text{span.source\_range.start}$, returns $T = \text{span.target\_range.start}$.
   - If $S == \text{span.source\_range.end}$, returns $T = \text{span.target\_range.end}$.
3. **Interior Source Edits (Left Affinity):**
   - If $S_{start} < S < S_{end}$, returns $T = \text{span.target\_range.start}$.
   - *Rationale:* In Avro-style transliteration, multiple Latin keystrokes collapse into a single Bangla character (e.g. `k` + `h` $\rightarrow$ `খ`). When the cursor is positioned in the interior of the Latin sequence, the visual preedit caret has left-affinity, anchoring before the uncommitted composite token.

### 3.3 Formal Inverse Mapping Contract (`map_target_to_source`)

The inverse mapping ($T \rightarrow S$) converts a public codepoint offset (e.g. from `MoveCursorLeft`, `MoveCursorRight`, `SetCursor`) into an internal byte offset:

#### 3.3.1 Explicit Preconditions & Span Geometric Relations in Target Coordinate Space
`map_target_to_source` can guarantee returning a valid source UTF-8 character boundary within `[0, raw_buffer.len()]` **only** under the following strict preconditions derived from the existing tokenizer and mapping implementation:
1. **Valid UTF-8 Buffer:** `raw_buffer` must be well-formed UTF-8.
2. **Explicit Source Span Bounds:**
   - For every span in `self.spans`, $0 \le \text{span.source\_range.start} \le \text{span.source\_range.end} \le \text{raw\_buffer.len()}$.
   - Both `span.source_range.start` and `span.source_range.end` must be valid UTF-8 character boundaries in `raw_buffer` (`raw_buffer.is_char_boundary`).
3. **Explicit Target Span Bounds:**
   - For every span in `self.spans`, $0 \le \text{span.target\_range.start} \le \text{span.target\_range.end} \le \text{preedit.chars().count()}$.
4. **Span Ordering:**
   - Spans in `self.spans` must be ordered strictly sequentially in forward order ($k < j \implies \text{span}[k].\text{source\_range.start} \le \text{span}[j].\text{source\_range.start}$ and $\text{span}[k].\text{target\_range.start} \le \text{span}[j].\text{target\_range.start}$).
5. **Geometric Relations in Target Coordinate Space:**
   - **Positive-Width Overlaps ($T_{start}[j] < T_{end}[k]$ where $k < j$ and $T_{end}[j] > T_{start}[j]$):**
     True overlaps where two spans share non-zero interior target codepoints. Strictly prohibited by the tokenizer architecture; never emitted.
   - **Shared Endpoints ($T_{end}[k] == T_{start}[k+1]$ for positive-width spans):**
     Consecutive contiguous spans meet at a single boundary point. Their half-open intervals do not overlap ($[T_{start}[k], T_{end}[k]) \cap [T_{start}[k+1], T_{end}[k+1]) = \emptyset$). Because range containment in `map_target_to_source` is inclusive ($T_{start} \le T \le T_{end}$), at the boundary $T = T_{end}[k] = T_{start}[k+1]$, both spans contain $T$. By linear loop iteration order, the earlier span $k$ matches first at `target_pos == span.target_range.end` and returns $S_{end}[k]$. Since $S_{end}[k] == S_{start}[k+1]$ for contiguous source spans, both spans resolve to the exact same source byte boundary.
   - **Zero-Width Target Spans ($T_{start} == T_{end}$):**
     Emitted when a source token produces 0 target codepoints (e.g. backtick token break, `tokenizer.rs:104-113`, where `target_range: start_codepoints..start_codepoints`). A zero-width span has an empty half-open range `start..start`, but in inclusive checking ($T_{start} \le T \le T_{end}$), it matches when $T == T_{start}$. When positioned between positive-width spans $k$ and $k+2$, the zero-width span $k+1$ shares the exact target coordinate with $T_{end}[k]$ and $T_{start}[k+2]$ ($T_{end}[k] == T_{start}[k+1] == T_{end}[k+1] == T_{start}[k+2]$). Under linear iteration order, span $k$ matches first, returning $S_{end}[k]$ (before the zero-width token).
6. **Separation of Proven Invariants vs. Runtime-Verification Properties:**
   - **Proven Invariants (from Tokenizer Inspection):**
     - Incremental scanning in `Tokenizer::tokenize` partitions `raw_buffer` contiguously: $S_{start}[0] == 0$, $S_{end}[k] == S_{start}[k+1]$, and $S_{end}[\text{last}] == \text{raw\_buffer.len()}$. No source gaps, no source overlaps.
     - Pre-NFC target ranges are contiguous and non-decreasing: $T_{start}[0] == 0$, $T_{end}[k] == T_{start}[k+1]$, and $T_{end}[\text{last}] == \text{output.chars().count()}$. Positive-width overlaps do not exist.
     - In `realign_spans_after_nfc` (`tokenizer.rs:389-411`), since the exact same scalar value $T = \text{span}[k].target\_range.end = \text{span}[k+1].target\_range.start$ is passed to the deterministic function `map_cursor_after_normalization`, the outputs `realigned_end[k]` and `realigned_start[k+1]` are identical ($T'_{end}[k] == T'_{start}[k+1]$). Thus, endpoint sharing is preserved, and gaps between adjacent spans cannot be introduced at shared endpoints.
   - **Properties Requiring Runtime Test Verification:**
     - Whether `map_cursor_after_normalization` is strictly monotonic non-decreasing ($c_1 \le c_2 \implies \text{map}(c_1) \le \text{map}(c_2)$) across all conceivable non-canonical or complex Unicode decomposition and recomposition sequences cannot be confirmed from static inspection alone.
     - Whether any edge-case normalization can cause inverted span endpoints ($T'_{start} > T'_{end}$) prior to the `.max(realigned_start)` clamp, or produce unexpected zero-width collapses, is identified as **requiring runtime verification** during test execution rather than asserted as an established fact.
7. **Mutually Consistent State & Non-Repair:**
   - `self.spans`, `self.state.raw_buffer`, and `self.state.preedit` must represent a mutually consistent composition state (as produced by `Tokenizer::tokenize` or `PhoneticEngine::transliterate`).
   - While the proportional interpolation branch executes `floor_char_boundary`, this adjustment snaps fractional candidate positions to the nearest preceding character boundary; it does **not** repair arbitrary, stale, or out-of-sync spans from an invalid engine state. Fast-path returns rely directly on these preconditions holding true.

#### 3.3.2 Inverse Mapping Lookup Flow & Fallback Unreachability Analysis
In `map_target_to_source(target_pos)` (`lib.rs:1286-1308` / §4.5), coordinate conversion executes through the following pipeline:
1. **Initial Fast Paths:**
   - `if target_pos == 0 || self.spans.is_empty() { return 0; }`
   - `if target_pos >= max_target { return self.state.raw_buffer.len(); }`
   Any target cursor reaching the linear search loop satisfies:
   $$1 \le \text{target\_pos} < \text{max\_target}$$
2. **Span Iteration and Exhaustive Branch Return:**
   The loop iterates through `&self.spans` with the inclusive containment check:
   `if target_pos >= span.target_range.start && target_pos <= span.target_range.end`
   Inside this matching guard, every single control-flow path returns an integer offset immediately:
   - Zero-width match ($T_{start} == T_{end}$): `return span.source_range.start;`
   - Boundary match start ($T == T_{start}$): `return span.source_range.start;`
   - Boundary match end ($T == T_{end}$): `return span.source_range.end;`
   - Interior interpolation ($T_{start} < T < T_{end}$): `return aligned.max(span.source_range.start).min(span.source_range.end);`
   There is no non-returning path inside the matching block.
3. **Mathematical Proof of Unreachability for Valid Tokenizer State:**
   - Because `Tokenizer::tokenize` guarantees contiguous target intervals covering $[0, \text{max\_target}]$ with shared endpoints ($T_{end}[k] == T_{start}[k+1]$) and no target gaps, the union of all closed intervals covers the target space completely:
     $$\bigcup_{k=0}^{N-1} [\text{span}[k].target\_range.start, \text{span}[k].target\_range.end] = [0, \text{max\_target}]$$
   - For every in-range target cursor $1 \le \text{target\_pos} < \text{max\_target}$, there exists at least one span $k$ such that $\text{target\_pos} \in [\text{span}[k].target\_range.start, \text{span}[k].target\_range.end]$.
   - Because every branch inside the matching guard returns, the loop is guaranteed to return on the first matching span.
   - **Conclusion:** For any valid, mutually consistent tokenizer-produced state with contiguous target spans, the trailing fallback `self.state.raw_buffer.len()` at line 408 is **provably unreachable** for an in-range target cursor.
4. **Defensive Role & Architectural Limitations of the Fallback:**
   - The trailing `self.state.raw_buffer.len()` serves two functions:
     1. It satisfies Rust's compiler exhaustiveness requirement outside the `for` loop.
     2. It acts as a defensive fallback against corrupt, disconnected, or gapped span lists. If an invalid or stale `self.spans` array contains a gap (e.g. $T_{end}[k] < T_{start}[k+1]$) and `target_pos` falls within that gap, the loop completes without matching any span and returns `raw_buffer.len()`.
   - Existing implementation behavior is preserved as the baseline; no new fallback is invented or redesigned.

#### 3.3.3 Mapping Rules by Span Geometry
1. **Buffer Boundaries:**
   - If $T = 0$ or `spans.is_empty()`, returns $S = 0$.
   - If $T \ge \text{preedit.chars().count()}$, returns $S = \text{raw\_buffer.len()}$.
2. **Span Boundary Matches:**
   - If $T == \text{span.target\_range.start}$, returns $S = \text{span.source\_range.start}$.
   - If $T == \text{span.target\_range.end}$, returns $S = \text{span.source\_range.end}$.
   - For contiguous adjacent spans $k$ and $k+1$, $\text{span}[k].target\_range.end == \text{span}[k+1].target\_range.start$ and $\text{span}[k].source\_range.end == \text{span}[k+1].source\_range.start$. Both spans resolve to the exact same shared byte boundary.
   - Since source span boundaries are guaranteed to be valid UTF-8 character boundaries by construction, span boundary matches always yield valid character boundaries.
3. **Single-Codepoint Target Spans ($T_{end} - T_{start} == 1$):**
   - For single-codepoint target spans (such as standard consonants, vowels, and simple conjuncts), interior target positions do not exist ($T$ is either $T_{start}$ or $T_{end}$).
4. **Multi-Codepoint Target Spans ($T_{end} - T_{start} > 1$):**
   - Governed by the deterministic proportional interpolation algorithm specified in §3.6.
   - Proportional interpolation serves strictly as an internal positioning heuristic and does not guarantee semantic round-tripping across linguistic dimensions.
5. **Multibyte Source Characters:**
   - If `raw_buffer` contains non-ASCII characters (e.g. passthrough Bengali characters), the source range spans 3 bytes (e.g. $[0, 3)$) while the target spans 1 codepoint ($[0, 1)$).
   - Target positions are strictly $0$ and $1$, mapping to byte $0$ and byte $3$. Intermediate byte offsets $1$ and $2$ cannot be generated by target navigation.

### 3.4 Inherent Mapping Asymmetry and Digraph Exemplar (`"kh"` $\rightarrow$ `"খ"`)

A fundamental property of phonetic transliteration is that **$S \rightarrow T \rightarrow S$ is NOT a 1:1 bijection for interior source positions**:
- Consider digraph rule `latin: "kh"` $\rightarrow$ `bangla: "\u{0996}"` (`'খ'`, `rules.rs:556-559`), producing `TokenSpan { source_range: 0..2, target_range: 0..1 }`:
  - $S = 0 \rightarrow T = 0 \rightarrow S = 0$ (Exact identity at boundary).
  - $S = 2 \rightarrow T = 1 \rightarrow S = 2$ (Exact identity at boundary).
  - $S = 1 \rightarrow T = 0 \rightarrow S = 0$ (Collapsed by Left Affinity).
- **Architectural Fact:** Exact round-trip identity holds **at token boundaries**, but collapses to the token start for interior source offsets due to Rule 3. This behavior is intentional, preserves Avro compatibility, and prevents invalid in-between states.

### 3.5 Zero-Width Spans (Backtick Token Break Reference Case)

In production code (`tokenizer.rs:104-113`), a single backtick token break consumes 1 Latin source byte but emits 0 target codepoints (`target_range: start..start`).

Consider the reference case `k` + `` ` `` + `h` (`"k`h"`):
- `span[0]`: `"k"` $\rightarrow$ `'ক'`, `source_range: 0..1`, `target_range: 0..1`.
- `span[1]`: `` "`" `` $\rightarrow$ `""`, `source_range: 1..2`, `target_range: 1..1` (zero-width target span).
- `span[2]`: `"h"` $\rightarrow$ `'হ'`, `source_range: 2..3`, `target_range: 1..2`.

Normalized preedit is `"কহ"` (length 2 codepoints). Observable target cursor positions are 0 (before `'ক'`), 1 (between `'ক'` and `'হ'`), and 2 (after `'হ'`).

#### 3.5.1 Observed Span-Order Precedence Policy & Mechanics
In `crates/bornomala-core/src/lib.rs:1287-1308`, the inverse mapping loop executes with the following concrete mechanics:
1. **Span Visitation Order:** Spans in `self.spans` are visited in strict linear forward sequential order (`for span in &self.spans`).
2. **Range Containment Inclusivity:** The containment check `target_pos >= span.target_range.start && target_pos <= span.target_range.end` is **inclusive** on both ends ($T_{start} \le T \le T_{end}$).
3. **Evaluation of Zero-Width vs. Boundary Matches:** Zero-width spans are evaluated in the order they appear in `self.spans`. When an earlier span has `target_range.end == target_pos`, that earlier span matches immediately upon the inclusive upper-bound check before any subsequent span (including a zero-width span at the same position) is inspected.
4. **Precedence for Overlapping/Adjacent Boundaries:** When adjacent spans or zero-width spans share the same target coordinate $T$, the **earliest (left-hand) span in iteration order wins** and returns immediately.

**Documented Reference Case (`"k`h"`):**
- In `"k`h"`, `span[0]` has `target_range: 0..1`, `span[1]` has `target_range: 1..1` (zero-width), and `span[2]` has `target_range: 1..2`.
- For target position $T = 1$:
  - `span[0]` is evaluated first: $1 \ge 0 \land 1 \le 1$ evaluates to `true`.
  - The boundary match condition `target_pos == span.target_range.end` matches, returning `span[0].source_range.end = 1`.
  - The function returns `1` immediately without inspecting `span[1]` or `span[2]`.
  - Thus, visual caret boundary $T = 1$ deterministically resolves to **source byte 1** (positioned after `'k'` and before the backtick `` ` ``).

**Scope & Policy Note:** This rule is documented strictly to the extent supported by the existing implementation and is explicitly limited to the observed `"k`h"` reference case. We do not claim that this precedence is the intended UX or that it prevents accidental deletion, nor is token-break navigation redesigned within WP-03. Existing implementation behavior is preserved as the baseline.

#### 3.5.2 Specified Mutation Behavior at Ambiguous Position $T = 1$ ($S = 1$)
When the cursor is at $T = 1$ ($S = 1$):
- **Insertion at $T = 1$ ($S = 1$):**
  Inserting character `'x'` inserts at byte offset 1 in `raw_buffer`, between `'k'` and `` ` ``. `raw_buffer` becomes `"kx`h"`. Composition updates with `'x'` inserted prior to the token break.
- **Backspace at $T = 1$ ($S = 1$):**
  Backspace deletes the character preceding byte 1, which is `'k'`. `raw_buffer` becomes `` "`h" ``. `source_cursor` becomes 0. Recomposition updates preedit to `"হ"` with `cursor_pos = 0`.
- **Forward Delete at $T = 1$ ($S = 1$):**
  Forward Delete deletes the character following byte 1, which is the backtick `` "`" ``. `raw_buffer` becomes `"kh"`. `source_cursor` remains at byte 1. Recomposition of `"kh"` invokes digraph rule `"kh"` $\rightarrow$ `'খ'`. Observable preedit transforms from `'কহ'` into single conjunct `'খ'`, and `cursor_pos` resolves to 0 (under left affinity for $S = 1$).

#### 3.5.3 Unresolved Product / Engineering Decision
Whether Forward Delete at the ambiguous boundary $T = 1$ visually deleting the invisible backtick and collapsing `'কহ'` into conjunct `'খ'` is the intended user experience, or whether users expect Forward Delete to skip zero-width spans and delete the visible following character `'হ'`.
Because existing test suites and specifications provide no evidence for zero-width token-skipping during Forward Delete, this is marked as an **unresolved product/engineering decision** rather than inventing an answer. As required by scope governance, existing implementation behavior (deleting the raw byte immediately following `source_cursor = 1`, i.e., the backtick) is preserved as the baseline unless a deliberate change is explicitly specified and approved. Token-break navigation is not redesigned as part of WP-03, and existing implementation behavior is strictly preserved as the baseline for this scope.

### 3.6 Deterministic Proportional Interpolation Algorithm & Multibyte Handling

For multi-codepoint target spans ($T_{end} - T_{start} > 1$), the engine implements a single, deterministic proportional interpolation algorithm:

#### 3.6.1 Mathematical Formulation
1. **Calculate Proportional Ratio:**
   $$\text{ratio} = \frac{T - T_{start}}{T_{end} - T_{start}}$$
   Where $T \in (T_{start}, T_{end})$. (Boundary points $T = T_{start}$ and $T = T_{end}$ are resolved directly by boundary matching and do not trigger interpolation.)
2. **Compute Candidate Byte Offset:**
   $$\text{offset} = \text{round}(\text{ratio} \times (S_{end} - S_{start}))$$
   $$\text{candidate} = S_{start} + \text{offset}$$
   Where $\text{round}(x)$ rounds half-way cases away from zero (`f32::round()`), yielding a deterministic integer offset.
3. **Backward Character-Boundary Adjustment:**
   The candidate is adjusted backward to the greatest valid character boundary not exceeding `candidate`, without crossing $S_{start}$:
   $$\text{aligned} = \max \{ b \in [S_{start}, \text{candidate}] \mid \text{raw\_buffer.is\_char\_boundary}(b) \}$$
   Since source span boundaries are guaranteed by `Tokenizer` to be valid UTF-8 character boundaries, $S_{start}$ is a valid character boundary. Because $\text{candidate} \ge S_{start}$, moving backward from `candidate` to the greatest valid character boundary will never cross below $S_{start}$. In code, bounding with `.max(span.source_range.start).min(span.source_range.end)` guarantees that the result is strictly bounded within the source span $[S_{start}, S_{end}]$ defensively.

#### 3.6.2 Algorithm Implementation (Rust)
```rust
let ratio = (target_pos - span.target_range.start) as f32
    / (span.target_range.end - span.target_range.start) as f32;
let source_len = span.source_range.end - span.source_range.start;
let offset = (ratio * source_len as f32).round() as usize;
let candidate = span.source_range.start + offset;
let aligned = floor_char_boundary(&self.state.raw_buffer, candidate);
aligned
    .max(span.source_range.start)
    .min(span.source_range.end)
```

#### 3.6.3 Specified Operation Sequence & Determinism
The deterministic interpolation pipeline executes the following specified operation sequence:
1. **Calculate the proportional ratio using `f32`:**
   $$\text{ratio} = \frac{(T - T_{start}) \text{ as f32}}{(T_{end} - T_{start}) \text{ as f32}}$$
2. **Multiply by the source span length:**
   $$\text{offset\_f32} = \text{ratio} \times ((S_{end} - S_{start}) \text{ as f32})$$
3. **Apply Rust `f32::round()`:**
   $$\text{offset} = \text{offset\_f32.round() as usize}$$
   $$\text{candidate} = S_{start} + \text{offset}$$
4. **Adjust backward to a valid UTF-8 character boundary:**
   $$\text{aligned} = \text{floor\_char\_boundary}(\text{raw\_buffer}, \text{candidate})$$
5. **Bound the result to the source span:**
   $$\text{result} = \text{aligned.max}(S_{start})\text{.min}(S_{end})$$
   The final result satisfies:
   - $\text{result} \ge \text{span.source\_range.start}$
   - $\text{result} \le \text{span.source\_range.end}$
   - $\text{result} \le \text{raw\_buffer.len()}$
   - $\text{raw\_buffer.is\_char\_boundary}(\text{result})$

Determinism is achieved through this explicit sequence of operations. Rather than relying on broad claims of cross-platform floating-point equivalence, the algorithm defines a bounded integer-snapping pipeline: floating-point arithmetic is restricted to computing a preliminary candidate offset, which is immediately rounded to an integer via Rust's standard `f32::round()`, adjusted backward to the nearest valid UTF-8 character boundary via `floor_char_boundary`, and clamped to the span's known UTF-8 character boundaries $[S_{start}, S_{end}]$ via `.max(span.source_range.start).min(span.source_range.end)`.

#### 3.6.4 Multibyte Character Adjustment Reference Case (Synthetic Fixture)
To isolate and verify multi-byte boundary stepping mechanics (independent of tokenizer rules), consider a synthetic mapping fixture with source range $[0, 4)$ where bytes $0..3$ contain `'ক'` (U+0995, 3 bytes: `0xE0 0xA6 0x95`) and byte $3..4$ contains `'a'` (`0x61`, 1 byte). Total source length is 4 bytes. Both $S_{start} = 0$ and $S_{end} = 4$ are valid character boundaries. Suppose the synthetic target span has 2 codepoints: $[0, 2)$.
For target position $T = 1$:
1. $\text{ratio} = (1 - 0) / (2 - 0) = 0.5$.
2. $\text{offset\_f32} = 0.5 \times 4.0 = 2.0$.
3. $\text{offset} = 2.0\text{.round() as usize} = 2$; $\text{candidate} = 0 + 2 = 2$.
4. Byte 2 is a continuation byte (`0x95`) of `'ক'` (which occupies bytes 0, 1, 2). Moving backward to the greatest valid character boundary not exceeding 2 without crossing $S_{start} = 0$:
   `floor_char_boundary(&raw_buffer, 2)` steps backward past `0x95` (byte 2) and `0xA6` (byte 1) to lead byte `0xE0` (byte 0).
5. Result: `aligned = 0`. Bounded by `0.max(0).min(4) = 0`.
The cursor safely snaps to byte 0 without splitting the multi-byte sequence.

#### 3.6.5 Semantic Distinction: Proportional Interpolation vs. Guaranteed Semantic Mapping
Proportional interpolation is strictly an **internal positioning heuristic** across composite tokens to ensure the cursor always resides on a valid UTF-8 character boundary within span limits.
- It does **not** guarantee exact semantic round-tripping across differing linguistic dimensions ($S \rightarrow T \rightarrow S$).
- Phonetic transliteration is non-linear: characters may reorder (e.g. dependent vowel signs like E-kar U+09C7 appearing visually to the left of consonants) or collapse (digraphs, conjuncts). Proportional interpolation does not attempt to reconstruct linguistic grapheme clusters; it merely provides safe, bounded caret navigation within composite spans.

---

## 4. Detailed Technical Specification of Core Mutations

### 4.1 Portable Boundary Helper

A private inline helper in `crates/bornomala-core/src/lib.rs` guarantees portability across all compiler versions without relying on unpinned compiler features (`str::floor_char_boundary`):

```rust
#[inline]
fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}
```

### 4.2 Character Insertion (`insert_char_at_cursor`)

```rust
fn insert_char_at_cursor(&mut self, c: char) {
    debug_assert!(self.source_cursor <= self.state.raw_buffer.len());
    debug_assert!(self.state.raw_buffer.is_char_boundary(self.source_cursor));

    self.state.raw_buffer.insert(self.source_cursor, c);
    self.source_cursor += c.len_utf8();

    self.update_composition();
    self.state.cursor_pos = phonetic::PhoneticEngine::map_cursor_to_target(
        self.source_cursor,
        &self.state.raw_buffer,
        &self.spans,
        &self.state.preedit,
    );
}
```

### 4.3 Backspace (`delete_before_cursor`)

```rust
fn delete_before_cursor(&mut self) {
    debug_assert!(self.source_cursor <= self.state.raw_buffer.len());
    debug_assert!(self.state.raw_buffer.is_char_boundary(self.source_cursor));

    if self.source_cursor > 0 && !self.state.raw_buffer.is_empty() {
        if let Some(prev_char) = self.state.raw_buffer[..self.source_cursor].chars().next_back() {
            let prev_start = self.source_cursor - prev_char.len_utf8();
            self.state.raw_buffer.drain(prev_start..self.source_cursor);
            self.source_cursor = prev_start;

            self.update_composition();
            self.state.cursor_pos = phonetic::PhoneticEngine::map_cursor_to_target(
                self.source_cursor,
                &self.state.raw_buffer,
                &self.spans,
                &self.state.preedit,
            );
        }
    }
}
```

### 4.4 Forward Delete (`delete_after_cursor`)

```rust
fn delete_after_cursor(&mut self) {
    debug_assert!(self.source_cursor <= self.state.raw_buffer.len());
    debug_assert!(self.state.raw_buffer.is_char_boundary(self.source_cursor));

    if self.source_cursor < self.state.raw_buffer.len() {
        if let Some(next_char) = self.state.raw_buffer[self.source_cursor..].chars().next() {
            let next_end = self.source_cursor + next_char.len_utf8();
            self.state.raw_buffer.drain(self.source_cursor..next_end);

            self.update_composition();
            self.state.cursor_pos = phonetic::PhoneticEngine::map_cursor_to_target(
                self.source_cursor,
                &self.state.raw_buffer,
                &self.spans,
                &self.state.preedit,
            );
        }
    }
}
```

### 4.5 Target-to-Source Coordinate Conversion (`map_target_to_source`)

```rust
pub fn map_target_to_source(&self, target_pos: usize) -> usize {
    if target_pos == 0 || self.spans.is_empty() {
        return 0;
    }
    let max_target = self.state.preedit.chars().count();
    if target_pos >= max_target {
        return self.state.raw_buffer.len();
    }
    for span in &self.spans {
        if target_pos >= span.target_range.start && target_pos <= span.target_range.end {
            if span.target_range.start == span.target_range.end {
                return span.source_range.start;
            }
            if target_pos == span.target_range.start {
                return span.source_range.start;
            }
            if target_pos == span.target_range.end {
                return span.source_range.end;
            }
            let ratio = (target_pos - span.target_range.start) as f32
                / (span.target_range.end - span.target_range.start) as f32;
            let source_len = span.source_range.end - span.source_range.start;
            let offset = (ratio * source_len as f32).round() as usize;
            let candidate = span.source_range.start + offset;
            let aligned = floor_char_boundary(&self.state.raw_buffer, candidate);
            return aligned
                .max(span.source_range.start)
                .min(span.source_range.end);
        }
    }
    self.state.raw_buffer.len()
}
```

---

## 5. Concrete Test Suite Specification

### 5.1 Test Visibility & Partitioning

`source_cursor` is private to `BornoEngine`. Tests are strictly partitioned into:
1. **Internal Unit Tests (`crates/bornomala-core/src/lib.rs` under `#[cfg(test)] mod tests`):** Direct assertions on `source_cursor`, UTF-8 byte boundary predicates, internal mutation arithmetic, and proportional interpolation.
2. **Integration Tests (`crates/bornomala-core/tests/contract_tests.rs`):** Observable public state (`cursor_pos`, `preedit`, `raw_buffer`) and event handling (`EngineEvent`).

*Note on Evidence & Test Status:* The test cases below represent proposed tests to be added and executed in the working tree upon milestone implementation authorization. Exploratory algorithm behavior has been observed in standalone scratch runners, but proposed tests must not be described as passed or independently verified in the repository until formal execution occurs in the working tree.

### 5.2 Concrete Test Cases

#### Test 1: ASCII Baseline Invariant & Interior Deletions (Integration Test)
- **File:** `tests/contract_tests.rs`
- **Initial State:** `raw_buffer = ""`, `preedit = ""`, `cursor_pos = 0`.
- **Event Sequence:**
  1. `InputString("ami")` $\rightarrow$ `raw_buffer = "ami"`, `preedit = "আমি"`, `cursor_pos = 3`.
  2. `MoveCursorLeft` $\rightarrow$ `cursor_pos = 2`.
  3. `Backspace` $\rightarrow$ deletes `'m'`.
- **Expected Results:**
  - `raw_buffer = "ai"`
  - `cursor_pos = 1`
  - `preedit = "আই"`
- **Status:** Proposed integration test to be added to `crates/bornomala-core/tests/contract_tests.rs` and executed upon milestone authorization; exploratory behavior observed in scratch runner.

#### Test 2: Multibyte Insertion and Byte Alignment (Internal Unit Test)
- **File:** `src/lib.rs` (`mod tests`)
- **Setup & Fixture Verification Contract:**
  1. **Construct Engine:** `let mut engine = BornoEngine::new();` (`raw_buffer = ""`, `source_cursor = 0`, `is_composing = false`).
  2. **Direct Insertion & `update_composition()` Contract:**
     The implementation must verify how `update_composition()` handles direct insertion into a newly constructed engine:
     - In `BornoEngine::new()`, `engine.state.is_composing` is initialized to `false`.
     - When `engine.insert_char_at_cursor(...)` is invoked directly, it mutates `raw_buffer` and calls `self.update_composition()`.
     - In the existing implementation (`crates/bornomala-core/src/lib.rs:1391`), `update_composition()` computes `self.state.is_composing = !self.state.preedit.is_empty();`, thereby transitioning `is_composing` from `false` to `true` whenever preedit is produced.
     - The implementation must verify that direct insertion activates composition in this manner. If the existing implementation requires initializing an explicit composition fixture (e.g. setting `engine.state.is_composing = true;`) prior to internal mutation, the test fixture must initialize it accordingly. Production behavior must not be changed merely to make the test pass.
- **Call Sequence & Grounded Code Path Evidence:**
  1. `engine.insert_char_at_cursor('ক');` (U+0995, 3 bytes: `0xE0 0xA6 0x95`):
     - Inserts `'ক'` at `source_cursor = 0`. `raw_buffer` becomes `"ক"` (len 3).
     - Advances `source_cursor` by `c.len_utf8() = 3` to byte offset 3.
     - `update_composition()` transliterates `"ক"`. In `Tokenizer::tokenize`, `'ক'` does not match Latin rules 1–6 and falls through to Rule 7 (non-Bangla passthrough, `tokenizer.rs:373-381`), emitting `source_range: 0..3, target_range: 0..1`.
     - Preedit is `"ক"`, `is_composing` transitions to `true`.
     - `map_cursor_to_target(3)`: $S = 3 \ge \text{len}$ returns target cursor 1.
  2. `engine.insert_char_at_cursor('b');` (ASCII, 1 byte: `0x62`):
     - Inserts `'b'` at `source_cursor = 3`. `raw_buffer` becomes `"কb"` (4 bytes total: bytes 0..3 for `'ক'`, byte 3..4 for `'b'`).
     - Advances `source_cursor` by `1` to byte offset 4.
     - `update_composition()` transliterates `"কb"`. In `Tokenizer::tokenize`, `'ক'` passes through Rule 7 (`source_range: 0..3, target_range: 0..1`) and `'b'` matches consonant rule `"b"` $\rightarrow$ `"ব"` (`rules.rs:596`, `source_range: 3..4, target_range: 1..2`).
     - Preedit is `"কব"`, `is_composing` remains `true`.
     - `map_cursor_to_target(4)`: $S = 4 \ge \text{len}$ returns target cursor 2.
- **Strict Invariants vs. Target-State Verification Contract:**
  - **Strict Byte-Offset and UTF-8 Boundary Invariants (Guaranteed by Construction):**
    - `assert_eq!(engine.state.raw_buffer, "কb");` (4 bytes total: bytes 0..3 for `'ক'`, byte 3..4 for `'b'`).
    - `assert_eq!(engine.source_cursor, 4);` (exact byte offset, NOT scalar index 2).
    - `assert!(engine.state.raw_buffer.is_char_boundary(engine.source_cursor));` (valid character boundary invariant).
  - **Composed Target-State Expectations:**
    - `assert_eq!(engine.state.cursor_pos, 2);` (observable codepoint offset).
    - `assert_eq!(engine.state.preedit, "কব");` (observable preedit string).
    - `assert_eq!(engine.state.is_composing, true);` (asserted alongside `raw_buffer`, `preedit`, `source_cursor`, and `cursor_pos`).
    - *Reporting Distinction:* While the byte offset and character boundary invariants are strictly guaranteed by construction, the composed target state values (`preedit == "কব"`, `cursor_pos == 2`, `is_composing == true`) are expectations grounded in the tokenizer fallback code paths cited above. These expectations must be formally confirmed by the crate test execution in the working tree upon milestone authorization, rather than presented as already verified facts in the repository.
- **Status:** Proposed internal unit test to be added to `crates/bornomala-core/src/lib.rs` and executed upon milestone authorization; exploratory behavior observed in scratch runner.

#### Test 3: Multibyte Interior Backspace (Internal Unit Test)
- **File:** `src/lib.rs` (`mod tests`)
- **Setup & Fixture Initialization:**
  Construct a valid engine fixture and initialize state consistently with the existing implementation:
  1. Construct engine: `let mut engine = BornoEngine::new();`.
  2. Set composition state: `engine.state.is_composing = true;`.
  3. Set raw buffer: `engine.state.raw_buffer = "কb".to_string();` (4 bytes: `'ক'` at 0..3, `'b'` at 3..4).
  4. Initialize preedit and spans via transliteration for `"কb"`:
     `let (preedit, spans, _) = phonetic::PhoneticEngine::transliterate(&engine.state.raw_buffer);`
     `engine.state.preedit = preedit;`
     `engine.spans = spans;`
  5. Position source cursor between `'ক'` and `'b'`: `engine.source_cursor = 3;` (valid character boundary: `engine.state.raw_buffer.is_char_boundary(3)` is `true`).
  6. Synchronize initial target cursor position:
     `engine.state.cursor_pos = phonetic::PhoneticEngine::map_cursor_to_target(engine.source_cursor, &engine.state.raw_buffer, &engine.spans, &engine.state.preedit);`
- **Call Sequence:** `engine.delete_before_cursor();`.
- **Expected Results:**
  - **Source Byte Arithmetic (Strict Contract):**
    - Deleting the 3-byte Bengali character `'ক'` preceding byte offset 3 leaves `raw_buffer = "b"` (1 byte).
    - `source_cursor = 0` (decremented by 3 bytes from 3 to 0, not by 1 scalar index).
    - Invariant holds: `engine.state.raw_buffer.is_char_boundary(engine.source_cursor) == true` and `engine.source_cursor <= engine.state.raw_buffer.len()`.
  - **Composition & Preedit Re-evaluation:**
    - Following buffer mutation, `delete_before_cursor()` invokes `update_composition()`, which re-transliterates `"b"`, updates `spans` and `state.preedit`, and remaps `cursor_pos = map_cursor_to_target(0, ...)`.
    - Resulting values (`state.preedit = "ব"`, `state.cursor_pos = 0`) are expected based on transliteration of `"b"`, but are not asserted solely from raw byte arithmetic; these expectations must be confirmed by the actual implementation and crate test.
- **Status:** Proposed internal unit test to be added to `crates/bornomala-core/src/lib.rs` and executed upon milestone authorization; exploratory arithmetic observed in scratch runner.

#### Test 4: Multibyte Forward Delete (Internal Unit Test)
- **File:** `src/lib.rs` (`mod tests`)
- **Setup & Fixture Initialization:**
  Construct a valid engine fixture and initialize state consistently with the existing implementation:
  1. Construct engine: `let mut engine = BornoEngine::new();`.
  2. Set composition state: `engine.state.is_composing = true;`.
  3. Set raw buffer: `engine.state.raw_buffer = "কb".to_string();` (4 bytes: `'ক'` at 0..3, `'b'` at 3..4).
  4. Initialize preedit and spans via transliteration for `"কb"`:
     `let (preedit, spans, _) = phonetic::PhoneticEngine::transliterate(&engine.state.raw_buffer);`
     `engine.state.preedit = preedit;`
     `engine.spans = spans;`
  5. Position source cursor at start (before `'ক'`): `engine.source_cursor = 0;` (valid character boundary: `engine.state.raw_buffer.is_char_boundary(0)` is `true`).
  6. Synchronize initial target cursor position:
     `engine.state.cursor_pos = phonetic::PhoneticEngine::map_cursor_to_target(engine.source_cursor, &engine.state.raw_buffer, &engine.spans, &engine.state.preedit);`
- **Call Sequence:** `engine.delete_after_cursor();`.
- **Expected Results:**
  - **Source Byte Arithmetic (Strict Contract):**
    - Deleting the 3-byte Bengali character `'ক'` at source cursor byte offset 0 leaves `raw_buffer = "b"` (1 byte; bytes 0..3 drained, trailing bytes shifted left).
    - `source_cursor = 0` (preserved at byte offset 0).
    - Invariant holds: `engine.state.raw_buffer.is_char_boundary(engine.source_cursor) == true` and `engine.source_cursor <= engine.state.raw_buffer.len()`.
  - **Composition & Preedit Re-evaluation:**
    - Following buffer mutation, `delete_after_cursor()` invokes `update_composition()`, which re-transliterates `"b"`, updates `spans` and `state.preedit`, and remaps `cursor_pos = map_cursor_to_target(0, ...)`.
    - Resulting values (`state.preedit = "ব"`, `state.cursor_pos = 0`) are expected based on transliteration of `"b"`, but are not asserted solely from raw byte arithmetic; these expectations must be confirmed by the actual implementation and crate test.
- **Status:** Proposed internal unit test to be added to `crates/bornomala-core/src/lib.rs` and executed upon milestone authorization; exploratory behavior observed in scratch runner.

#### Test 5: Boundary Deletion Handled Semantics at Buffer Limits (Integration Test)
- **File:** `tests/contract_tests.rs`
- **Assertion Contract Requirement:**
  After each and every event dispatched in this test, the integration test must explicitly assert the complete observable state across all 5 properties:
  1. `res.handled` (boolean event-dispatch outcome).
  2. `res.state.raw_buffer` (underlying source string).
  3. `res.state.preedit` (composed visual preedit string).
  4. `res.state.cursor_pos` (target codepoint caret index).
  5. `res.state.is_composing` (composition active status).
- **Initial Composition:**
  `let res0 = engine.process_event(EngineEvent::InputString("k".to_string()));`
  - Assertions:
    - `assert_eq!(res0.handled, true);`
    - `assert_eq!(res0.state.raw_buffer, "k");`
    - `assert_eq!(res0.state.preedit, "ক");`
    - `assert_eq!(res0.state.cursor_pos, 1);`
    - `assert_eq!(res0.state.is_composing, true);`
- **Event Sequence & Verified State Expectations:**
  1. `let res1 = engine.process_event(EngineEvent::MoveCursorLeft);`:
     - Post-state Assertions:
       - `assert_eq!(res1.handled, true);`
       - `assert_eq!(res1.state.raw_buffer, "k");`
       - `assert_eq!(res1.state.preedit, "ক");`
       - `assert_eq!(res1.state.cursor_pos, 0);`
       - `assert_eq!(res1.state.is_composing, true);`
     - Evidence / Reference: `crates/bornomala-core/src/lib.rs:798-808` (`cursor_pos -= 1`, `source_cursor = map_target_to_source(0) = 0`). Confirmed zero discrepancy with source.
  2. `let res2 = engine.process_event(EngineEvent::Backspace);` (at composition start):
     - Post-state Assertions:
       - `assert_eq!(res2.handled, false);`
       - `assert_eq!(res2.state.raw_buffer, "k");`
       - `assert_eq!(res2.state.preedit, "ক");`
       - `assert_eq!(res2.state.cursor_pos, 0);`
       - `assert_eq!(res2.state.is_composing, true);`
     - Evidence / Reference: Supported by production dispatch in `crates/bornomala-core/src/lib.rs:641-647` (`else if self.source_cursor == 0 && self.state.cursor_pos == 0 { ... handled: false }`). Confirmed zero discrepancy with source.
  3. `let res3 = engine.process_event(EngineEvent::MoveCursorRight);`:
     - Post-state Assertions:
       - `assert_eq!(res3.handled, true);`
       - `assert_eq!(res3.state.raw_buffer, "k");`
       - `assert_eq!(res3.state.preedit, "ক");`
       - `assert_eq!(res3.state.cursor_pos, 1);`
       - `assert_eq!(res3.state.is_composing, true);`
     - Evidence / Reference: `crates/bornomala-core/src/lib.rs:819-830` (`cursor_pos += 1`, `source_cursor = map_target_to_source(1) = 1`). Confirmed zero discrepancy with source.
  4. `let res4 = engine.process_event(EngineEvent::Delete);` (at composition end):
     - Action & Dispatch: `self.source_cursor >= self.state.raw_buffer.len()` ($1 \ge 1$) evaluates to `true` in `crates/bornomala-core/src/lib.rs:659-665`.
     - Returns `handled: false`. Because no deletion occurs and the engine returns `state: self.state.clone()`, the cursor and composition remain completely unchanged.
     - Post-state Assertions:
       - `assert_eq!(res4.handled, false);`
       - `assert_eq!(res4.state.raw_buffer, "k");`
       - `assert_eq!(res4.state.preedit, "ক");`
       - `assert_eq!(res4.state.cursor_pos, 1);`
       - `assert_eq!(res4.state.is_composing, true);`
     - Evidence / Reference: Supported by production dispatch in `crates/bornomala-core/src/lib.rs:659-665`. Confirmed zero discrepancy with source.
  5. `let res5 = engine.process_event(EngineEvent::Backspace);` (at composition end):
     - Starting State: Completely unchanged post-state from step 4 (`raw_buffer: "k"`, `preedit: "ক"`, `cursor_pos: 1`, `is_composing: true`).
     - Action & Dispatch: `self.source_cursor == 1 > 0`, so dispatch invokes `delete_before_cursor()` (`crates/bornomala-core/src/lib.rs:648-655`).
     - Character `'k'` preceding byte 1 is deleted; `raw_buffer` becomes empty, triggering `clear_composition()` (`lib.rs:1366-1368`).
     - Post-state Assertions:
       - `assert_eq!(res5.handled, true);`
       - `assert_eq!(res5.state.raw_buffer, "");`
       - `assert_eq!(res5.state.preedit, "");`
       - `assert_eq!(res5.state.cursor_pos, 0);`
       - `assert_eq!(res5.state.is_composing, false);`
     - Evidence / Reference: Supported by `crates/bornomala-core/src/lib.rs:648-655` and `lib.rs:1366-1368`. Confirmed zero discrepancy with source.
  6. **Reset & Delete at Start:**
     - 6a. Re-input: `let res6a = engine.process_event(EngineEvent::InputString("k".to_string()));`:
       - Assertions:
         - `assert_eq!(res6a.handled, true);`
         - `assert_eq!(res6a.state.raw_buffer, "k");`
         - `assert_eq!(res6a.state.preedit, "ক");`
         - `assert_eq!(res6a.state.cursor_pos, 1);`
         - `assert_eq!(res6a.state.is_composing, true);`
     - 6b. Move cursor to start: `let res6b = engine.process_event(EngineEvent::MoveCursorLeft);`:
       - Assertions:
         - `assert_eq!(res6b.handled, true);`
         - `assert_eq!(res6b.state.raw_buffer, "k");`
         - `assert_eq!(res6b.state.preedit, "ক");`
         - `assert_eq!(res6b.state.cursor_pos, 0);`
         - `assert_eq!(res6b.state.is_composing, true);`
     - 6c. Forward delete at start: `let res6c = engine.process_event(EngineEvent::Delete);`:
       - Action & Dispatch: `self.source_cursor == 0 < raw_buffer.len() (1)`, so dispatch invokes `delete_after_cursor()` (`crates/bornomala-core/src/lib.rs:666-673`). Character `'k'` is deleted; `raw_buffer` becomes empty, triggering `clear_composition()` (`lib.rs:1366-1368`).
       - Post-state Assertions:
         - `assert_eq!(res6c.handled, true);`
         - `assert_eq!(res6c.state.raw_buffer, "");`
         - `assert_eq!(res6c.state.preedit, "");`
         - `assert_eq!(res6c.state.cursor_pos, 0);`
         - `assert_eq!(res6c.state.is_composing, false);`
     - Evidence / Reference: Supported by `crates/bornomala-core/src/lib.rs:666-673` and `lib.rs:1366-1368`; verified by existing test pattern in `crates/bornomala-core/tests/contract_tests.rs:285-295`. Confirmed zero discrepancy with source.
- **Status:** Proposed integration test to be added to `crates/bornomala-core/tests/contract_tests.rs` and executed upon milestone authorization; exploratory behavior observed in scratch runner.

#### Test 6: In-Preedit Cursor Navigation and Interior Insertion (Integration Test)
- **File:** `tests/contract_tests.rs`
- **Initial State:** `raw_buffer = ""`, `preedit = ""`, `cursor_pos = 0`.
- **Event Sequence & Step-by-Step Expected Values:**
  1. `InputString("bha".to_string())`:
     - `raw_buffer = "bha"` (3 bytes: `"bh"` at 0..2, `'a'` at 2..3).
     - `preedit = "ভা"` (2 codepoints: U+09AD `'ভ'`, U+09BE `'া'`).
     - `cursor_pos = 2`.
  2. `MoveCursorLeft`:
     - `cursor_pos` decrements from `2` to `1` (positioned between `'ভ'` and `'া'`).
     - `map_target_to_source(1)` resolves to byte offset `2` (the exact boundary between `"bh"` and `"a"`).
  3. `InputChar('k')`:
     - `'k'` is inserted at byte offset `2`.
     - `raw_buffer` becomes `"bhka"` (4 bytes: `"bh"` at 0..2, `'k'` at 2..3, `'a'` at 3..4).
     - `update_composition()` tokenizes `"bhka"` $\rightarrow$ `"ভকা"` (3 codepoints: `'ভ'` at 0..1, `'ক'` at 1..2, `'া'` at 2..3).
     - `source_cursor` advances from `2` to `3`.
     - `map_cursor_to_target(3)` maps to target codepoint `2` (after `'ভক'`, before `'া'`).
     - Observable `state.preedit = "ভকা"`.
     - Observable `state.cursor_pos = 2`.
- **Status:** Proposed integration test to be added to `crates/bornomala-core/tests/contract_tests.rs` and executed upon milestone authorization; exploratory behavior observed in scratch runner.

#### Test 7: Pure Mapping & Left Affinity Invariant with Digraph (Internal Unit Test)
- **File:** `src/lib.rs` (`mod tests`)
- **Explicit Setup & Execution Sequence:**
  1. **Tokenize `"kh"` via existing API:** Call `let (tokenized_preedit, tokenized_spans) = phonetic::Tokenizer::tokenize("kh");`. Real tokenization output produces preedit `"খ"` (1 codepoint) and a single span `TokenSpan { source_range: 0..2, target_range: 0..1 }`.
  2. **Initialize Engine State:** Construct engine `let mut engine = BornoEngine::new();` and set `engine.state.raw_buffer = "kh".to_string();` so `raw_buffer` is exactly `"kh"`.
  3. **Set Preedit:** Set `engine.state.preedit = tokenized_preedit;` (expected `"খ"`).
  4. **Set Spans:** Set `engine.spans = tokenized_spans;` to the actual spans returned by tokenization.
  5. **Invoke Mapping & Assert Expected Results:**
     - Forward mapping via `phonetic::PhoneticEngine::map_cursor_to_target(source_cursor, &engine.state.raw_buffer, &engine.spans, &engine.state.preedit)`:
       - Source offset `0` $\rightarrow$ Target position `0`.
       - Source offset `1` $\rightarrow$ Target position `0` (Left affinity interior collapse).
       - Source offset `2` $\rightarrow$ Target position `1` (Token boundary end).
     - Inverse mapping via `engine.map_target_to_source(target_pos)`:
       - Invariant contract: for each inverse mapping call, explicitly assert that the returned source offset is no greater than `raw_buffer.len()` and satisfies `raw_buffer.is_char_boundary(source_pos)`.
       - Target position `0` $\rightarrow$ Source offset `0`:
         `let s0 = engine.map_target_to_source(0); assert!(s0 <= engine.state.raw_buffer.len()); assert!(engine.state.raw_buffer.is_char_boundary(s0)); assert_eq!(s0, 0);`
       - Target position `1` $\rightarrow$ Source offset `2`:
         `let s1 = engine.map_target_to_source(1); assert!(s1 <= engine.state.raw_buffer.len()); assert!(engine.state.raw_buffer.is_char_boundary(s1)); assert_eq!(s1, 2);`
- **Status:** Proposed internal unit test to be added to `crates/bornomala-core/src/lib.rs` and executed upon milestone authorization; exploratory behavior observed in scratch runner.

#### Test 8: Public Target-Cursor Ingress Clamping (Integration Test)
- **File:** `tests/contract_tests.rs`
- **Initial State:** `raw_buffer = ""`, `preedit = ""`, `cursor_pos = 0`.
- **Event Sequence & Assertions:**
  1. `InputString("ami".to_string())` $\rightarrow$ `raw_buffer = "ami"`, `preedit = "আমি"`, `cursor_pos = 3`.
  2. Send public ingress event `SetCursor(1000)`:
     - Target cursor clamps to `preedit.chars().count() = 3`.
     - Observable `state.cursor_pos` is `3`.
  3. Send `Backspace`:
     - Character before cursor (`'i'`) is deleted.
     - Resulting `state.raw_buffer` is `"am"`.
     - Resulting `state.preedit` is `"আম"`.
     - Resulting `state.cursor_pos` is `2`.
     - `handled` is `true`.
- **Status:** Proposed integration test to be added to `crates/bornomala-core/tests/contract_tests.rs` and executed upon milestone authorization; exploratory behavior observed in scratch runner.

#### Test 9: Deterministic Proportional Interpolation & Multibyte Boundary Adjustment (Internal Unit Test)
- **File:** `src/lib.rs` (`mod tests`)
- **Nature of Fixtures (Synthetic Arithmetic Tests):** These test fixtures are explicitly identified as **synthetic coordinate arithmetic tests** to verify internal proportional interpolation formulas and UTF-8 byte boundary alignment mechanics in complete isolation. They do **not** represent production tokenizer behavior or real linguistic transliteration rules.
- **Span Ownership & Bounds Assertion Requirements (Borrow Conflict Prevention):**
  To perform exact source-span bounds assertions on synthetic spans without creating Rust borrow-checker conflicts:
  1. **Cloned Local Fixture Requirement:** Tests must retain a cloned local `TokenSpan` fixture prior to assigning it to the engine:
     `let test_span = TokenSpan { source_range: ..., target_range: ... };`
     `engine.spans = vec![test_span.clone()];`
     Retrieving a reference directly from `engine.spans` (e.g. `let test_span = &engine.spans[0];`) is strictly prohibited, as holding an immutable reference to `engine` while subsequently invoking mutable operations or methods would trigger borrow conflicts.
  2. **Bounds & Invariant Verification:** Every mapped source offset `s = engine.map_target_to_source(t)` must be asserted against the independent local `test_span` fixture and raw buffer:
     - `assert!(s <= engine.state.raw_buffer.len());` (Does not exceed raw buffer length).
     - `assert!(engine.state.raw_buffer.is_char_boundary(s));` (Satisfies UTF-8 character boundary invariant in `raw_buffer`).
     - `assert!((test_span.source_range.start..=test_span.source_range.end).contains(&s));` (Mapped offset falls strictly within the exact synthetic span under test, inclusively).
     - `assert_eq!(s, expected);` (Matches deterministic expected byte offset).
- **Multibyte Character Case (Synthetic Arithmetic Test):**
  - Setup: `engine.state.raw_buffer = "কa".to_string()` (4 bytes: `'ক'` at 0..3, `'a'` at 3..4).
  - Preedit: `engine.state.preedit = "XY".to_string()` (2 codepoints).
  - Span Retention & Assignment (Cloned Local Fixture):
    `let test_span = TokenSpan { source_range: 0..4, target_range: 0..2 }; engine.spans = vec![test_span.clone()];` (both $S_{start}=0$ and $S_{end}=4$ are valid character boundaries).
  - Assertions:
    - Boundary match ($t=0$): `let s0 = engine.map_target_to_source(0); assert!(s0 <= engine.state.raw_buffer.len()); assert!(engine.state.raw_buffer.is_char_boundary(s0)); assert!((test_span.source_range.start..=test_span.source_range.end).contains(&s0)); assert_eq!(s0, 0);`
    - Boundary match ($t=2$): `let s2 = engine.map_target_to_source(2); assert!(s2 <= engine.state.raw_buffer.len()); assert!(engine.state.raw_buffer.is_char_boundary(s2)); assert!((test_span.source_range.start..=test_span.source_range.end).contains(&s2)); assert_eq!(s2, 4);`
    - Interpolated candidate in multibyte ($t=1$): yields candidate offset 2 (inside `'ক'`). Backward adjustment moves to byte 0 without crossing $S_{start} = 0$ (bounded by `0.max(0).min(4) = 0`):
      `let s1 = engine.map_target_to_source(1); assert!(s1 <= engine.state.raw_buffer.len()); assert!(engine.state.raw_buffer.is_char_boundary(s1)); assert!((test_span.source_range.start..=test_span.source_range.end).contains(&s1)); assert_eq!(s1, 0);`
- **ASCII Uniform Proportional Case (Synthetic Arithmetic Test):**
  - Setup: `engine.state.raw_buffer = "abcdef".to_string()` (6 bytes).
  - Preedit: `engine.state.preedit = "XYZ".to_string()` (3 codepoints).
  - Span Retention & Assignment (Cloned Local Fixture):
    `let test_span = TokenSpan { source_range: 0..6, target_range: 0..3 }; engine.spans = vec![test_span.clone()];`
  - Assertions:
    - Boundary match ($t=0$): `let a0 = engine.map_target_to_source(0); assert!(a0 <= engine.state.raw_buffer.len()); assert!(engine.state.raw_buffer.is_char_boundary(a0)); assert!((test_span.source_range.start..=test_span.source_range.end).contains(&a0)); assert_eq!(a0, 0);`
    - Interior ratio $1/3$ ($t=1$): $\text{round}(1/3 \times 6) = 2$. Byte 2 is a character boundary:
      `let a1 = engine.map_target_to_source(1); assert!(a1 <= engine.state.raw_buffer.len()); assert!(engine.state.raw_buffer.is_char_boundary(a1)); assert!((test_span.source_range.start..=test_span.source_range.end).contains(&a1)); assert_eq!(a1, 2);`
    - Interior ratio $2/3$ ($t=2$): $\text{round}(2/3 \times 6) = 4$. Byte 4 is a character boundary:
      `let a2 = engine.map_target_to_source(2); assert!(a2 <= engine.state.raw_buffer.len()); assert!(engine.state.raw_buffer.is_char_boundary(a2)); assert!((test_span.source_range.start..=test_span.source_range.end).contains(&a2)); assert_eq!(a2, 4);`
    - Boundary match ($t=3$): `let a3 = engine.map_target_to_source(3); assert!(a3 <= engine.state.raw_buffer.len()); assert!(engine.state.raw_buffer.is_char_boundary(a3)); assert!((test_span.source_range.start..=test_span.source_range.end).contains(&a3)); assert_eq!(a3, 6);`
- **Status:** Proposed internal unit test to be added to `crates/bornomala-core/src/lib.rs` and executed upon milestone authorization; exploratory arithmetic observed in scratch runner.

---

## 6. Scope Boundaries & Implementation Plan

### 6.1 Authorized Implementation Scope
When approved, modifications are strictly restricted to:
1. `crates/bornomala-core/src/lib.rs` (mutation logic, boundary helper, and internal unit tests).
2. `crates/bornomala-core/tests/contract_tests.rs` (public integration tests).

### 6.2 Strict Exclusions
- **Phonetic Engine & Tokenizer Rules:** No changes to `rules.rs`, `candidates.rs`, or `tokenizer.rs`.
- **Dictionary & Fixed Engine:** No changes to `dictionary/` or `layout/`.
- **Public API & C ABI:** `BornoEngine` public methods and `BornoStateC` remain 100% unchanged.
- **Platform Adapters:** No edits to Windows TSF, Linux Fcitx 5, or Tauri wrapper.
- **Pre-Existing Work:** All WP-01 and WP-02 modifications are preserved untouched.

---

## 7. Verification Targets & Execution Plan

### 7.1 Confirmed Test Targets in `bornomala-core`

The `bornomala-core` package contains exactly 10 test targets:

1. `cargo test -p bornomala-core --lib`
2. `cargo test -p bornomala-core --test candidate_provenance_tests`
3. `cargo test -p bornomala-core --test contract_tests`
4. `cargo test -p bornomala-core --test dictionary_benchmarks`
5. `cargo test -p bornomala-core --test dictionary_tests`
6. `cargo test -p bornomala-core --test engine_config_tests`
7. `cargo test -p bornomala-core --test ffi_tests`
8. `cargo test -p bornomala-core --test layout_tests`
9. `cargo test -p bornomala-core --test phonetic_tests`
10. `cargo test -p bornomala-core --test unicode_golden_tests`

*(Note: There is no `regression_tests` binary in `bornomala-core`.)*

### 7.2 Implementation Verification Commands

Once implementation is authorized, verification will execute:

1. **Compilation Check:**
   ```powershell
   cargo check -p bornomala-core
   ```
2. **Run New Internal Cursor Unit Tests:**
   ```powershell
   cargo test -p bornomala-core --lib internal_cursor
   ```
3. **Run Contract Integration Tests:**
   ```powershell
   cargo test -p bornomala-core --test contract_tests
   ```
4. **Run Full Crate Test Suite Across All 10 Targets:**
   ```powershell
   cargo test -p bornomala-core --lib
   cargo test -p bornomala-core --test candidate_provenance_tests
   cargo test -p bornomala-core --test contract_tests
   cargo test -p bornomala-core --test dictionary_benchmarks
   cargo test -p bornomala-core --test dictionary_tests
   cargo test -p bornomala-core --test engine_config_tests
   cargo test -p bornomala-core --test ffi_tests
   cargo test -p bornomala-core --test layout_tests
   cargo test -p bornomala-core --test phonetic_tests
   cargo test -p bornomala-core --test unicode_golden_tests
   ```
5. **Code Hygiene & Non-Mutating Formatting Check:**
   Verify formatting compliance on modified files only without altering existing workspace code:
   ```powershell
   rustfmt --edition 2021 --check crates/bornomala-core/src/lib.rs crates/bornomala-core/tests/contract_tests.rs
   ```
   *(Note: Passing all 10 core targets verifies the core engine logic; full workspace and native platform integration remain subject to environment prerequisites.)*
