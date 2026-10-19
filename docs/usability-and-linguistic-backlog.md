# Usability and Linguistic Backlog

> **Purpose:** Formal tracking and acceptance test backlog for usability, punctuation, and linguistic refinements identified during interactive testing.  
> **Master Source of Truth:** [`ROADMAP.md`](../ROADMAP.md)  
> **Governance:** Items in this backlog are strictly mapped to their designated milestones. When a target milestone begins, its implementation plan must include the designated backlog items in its acceptance test gates.

---

## 1. Backlog Summary Table

| ID | Title | Discovered In | Target Milestone | Severity / Type | Current Status |
| :---: | :--- | :---: | :---: | :---: | :---: |
| [`BACKLOG-01`](#backlog-01-bengali-dáñṛi--phonetic-input--codepoint-typo) | Bengali Dáñṛi (`।`) Phonetic Input & Codepoint Bug | M8 | **M9** | Bug & Usability | `Verified in M9` |
| [`BACKLOG-02`](#backlog-02-casual-vowel-ambiguity-auto-suggestion-tomake--তোমাকে) | Casual Vowel Ambiguity Auto-Suggestion (`tomake` → `তোমাকে`) | M8 | **M9** | Usability / Phonetics | `Verified in M9` |
| [`BACKLOG-03`](#backlog-03-n-twa-bidhan-nasal-suggestion-bornomala--বর্ণমালা) | ণ-ত্ব বিধান Nasal Homophone Suggestion (`bornomala` → `বর্ণমালা`) | M8 | **M9** | Usability / Lexicon | `Verified in M9` |
| [`BACKLOG-04`](#backlog-04-accent-color-selector-for-high-contrast--dark-modes) | Accent Color Selector for High Contrast & Dark Modes | M9 | **M9 Refinement** | UI / Customization | `Planned` |
| [`BACKLOG-05`](#backlog-05-light-theme-contrast--legibility-overhaul) | Light Theme Contrast & Legibility Overhaul | M9 | **M9 Refinement** | UI / Accessibility | `Planned` |
| [`BACKLOG-06`](#backlog-06-candidate-window-orientation-default-horizontal) | Candidate Window Orientation Default (`Horizontal`) | M9 | **M9 Refinement** | Usability / Defaults | `Planned` |
| [`BACKLOG-07`](#backlog-07-windows-tsf-configuration-path-synchronization) | Windows TSF Configuration Path Synchronization | M9 | **M9 Refinement** | Native Adapter | `Planned` |
| [`BACKLOG-08`](#backlog-08-windows-input-method-switching-scope-per-window-vs-global) | Windows Input Method Switching Scope (Per-Window vs Global) | M9 | **M9 / M12** | OS Integration | `Documented & Planned` |
| [`BACKLOG-09`](#backlog-09-windows-tsf-language-profile-ordering-bangladesh-priority) | Windows TSF Language Profile Locale Ordering (Bangladesh Priority) | M9 | **M9 Refinement** | Native TSF | `Planned` |
| [`BACKLOG-10`](#backlog-10-independent-theming-for-floating-bar-vs-sandbox-app) | Independent Theming & Appearance for Floating Bar vs. Sandbox App | M9 | **M9 Refinement** | UI / Customization | `Planned` |
| [`BACKLOG-11`](#backlog-11-deterministic-2-way-hotkey-toggle-phonetic--english-sandbox-parity) | Deterministic 2-Way Hotkey Toggle (Phonetic ↔ English) Sandbox Parity | M9 | **M9 Refinement** | Ergonomics / Hotkeys | `Planned` |
| [`BACKLOG-12`](#backlog-12-ergonomic-1-press-down-arrow--candidate-navigation) | Ergonomic 1-Press Down Arrow (↓) Candidate Navigation | M9 | **M9 Refinement** | Core / Ergonomics | `Planned` |
| [`BACKLOG-13`](#backlog-13-modular-sandbox-dashboard-section-reordering) | Modular Sandbox Dashboard Section Reordering | M9 | **M9 Refinement** | UI / Customization | `Planned` |
| [`BACKLOG-14`](#backlog-14-system-wide-floating-detached-virtual-keyboard) | System-Wide Floating Detached Virtual Keyboard | M9 | **Milestone M12** | Desktop Utility | `Deferred to M12` |

---

## 2. Detailed Item Specifications & Verification Tests

### BACKLOG-01: Bengali Dáñṛi (`।`) Phonetic Input & Codepoint Typo

- **Discovery Context:**  
  During Milestone M8 interactive testing in Ubuntu WSL2, typing `আমি খালিদ বলছি.` in phonetic mode emitted an ASCII period (`.`) rather than a Bengali Dáñṛi (`।`).
- **Technical Analysis:**
  1. **Codepoint Typo:** In [`crates/bornomala-core/src/phonetic/tokenizer.rs:L141`](../crates/bornomala-core/src/phonetic/tokenizer.rs#L141), the tokenizer emits `\u{09E4}` (an unassigned/currency sign in Unicode) instead of `\u{0964}` (Bengali Danda / Dari `।`), and `\u{09E5}` instead of `\u{0965}` (Double Dari `॥`).
  2. **Typing Convention:** Users expect typing `..` (double period) to emit `।` (matching standard Avro conventions), with an optional user configuration setting to auto-convert single `.` to `।`.
- **Target Milestone:** **Milestone M9 (Customization & Usability)**.
- **Resolution Plan:**
  1. Correct `\u{09E4}` $\to$ `\u{0964}` and `\u{09E5}` $\to$ `\u{0965}` in `tokenizer.rs`.
  2. Implement `..` $\to$ `।` tokenization rule in phonetic mode.
  3. Provide a configuration toggle in `BornoConfig`: `auto_convert_period_to_dari: bool` (default: false for double-dot, true if opted-in).
- **Status:** **Verified in M9** via `test_gate_01_phonetic_period_sequence_matrix` and `test_gate_02_single_period_auto_dari_with_safeguards`.

---

### BACKLOG-02: Casual Vowel Ambiguity Auto-Suggestion (`tomake` → `তোমাকে`)

- **Discovery Context:**  
  Typing `tomake` emits `তমাকে` with no candidate suggestion for `তোমাকে`, even though `তোমাকে` exists in `lexicon.tsv` (rank 8644).
- **Technical Analysis:**
  1. In Avro phonetic grammar ([`rules.rs:L166-L172`](../crates/bornomala-core/src/phonetic/rules.rs#L166-L172)), lowercase `o` is the *inherent vowel bridge* (`অ`, `dependent: None`). Lowercase `o` emits no dependent vowel kar.
  2. The explicit o-kar (`ো`) is mapped to uppercase `O` (`tOmake` $\to$ `তোমাকে`).
  3. Milestone M6 introduced the *Phonetic Ambiguity Pipeline* ([`ambiguity.rs`](../crates/bornomala-core/src/dictionary/ambiguity.rs)), but restricted rewrites to consonant homophones (`kh` $\to$ `ক্ষ`, `gg` $\to$ `জ্ঞ`, `sh` $\to$ `ষ`) to prevent combinatorial latency explosion.
- **Target Milestone:** **Milestone M9 / M10 (Customization / Prediction)**.
- **Resolution Plan:**
  1. Add a bounded vowel-alternation rewrite rule: where an inherent `o` follows a consonant, generate a candidate variant with o-kar (`O` $\to$ `ো`).
  2. Gate the variant against the embedded trie (`PrefixTrie::exact_search`). Because `তোমাকে` exists in the lexicon, it surfaces with origin `CandidateKind::ExactLexiconMatch` (badge `✓ Word`).
- **Status:** **Verified in M9** via `test_gate_03_ambiguity_vowel_tomake_ranked_slot_1`.

---

### BACKLOG-03: ণ-ত্ব বিধান Nasal Homophone Suggestion (`bornomala` → `বর্ণমালা`)

- **Discovery Context:**  
  In standard Bengali grammar (ণ-ত্ব বিধান / *N-twa Bidhan*), Tatsama words with Reph (`র্`) require মূর্ধন্য-ণ (`ণ`), making **`বর্ণমালা`** the grammatically correct spelling. However, typing casual lowercase `bornomala` emits `বর্নমালা` (dental `ন`), requiring the user to know that uppercase `N` must be typed (`borNomala`).
- **Technical Analysis:**
  1. Lowercase `n` maps to `ন` ([`rules.rs:L628`](../crates/bornomala-core/src/phonetic/rules.rs#L628)).
  2. Uppercase `N` maps to `ণ` ([`rules.rs:L613`](../crates/bornomala-core/src/phonetic/rules.rs#L613)).
  3. Casual users frequently type all-lowercase Latin strings for Bengali words with `ণ`.
- **Target Milestone:** **Milestone M9 / M10 (Customization / Prediction)**.
- **Resolution Plan:**
  1. Add a phonetic ambiguity rewrite rule for post-Reph nasals: rewrite `rn` $\to$ `rN` (`borNomala`).
  2. Validate against the lexicon trie: `বর্ণমালা` exists in the dictionary, so `বর্ণমালা` will be surfaced as a verified lexicon candidate.
- **Status:** **Verified in M9** via `test_gate_04_ambiguity_nasal_bornomala_and_negative_cases`.

---

### BACKLOG-04: Accent Color Selector for High Contrast & Dark Modes

- **Discovery Context:**  
  During M9 desktop testing, users loved the OLED High Contrast theme but expressed the desire to customize the accent color beyond fixed yellow (`#facc15`).
- **Technical Analysis:**
  1. `BornoConfig` currently binds theme solely via `ThemeId` (`dark-glass`, `light-glass`, `high-contrast`).
  2. High Contrast and focus indicators can support a curated `AccentColor` enum (`Yellow`, `Cyan`, `Emerald`, `Indigo`, `Rose`, `White`).
  3. The chosen accent updates `--accent-focus` and `--border-focus` across the UI, and translates to Windows GDI selection brushes in `candidate_window.cpp`.
- **Target Milestone:** **Milestone M9 Refinement Pass**.
- **Resolution Plan:**
  1. Add `accent_color: AccentColor` to `BornoConfig` in `crates/bornomala-core/src/config.rs`.
  2. Add interactive accent color picker chips in `SettingsDrawer.tsx`.
  3. Map accent color in GDI `candidate_window.cpp`.

---

### BACKLOG-05: Light Theme Contrast & Legibility Overhaul

- **Discovery Context:**  
  Interactive testing showed that `Light Glass` suffered from muddy contrast. The candidate panel in `SandboxTypingArea.tsx` retained hardcoded dark background (`rgba(11, 15, 25)`) and `#ffffff` text, resulting in inverted dark patches on a white page. Preview card text in settings was white-on-white.
- **Technical Analysis:**
  1. Eliminate all hardcoded dark fills from `SandboxTypingArea.tsx`, binding them to CSS custom properties (`var(--bg-card)`, `var(--text-primary)`, `var(--border-subtle)`).
  2. Restyle Light Glass with crisp white cards (`#ffffff`), dark slate typography (`#0f172a`), soft borders (`#cbd5e1`), and high-legibility candidate pills (`#e0f2fe` fill with `#0369a1` text).
- **Target Milestone:** **Milestone M9 Refinement Pass**.

---

### BACKLOG-06: Candidate Window Orientation Default (`Horizontal`)

- **Discovery Context:**  
  Users preferred the Horizontal candidate bar as the primary default. While `SettingsDrawer.tsx` labeled Horizontal as default, `default_candidate_orientation()` in `crates/bornomala-core/src/config.rs` returned `CandidateOrientation::Vertical`.
- **Resolution Plan:**
  1. Align `default_candidate_orientation()` to `CandidateOrientation::Horizontal` in core schema.
  2. Update unit and persistence test assertions to enforce `Horizontal` as the canonical default.
- **Target Milestone:** **Milestone M9 Refinement Pass**.

---

### BACKLOG-07: Windows TSF Configuration Path Synchronization

- **Discovery Context:**  
  `native/borno-tsf/borno_tsf.cpp` resolved config from `%APPDATA%\com.bornomala.desktop\config.json`, whereas Tauri v2 (`tauri.conf.json`) uses identifier `"com.bornomala.sandbox"`, writing to `%APPDATA%\com.bornomala.sandbox\config.json`.
- **Resolution Plan:**
  1. Update `borno_tsf.cpp` `GetConfigFilePath()` to inspect `com.bornomala.sandbox\config.json` first, falling back to `com.bornomala.desktop\config.json`.
  2. Ensure changes made in the settings drawer dynamically propagate to the TSF floating candidate window in real time.
- **Target Milestone:** **Milestone M9 Refinement Pass**.

---

### BACKLOG-08: Windows Input Method Switching Scope (Per-Window vs Global)

- **Discovery Context:**  
  When navigating between Notepad, Google Chrome, and Sublime Text, Windows resets the active keyboard to default `ENG`.
- **Technical Analysis:**
  1. By default, Windows 10/11 isolates input methods per window thread (`ITfThreadMgr`), controlled by the Windows setting *"Let me use a different input method for each app window"*.
  2. If a user sets Borno as the *Windows Default Input Method* or unchecks per-app switching, Borno remains active across all windows, with `F12`/`F4` toggling Bangla vs English internally.
  3. For global hotkey interception outside TSF focus (similar to legacy Avro background hook), a dedicated background agent or tray daemon (`bornomala-tray.exe`) is targeted for M12 packaging.
- **Target Milestone:** **Milestone M9 (Documentation & User Guidance) / Milestone M12 (Background Daemon)**.

---

### BACKLOG-09: Windows TSF Language Profile Locale Ordering (Bangladesh Priority)

- **Discovery Context:**  
  In the Windows Language Switcher (`Win + Space`), `Borno Bangla (India)` is listed before `Borno Bangla (Bangladesh)` due to numerical LCID sorting (`0x0445` < `0x0845`).
- **Resolution Plan:**
  1. In `native/borno-tsf/borno_tsf.cpp` `DllRegisterServer()`, register Bangladesh (`0x0845`) as the primary text service.
  2. Provide option or default to register exclusively Bangladesh (`0x0845`) unless Indian Bengali is explicitly requested, eliminating duplicate language switcher entries.
- **Target Milestone:** **Milestone M9 Refinement Pass**.

---

### BACKLOG-10: Independent Theming & Appearance for Floating Bar vs. Sandbox App

- **Discovery Context:**  
  Users requested the flexibility to either synchronize the floating candidate bar's appearance with the main sandbox application theme, or configure the floating bar independently (e.g. keeping the sandbox in Light Glass while typing with an OLED High Contrast or Dark Glass floating bar in external editors).
- **Technical Analysis:**
  1. **Schema Extension (`crates/bornomala-core/src/config.rs`):**
     - `sync_floating_bar_theme: bool` (default: `true`).
     - `floating_bar_theme: ThemeId` (default: `dark-glass`, active when sync is false).
     - `floating_bar_accent: AccentColor` (default: `Yellow`, active when sync is false).
     - `floating_bar_font_size: u32` (default: `14`, range: 10–24 pt).
  2. **Settings UI (`src/components/SettingsDrawer.tsx`):**
     - Add a "Floating Candidate Bar" settings group with a master toggle switch: `"Sync Floating Bar with App Theme"`.
     - When toggled ON, the floating bar automatically adopts the active sandbox theme and accent, displaying an explanatory subtitle: *"Floating candidate window automatically inherits the app theme and accent."*
     - When toggled OFF, an independent appearance control panel dynamically unfolds with separate Theme selection chips, Accent color picker, and font size adjustment.
  3. **Native TSF Window (`native/borno-tsf/candidate_window.cpp`):**
     - Read `sync_floating_bar_theme`. If true, paint using the primary `theme_id` and `accent_color`. If false, paint using `floating_bar_theme` and `floating_bar_accent`.
     - Render background, borders, and selection highlight pills matching the specified theme palette in real time.
- **Target Milestone:** **Milestone M9 Refinement Pass**.

---

### BACKLOG-11: Deterministic 2-Way Hotkey Toggle (Phonetic ↔ English) Sandbox Parity

- **Discovery Context:**  
  When typing bilingual documents, users frequently toggle between Bengali and English. If pressing a hotkey cycles through all installed Bengali layouts (e.g., Phonetic $\to$ Probhat $\to$ National $\to$ English), users experience friction and accidental layout switches, requiring multiple presses to return to English or their desired Bengali layout.
- **Technical Analysis:**
  1. **Native TSF Precedent:** In [`native/borno-tsf/borno_tsf.cpp:L277`](../native/borno-tsf/borno_tsf.cpp#L277), pressing <kbd>F12</kbd> or <kbd>F4</kbd> operates as a strict deterministic 2-way toggle: `active_bengali_layout <-> English Passthrough`. It never inadvertently cycles through fixed keyboard layouts.
  2. **Sandbox Parity:** In [`src/components/SandboxTypingArea.tsx`](../src/components/SandboxTypingArea.tsx), hotkey handling (<kbd>F12</kbd> / <kbd>F4</kbd>) must match this exact behavior:
     - Toggling switches strictly between the user's selected Bengali layout (e.g., `Phonetic`) and `English`.
     - When English is active, pressing the hotkey returns directly to the user's previously selected Bengali layout.
     - Persistent setting `preferred_bangla_layout: LayoutId` ensures this 2-way toggle remains consistent across sessions.
- **Target Milestone:** **Milestone M9 Refinement Pass**.

---

### BACKLOG-12: Ergonomic 1-Press Down Arrow (↓) Candidate Navigation

- **Discovery Context:**  
  In both the floating candidate window and sandbox typing area, Candidate Slot 0 (the primary prediction or direct phonetic transliteration) is already rendered inline in the preedit text and automatically commits upon pressing <kbd>Space</kbd> or punctuation. When a user presses <kbd>↓</kbd> (Down Arrow), their intent is almost universally to select an *alternative* word (Candidate Slot 1, the 2nd item). Requiring two Down Arrow presses to reach the 2nd suggestion adds unnecessary keystroke friction.
- **Technical Analysis:**
  1. In [`crates/bornomala-core/src/state.rs`](../crates/bornomala-core/src/state.rs), `selected_candidate_index: Option<usize>` starts at `None`.
  2. Currently, calling `navigate_candidate(Direction::Next)` moves `None` $\to$ `Some(0)` (Slot 0). Because Slot 0 is already the default commit target, highlighting Slot 0 on the first down arrow press is redundant.
  3. **Ergonomic Jump:** When `selected_candidate_index` is `None` and the user presses <kbd>↓</kbd>:
     - If candidate list length $\ge 2$, jump immediately to `Some(1)` (Slot 1, the first alternative candidate).
     - If candidate list length == 1, set `Some(0)`.
     - Pressing <kbd>↑</kbd> (Up Arrow) from Slot 1 moves to Slot 0 (`Some(0)`), and another Up Arrow clears the selection back to `None`.
  4. This eliminates 1 keystroke per candidate selection across all typing workflows, improving word selection speed by up to 50%.
- **Target Milestone:** **Milestone M9 Refinement Pass**.

---

### BACKLOG-13: Modular Sandbox Dashboard Section Reordering

- **Discovery Context:**  
  Users expressed a desire to customize the arrangement of panels in the Sandbox application to match their personal workflow (e.g., placing the interactive Virtual Keyboard directly beneath or adjacent to the typing area, or collapsing metrics panels for a cleaner view).
- **Technical Analysis:**
  1. The Sandbox application currently has a hardcoded layout in `src/App.tsx`:
     - Top: Header & Global Controls
     - Section 1: Sandbox Typing Area & Candidate Window
     - Section 2: Layout Selector & Engine Diagnostics
     - Section 3: Interactive Virtual Keyboard
     - Section 4: Engine Status, Telemetry & Performance Metrics
  2. **Modular Layout Architecture:**
     - Define component section identifiers: `typing-area`, `layout-selector`, `virtual-keyboard`, `engine-metrics`.
     - Store `dashboard_section_order: Vec<String>` in `BornoConfig`.
     - Provide reordering controls (Up/Down section shift buttons in section headers or a Layout Customization modal in Settings).
     - Dynamically map and render sections according to the persisted order.
- **Target Milestone:** **Milestone M9 Refinement Pass**.

---

### BACKLOG-14: System-Wide Floating Detached Virtual Keyboard

- **Discovery Context:**  
  Users inquired whether the built-in interactive virtual keyboard from the sandbox can be used anywhere in Windows (e.g., floating over Notepad, Microsoft Word, or web browsers) like the native Windows on-screen keyboard.
- **Technical Analysis & Roadmap Mapping:**
  1. **Milestone M9 Boundary:** In Milestone M9, the Virtual Keyboard is implemented as an interactive, accessible React component within the Tauri sandbox environment (`src/components/VirtualKeyboard.tsx`).
  2. **System-Wide Architecture:** Delivering a standalone floating virtual keyboard across arbitrary third-party Windows applications requires:
     - A detached, borderless, always-on-top desktop utility window with Win32 styles `WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST`.
     - The `WS_EX_NOACTIVATE` style is critical so that clicking virtual keys does not steal input focus from the user's active document (Notepad, browser, text editor).
     - A background system tray daemon and multi-window manager to spawn and position the floating keyboard.
  3. **Milestone Roadmap Target:** This functionality is officially mapped to **Milestone M12 (Desktop Packaging, Production Installer & Update Channel)**, which introduces the standalone desktop shell, background tray process (`bornomala-tray.exe`), and detached utility windows.
- **Target Milestone:** **Milestone M12 (Desktop Packaging & Utilities)**.

---

## 3. Verification Protocol

When closing any target milestone:
1. Open this file.
2. Execute the automated test command:
   ```bash
   cargo test -p bornomala-core
   ```
3. Run the interactive manual test in both Windows TSF and Linux Fcitx 5.
4. Update the item's status in Section 1 to `Verified` with date and commit hash.
