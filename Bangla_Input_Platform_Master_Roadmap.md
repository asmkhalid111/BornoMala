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

---

## 4. Milestone Overview

The sequence below is the master dependency pathway. It may be adjusted when real evidence requires it, but changes must be recorded with reasons. Milestones are not promises that every stage will take equal time.

| Milestone | Name | Main outcome | Completion gate |
|---|---|---|---|
| M0 | Runnable foundation | Visible app/sandbox, Rust core linked, commands, tests, CI | Clean, reproducible launch and test |
| M0.1 | Native integration feasibility | Early Windows TSF and Linux integration experiments; Rust/native boundary decision | Risks and a practical first vertical slice documented |
| M1 | Engine contract and sandbox | Defined event/state/API contract and inspectable interaction | Contract tests and sandbox behavior pass |
| M2 | Unicode and composition foundation | Correct, documented Unicode/composition/editing behavior | Golden regression suite passes |
| M3 | Phonetic input v1 | Useful phonetic typing and candidate selection | Reviewed corpus evaluation and latency baseline |
| M4 | Fixed layouts | Versioned layout format and first validated layout(s) | Mapping and editing tests pass; compatibility documented |
| M5 | Settings foundation | Persistent configuration and working settings UI | Settings persist, validate, reset, and affect behavior |
| M6 | Dictionary and correction | Completion and spelling suggestions; conservative correction option if justified | Evaluation set and quality criteria pass |
| M7 | First native IME — Windows | Real system input method works end-to-end on Windows | Tested in multiple real applications on Windows |
| M8 | Linux integration | Native input integration and package for defined Linux configuration | Real Linux end-to-end test and support statement |
| M9 | Customization and usability | User-facing configuration, candidate settings, supported themes/layout tooling | Usability/accessibility checks and regression pass |
| M10 | Contextual prediction | Optional offline next-word prediction with reproducible evaluation | Meets quality/latency/memory thresholds or is deferred |
| M11 | Hardening and cross-platform regression | Reliability, privacy, performance, accessibility, security/license review | Release-blocking issues resolved or documented |
| M12 | Packaging and release candidate | Installers/packages, upgrade/uninstall checks, release notes | Clean install and release-candidate verification |
| M13 | v1.0.0 release decision | Feature freeze, verified artifacts, final support matrix and tagged release | Every claimed platform meets its release gate |

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

- There is a recorded Windows TSF feasibility result, with real test evidence or a clearly documented blocker.
- Linux feasibility and access are classified as verified, unverified, or blocked.
- The Rust/native boundary is documented and justified.
- The roadmap has an explicit platform strategy.
- The project does not falsely claim that all platforms are already feasible or verified.

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

- The event/state contract is documented and version-controlled.
- Tests cover normal input, candidate selection, commit, cancellation, reset, and error handling.
- The sandbox visibly demonstrates the contract.
- The app remains runnable.
- No adapter independently invents conflicting composition semantics.

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
- Defer complex visual editing or advanced personalization if it threatens core correctness or release readiness.

### Acceptance criteria

- Supported settings persist and actually affect behavior.
- Invalid imports are rejected safely and do not corrupt existing settings.
- Keyboard-only navigation and basic accessibility checks are documented.
- Customization changes do not break engine, Unicode, or native integration tests.
- The release does not claim settings that are only decorative or unsupported on a platform.

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
- If quality or performance fails the agreed thresholds, mark prediction experimental or defer it; do not hold v1.0.0 hostage to a weak model.

### Acceptance criteria

- Evaluation can be reproduced from documented steps.
- Quality and performance thresholds are set and results are recorded.
- Prediction meets the agreed thresholds or is explicitly deferred/experimental.
- Ordinary typing remains responsive when prediction is disabled and when it is enabled.
- Privacy implications and model/resource licensing are documented.

## M11 — Hardening and Cross-Platform Regression

**Purpose:** Turn a feature-complete prototype into reliable software.

### Required work

- Expand automated regression coverage for Unicode, composition, layouts, phonetic input, settings, dictionary assistance, and platform adapters.
- Add fuzzing/property-style tests where practical for parsers, layout files, configuration, and event sequences.
- Measure keystroke-to-result latency, candidate latency, startup, memory, idle CPU/resource use, and large-resource behavior on reference hardware.
- Test crash recovery and interrupted composition.
- Review privacy behavior, logs, diagnostics, telemetry (if any), and storage.
- Review dependency licenses, dictionary/model provenance, packaging dependencies, and third-party notices.
- Review accessibility and keyboard navigation.
- Triage bugs by severity and record reproducible steps.
- Run the same core regression suite on available environments; clearly identify unavailable platform tests.
- Do not use unmeasured performance claims.

### Acceptance criteria

- Release-blocking correctness and data-loss issues are fixed or release is blocked.
- Core regression tests pass.
- Performance results and test hardware/software are documented.
- Privacy and license reviews are recorded.
- Known limitations and unverified platforms are visible.
- No platform is marked verified solely because the shared Rust core passes.

## M12 — Packaging and Release Candidate

**Purpose:** Verify that users can install, use, upgrade, and remove the software.

### Required work

- Create platform-specific distribution artifacts for platforms that have passed their native integration gates.
- Test clean install, first run, enabling the input method, normal use, upgrade, and uninstall.
- Verify required language resources and configuration are included or downloaded only through an explicit documented flow.
- Document prerequisites and supported OS versions/configurations.
- Prepare release notes, user documentation, troubleshooting, license notices, and privacy statement.
- Freeze feature additions while resolving release blockers.
- Prepare a capability matrix for every platform and feature.
- Make an explicit macOS scope decision based on whether real Mac testing is available.
- Do not ship an artifact as production-ready solely because it compiles.

### Acceptance criteria

- Clean installation and uninstall are verified for every claimed release platform.
- Upgrade behavior is tested or explicitly documented as not supported for the first release.
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
| Windows — exact version to record | Unknown | Unknown | Unknown | Unknown | Not verified until tested |
| Linux — distribution/session to define | Unknown | Unknown | Unknown | Unknown | Environment must be confirmed |
| macOS — version to define | Unknown | Unknown | Unknown | Unknown | Requires real Mac access |

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
