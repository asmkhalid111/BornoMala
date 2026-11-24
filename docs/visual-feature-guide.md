# BornoMala — Visual Feature Guide

This guide highlights the **BornoMala Desktop Sandbox (`bornomala-desktop`)** user interface, diagnostics, themes, and configuration architecture. All screenshots below are captured directly from the running Tauri v2 + React application backed by `bornomala-core`.

> **Scope Note:** These captures demonstrate the standalone Desktop Sandbox and settings UI. System-wide OS input integration (Windows TSF COM and Linux Fcitx 5) is documented separately in [Platform Support](platform-support.md).

---

## 1. Phonetic Sandbox & Keystroke Reference

| Phonetic Composition & Candidates | Avro Phonetic Keystroke Reference |
| :---: | :---: |
| <img src="assets/screenshots/phonetic-sandbox-showcase.png" alt="Phonetic typing sandbox showing active Bangla preedit, completion badges, and committed text" width="440" /> | <img src="assets/screenshots/avro-phonetic-reference.png" alt="Collapsible Avro phonetic keystroke guide showing vowels, consonants, and conjuncts" width="440" /> |
| *Live preedit (`বাংলা`), completion candidates, and committed output buffer.* | *Built-in phonetic keystroke reference for vowels, consonants, conjuncts, and phalas.* |

---

## 2. Unicode & Conjunct Shaping Verification

<p align="center">
  <img src="assets/screenshots/unicode-conjunct-verification.png" alt="Unicode NFC normalization and Bangla conjunct shaping verification panel" width="760" />
</p>

*Interactive verification of complex Bangla conjuncts, two-part vowel NFC composition, nukta letters, Khanda-Ta, and `ZWJ`/`ZWNJ` joiner behavior.*

---

## 3. Interactive Virtual Keyboard & Event Inspector

### Interactive Virtual Keyboard

<p align="center">
  <img src="assets/screenshots/virtual-keyboard.png" alt="Interactive virtual keyboard with live Shift and AltGr state indicators" width="820" />
</p>

*Physical key-mapping preview with live illumination and `Shift`/`AltGr` toggles.*

### Normalized Event & State Inspector

<p align="center">
  <img src="assets/screenshots/event-inspector.png" alt="Normalized Event and State Inspector showing round-trip IPC and engine latency" width="820" />
</p>

*Deterministic event log showing raw buffer, Bangla preedit, cursor position, and round-trip IPC latency.*

<details>
<summary>View combined Virtual Keyboard, M0 Verification Gate, and Event Inspector capture</summary>

<p align="center">
  <img src="assets/screenshots/virtual-keyboard-and-event-inspector.png" alt="Combined view of Virtual Keyboard, Deterministic Engine Verification M0 Gate, and Event Inspector" width="780" />
</p>

</details>

---

## 4. High-Contrast Theme

<p align="center">
  <img src="assets/screenshots/high-contrast-theme.png" alt="BornoMala Desktop Sandbox rendered in High-Contrast theme" width="760" />
</p>

*High-contrast theme (`high-contrast` in `BornoConfig`) removing translucent blur and increasing foreground and border contrast.*

---

## 5. Configuration & Persistence Architecture

<p align="center">
  <img src="assets/diagrams/configuration-architecture.svg" alt="Configuration, validation, SemVer schema management, and atomic persistence flow" width="840" />
</p>

The Settings Drawer (`src/components/SettingsDrawer.tsx`) manages schema-versioned (`v1.2.0`) configuration across five tabs; four operational tabs are illustrated below (the fifth tab covers About and Third-Party Notices):

| Typing & Punctuation | Appearance & Themes |
| :---: | :---: |
| <img src="assets/screenshots/settings-typing-punctuation.png" alt="Settings Drawer Typing and Punctuation tab" width="380" /> | <img src="assets/screenshots/settings-appearance.png" alt="Settings Drawer Appearance tab" width="380" /> |
| *Candidate display limit (`1–9`), dictionary assistance, word completion, and Dari (`. → ।`) toggle.* | *Theme selector (`dark-glass`, `light-glass`, `high-contrast`), candidate orientation, and font size.* |

| Shortcuts & Fixed Layouts | Persistence & Diagnostics |
| :---: | :---: |
| <img src="assets/screenshots/settings-shortcuts-layouts.png" alt="Settings Drawer Shortcuts and Fixed Layouts tab" width="380" /> | <img src="assets/screenshots/settings-persistence-diagnostics.png" alt="Settings Drawer Persistence and Diagnostics tab with generic config path" width="380" /> |
| *Default startup mode, preferred fixed layout (`National`, `Probhat`, `Bijoy`), and mode toggle hotkey (`F12` / `F4`).* | *JSON export/import, factory reset, schema version status, and persistence diagnostics.* |

---

Back to [README](../README.md).
