# BornoMala Design System & Usability Specification

> **Status:** Living Master Specification  
> **Initial Baseline Version:** `UX 0.1.0`  
> **Master Source of Truth:** [`ROADMAP.md`](../ROADMAP.md)  
> **Scope:** Design tokens, component conventions, accessibility rules, platform rendering bridges, and usability change management for the BornoMala Bangla Input Platform.

---

## 1. Governance & Three Decoupled Version Domains

To ensure that BornoMala's appearance, usability, and accessibility evolve systematically without introducing premature redesigns or process overhead, the project enforces **three strictly decoupled version domains**:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                            THREE VERSION DOMAINS                             │
├──────────────────────────┬────────────────────────────┬──────────────────────┤
│ 1. Product Milestones    │ 2. Design System / UX      │ 3. Config Schema     │
│    (M0 ... M9 ... M13)   │    (UX 0.1.0, UX 0.2.0)    │    (v1.0.0, v1.2.0)  │
├──────────────────────────┼────────────────────────────┼──────────────────────┤
│ • Scope: Delivered       │ • Scope: Visual tokens,    │ • Scope: Serialized  │
│   functional features,   │   component patterns,      │   JSON data contract │
│   OS integrations,       │   typography, ergonomics,  │   in config.json     │
│   and language engine.   │   a11y, and user patterns. │                      │
│ • Governed by:           │ • Governed by:             │ • Governed by:       │
│   ROADMAP.md             │   This Living Spec         │   Rust Core (Serde)  │
│ • Progress Gate:         │ • Progress Gate:           │ • Progress Gate:     │
│   Milestone acceptance   │   Design Impact Assessment │   Persistence Suite  │
│   test suite & report.   │   and UX Changelog entry.  │   & Migration Tests. │
└──────────────────────────┴────────────────────────────┴──────────────────────┘
```

### 1.1 Governance Invariants
1. **No 1-to-1 Coupling:** A product milestone (such as `M10` for Contextual Prediction) is **not required** to increment the UX version. When a milestone implements backend or engine capabilities without altering UI foundations, the existing UX version is reused.
2. **Standardized Three-Part UX Versioning (`UX X.Y.Z`):**
   - **Baseline:** `UX 0.1.0` (Documented baseline of active development at Milestone M9).
   - **Patch Revision (`UX 0.1.x`):** Token adjustments for verified contrast/legibility, alignment bug fixes, or minor styling corrections that do not add new components, introduce new configuration options, or change user interaction patterns.
   - **Minor Revision (`UX 0.x.0`):** Introduction of new component patterns (e.g. prediction suggestion chips, section reordering) or new user-facing appearance preferences (e.g. accent color selector).
   - **Major Revision (`UX 1.0.0`):** Foundational redesign of visual foundations or breaking changes to interaction models across multiple platforms.
3. **Decoupling from Configuration Schema:**
   - Developer-controlled design tokens (spacing scales, typography stacks, border-radius, blur radius, shadow definitions) live exclusively in stylesheet and adapter layout code.
   - Only user-configurable preferences (e.g. `active_theme`, `candidate_font_size`, `candidate_orientation`) reside in the Rust `BornoConfig` schema. Presentation-only tokens are never added to `BornoConfig`.
   - Introducing or modifying visual design tokens does **not** trigger a configuration schema migration unless a user-facing persistent setting is added.

---

## 2. Canonical Configuration Lifecycle & Cross-Process Contract

### 2.1 Single Writer vs. Read-Only Consumer
To prevent race conditions, file corruption, and write conflicts across processes:
- **Sole Authorized Writer:** The Tauri desktop application (and in M12, the standalone desktop manager) is the **exclusive read/write owner** of `config.json`. It owns safe atomic replacement via temporary file rename (`save_config_safe`), corrupted-file backups (`.corrupt.<timestamp>`), invalid schema backups (`.invalid.<timestamp>`), and structural schema validation.
- **Read-Only Consumer:** The native Windows TSF DLL (`bornomala_tsf.dll`) is an in-process COM server loaded by arbitrary host processes (Notepad, Microsoft Word, web browsers). It is strictly a **read-only consumer**. It must **never** create, write, delete, or rename configuration files.

### 2.2 Canonical Path Resolution & Coexistence Lifecycle
The configuration directory evolves across project milestones according to a single canonical contract:

1. **Active Development Path (Milestones M0–M11):**
   - Path: `%APPDATA%\com.bornomala.sandbox\config.json`
   - Owned by: `src-tauri/tauri.conf.json` (`"identifier": "com.bornomala.sandbox"`).
2. **Production Suite Path (Milestone M12 onward):**
   - Path: `%APPDATA%\com.bornomala.desktop\config.json`
   - Governed by: The unified production desktop installer (WiX / NSIS).
3. **M12 Migration Specification:**
   - **Migration Trigger:** Executed once by the M12 installer or during first launch of the production desktop application.
   - **Fresh Install:** If `com.bornomala.desktop\config.json` does not exist and no sandbox configuration exists, write default configuration to `com.bornomala.desktop\config.json`.
   - **Clean Migration:** If `com.bornomala.desktop\config.json` does not exist but `com.bornomala.sandbox\config.json` exists, validate the sandbox file. If valid, copy it to `com.bornomala.desktop\config.json`.
   - **Coexistence / Conflict Resolution:** If configuration files exist at *both* locations (e.g. developer running both sandbox and desktop builds), the production desktop application treats `%APPDATA%\com.bornomala.desktop\config.json` as authoritative. The sandbox file is left untouched to prevent breaking active developer sandbox testing.
   - **Corrupt Source Handling:** If the sandbox configuration is malformed or invalid during migration, the migration halts, leaves the corrupted file in place for developer inspection, and writes clean defaults to the desktop directory.
4. **Startup Loading vs. Dynamic Reload:**
   - **Startup Loading:** Read once during input service initialization.
   - **Dynamic Reload:** Windows TSF polls file attributes (`GetFileAttributesExW` with tick throttle $\ge 50\text{ms}$) to detect changes in write-time, file size, or hash.
   - *Verification Status:* Startup loading is verified in code inspection. Dynamic live reload in external host applications remains an **unverified hypothesis** until an end-to-end runtime test is executed.

---

## 3. Candidate Window Orientation: Product Decision & Platform Status

### 3.1 Product Preference
Based on interactive testing and ergonomic considerations, **`Horizontal`** orientation is the intended product preference for desktop typing. A horizontal candidate bar directly beneath the active composition line mirrors standard Avro/IME conventions, reduces vertical occlusion of surrounding document text, and enables natural left-to-right reading of suggestions.

### 3.2 Platform Implementation Status Matrix

| Component / Platform | Current Default / Implementation | Implementation Status |
| :--- | :--- | :--- |
| **Rust Core Schema** | `CandidateOrientation::Vertical` (`default_candidate_orientation()`) | `Verified in code` ([`config.rs:L99`](../crates/bornomala-core/src/config.rs#L99)) |
| **Sandbox UI** | Dynamic flex layout (`column` vs `row`), defaults to `"horizontal"` | `Verified in code` ([`SandboxTypingArea.tsx:L288`](../src/components/SandboxTypingArea.tsx#L288)) |
| **Windows TSF GDI** | Hardcoded vertical row height calculation; horizontal layout not implemented | `Verified in code` ([`candidate_window.cpp:L119`](../native/borno-tsf/candidate_window.cpp#L119)) |
| **Linux Fcitx 5** | Hardcoded `fcitx::CandidateLayoutHint::Vertical` | `Verified in code` ([`bornomala.cpp:L434`](../native/bornomala-fcitx5/bornomala.cpp#L434)) |

### 3.3 Prerequisites for Changing Rust Core Default
The Rust core schema default must **not** be changed merely to force superficial consistency. The following technical gates must be satisfied first:
1. Windows TSF `candidate_window.cpp` must implement true horizontal bounding-box and text-width calculations using `GetTextExtentPoint32W`.
2. Linux Fcitx 5 `bornomala.cpp` must dynamically map `CandidateOrientation::Horizontal` to `fcitx::CandidateLayoutHint::Horizontal`.
3. Persistence test suite (`persistence_suite.rs`) must verify that existing saved configurations specifying `"vertical"` are preserved without silent override.

---

## 4. Multi-Platform Rendering Architecture & Shared Design Intent

Because BornoMala spans web and native C++ environments, shared design intent is maintained through explicit platform-specific bridges:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          SHARED DESIGN INTENT                               │
│           (Palette, Typography, Spacing, Contrast, Ergonomics)             │
└──────────────┬───────────────────────────────┬──────────────────────────────┘
               │                               │
               ▼                               ▼
┌──────────────────────────────┐ ┌────────────────────────────────────────────┐
│ Web / React (Sandbox UI)     │ │ Windows Native TSF (Floating Candidate Bar)│
├──────────────────────────────┤ ├────────────────────────────────────────────┤
│ • Implemented in:            │ │ • Implemented in:                          │
│   src/index.css              │ │   native/borno-tsf/candidate_window.cpp    │
│ • Technology:                │ │ • Technology:                              │
│   CSS Custom Properties      │ │   Win32 GDI C++ (Double-buffered memDC)    │
│ • Primitives:                │ │ • Primitives:                              │
│   flex, var(--bg-card),      │ │   COLORREF, HBRUSH, HFONT (CreateFontW),   │
│   backdrop-filter            │ │   FillRect, DrawTextW                      │
│ • Rule:                      │ │ • Rule:                                    │
│   Stylesheets govern web;    │ │   CSS custom properties have ZERO effect   │
│   no hardcoded inline hex    │ │   on GDI; GDI maps user config to native   │
│   when token exists.         │ │   Win32 color constants explicitly.        │
└──────────────────────────────┘ └────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ Linux Native Fcitx 5 Addon                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│ • Implemented in: native/bornomala-fcitx5/bornomala.cpp                     │
│ • Technology: fcitx5::InputMethodEngineV2 & fcitx::CommonCandidateList      │
│ • Rendering Architecture: Host-Delegated.                                   │
│ • Rule: Borno provides semantic candidate strings, cursor index, and        │
│   comments (e.g. "✓ Word"). Visual rendering (fonts, colors, borders, blur) │
│   is managed entirely by the host desktop environment's Fcitx 5 theme       │
│   (KDE Breeze, GNOME Shell, or Classic UI). Borno does not control pixels.  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Living Design Tokens Specification (Web / Sandbox)

The following design tokens are authoritative for the web/sandbox interface in [`src/index.css`](../src/index.css).

### 5.1 Color Palette & Themes

```css
/* Dark Glass (Default Web Theme) */
[data-theme="dark-glass"] {
  --bg-primary: #0f172a;           /* Base viewport background */
  --bg-secondary: #111827;         /* Secondary panel background */
  --bg-tertiary: #1f2937;          /* Input container background */
  --bg-card: rgba(17, 24, 39, 0.75); /* Translucent glass card */
  --bg-card-hover: rgba(31, 41, 55, 0.85);

  --border-subtle: rgba(255, 255, 255, 0.1);
  --border-focus: #38bdf8;
  --accent-focus: #38bdf8;

  --text-primary: #f9fafb;         /* High-contrast body text */
  --text-secondary: #9ca3af;       /* Secondary labels */
  --text-muted: #6b7280;           /* Hints and shortcuts */
}

/* Light Glass Theme */
[data-theme="light-glass"] {
  --bg-primary: #f8fafc;
  --bg-secondary: #f1f5f9;
  --bg-tertiary: #e2e8f0;
  --bg-card: rgba(255, 255, 255, 0.85);
  --bg-card-hover: rgba(255, 255, 255, 0.95);

  --border-subtle: rgba(15, 23, 42, 0.12);
  --border-focus: #0284c7;
  --accent-focus: #0284c7;

  --text-primary: #0f172a;
  --text-secondary: #475569;
  --text-muted: #64748b;
}

/* High Contrast (OLED Black) Theme */
[data-theme="high-contrast"] {
  --bg-primary: #000000;
  --bg-secondary: #000000;
  --bg-tertiary: #121212;
  --bg-card: #000000;
  --bg-card-hover: #1a1a1a;

  --border-subtle: #ffffff;
  --border-focus: #facc15;
  --accent-focus: #facc15;

  --text-primary: #ffffff;
  --text-secondary: #ffffff;
  --text-muted: #e5e5e5;
}
```

### 5.2 Token Usage Policy
- **Prohibition on Unexplained Values:** Components must not use hardcoded hex fills (e.g. `#0b0f19` or `#ffffff`) when an equivalent semantic token (`var(--bg-card)`, `var(--text-primary)`) exists.
- **Dynamic Inline Calculations:** Inline styles are permitted only for dynamic, user-driven metrics (e.g. `fontSize: `${candidateFontSize}pt``, flex orientation from config, or runtime caret coordinates).

### 5.3 Typography Hierarchy
- **UI Latin Font Stack:** `'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;`
- **Bangla Script Font Stack:** `'Noto Sans Bengali', 'Kalpurush', 'SolaimanLipi', 'Nirmala UI', 'Vrinda', sans-serif;`
- **Monospace Font Stack:** `'Fira Code', monospace;`

---

## 6. Rigorous Accessibility & Usability Standards

### 6.1 Normative WCAG Standards vs. BornoMala Design Decisions

| Requirement Area | Specification / Standard | Authority / Level | BornoMala Technical Implementation |
| :--- | :--- | :--- | :--- |
| **Normal Text Contrast** | $\ge 4.5:1$ contrast ratio | WCAG 2.1 Criterion 1.4.3 (Level AA) | Measured against rendered card backgrounds, accounting for alpha blending. |
| **Large Text Contrast** | $\ge 3.0:1$ ($\ge 18\text{pt}$ or $\ge 14\text{pt}$ bold) | WCAG 2.1 Criterion 1.4.3 (Level AA) | Section headings and candidate numbers. |
| **Non-Text Component Contrast** | $\ge 3.0:1$ against adjacent colors | WCAG 2.1 Criterion 1.4.11 (Level AA) | Candidate pill borders, toggle switches, and input boundaries. |
| **Focus Visibility** | Visible indicator on keyboard focus | WCAG 2.1 Criterion 2.4.7 (Level AA) | `:focus-visible` present on all interactive controls. |
| **Focus Contrast & Area** | $\ge 3.0:1$ contrast, $\ge 2\text{px}$ perimeter | WCAG 2.2 Criterion 2.4.13 (Level AAA) | Distinct outline with 2px offset; yellow (`#facc15`) in High Contrast. |
| **Dialog Focus Trapping** | Focus contained within open modal | WCAG 2.1 Criterion 2.1.2 (Level A) | `SettingsDrawer` traps Tab navigation; <kbd>Escape</kbd> dismisses modal. |
| **Focus Restoration** | Focus returns to opening trigger on close | WCAG 2.1 Criterion 2.4.3 (Level A) | `triggerElementRef.current?.focus()` executed on drawer dismiss. |
| **Reduced Motion** | Honor user motion preferences | WCAG 2.1 Criterion 2.3.3 (Level AAA) & BornoMala Policy | `@media (prefers-reduced-motion: reduce)` disables non-essential transitions. |
| **Candidate Announcements** | Dynamic candidate updates spoken | WCAG 2.1 Criterion 4.1.2 & 4.1.3 (Level AA) | Live region announcements for candidate suggestions and status changes. |

### 6.2 Contrast Measurement Protocol
Contrast claims must be validated using rendered foreground and background color combinations. Where translucent backgrounds (`rgba`) are used over a dark background (`#0f172a`), the composite color must be computed before calculating the contrast ratio:
$$\text{Composite} = \alpha \cdot \text{Foreground} + (1 - \alpha) \cdot \text{Background}$$

---

## 7. Reusable Milestone Design-Impact Assessment Template

This template must be incorporated into future milestone instruction files (`M10.md`, `M11.md`, etc.) whenever user-facing presentation or interaction is touched:

```markdown
---

## Milestone Design Impact Assessment

1. **User Experience Impact:**
   - [ ] No visual or interaction changes (Reuses existing UX version without bump).
   - [ ] Modifies visual tokens, component patterns, or interaction models (Requires UX version bump).

2. **Impact Breakdown:**
   - **Visual Tokens:** [List new tokens or state colors, or state None]
   - **Component Patterns:** [Describe changes to Candidate Window, Drawers, Virtual Keyboard]
   - **Keyboard & Ergonomics:** [Specify keystroke changes, navigation shortcuts, focus order]
   - **Accessibility Verification:** [State WCAG criteria evaluated and verification method]
   - **Platform Boundaries:** [Explain impact on Windows TSF, Linux Fcitx 5, and Sandbox]

3. **Design & Usability Acceptance Gate:**
   - [ ] Text contrast meets WCAG 2.1 AA ($\ge 4.5:1$ normal, $\ge 3.0:1$ large) against rendered composite backgrounds.
   - [ ] Non-text UI components meet WCAG 2.1 AA ($\ge 3.0:1$) for boundaries and focus states.
   - [ ] Full keyboard navigation operational; visible focus rings present on all interactive elements.
   - [ ] Dialogs trap focus on entry and restore focus to trigger element upon dismissal.
   - [ ] Live regions announce candidate suggestions without interrupting active typing.
   - [ ] No unexplained inline hardcoded presentation values introduced.
```

---

## 8. UX Version Changelog & Initial Baseline Record

### [UX 0.1.0] — 2026-10-20
**Documented Active Baseline (Milestone M9 Current State)**

#### Motivation
Establish the initial documented baseline of visual foundations, component patterns, and platform rendering behaviors as verified during Milestone M9 interactive testing.

#### Scope & Platform Matrix
- [x] Web / Sandbox UI (`src/App.tsx`, `src/components/`, `src/index.css`)
- [x] Windows Native TSF (`native/borno-tsf/candidate_window.cpp`)
- [x] Linux Fcitx 5 (`native/bornomala-fcitx5/bornomala.cpp`)

#### Factual Code-Inspection Baseline
1. **Web / Sandbox UI:**
   - Three themes defined in `index.css`: `dark-glass`, `light-glass`, `high-contrast`.
   - `SettingsDrawer.tsx`: Implements focus trapping, <kbd>Escape</kbd> listener, `aria-modal="true"`, `aria-live` polite/assertive regions, and slider `aria-valuetext`.
   - `SandboxTypingArea.tsx`: Candidate panel implements dynamic flex layout based on orientation prop.
2. **Windows Native TSF:**
   - Double-buffered Win32 GDI candidate window with `WS_EX_NOACTIVATE`.
   - Font scaling via `MulDiv` for DPI.
   - Supports Vertical layout rendering with GDI brushes for dark, light, and high-contrast modes.
3. **Linux Native Fcitx 5:**
   - Delegates candidate presentation to `fcitx::CommonCandidateList`.
   - Passes candidate text, cursor selection index, page size (9), and comment badges (`✓ Word`, `★ Spell`).

#### Explicit Verification Gaps & Known Limitations
> [!WARNING]
> The following items are explicitly recorded as pending runtime verification or known defects:
1. **Light Theme Candidate Panel Contrast:** In `SandboxTypingArea.tsx:L670, L708`, the candidate panel uses hardcoded dark values (`rgba(11, 15, 25, 0.6)` and `#ffffff` text). Contrast ratio in Light mode requires formal rendered measurement and token alignment.
2. **TSF Horizontal Candidate Layout:** `candidate_window.cpp` does not yet calculate text widths or render horizontal candidate pills in GDI.
3. **TSF Dynamic Reload in Host:** Live reload via file attribute polling (`GetFileAttributesExW`) is verified in code structure, but unverified in external application runtimes (e.g. Notepad).
4. **Screen Reader Runtime Verification:** Live region candidate announcements in `SandboxTypingArea.tsx` have not yet been evaluated with NVDA or Windows Narrator.
5. **High-DPI GDI Scaling:** Verification of GDI candidate window layout at $125\%$, $150\%$, and $200\%$ display scaling remains pending.

---

### Future Version Placeholders (Not Implemented)
- `[UX 0.1.1]` *(Planned Patch)*: Light theme token alignment in `SandboxTypingArea.tsx` and contrast verification.
- `[UX 0.2.0]` *(Planned Minor)*: Multi-accent color selector, 1-press down-arrow candidate navigation, and horizontal GDI candidate rendering.
