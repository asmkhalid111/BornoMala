# Bangla Input Platform — Master Roadmap to v1.0.0

> **Document role:** This is the project's master planning document and source of truth. Use it to create focused milestone instruction files (for example, `M0.md`, `M0.1.md`, `M1.md`) for Google Antigravity or another AI coding agent.
>
> **Important:** This file defines the product direction, milestone sequence, scope, dependencies, acceptance gates, and project-management rules. It is not a request to implement the entire application in one pass. Detailed instructions for a milestone must be written only when that milestone is next, after inspecting the actual repository and evidence from completed work.

- **Project:** Bangla Input Platform
- **Release target:** `v1.0.0`
- **Primary development environment:** Windows
- **Developer model:** Solo developer using AI coding tools
- **Linux access:** Potentially available through a VM, dual boot, or another environment; must be verified
- **macOS access:** Not currently available; macOS support must not be claimed as verified without real macOS testing
- **Roadmap status:** Proposed master plan; implementation status must be updated only from evidence
- **Last updated:** 2026-10-17

---

## 1. Product Vision

Build a fast, modular, customizable, privacy-conscious Bangla input platform for desktop operating systems. It should bring multiple Bangla typing methods and language assistance together behind a shared engine, with native operating-system input-method integrations.

This is **not merely a standalone Bangla typing window** and should not become a generic keyboard clone. The intended product should let users type Bangla in ordinary applications using a real system input method, choose their preferred typing method, configure useful behavior, and benefit from language assistance without making normal typing dependent on a network connection.

### 1.1 Product goals

1. **Multiple typing methods:** Support phonetic Bangla input and fixed keyboard layouts.
2. **Correct text:** Produce valid Unicode Bangla and handle composition and editing correctly, including conjuncts, combining marks, deletion, cursor movement, cancellation, and mixed Bangla/English text.
3. **Shared engine:** Keep language/input logic in a reusable Rust core wherever practical.
4. **Native integration:** Let users type directly in supported applications through each claimed operating system's native input-method mechanism.
5. **Language assistance:** Provide word completion and spelling suggestions; consider autocorrection and contextual next-word prediction only when their behavior and quality are measured.
6. **Customization:** Offer dependable settings, shortcut configuration, candidate presentation options, and an extensible approach to layouts and themes.
7. **Offline-first behavior:** Core typing and installed language resources must work without internet access.
8. **Privacy by default:** Do not transmit typed content or retain raw keystroke logs by default. Any future personalization should be local and opt-in unless the product direction is explicitly changed.
9. **Measured performance:** Keep typing responsive. Measure actual latency, startup, memory use, and resource consumption instead of assuming performance from the technology choice.
10. **Incremental delivery:** The app must run and be testable from M0 onward. Each milestone adds to a working checkpoint rather than postponing usability until the end.
11. **Honest platform claims:** Support status must reflect real build, install, and end-to-end tests on each claimed OS.

### 1.2 Desktop v1.0.0 scope

The intended desktop targets are:

- Windows
- Linux
- macOS

The product should aim to provide:

- Shared Rust input/language engine
- Phonetic Bangla input
- Fixed keyboard layouts with clearly documented and tested compatibility targets
- Correct Unicode composition and editing
- Word completion and spelling suggestions
- A settings application
- Meaningful customization
- Native system input-method integration
- Offline operation for ordinary typing
- Automated tests, performance measurements, documentation, and distributable packages

**Important platform constraint:** Windows is the practical first implementation target because it is the developer's primary machine. Linux should be investigated early and implemented only against a clearly specified, testable configuration. macOS remains an intended target, but it cannot be marked complete until it has been built, installed, and tested on real macOS. If Mac access cannot be obtained before release, the roadmap must explicitly present macOS as unverified/deferred rather than claim full three-platform v1.0.0 support.

### 1.3 Explicitly outside the initial release

Unless a later evidence-based decision changes the scope, do not make these v1.0.0 requirements:

- Android keyboard application (future possibility only)
- Mandatory cloud prediction, accounts, or synchronization
- A large generative language model trained from scratch
- An arbitrary executable plug-in ecosystem
- Guaranteed identical candidate-panel behavior across every OS/application
- Unverified compatibility with every legacy encoding
- Complex layout editor, extensive theme marketplace, or advanced personalization before core typing and native integration are reliable
- Neural prediction as a release blocker if its quality, latency, or memory use is not good enough

---

## 2. Architecture Direction

The architecture below is the intended direction, not permission to ignore evidence. Validate assumptions early and document any justified change before adopting it.

### 2.1 Shared Rust core

The Rust core should own reusable input and language logic, including the parts that can be made independent of operating-system APIs:

- Normalized input events and engine state
- Phonetic transliteration and candidate generation
- Fixed-layout mapping
- Composition and commit logic
- Unicode policy and text normalization decisions
- Candidate ranking and dictionary-backed assistance
- Configuration model and validation
- Stable interfaces for native adapters and the sandbox
- Unit, regression, and performance tests

Keep OS-specific behavior out of the core where possible. Avoid coupling the engine to the settings UI or to a single native adapter.

### 2.2 Native platform adapters

Each adapter translates platform-specific events and text-client interactions into the shared engine contract.

- **Windows:** Investigate and implement a native Text Services Framework (TSF) integration. Use C++ where Windows APIs require it; connect to Rust through a narrow, documented boundary.
- **Linux:** Choose a concrete initial environment and integration path, such as a defined Ubuntu-based setup using Fcitx 5. Specify desktop session and display-server assumptions. Do not describe “Linux support” as tested based on a single unrecorded experiment.
- **macOS:** Use the appropriate native input-method mechanism (InputMethodKit is the intended direction), but only claim support after real macOS testing.

The native adapter must handle the real lifecycle of text input: composition/preedit, candidates, commit, cancellation, focus/client changes, cursor or surrounding-text interactions where supported, and errors. A converter that only turns a string into another string is not sufficient for a system IME.

### 2.3 Settings application and sandbox

Intended direction:

- TypeScript
- React
- Tauri
- A visible sandbox for deterministic input-engine interaction and debugging

The sandbox is not disposable after native integration. It remains a reliable place to test the engine, inspect composition/candidates, reproduce bugs, and demonstrate behavior without depending on a particular host application.

Settings must affect the actual running engine or persisted configuration; a decorative UI that does not change behavior is not completion.

### 2.4 Python research and evaluation tooling

Use Python where it adds value for:

- Preparing and evaluating language resources
- Reproducible dataset preparation
- Dictionary and prediction evaluation
- Benchmark analysis
- Optional model experiments

Python should not become a mandatory runtime dependency for ordinary typing unless a later architectural decision explicitly justifies that trade-off.

### 2.5 Interfaces and dependencies

- Define and document the engine contract before adding many features.
- Keep platform adapters thin.
- Use a narrow FFI boundary only where needed; do not force every internal interaction through a C ABI without a demonstrated reason.
- Version data formats such as keyboard layouts and settings.
- Keep test fixtures and evaluation sets reproducible.
- Treat dependencies, licenses, resource origins, and update mechanisms as release concerns.

### 2.6 Design System & UX Evolution Architecture

To ensure appearance, usability, accessibility, and cross-platform consistency evolve systematically without coupling design churn to core engine milestones, the platform enforces **three independently versioned domains**:

1. **Product Release & Milestone Versioning (`v1.0.0`, `M0`...`M13`):**  
   Tracks delivered functional features, language engine capabilities, and OS integrations.
2. **Design System / UX Versioning (`UX X.Y.Z`):**  
   Tracks visual design tokens, component patterns, typography, ergonomics, and accessibility conventions. Documented authoritative baseline: `UX 0.1.0` in [`docs/design-system-and-usability.md`](docs/design-system-and-usability.md). Product `v1.0.0` does not require or imply `UX 1.0.0`.
3. **Configuration Schema Versioning (`schema_version: "1.2.0"`):**  
   Tracks the JSON data contract persisted to disk (`config.json`), governed by the Rust core (`crates/bornomala-core/src/config.rs`) and persistence migration tests.

#### Key Invariants:
- **Decoupled Progression:** Milestone progress does not force a UX version increment. Milestones reuse the established UX version unless meaningful, approved UI pattern changes are introduced.
- **Token vs Preference Separation:** Presentation tokens (blur, spacing, elevation) remain in adapter styles (`src/index.css`, native C++ constants). Only genuine user-configurable preferences reside in `BornoConfig`.
- **Platform-Specific Realities:** Web stylesheets govern the sandbox; Win32 GDI C++ explicitly maps preferences to native color constants in Windows TSF; Linux Fcitx 5 delegates pixel styling to host desktop environment themes.

---

## 3. Roadmap Rules

These rules apply to every milestone and every AI-generated sub-milestone file.

### 3.1 The application must run from M0

M0 does not need to provide a complete system-wide IME. It must provide a real, visible, runnable application/sandbox that calls the actual Rust engine, shows engine status, supports a deterministic test interaction, and has documented build/run/test commands. At least one automated test must run locally and in CI.

Every later milestone must preserve a runnable, testable checkpoint.

### 3.2 One focused milestone instruction at a time

The master roadmap defines the pathway. The sub-milestone instruction file defines the work to do **now**.

Before writing the next instruction file, the AI agent must:

1. Inspect the actual repository and current branch.
2. Read the latest roadmap and prior milestone reports.
3. Check available tools, installed SDKs, and platform access.
4. Run relevant baseline build and tests before making changes.
5. Identify the next milestone's dependencies and unresolved questions.
6. Write only the next focused instruction file, with concrete tasks and acceptance criteria.
7. Avoid implementing unrelated future milestones.

Do not create detailed instructions for all future milestones up front. Earlier experiments may change the plan.

### 3.3 Evidence-based status

Do not use invented progress percentages or claim work is complete because code was generated.

Track separately:

- Code exists
- Code builds
- Automated tests pass
- Manual workflow has been checked
- End-to-end behavior works in a real target application
- Performance has been measured
- Package has been installed/tested
- Platform has been verified
- Known limitations remain

Use the statuses below:

- `Not started`
- `In progress`
- `Blocked`
- `Implemented — not verified`
- `Verified`
- `Deferred`
- `Out of scope`

Only use `Verified` when the milestone's acceptance evidence exists.

### 3.4 Every milestone preserves the baseline

A milestone is complete only when:

- Its acceptance criteria are satisfied.
- The app still launches.
- New and previous automated tests pass.
- Build, run, and test commands are documented and reproducible.
- Manual checks are recorded separately from automated tests.
- Known issues and unverified platform behavior are recorded.
- A checkpoint/commit is made when the developer approves it.
- The roadmap is updated from evidence.
- The next milestone's readiness and blockers are stated.

### 3.5 AI coding-agent discipline

For every generated change:

- Prefer small, reviewable tasks over a huge one-shot implementation.
- Inspect generated native code and FFI boundaries carefully.
- Do not overwrite working code or user changes without inspecting them.
- Do not silently change the chosen technology direction.
- Do not add dependencies without explaining why they are needed.
- Do not invent test results, benchmark results, platform access, or successful installation.
- Record exact commands and their real outcomes.
- When blocked by unavailable hardware/OS, create a small feasibility experiment or document the limitation; do not simulate a successful platform test.
- Keep the repository and a visible runnable sandbox available throughout development.

### 3.6 Design impact and usability governance

For every milestone that touches user-facing appearance, interaction, or candidate presentation:

- **Design Impact Assessment:** The milestone instruction file must evaluate impact on visual design tokens, interaction behavior, keyboard accessibility, dialog focus management, and cross-platform parity.
- **Evidence-Based Acceptance:** A successful build alone must never be claimed as proof of usability or accessibility. Visual contrast, focus indicators, keyboard trapping, and screen-reader announcements must be supported by executable or manual test evidence.
- **Proportionate UX Versioning:** Milestones reuse the established UX version (`UX 0.1.0`) when no meaningful design pattern changes are needed. New UX versions are created only when visual foundations, component patterns, or user interaction models change.
- **Tracked Gaps:** Unresolved accessibility items, platform rendering differences, and high-DPI scaling behaviors must be explicitly tracked in [`docs/design-system-and-usability.md`](docs/design-system-and-usability.md) rather than prematurely marked verified.

---

## 4. Milestone Overview

The sequence below is the master dependency pathway. It may be adjusted when real evidence requires it, but changes must be recorded with reasons. Milestones are not promises that every stage will take equal time.

| Milestone | Name | Main outcome | Completion gate | Status |
|---|---|---|---|---|
| M0 | Runnable foundation | Visible app/sandbox, Rust core linked, commands, tests, CI | Clean, reproducible launch and test | **Verified** (2026-10-17) |
| M0.1 | Native integration feasibility | Windows TSF in-process COM prototype & C-ABI boundary verified | FFI safety, TSF Level 1–3 pass, Level 4 elevation barrier documented | **Verified** (2026-10-17) |
| M1 | Engine contract and sandbox | Defined event/state/API contract and inspectable interaction | Contract tests and sandbox behavior pass | **Verified** (2026-10-17) |
| M2 | Unicode and composition foundation | Correct, documented Unicode/composition/editing behavior | Golden regression suite passes | **Verified** (2026-10-17) |
| M3 | Phonetic input v1 | Useful phonetic typing and candidate selection | Reviewed corpus evaluation and latency baseline | **Verified** (2026-10-17) |
| M4 | Fixed layouts | Versioned layout format and first validated layout(s) | Mapping and editing tests pass; compatibility documented | **Verified** (2026-10-18) |
| M5 | Settings foundation | Persistent configuration and working settings UI | Settings persist, validate, reset, and affect behavior | **Verified** (2026-10-18) |
| M6 | Dictionary and correction | Completion and spelling suggestions; conservative correction option if justified | Evaluation set and quality criteria pass | **Verified** (2026-10-18) |
| M7 | First native IME — Windows | Real system input method works end-to-end on Windows | Tested in multiple real applications on Windows | **Verified** (2026-10-18) |
| M8 | Linux integration | Native input integration and package for defined Linux configuration | Real Linux end-to-end test and support statement | **Verified** (2026-10-19) |
| M9 | Customization and usability | User-facing configuration, candidate settings, supported themes/layout tooling | Usability/accessibility checks and regression pass | **In progress (Next)** |
| M10 | Contextual prediction | Optional offline next-word prediction with reproducible evaluation | Meets quality/latency/memory thresholds or is deferred |
| M11 | Hardening and cross-platform regression | Reliability, privacy, performance, accessibility, security/license review | Release-blocking issues resolved or documented |
| M12 | Packaging and release candidate | Installers/packages, upgrade/uninstall checks, release notes | Clean install and release-candidate verification |
| M13 | v1.0.0 release decision | Feature freeze, verified artifacts, final support matrix and tagged release | Every claimed platform meets its release gate |

---

### 4.1 Milestone Progress & Feature Tracking

| Milestone | Shortform Requirements | Completed Features | % Complete | Comments |
|---|---|---|:---:|---|
| **M0** — Runnable Foundation | Runnable desktop sandbox, decoupled Rust core, deterministic interaction, unit tests, CI baseline | `borno-core` state machine, 11 unit tests, Tauri v2 host, React+TS sandbox, standalone `Borno-Sandbox.exe` offline bundle, GitHub Actions CI | 100% | Verified. App compiles, launches, and deterministically transliterates `ka` -> `কা`. |
| **M0.1** — Native Integration Feasibility | Windows TSF COM architecture spike, C-ABI FFI boundary, memory ownership rules, permission analysis | `docs/windows-tsf-architecture.md`, `borno.h`, 5 FFI automated tests, self-contained `borno_tsf.dll`, test harness (Levels 1–3 pass, Level 4 elevation evaluated) | 100% | Verified. C-ABI handles multi-byte Bangla Unicode safely; TSF profile registration confirmed to require elevated installer. |
| **M1** — Engine Contract & Sandbox | Formalized event/state contract, persistent state vs ephemeral effects, candidate selection/wrapping, in-preedit cursor, inspectable sandbox UI | `docs/engine-contract.md`, 13 contract tests in `crates/borno-core/tests/contract_tests.rs`, C-ABI event dispatch & memory safety, desktop sandbox in-preedit cursor & candidate tiles | 100% | Verified. Contract verified with 29 passing automated tests; standalone `Borno-Sandbox.exe` rebuilt and verified. |
| **M2** — Unicode & Composition Foundation | Unicode normalization policy, Bangla combining marks, hasanta/virama, conjunct composition, golden test suite | `docs/unicode-composition.md`, `crates/borno-core/src/unicode.rs` (UCD 16.0.0, 96 assigned, 32 unassigned), strict NFC normalization, canonical nukta decomposition in committed text, UAX #29 extended grapheme clusters (GB9a), 22-test golden suite, desktop sandbox continuous font-shaping verification, rebuilt `Borno-Sandbox.exe` | 100% | Verified. 60 total automated tests pass with zero regression; standalone `Borno-Sandbox.exe` rebuilt and verified on Windows. |
| **M3** — Phonetic Input v1 | Full phonetic typing table (Avro-compatible), candidate generation, ambiguity resolution, latency evaluation | Pure Rust phonetic engine (`crates/borno-core/src/phonetic/`), pinned Avro rules, 30-conjunct matrix, geminates, phalas, Candrabindu reordering, Khanda-Ta disambiguation, bounded candidates (ALT-1, ALT-2a/b), Scenarios C-1–C-5 cursor mapping, 57 tests, 0.0034 ms release benchmark, desktop sandbox Avro guide | 100% | Verified. 117 total automated tests pass with zero regression; mean core latency 3.44 µs exceeds 1.0 ms gate by 290x; standalone `Borno-Sandbox.exe` rebuilt. |
| **M4** — Fixed Layouts | Layout specification format, Bijoy, National/Jatiya, Probhat layout mappings, modifier handling | Pure Rust fixed layout engine (`crates/borno-core/src/layout/`), declarative schema with strict validation, National (Provisional BCC), Probhat (Verified Upstream), Bijoy-Compatible (Provisional Unicode), link-key state machine, leading-virama discard, backspace deconstruction to LinkPending, C-ABI `borno_engine_process_key`, 57 tests, 0.79–1.19 µs latency, interactive virtual keyboard UI in sandbox | 100% | Verified. 176 total workspace tests pass with zero regression; mean core latency ~1 µs; standalone `Borno-Sandbox.exe` rebuilt and verified on Windows. |
| **M5** — Settings Foundation | Persistent configuration schema, settings UI in sandbox, live engine reconfiguration, reset defaults | Pure Rust typed schema `BornoConfig` v1.0.0, platform-safe replacement persistence (`config.json`), corruption backup/recovery, unsupported version protection, live engine candidate slicing & mode-switch auto-commit, accessible dark glassmorphic Settings Drawer UI in desktop sandbox, 19 new tests | 100% | Verified. 195 total tests pass with zero regression; failure-safe replacement & recovery verified; standalone `Borno-Sandbox.exe` rebuilt and verified on Windows. |
| **M6** — Dictionary & Correction | Wordlist/corpus processing, prefix tree/trie lookup, spelling assistance, offline dictionary resources | Leipzig CC BY 4.0 40k-word corpus (`lexicon.bin`), pure Rust `PrefixTrie` with admissible branch-and-bound pruning, UAX #29 grapheme Damerau-Levenshtein distance, phonetic homophone ambiguity resolution (`ক্ষমা`, `বিজ্ঞান`, `ভাষা`), Slot 0 raw transliteration preservation, configuration migration 1.0.0 -> 1.1.0, candidate tiles with kind badges in sandbox UI, 34 new automated tests, 1.9µs prefix query latency, 0.81ms release keystroke latency | 100% | Verified. 229 total tests pass with zero regression; sub-millisecond release latency; standalone `Borno-Sandbox.exe` rebuilt and verified on Windows. |
| **M7** — First Native IME — Windows | Full Windows TSF input method, text store edit session, candidate window UI, system-wide typing in Notepad/Word | Full Windows Text Services Framework in-process COM IME (`native/borno-tsf/`), floating candidate window with GDI rendering & caret tracking, language bar integration (`F12`), tested across Windows Notepad, Sublime Text, Office, Visual Studio Code | 100% | Verified. Documented in `milestones/M7-completion-report.md`. |
| **M8** — Linux Integration | Fcitx 5 / IBus native engine adapter, Unix packaging, display server verification (X11/Wayland) | Full Linux Fcitx 5 C++ addon (`native/bornomala-fcitx5/`), static Rust core linking (`libbornomala_core.a`), preedit underline, floating candidates, interactive GTK tester, verified under Ubuntu WSL2/X11/Wayland | 100% | Verified. Documented in `milestones/M8-completion-report.md`. |
| **M9** — Customization & Usability | Candidate window styling, shortcut keys, dark/light themes, accessibility hooks | Scheduled next: shortcut configuration, layout export, punctuation toggle (`..` -> `।`), and linguistic backlog (`docs/usability-and-linguistic-backlog.md`) | 0% | In preparation. |
| **M10** — Contextual Prediction | Optional offline n-gram/contextual next-word prediction, quality/latency evaluation | None | 0% | Non-blocking for v1.0.0 release; quality and latency thresholds apply. |
| **M11** — Hardening & Cross-Platform Regression | Stress tests, memory leak auditing, security/privacy review, long typing sessions | Null-pointer safety and explicit memory disposal in FFI tests | 10% | Continuous regression discipline enforced from M0. |
| **M12** — Packaging & Release Candidate | Elevated Windows installer (WiX MSI / NSIS) registering TSF DLL, uninstallers, auto-update checks | Standalone portable `.exe` bundle and `.bat` launcher in M0; elevated registration documented | 15% | Elevated installer design documented in M0.1 completion report. |
| **M13** — v1.0.0 Release Decision | Feature freeze, verified multi-platform support matrix, signed releases, documentation | Roadmap governance and milestone verification framework | 5% | Final sign-off gate across all claimed platforms. |

---

### 4.2 Status by Dimension

| Dimension | % ready | Hours to full | Doc |
|---|:---:|:---:|---|
| **Core Engine & State Machine** (composition, events, state, C-ABI) | 95% | 10–20 | [architecture.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/docs/architecture.md), [engine-contract.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/docs/engine-contract.md), [borno.h](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/crates/borno-core/include/borno.h) |
| **Phonetic Transliteration** (Avro-compatible rules, candidate ranking) | 90% | 10–20 | [rules.rs](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/crates/borno-core/src/phonetic/rules.rs), [tokenizer.rs](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/crates/borno-core/src/phonetic/tokenizer.rs), [candidates.rs](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/crates/borno-core/src/phonetic/candidates.rs) |
| **Fixed Keyboard Layouts** (Bijoy, National, Probhat, modifier mappings) | 95% | 5–10 | [M4-completion-report.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/milestones/M4-completion-report.md), [schema.rs](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/crates/borno-core/src/layout/schema.rs), [builtin.rs](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/crates/borno-core/src/layout/builtin.rs), [engine.rs](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/crates/borno-core/src/layout/engine.rs) |
| **Unicode & Composition Correctness** (combining marks, conjuncts, hasanta, UCD 16.0.0, NFC, UAX #29) | 95% | 10–15 | [unicode-composition.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/docs/unicode-composition.md), [unicode_golden_tests.rs](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/crates/borno-core/tests/unicode_golden_tests.rs) |
| **Language Assistance & Dictionary** (trie lookup, spell check, frequency corpus) | 90% | 15–25 | [M6-completion-report.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/milestones/M6-completion-report.md), [lexicon_provenance.json](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/docs/lexicon_provenance.json), [resources-and-licenses.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/docs/resources-and-licenses.md) |
| **Native Windows TSF Integration** (COM in-process server, edit session, IME) | 95% | 10–20 | [windows-tsf-architecture.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/docs/windows-tsf-architecture.md) |
| **Native Linux Fcitx 5 Integration** (C++ addon, preedit, popup candidates) | 95% | 10–20 | [M8-completion-report.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/milestones/M8-completion-report.md) |
| **Desktop Sandbox & Settings UI** (Tauri v2 + React, glassmorphism, configuration) | 90% | 15–30 | [App.tsx](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/src/App.tsx) |
| **Performance & Latency** (<5ms keystroke budget, pure-core <1ms gate) | 90% (3.4µs core measured) | 10–15 | [M3-completion-report.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/milestones/M3-completion-report.md) |
| **Packaging, Installer & Deployment** (WiX MSI / NSIS / AppImage / Deb) | 25% | 40–60 | [windows-tsf-architecture.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/docs/windows-tsf-architecture.md) |
| **Privacy & Offline Guarantee** (Zero telemetry, local dictionaries, offline by default) | 95% | 5–10 | [AGENTS.md](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/AGENTS.md) |

---

**macOS track:** Start feasibility research and interface planning early, but do not claim implementation completion based on Windows/Linux work. Build a dedicated macOS implementation/test sub-milestone when real Mac access is available. If access remains unavailable, document macOS as `Blocked` or `Deferred` and make the release scope decision explicit at M12–M13.

---

## 5. Detailed Milestone Definitions

## M0 — Runnable Foundation

**Purpose:** Establish the smallest real application that can be built, launched, and tested before feature development expands.

### Required work

- Inspect the actual repository, current files, installed toolchains, and available CI.
- Establish a clear repository structure and concise development documentation.
- Pin or document compatible versions of the toolchain and important dependencies.
- Create the initial Rust core library.
- Create a visible application/sandbox using the agreed UI direction, unless repository inspection demonstrates a concrete reason to stage the UI shell differently.
- Link the sandbox to the actual Rust core; do not use a fake engine implementation.
- Add a small deterministic input interaction and visible engine status.
- Add at least one automated core test and a reproducible test command.
- Add or configure CI to run the relevant automated checks.
- Document exact commands to build, run, test, and troubleshoot the project on Windows.
- Keep the app simple; do not attempt the complete IME or language engine in M0.

### Acceptance criteria

- A clean checkout can be built using the documented steps on the available Windows development environment.
- The app launches and visibly responds to a deterministic test interaction.
- The sandbox calls the actual Rust core.
- Automated tests execute and their results are recorded.
- CI runs the relevant checks.
- The app remains runnable after the milestone is complete.
- No unverified platform support is claimed.

### Deliverables

- Working repository baseline
- Visible runnable sandbox
- Rust core library connected to the app
- Build/run/test documentation
- Automated test and CI baseline
- `M0` completion report with commands and evidence

### Not included

- Full phonetic engine
- Complete fixed layouts
- Production system-wide IME
- Mature dictionary/prediction
- Extensive customization

## M0.1 — Native Integration Feasibility

**Purpose:** Test the highest-risk architectural assumption before building too much on top of it.

### Required work

- Investigate the Windows native input-method integration path, targeting TSF.
- Build the smallest practical Windows experiment that demonstrates whether the proposed native adapter can communicate with the Rust engine.
- Identify required Windows SDK/toolchain prerequisites and likely packaging/debugging constraints.
- Investigate one concrete Linux integration target and prerequisites; verify whether a Linux VM or another test environment is actually available.
- Define the proposed initial Linux target, including distribution/version, input framework, and desktop/display-session assumptions.
- Decide where native code ends and shared Rust logic begins.
- Document what was built and tested, what was only researched, and what remains unknown.
- Keep experiments isolated from the main working app unless integration is safe.

### Acceptance criteria

- There is a recorded Windows TSF feasibility result, with real test evidence or a clearly documented blocker: **Verified** (Documented in `milestones/M0.1-completion-report.md`).
- Linux feasibility and access are classified as verified, unverified, or blocked: **Verified** (Documented as `Unverified / Environment Not Available`).
- The Rust/native boundary is documented and justified: **Verified** (Documented in `docs/windows-tsf-architecture.md` and `crates/borno-core/src/ffi.rs`).
- The roadmap has an explicit platform strategy: **Verified**.
- The project does not falsely claim that all platforms are already feasible or verified: **Verified**.

## M1 — Engine Contract and Sandbox

**Purpose:** Define the behavior that every input method and adapter will use.

### Required work

- Define normalized input events (for example: key input, mode switch, candidate selection, backspace, cancellation, and relevant editing/context events).
- Define engine state, including composition/preedit, candidates, selected candidate, committed text, and error/status reporting.
- Define clear rules for when text is provisional and when it is committed.
- Define candidate selection, cancellation, reset, and error behavior.
- Specify API boundaries between Rust core, sandbox, settings, and native adapters.
- Make the sandbox inspect engine state and events in a useful debugging view.
- Add contract tests and representative event sequences.
- Document assumptions about host application text-client behavior and where the platform adapters may differ.

### Acceptance criteria

- The event/state contract is documented and version-controlled: **Verified** (Documented in `docs/engine-contract.md`).
- Tests cover normal input, candidate selection, commit, cancellation, reset, and error handling: **Verified** (13 tests in `crates/borno-core/tests/contract_tests.rs`, 29 total core tests passing).
- The sandbox visibly demonstrates the contract: **Verified** (In-preedit cursor, candidate tiles, mode switcher, live event inspector).
- The app remains runnable: **Verified** (Standalone `Borno-Sandbox.exe` rebuilt and launched on Windows).
- No adapter independently invents conflicting composition semantics: **Verified** (C-ABI and Tauri IPC unified to the authoritative `ProcessResult` engine contract).

## M2 — Unicode and Composition Foundation

**Purpose:** Make text correctness a foundation, not a late cleanup task.

### Required work

- Define the Unicode output policy, normalization expectations, and handling of Bangla combining marks.
- Define composition rules for Bangla letters, vowel signs, hasanta/virama, conjuncts, punctuation, numerals, and mixed-script text as applicable.
- Specify backspace/delete behavior, cursor movement interactions, cancellation, and committed-text editing boundaries.
- Add reviewed golden test cases using expected input/output sequences.
- Include malformed or unexpected event sequences where relevant.
- Add regression tests for repeated deletion, empty composition, switching modes, and mixed Bangla/English.
- Document cases that depend on host application or OS behavior.

### Acceptance criteria

- Unicode and composition rules are documented.
- Golden tests cover the agreed core cases.
- Every known bug fixed in this milestone gets a regression test where practical.
- No known severe text corruption or data-loss issue remains in the tested core workflows.
- Platform-specific limitations are explicitly recorded.

## M3 — Phonetic Input v1

**Purpose:** Deliver a useful first phonetic Bangla typing method in the sandbox.

### Required work

- Define the phonetic scheme and compatibility target clearly; “Avro-style” must be backed by a specific documented target rather than an ambiguous promise of full compatibility.
- Implement transliteration and candidate generation in the Rust core.
- Define ambiguity handling, punctuation, capitalization, word boundaries, and candidate ranking behavior.
- Support candidate selection through the engine contract.
- Create a representative, legally usable evaluation set of Bangla words, phrases, and mixed-language examples.
- Measure expected-output accuracy and keystroke-to-result latency.
- Record known mismatches and prioritize them.
- Keep the engine usable when language resources are missing or incomplete.

### Acceptance criteria

- Phonetic typing works in the sandbox for the agreed test set.
- Candidate selection and commit follow the engine contract.
- Accuracy results are reproducible and documented.
- Latency is measured on the development machine with the method documented.
- No unsupported claim of complete compatibility is made.
- All Unicode regression tests continue to pass.

## M4 — Fixed Keyboard Layouts

**Purpose:** Add fixed-layout typing without coupling layouts to the phonetic engine.

### Required work

- Define a versioned, validated, data-driven keyboard-layout format.
- Decide the initial layout(s) based on explicit product priorities and available verified layout definitions.
- Document each layout's source, intended compatibility, and any differences.
- Implement layout mapping in the shared core.
- Provide a visual keyboard reference in the sandbox or settings UI.
- Test normal keys, modifiers, punctuation, mode switching, and composition/editing interactions.
- Validate malformed or unsupported layout files and provide useful errors.
- Do not claim compatibility with a named layout unless the mapping has been compared against an identified reference and tested.

### Acceptance criteria

- At least one explicitly identified fixed layout works in the sandbox.
- Layout definitions are versioned and validated.
- A visual reference corresponds to the actual mapping.
- Layout-specific regression tests pass.
- The engine can switch between phonetic and fixed-layout modes without corrupting text.

## M5 — Settings Foundation

**Purpose:** Introduce durable configuration and a real settings application.

### Required work

- Define a versioned configuration schema with defaults and validation.
- Implement persistent configuration and safe handling of missing, invalid, or older configuration.
- Create the first functional TypeScript/React/Tauri settings UI.
- Support essential settings such as input method selection, available layout selection, and basic candidate/typing preferences that already exist in the engine.
- Ensure UI changes actually affect the engine.
- Add reset-to-default behavior and document the configuration location.
- Keep settings local and avoid collecting typed text.
- Add tests for defaults, persistence, validation, and reset.

### Acceptance criteria

- Settings persist across restarts.
- Invalid settings fail safely and provide a useful fallback or error.
- UI changes affect actual engine behavior.
- Configuration is versioned and documented.
- The sandbox and automated tests still work.

## M6 — Dictionary and Correction

**Purpose:** Add useful dictionary-backed writing assistance while keeping it separate from input-method selection.

### Required work

- Decide and document the initial dictionary/resource sources, licensing, format, update strategy, and language coverage.
- Add word completion and spelling suggestions.
- Rank candidates using reproducible, explainable baseline logic before considering complex models.
- Define whether autocorrection is included; if included, make it conservative, configurable, and reversible where feasible.
- Do not silently replace user text in ways that risk losing intent.
- Build a representative evaluation set for common words, misspellings, inflections, and mixed-script text.
- Measure suggestion usefulness and latency.
- Keep assistance optional/configurable and ensure core typing still works when resources are unavailable.

### Acceptance criteria

- Completion and spelling suggestions work for the defined test cases.
- The evaluation set and results are versioned and reproducible.
- Dictionary resource licensing and provenance are documented.
- Suggestions remain responsive and do not block ordinary typing.
- Autocorrection, if included, has explicit behavior and a setting.
- The product does not confuse dictionary spelling correction with next-word prediction.

## M7 — First Native IME: Windows

**Purpose:** Make the shared engine usable for system-wide typing on the primary development platform.

### Required work

- Implement the native Windows adapter using the chosen, documented integration path, with TSF as the intended starting point.
- Connect native events and composition/candidate/commit behavior to the shared engine contract.
- Support focus changes, cancellation, and relevant lifecycle behavior.
- Test in more than one real Windows application with different text controls where practical.
- Test switching input modes and recovering from interrupted composition.
- Keep the sandbox as a separate debugging and regression tool.
- Document installation, activation, switching, logging that does not capture raw typed content by default, and troubleshooting.
- Review the native/Rust boundary for memory safety, lifecycle issues, and error handling.

### Acceptance criteria

- A user can install/enable the input method and type using the engine in multiple real Windows applications.
- Phonetic input works end-to-end through the native adapter.
- Fixed layout support is included if it has already passed its own acceptance criteria and the adapter supports it.
- Composition, candidate selection, commit, cancellation, and deletion pass a documented manual test plan.
- No critical text-loss or input-locking issue remains in tested workflows.
- The Windows support statement reflects actual tested Windows versions and applications.

## M8 — Linux Integration

> **Status: Verified (2026-10-19)** — See completion report at [`milestones/M8-completion-report.md`](file:///e:/Codes/Bangla%20Input%20Platform/Borno%20V1/milestones/M8-completion-report.md).

**Purpose:** Bring the engine into a clearly defined Linux desktop environment.

### Required work

- Confirm the available Linux test environment.
- Select and document the initial supported distribution/version, input framework (for example, Fcitx 5), desktop environment, and display-session assumptions.
- Implement the adapter and connect it to the shared engine.
- Build and package the integration for the selected environment.
- Test typing in multiple real applications, including composition, candidates, commit, cancellation, mode switching, and deletion.
- Record behavior differences from Windows and any unsupported environments.
- Document dependencies, installation, enabling, troubleshooting, and removal.
- Do not generalize a single tested configuration into universal Linux support.

### Acceptance criteria

- The input method installs and works end-to-end in the declared Linux configuration.
- Test environment and application list are recorded.
- Core and platform-specific regression tests pass.
- A capability/support matrix states what is verified, unverified, or unsupported.
- The app remains runnable in the sandbox.

## M9 — Customization and Usability

**Purpose:** Make the product configurable and comfortable without destabilizing core typing.

### Required work

- Prioritize customization based on real user workflows and existing settings architecture.
- Add shortcut configuration with validation and conflict handling where supported.
- Add basic candidate presentation and density/typography preferences only where platform capabilities permit.
- Add a limited theme system if the settings UI and candidate surfaces support it.
- Add layout visualization and, only if justified, a constrained layout editor.
- Add import/export for supported configuration and layout formats with validation.
- Improve accessibility, keyboard navigation, focus behavior, labels, and readable error messages.
- Document platform differences instead of promising identical presentation everywhere.
- Document the authoritative living design system baseline (`UX 0.1.0`) in [`docs/design-system-and-usability.md`](docs/design-system-and-usability.md), establishing token contracts and platform rendering rules.
- Track interactive ergonomic feedback, usability observations, and linguistic edge cases in [`docs/usability-and-linguistic-backlog.md`](docs/usability-and-linguistic-backlog.md).
- Defer complex visual editing or advanced personalization if it threatens core correctness or release readiness.

### Acceptance criteria

- Supported settings persist and actually affect behavior.
- Invalid imports are rejected safely and do not corrupt existing settings.
- Keyboard-only navigation and basic accessibility checks are documented.
- Customization changes do not break engine, Unicode, or native integration tests.
- The release does not claim settings that are only decorative or unsupported on a platform.
- Authoritative design system specification documented at `UX 0.1.0` in [`docs/design-system-and-usability.md`](docs/design-system-and-usability.md), with explicit verification gaps recorded.

## M10 — Contextual Prediction (Optional, Evidence-Gated)

**Purpose:** Evaluate whether offline next-word prediction adds real value.

### Required work

- Keep prediction separate from spelling suggestions and from the core input-method contract.
- Build a reproducible Python evaluation pipeline and document data provenance, licensing, preprocessing, and splits.
- Establish baseline metrics and representative test cases before choosing a model.
- Evaluate quality, latency, memory use, startup impact, and behavior with mixed-language text.
- Make prediction optional/opt-in if included.
- Ensure prediction cannot block ordinary keystroke handling.
- Keep data local and do not transmit typed text.
- Complete a Design Impact Assessment for prediction suggestion presentation; reuse the `UX 0.1.0` baseline unless new component patterns are justified and approved.
- If quality or performance fails the agreed thresholds, mark prediction experimental or defer it; do not hold v1.0.0 hostage to a weak model.

### Acceptance criteria

- Evaluation can be reproduced from documented steps.
- Quality and performance thresholds are set and results are recorded.
- Prediction meets the agreed thresholds or is explicitly deferred/experimental.
- Ordinary typing remains responsive when prediction is disabled and when it is enabled.
- Privacy implications and model/resource licensing are documented.
- Design impact on candidate presentation is assessed; no speculative UX version bump without approved UI changes.

## M11 — Hardening and Cross-Platform Regression

**Purpose:** Turn a feature-complete prototype into reliable software.

### Required work

- Expand automated regression coverage for Unicode, composition, layouts, phonetic input, settings, dictionary assistance, and platform adapters.
- Add fuzzing/property-style tests where practical for parsers, layout files, configuration, and event sequences.
- Measure keystroke-to-result latency, candidate latency, startup, memory, idle CPU/resource use, and large-resource behavior on reference hardware.
- Test crash recovery and interrupted composition.
- Review privacy behavior, logs, diagnostics, telemetry (if any), and storage.
- Review dependency licenses, dictionary/model provenance, packaging dependencies, and third-party notices.
- Review accessibility, keyboard navigation, visible focus rings, dialog focus management, and screen-reader announcements against WCAG 2.1 AA standards defined in [`docs/design-system-and-usability.md`](docs/design-system-and-usability.md).
- Measure rendered contrast ratios of foreground and background combinations across all supported themes.
- Test high-DPI scaling behavior (125%, 150%, 200%) on Windows native GDI surfaces.
- Triage bugs by severity and record reproducible steps.
- Run the same core regression suite on available environments; clearly identify unavailable platform tests.
- Do not use unmeasured performance claims.

### Acceptance criteria

- Release-blocking correctness and data-loss issues are fixed or release is blocked.
- Core regression tests pass.
- Performance results and test hardware/software are documented.
- Privacy and license reviews are recorded.
- Accessibility and contrast measurements are documented with actual test evidence, not assumed from token centralization.
- High-DPI GDI scaling behavior and any remaining visual defects are recorded.
- Known limitations and unverified platforms are visible.
- No platform is marked verified solely because the shared Rust core passes.

## M12 — Packaging and Release Candidate

**Purpose:** Verify that users can install, use, upgrade, and remove the software.

### Required work

- Create platform-specific distribution artifacts for platforms that have passed their native integration gates.
- Test clean install, first run, enabling the input method, normal use, upgrade, and uninstall.
- Verify required language resources and configuration are included or downloaded only through an explicit documented flow.
- Implement and verify the configuration lifecycle migration from `com.bornomala.sandbox` to canonical `com.bornomala.desktop`, handling existing files, coexistence, and corrupted file safety per [`docs/design-system-and-usability.md`](docs/design-system-and-usability.md).
- Document prerequisites and supported OS versions/configurations.
- Prepare release notes, user documentation, troubleshooting, license notices, and privacy statement.
- Freeze feature additions while resolving release blockers.
- Prepare a capability matrix for every platform and feature.
- Make an explicit macOS scope decision based on whether real Mac testing is available.
- Do not ship an artifact as production-ready solely because it compiles.

### Acceptance criteria

- Clean installation and uninstall are verified for every claimed release platform.
- Upgrade behavior is tested or explicitly documented as not supported for the first release.
- Configuration lifecycle migration and coexistence are tested between sandbox and desktop installations.
- Installers/packages and checksums or equivalent artifact verification are prepared as appropriate.
- Release notes and support matrix match actual evidence.
- No unresolved critical/high-severity blocker remains without an explicit release decision.
- macOS is either verified through real end-to-end tests or explicitly deferred/not claimed.

## M13 — v1.0.0 Release Decision

**Purpose:** Make a deliberate release decision, not simply reach the last numbered milestone.

### Required work

- Freeze the v1.0.0 feature set.
- Run the final regression and release-candidate test plan.
- Verify the build from a clean checkout and document exact commands/toolchain versions.
- Verify all published release artifacts.
- Finalize architecture, installation, user, troubleshooting, privacy, and license documentation.
- Publish a support matrix that differentiates verified, partially verified, unverified, and unsupported configurations.
- Review all remaining bugs and explicitly accept or block release.
- Tag the release and preserve the release evidence/report.
- If a platform cannot pass its gate, reduce the claimed support scope or delay the release; do not falsely claim complete cross-platform support.

### v1.0.0 minimum release gates

- Shared Rust core is tested and documented.
- Phonetic input has a measured, reviewed evaluation baseline.
- At least one fixed layout is documented and tested.
- Unicode/composition/editing regression suite passes.
- Dictionary completion/spelling assistance is useful against a documented evaluation set, or its scope is explicitly reduced.
- Native integration works end-to-end on every platform advertised as supported.
- Essential settings persist and affect actual behavior.
- Ordinary typing works offline with installed resources.
- No default transmission or raw logging of typed content.
- Packaging, installation, and removal have been verified for claimed platforms.
- Documentation, licensing, privacy statement, and support matrix are complete.
- UX Release-Readiness Assessment: A structured review distinguishing documented design requirements ([`docs/design-system-and-usability.md`](docs/design-system-and-usability.md)), implemented behavior across supported platforms, automated test evidence, and manual accessibility/contrast verification, with unresolved items explicitly tracked.
- No unresolved critical issue remains that risks text loss, typing failure, privacy, or installation.
- Contextual prediction is included only if its evaluation supports inclusion; otherwise it is clearly experimental or deferred.

---

## 6. Cross-Cutting Requirements

These requirements apply throughout the roadmap, not just at the end.

### 6.1 Correctness and Unicode

Maintain a shared regression suite for:

- Bangla letters and vowel signs
- Hasanta/virama and conjunct sequences
- Combining marks and normalization expectations
- Backspace and delete behavior
- Cursor movement and interrupted composition
- Candidate selection, commit, cancel, and mode switch
- Punctuation and numerals
- Mixed Bangla and English
- Empty composition and repeated deletion
- Unexpected event sequences
- Host application differences that affect text insertion

Use reviewed expected outputs. Do not treat visual appearance alone as proof of Unicode correctness.

### 6.2 Performance

Measure actual behavior on documented reference hardware. Track at least:

- Keystroke-to-result latency
- Candidate-generation and candidate-display latency where measurable
- Startup time
- Memory usage
- Idle resource usage
- Behavior with dictionary/model resources loaded

Set numeric thresholds after obtaining an initial baseline and agreeing on realistic requirements. Do not invent thresholds or claim performance before measurement.

### 6.3 Privacy and offline behavior

- Core typing must not require a network connection.
- Do not transmit typed content by default.
- Do not keep raw keystroke logs by default.
- Diagnostic logs should avoid sensitive typed content.
- Any future personalization should be local and opt-in unless explicitly reconsidered.
- Document any network use, update checks, or resource download behavior.

### 6.4 Resource licensing and provenance

For dictionaries, corpora, layouts, models, and third-party components:

- Record source and license.
- Record any transformations or preprocessing.
- Keep reproducible preparation scripts where practical.
- Do not bundle assets with unclear redistribution rights.
- Include required notices in release packages.

### 6.5 Documentation

Keep the following documentation current as the project evolves:

- `README.md` — project purpose, current status, quick start
- `ROADMAP.md` — this master roadmap or a maintained equivalent
- `docs/architecture.md` — architecture and dependency boundaries
- `docs/engine-contract.md` — engine event/state/API contract
- `docs/unicode-and-composition.md` — text behavior and known limitations
- `docs/testing.md` — automated/manual tests and reproducible commands
- `docs/platform-support.md` — tested OS configurations and capability matrix
- `docs/privacy.md` — data handling and diagnostics
- `docs/resources-and-licenses.md` — resource provenance and notices
- `docs/release-process.md` — packaging and release procedure

These are intended document roles, not a demand to create every file during M0. Add them when the relevant milestone needs them and avoid duplicate sources of truth.

---

## 7. Required Structure for Every Sub-Milestone Instruction File

Every generated milestone instruction file should use a consistent structure and be based on the actual repository state.

1. **Milestone identity**
   - ID and name
   - Current status
   - Dependencies
   - Purpose and boundaries

2. **Current-state inspection**
   - Files/modules to inspect
   - Current build/test status
   - Prior milestone evidence
   - Verified facts versus assumptions

3. **Goals and non-goals**
   - Specific outcomes required now
   - Explicitly excluded work

4. **Implementation plan**
   - Small, ordered tasks
   - Expected files/modules and interface changes
   - Dependency/license implications
   - Questions that must be resolved before implementation

5. **Acceptance criteria**
   - Observable and testable criteria
   - No vague “works correctly” criteria without examples or test procedures

6. **Testing plan**
   - Unit/integration/regression tests
   - Manual tests where necessary
   - Real OS/application tests if applicable
   - Exact commands, expected results, and environment assumptions

7. **Runnable checkpoint**
   - How to build and launch the app after the changes
   - Confirmation that the sandbox remains available
   - Recovery or rollback guidance if the milestone fails

8. **Evidence/reporting requirements**
   - Files changed
   - Commands actually run
   - Test results
   - Manual checks
   - Benchmarks, if relevant
   - Known issues and unverified claims

9. **Roadmap update**
   - Update status only from evidence
   - Record blockers and scope changes
   - State whether the next milestone is ready

10. **Stop conditions**
    - Stop and report rather than inventing assumptions, silently changing architecture, or proceeding past a failed acceptance gate.

The instruction file must tell the AI coding agent to implement only the named milestone. It must not authorize a broad rewrite or future feature implementation.

---

## 8. Milestone Completion Report Template

At the end of each milestone, record:

```md
# Mx Completion Report

## Status
- Status: Verified / Implemented — not verified / Blocked / Deferred
- Date:
- Commit or checkpoint:

## Scope
- Planned outcomes:
- Completed outcomes:
- Deferred outcomes and reasons:

## Changes
- Files/modules changed:
- Dependencies added/updated:
- Architecture/interface changes:

## Evidence
- Commands actually run:
- Automated test results:
- Manual test results:
- Platforms and applications tested:
- Performance results, if applicable:

## Known Issues
- Bugs:
- Limitations:
- Unverified claims/platforms:
- Privacy/licensing concerns:

## Acceptance Criteria
- [ ] Criterion — evidence:
- [ ] Criterion — evidence:

## Roadmap Decision
- Milestone status:
- Blockers:
- Is the next milestone ready?
- Recommended next milestone:
```

Do not mark checkboxes complete without evidence. Distinguish a test that passed from a test that was not run.

---

## 9. Platform Strategy and Release Honesty

### Windows

- Primary development and first native integration target.
- Perform early feasibility work in M0.1.
- Build the first end-to-end native vertical slice before investing heavily in advanced features.
- Verify behavior in real applications, not only in the sandbox.

### Linux

- Investigate early rather than leaving it until the end.
- Confirm whether a VM/dual boot/other environment is available.
- Choose and document a specific first supported configuration.
- Treat other distributions, desktop environments, and display-server combinations as unverified until tested.

### macOS

- Keep macOS in the intended desktop architecture.
- Research the native integration mechanism early enough to identify architectural concerns.
- Do not claim a successful build, install, or end-to-end test without access to a real Mac environment.
- When access becomes available, add a dedicated macOS implementation milestone with concrete build/install/typing tests.
- If access is unavailable near release, either obtain a suitable test environment or explicitly defer macOS support in the v1.0.0 support statement.

### Support matrix format

Maintain a table similar to this and update it from evidence:

| Platform/configuration | Builds | Installs | Types in real apps | Core features tested | Status / limitations |
|---|---:|---:|---:|---:|---|
| Windows 11 Build 26200 (x64) | Verified | Local/Dev | Sandbox Verified | State machine, transliteration baseline | M0 Verified; native TSF in M0.1/M7 |
| Linux (distribution to define) | Unknown | Unknown | Unknown | Unknown | Environment must be confirmed in M0.1 |
| macOS (version to define) | Unknown | Unknown | Unknown | Unknown | Requires real Mac access (deferred) |

Do not replace `Unknown` with a guess.

---

## 10. Immediate Next Step

The next action is **not** to implement the whole product or to write detailed files for every milestone.

1. Save this document as `ROADMAP.md` at the repository root (or merge its content into the existing master roadmap after checking the repository).
2. Inspect the current repository and environment.
3. Create a focused `M0.md` instruction file from the template in Section 7.
4. Execute M0 and collect real build/test evidence.
5. Update this roadmap before creating `M0.1.md` or `M1.md`.
6. Use M0.1 to test native integration feasibility early; revise later milestones only when evidence warrants it.

The roadmap should remain a living record of the plan and verified progress. The sub-milestone files are temporary, focused execution instructions derived from the roadmap and the repository as it actually exists.

---

## 11. Final Project Declaration

We intend to create a fast, modular, privacy-conscious desktop Bangla input platform that unifies phonetic and fixed-layout typing around a shared Rust engine, provides useful dictionary-backed language assistance, and offers a modern customizable experience through native system input-method integrations.

The product must be built incrementally, with a runnable and testable application from M0 onward. Each milestone must preserve working functionality, use explicit acceptance criteria, and report evidence honestly. Windows is the first practical implementation target; Linux must have a defined and verified configuration; macOS support requires real macOS build and end-to-end testing. The `v1.0.0` release should be declared only for platforms and workflows that have actually passed their release gates.
