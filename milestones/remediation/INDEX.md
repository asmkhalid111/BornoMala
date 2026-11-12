# BornoMala — Remediation Program Directory

> **Master Index:** This directory contains all authoritative remediation reviews, work package specifications, and execution completion reports for the BornoMala desktop platform.

---

## Structure

```
milestones/remediation/
├── INDEX.md                                      # This master index
├── FINAL-IMPLEMENTATION-READINESS-REVIEW.md       # Final baseline review across M1-M8 & architecture
├── WP-01-linux-privacy-and-build/                 # First remediation phase (Closed)
│   ├── SPECIFICATION.md                          # Scope, diffs, and regression plan for WP-01
│   ├── COMPLETION-REPORT.md                      # Post-implementation validation report
│   └── WSL2-VERIFICATION-REPORT.md               # End-to-end WSL2/WSLg Linux build & GUI verification
├── WP-02-candidate-integrity/                     # Second remediation phase (Closed)
│   ├── SPECIFICATION.md                          # Scope, diffs, and test realignment plan for WP-02
│   ├── IMPLEMENTATION-REPORT.md                  # Implementation evidence and invariant audit
│   └── COMPLETION-REPORT.md                      # Post-implementation closure report
├── WP-03-cursor-arithmetic/                        # Third remediation phase (Closed)
│   ├── SPECIFICATION.md                          # Cursor arithmetic & UTF-8 synchronization spec
│   ├── IMPLEMENTATION-REPORT.md                  # Implementation evidence and invariant audit
│   └── COMPLETION-REPORT.md                      # Post-implementation closure report
├── WP-04-c-abi-header-unification/                 # Fourth remediation phase (Closed)
│   ├── SPECIFICATION.md                          # C-ABI header unification specification
│   └── COMPLETION-REPORT.md                      # Post-implementation closure report
├── WP-05-desktop-app-identifier-migration/         # Fifth remediation phase (Closed — Committed Locally)
│   ├── SPECIFICATION.md                          # Identifier migration & persistence unification spec
│   └── COMPLETION-REPORT.md                      # Post-implementation validation report
├── WP-06-licensing-compliance/                     # Sixth remediation phase (Closed w/ Open RV-11 Risk)
│   └── SPECIFICATION.md                          # CC BY 4.0 licensing compliance & notices spec
└── FINAL-REMEDIATION-CLOSURE-REPORT.md             # Consolidated closure for WP-05–WP-I, Selectability & Cursor Audit
```

---

## Work Packages Status

| Work Package ID | Title | Scope | Status | Date |
|---|---|---|:---:|:---:|
| **Review** | [Final Implementation-Readiness Review](FINAL-IMPLEMENTATION-READINESS-REVIEW.md) | Baseline audit reconciliation & risk assessment | **Complete** | 2026-10-26 |
| **WP-01** | [Linux Fcitx 5 Privacy & Build Integrity](WP-01-linux-privacy-and-build/WSL2-VERIFICATION-REPORT.md) | Keystroke log redaction & multi-tier CMake path discovery | **Closed** | 2026-10-26 |
| **WP-02** | [Candidate Integrity & Contract Test Alignment](WP-02-candidate-integrity/COMPLETION-REPORT.md) | Remove `candidates.rs` fixtures, realign contract and config tests | **Closed (w/ Limitations)** | 2026-10-26 |
| **WP-03** | [Unicode In-Composition Cursor Arithmetic](WP-03-cursor-arithmetic/COMPLETION-REPORT.md) | Correct `lib.rs:1316` scalar vs byte dimension mismatch | **Closed (w/ Limitations)** | 2026-10-26 |
| **WP-04** | [C-ABI Header Unification](WP-04-c-abi-header-unification/COMPLETION-REPORT.md) | Synchronize `borno.h` with `bornomala.h` | **Closed** | 2026-11-03 |
| **WP-05** | [Desktop App Identifier & Configuration Migration](WP-05-desktop-app-identifier-migration/COMPLETION-REPORT.md) | Unify `com.bornomala.desktop` across Tauri and native adapters | **Closed (Committed `223b7a3`, Local)** | 2026-11-06 |
| **WP-06** | [Licensing Compliance & Notices](WP-06-licensing-compliance/SPECIFICATION.md) | Add `THIRD-PARTY-NOTICES.md`, desktop About & Notices tab, Fcitx/TSF packaging rules | **Closed (w/ Open `RV-11` Risk)** | 2026-11-12 |
| **WP-A – WP-I & Selectability** | [Final Remediation Closure Report](FINAL-REMEDIATION-CLOSURE-REPORT.md) | Settings draft lifecycle (`WP-A`), ellipsis (`WP-B`), digit collision (`WP-C`), Windows GNU manifest (`WP-D`–`WP-I`), UI selectability, canonical `Borno-Sandbox.exe` reconciliation, deferred cursor audit | **Closed (w/ Deferred `RV-08.1` / Linux-macOS Scope)** | 2026-11-12 |

---

## Governance Rules
1. **Never mutate unapproved code:** Every work package requires an explicit specification before implementation.
2. **Every work package records a completion report:** Reports must include exact diffs, executed commands, and actual tool output.
3. **Continuous regression passing:** All existing workspace tests must pass at every checkpoint.
