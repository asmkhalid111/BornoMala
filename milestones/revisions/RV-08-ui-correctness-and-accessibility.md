# RV-08 — UI Correctness, Accessibility & Security

> **Revision ID:** RV-08
> **Status:** Proposed — awaiting developer approval
> **Priority:** High (Phase 1)
> **Estimated effort:** 6–8 days
> **Dependencies:** RV-07 (small components + state layer)
> **Sub-instruction:** [`RV-08.1-ime-and-focus.md`](RV-08.1-ime-and-focus.md)
> **Blocks:** RV-11

---

## 1. Why this revision exists

The sandbox works as a demo but fails as a product surface: an unreadable light theme, an incomplete modal, no reduced-motion support, a CSP-disabled WebView that loads remote fonts, and settings behaviour (“Discard”) that lies.

| ID | Defect | Evidence |
|---|---|---|
| U-THEME-1 | Light theme composites at ~1.07:1 for muted text; hard-coded dark panels survive into light mode | [`SandboxTypingArea.tsx:611`](../../src/components/SandboxTypingArea.tsx#L611), [`index.css`](../../src/index.css) |
| U-A11Y-1 | Focus not moved into the drawer; background not inert; nested dialogs untrapped; `role="tab"` without `aria-controls`; `outline: none`; no reduced-motion; `user-select: none` | [`SettingsDrawer.tsx:128-190`](../../src/components/SettingsDrawer.tsx#L128-L190), [`index.css:121`](../../src/index.css#L121) |
| U-CONFIG-1 | “Discard” keeps edits; save can clobber in-flight edits | [`SettingsDrawer.tsx:74-92`](../../src/components/SettingsDrawer.tsx#L74-L92), `:198-220` |
| U-SEC-1 | `csp: null`; remote Google Fonts; unused `opener` permission | [`tauri.conf.json:25`](../../src-tauri/tauri.conf.json#L25), [`index.html:7-9`](../../index.html#L7-L9) |
| Medium | Virtual keyboard renders Caps/Tab/Ctrl/Alt keys that do nothing | [`VirtualKeyboard.tsx:109`](../../src/components/VirtualKeyboard.tsx#L109), `:178-196` |
| Medium | Layout fetch failure leaves stale key labels | [`VirtualKeyboard.tsx:56-66`](../../src/components/VirtualKeyboard.tsx#L56-L66) |
| Medium | Candidate live region claimed but absent | `docs/design-system-and-usability.md:225` |
| Medium | No `lang="bn"` on Bangla regions | [`index.html:2`](../../index.html#L2) |

---

## 2. Current-state inspection

Read:

- `src/index.css`, `src/App.tsx`, all components
- `docs/design-system-and-usability.md` (tokens, accessibility claims, verification gaps)
- `docs/usability-and-linguistic-backlog.md` BACKLOG-04/05/06/10/11/12/13
- `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `index.html`

Measure before changing:

- count inline `style={{` objects per file,
- compute contrast for every foreground/background pair in each theme (record in a table),
- list every interactive element without a keyboard path.

---

## 3. Scope

### Goals

1. A single token source; no hard-coded palette in components.
2. All three themes meet WCAG 2.1 AA for their intended text sizes, with measured evidence.
3. Correct dialog semantics and keyboard operability.
4. Reduced-motion support, live region for candidates, `lang` attributes, selectable output.
5. Security: strict CSP, local fonts, no unused permissions.
6. Correct settings form behaviour (discard, save locking).
7. Honest documentation of what is and is not verified.

### Non-goals

- No new settings fields.
- No visual redesign beyond fixing tokens/contrast.
- No IME composition logic (RV-08.1).

### Explicit no-go

- **No** “accessibility verified” without a measured contrast table and a keyboard-only pass.
- **No** remote font/CDN dependency in a product that claims offline-first.
- **No** `csp: null` in a release build.
- **No** `outline: none` without an equivalent visible focus indicator.
- **No** decorative ARIA that does not match behaviour.

---

## 4. Work packages

### WP-8.1 — Token consolidation

1. Define the full token set per theme in `src/index.css` only: background layers, text, border, accent, candidate surfaces, status colours, focus ring, shadows.
2. Delete palette literals from components; replace with `var(--…)` or classes.
3. Remove duplicate palettes from `SettingsDrawer`.
4. Add a lint-style check (grep in CI) for `#[0-9a-fA-F]{3,8}` and `rgba(` inside `src/components/**` — allow a documented exceptions list (e.g. transparent overlays).

Gate: zero palette literals outside `index.css` and `src/data/`.

### WP-8.2 — Contrast

1. Build a table of every text/surface pair per theme.
2. Measure with a script (compute WCAG ratio from hex) and commit the output to `docs/design-system-and-usability.md`.
3. Fix failures: light theme candidate pill (use dark text), light panels (no dark overlay), muted text (raise to ≥4.5:1 where used below 18.66px).
4. Add a unit test that imports the token table and asserts ratios ≥ 4.5 (normal text) / 3.0 (large text and UI boundaries).

Gate: the contrast test passes for all themes; the table is in the docs.

### WP-8.3 — Dialog and tabs semantics

1. Render the settings drawer in a portal.
2. On open: move focus to the first focusable control (or the heading with `tabindex="-1"`).
3. Set `inert`/`aria-hidden` on the app root; restore focus on close.
4. Implement a full trap that handles focus starting outside the dialog.
5. Nested confirm dialogs get their own trap and return focus to the invoking control.
6. Implement the WAI-ARIA tabs pattern: `role="tablist"`, `role="tab"` with `aria-controls`, `aria-selected`, roving `tabindex`, arrow-key navigation, `role="tabpanel"` with `aria-labelledby`.
7. Remove inline `outline: none`; use `:focus-visible` with a 2px ring at 3:1 against adjacent colours.

Gate: keyboard-only walkthrough recorded (Tab, Shift+Tab, arrows, Escape, Enter/Space).

### WP-8.4 — Settings form correctness

1. Reset `formData` when `isOpen` becomes true (or unmount on close).
2. Disable the form while saving.
3. Use a save revision/dirty token; when the save resolves, do not overwrite edits made after the request.
4. “Discard & Close” must discard.

Gate: unit tests for open→edit→discard→reopen, and edit-during-save.

### WP-8.5 — Security

1. Remove the Google Fonts links; bundle WOFF2 locally (SIL OFL fonts — record the licence in `THIRD-PARTY-NOTICES`).
2. Set a strict CSP in `tauri.conf.json` (`default-src 'self'`; no remote origins; `style-src 'self' 'unsafe-inline'` only if inline styles remain, and remove that exception after WP-8.1).
3. Remove `tauri-plugin-opener` and its capability if unused.
4. Verify the app launches with no network requests (devtools network tab, recorded).

Gate: offline launch screenshot; CSP present; no opener permission.

### WP-8.6 — Motion, language, selection, live region

1. Add `@media (prefers-reduced-motion: reduce)` disabling transitions/animations.
2. Add `aria-live="polite"` to the candidate list and a concise announcement (“3 candidates, first: …”).
3. Set `lang="bn"` on Bangla output regions.
4. Allow text selection in the committed output and diagnostics (`user-select: text`).
5. Add `scope="col"` to table headers.
6. Add `aria-pressed`/`role="radio"` semantics to mode, theme, orientation, and hotkey selectors.

Gate: screen-reader smoke test (Narrator) recorded; reduced-motion verified with OS setting.

### WP-8.7 — Virtual keyboard honesty

1. Either implement Caps/Tab/Ctrl/Alt behaviour or remove the keycaps.
2. On layout fetch failure, clear the layout and show an error state.
3. Make Shift/AltGr badges real buttons with `aria-pressed`.
4. Keyboard-operable (Tab + Enter/Space).

Gate: no dead control remains; error state tested by forcing a failed invoke.

### WP-8.8 — Documentation truth

Update `docs/design-system-and-usability.md`:

- mark each claim as Implemented / Verified / Not implemented,
- remove the “reduced motion”, “live region”, “focus trap verified” claims if not implemented (they will be after this revision — link the test/evidence),
- add the contrast table,
- keep the “Verification Gaps” section honest.

---

## 5. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | No palette literals outside `index.css`/`data` | CI grep |
| AC-2 | Contrast ≥ AA for all themes | measured table + test |
| AC-3 | Focus trap and restoration work | keyboard walkthrough |
| AC-4 | Tabs follow the ARIA pattern | DOM inspection + test |
| AC-5 | Discard discards; save does not clobber | unit tests |
| AC-6 | CSP set; no remote fonts; opener removed | config + network capture |
| AC-7 | Reduced motion respected | OS setting test |
| AC-8 | Live region announces candidates | screen-reader note |
| AC-9 | No dead virtual keys | manual |
| AC-10 | Design doc matches code | review |

---

## 6. Testing plan

```powershell
npm test
npm run build
npm run tauri dev
```

Manual:

1. Keyboard-only: open settings, traverse tabs, change theme, save, close.
2. Narrator: confirm dialog name, tab names, candidate announcements.
3. Toggle Windows “Show animations” off; confirm no motion.
4. Disconnect network; launch; confirm fonts render and no request.
5. Light theme: read every panel.

---

## 7. Runnable checkpoint

- App launches, all themes legible, settings usable by keyboard.
- No functional regression in typing.

Rollback: token changes are CSS-only; dialog changes are component-local.

---

## 8. Evidence and reporting

`docs/evidence/RV-08/<date>-ui-a11y.md`: contrast table, keyboard walkthrough notes, Narrator notes, network capture, before/after screenshots per theme.

---

## 9. Roadmap update

- Update the UX baseline to `UX 0.2.0` if tokens/patterns changed (they will).
- Reconcile BACKLOG-04/05/06/10/11/12/13 statuses honestly.
- The M9 AC-09/AC-10 claims must be corrected in the M9 report or superseded by this evidence.

---

## 10. Stop conditions

- A contrast fix requires a visual redesign the developer has not approved → propose options and stop.
- Bundling fonts changes the app size materially → report the delta and get approval.
- CSP breaks Tauri functionality → narrow the policy with a documented reason, never disable it.

---

## 11. Handoff

The UI is now a credible product surface. RV-11 may package it.
