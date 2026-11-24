# BornoMala — Public Visual Asset Manifest

**Baseline:** `docs/visual-showcase` (`cc64ba3c573c65786e21be58af3f49aafff4ba35`)  
**Total Public Assets:** 16 (1 branding hero, 2 animated GIFs, 11 screenshots, 2 architecture diagrams)

---

## 1. Branding (`docs/assets/branding/`)

| Public Asset Path | Source Reference | Processing Performed | Dimensions | Size (bytes) | SHA-256 | Privacy Check | Intended Usage |
|---|---|---|---|---:|---|---|---|
| `docs/assets/branding/bornomala-showcase.png` | `manual-source/selected/phonetic-sandbox-showcase.png` | Composed branded hero frame (`#0b0f19` canvas, `#38bdf8` / `#10b981` palette, restrained header & badges, "Shared Rust Engine" footer) around native-resolution `1119×836` screenshot | `1220×980` | `122,327` | `AA815BD500C0A44BDCE4EB9C5A79B79C9874612B1B9E4ED860927BE321AE08D3` | Clean (no PII) | Primary README hero showcase banner |

---

## 2. Animated GIFs (`docs/assets/gifs/`)

| Public Asset Path | Source Reference | Processing Performed | Dimensions | Size (bytes) | Frames / Duration | SHA-256 | Privacy Check | Intended Usage |
|---|---|---|---|---:|---|---|---|
| `docs/assets/gifs/phonetic-typing-demo.gif` | `manual-source/selected/phonetic-typing-demo.gif` | Shortened idle pauses (`>220 ms` capped to `145 ms`, final frame `900 ms`) + FFmpeg delta-palette re-encode (`stats_mode=diff`, `diff_mode=rectangle`); all `106` frames preserved in order | `1154×807` | `481,455` | `106` frames / `13.28 s` | `A94B390891C324CAC4EB1D0A07C303B214E5D3280AAF0653719758FD746E783D` | Clean (no PII) | Hero phonetic typing & candidate selection demo |
| `docs/assets/gifs/dictionary-correction-demo.gif` | `manual-source/selected/dictionary-correction-demo.gif` | Retained unchanged at original quality and duration | `1121×817` | `292,149` | `46` frames / `13.00 s` | `0AA514408EFF749B8576E8F6F55204CF4CF28DDB7B6569455B75CABA5EE78043` | Clean (no PII) | Offline dictionary completion & typo correction demo |

---

## 3. Screenshots (`docs/assets/screenshots/`)

| Public Asset Path | Source Reference | Processing Performed | Dimensions | Size (bytes) | SHA-256 | Privacy Check | Intended Usage |
|---|---|---|---|---:|---|---|---|
| `docs/assets/screenshots/phonetic-sandbox-showcase.png` | `manual-source/selected/phonetic-sandbox-showcase.png` | Retained at native resolution (unchanged copy) | `1119×836` | `136,390` | `90AB85BC46138B1F5B0F24046CDC34A3F01228185C019C8FC775C4998B4D3863` | Clean (no PII) | Phonetic typing & candidate list showcase |
| `docs/assets/screenshots/high-contrast-theme.png` | `manual-source/selected/high-contrast-theme.png` | Retained at native resolution (unchanged copy) | `1121×826` | `131,619` | `69063DE4285D0E083B46FC22E800387D63BC744E4EC92249AAF3730F00BBBDC4` | Clean (no PII) | Accessibility / High-Contrast theme showcase |
| `docs/assets/screenshots/avro-phonetic-reference.png` | `manual-source/selected/avro-phonetic-reference.png` | Retained at native resolution (unchanged copy) | `1008×642` | `117,932` | `D87CBCA24333B70D280CFE35C0DF61D3C30B7DEC5EAA296DCEAAC4B00A1E95BC` | Clean (no PII) | In-app Avro phonetic keystroke guide |
| `docs/assets/screenshots/unicode-conjunct-verification.png` | `manual-source/selected/unicode-conjunct-verification.png` | Retained at native resolution (unchanged copy) | `1011×1008` | `162,372` | `2305F55A7EE86BE0370C2F2A0A6E7D127C4C113B85B90F922C150D735714EEB8` | Clean (no PII) | Unicode NFC & conjunct shaping verification |
| `docs/assets/screenshots/virtual-keyboard-and-event-inspector.png` | `manual-source/selected/virtual-keyboard-and-event-inspector.png` | Retained at native resolution (unchanged copy) | `1039×917` | `170,666` | `FCEB2E474D34819FCCEF9413466E7DB4054D3539779B388870E741F5C79F027C` | Clean (no PII) | Combined virtual keyboard, M0 gate & inspector view |
| `docs/assets/screenshots/virtual-keyboard.png` | `manual-source/selected/virtual-keyboard-and-event-inspector.png` | Focused vertical crop (`y=0..412`) isolating the Virtual Keyboard card | `1039×412` | `59,929` | `7941F1E8972463CE5633036B114DE148114491ED936BAF56344178B6FD47109E` | Clean (no PII) | Interactive virtual keyboard documentation |
| `docs/assets/screenshots/event-inspector.png` | `manual-source/selected/virtual-keyboard-and-event-inspector.png` | Focused vertical crop (`y=532..875`) isolating the Event & State Inspector card | `1039×343` | `56,324` | `BCD773F4353D469C59A7702D37ECB4D376E9D03C9969F7F0F146738261742F4D` | Clean (no PII) | Engine event & latency inspector documentation |
| `docs/assets/screenshots/settings-typing-punctuation.png` | `manual-source/selected/settings-typing-punctuation.png` | Retained at native resolution (unchanged copy) | `500×1023` | `121,094` | `6079F2AC6DE486CC0ED1091816E3E3DAFC056C42B74BBBEC242DA917770AD408` | Clean (no PII) | Settings: Typing & Punctuation options |
| `docs/assets/screenshots/settings-appearance.png` | `manual-source/selected/settings-appearance.png` | Retained at native resolution (unchanged copy) | `501×455` | `48,336` | `686C006CA6E703122EEE3B63AB7F0609464D0FDC8B8AF1901D6C4AFB8BBFC285` | Clean (no PII) | Settings: Appearance & theme selector |
| `docs/assets/screenshots/settings-shortcuts-layouts.png` | `manual-source/selected/settings-shortcuts-layouts.png` | Retained at native resolution (unchanged copy) | `500×529` | `55,329` | `39ECC8AE9A827A3505297582AAE8E82F27DFADBBC6E33F41CC6B80521CE04A14` | Clean (no PII) | Settings: Shortcuts & fixed layouts |
| `docs/assets/screenshots/settings-persistence-diagnostics.png` | `manual-source/selected/settings-persistence-diagnostics.png` | Permanently redacted via opaque solid RGB fill (`(9,13,21)`) over path box (`x=42..458, y=308..346`) and flattened with generic `%APPDATA%\com.bornomala.desktop\config.json` label | `500×473` | `28,228` | `4237D622A018031C30EE25041C4ABF8562B761AEF2EEA564E9730BCD3928AAAE` | **Redacted & Verified** (no username/local path) | Settings: Persistence, backup & schema diagnostics |

---

## 4. Architecture Diagrams (`docs/assets/diagrams/`)

| Public Asset Path | Source Reference | Processing Performed | Dimensions | Size (bytes) | SHA-256 | Privacy Check | Intended Usage |
|---|---|---|---|---:|---|---|---|
| `docs/assets/diagrams/shared-engine-architecture.svg` | `docs/architecture.md`, `docs/platform-support.md`, `crates/bornomala-core/src/lib.rs`, `src-tauri/src/lib.rs` | Self-contained SVG (`viewBox 0 0 960 540`) with dark-slate card, system font stack, C-ABI & `Tauri IPC + Direct Rust API` boundaries, and honest platform verification badges | `960×540` | `12,612` | `0CA7A408754B42D6D8CB563C0ED5140B11D4B1A408E18496F3FA1E4ECD924B92` | Clean (no PII) | README & portfolio system architecture overview |
| `docs/assets/diagrams/configuration-architecture.svg` | `crates/bornomala-core/src/config.rs`, `crates/bornomala-core/src/lib.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/config_store.rs`, `src/components/SettingsDrawer.tsx` | Self-contained SVG (`viewBox 0 0 960 520`) distinguishing `BornoEngine::with_config()` startup init, `BornoEngine::configure()` live updates (preserving active mode & composition), UI/adapter settings, and atomic persistence | `960×520` | `11,430` | `749D9C9F836D4FBF99D74D36D48DB0D0EC7179DD16815A5772DA02FBCA400625` | Clean (no PII) | README & docs configuration/persistence lifecycle |
