# AGENTS.md — Persistent Rules for AI Coding Agents

> **Project:** Bangla Input Platform  
> **Master Source of Truth:** `ROADMAP.md`  
> **Operating Environment:** Windows (x64) with pwsh

This file defines non-negotiable operational rules for all AI coding agents working on this codebase.

---

## 1. Scope Control & Roadmap Governance

1. **One Milestone at a Time:** Work ONLY on the currently approved milestone (e.g. `M0`). Do not start or implement future milestones (`M0.1`, `M1`, etc.) until the current milestone is verified and its completion report is approved.
2. **Never Speculate or Anticipate:** Do not create detailed implementation files for future milestones in advance. Each milestone instruction file must be created immediately before that milestone, informed by real evidence from prior work.
3. **No Decorative Placeholders:** Do not introduce mock or decorative components that pretend to be functional. All state, events, and UI interactions must connect to real code in the core engine.
4. **Preserve Existing Files:** Never delete or overwrite working code, user-created files, or master planning documents without clear technical justification and developer consent.

---

## 2. Evidence-Based Development & Reporting

1. **Strict Truth in Status:** Never claim a tool, test, build, or platform works unless you have executed the command and verified the actual output.
2. **Explicit Verification States:**
   - `Not started`
   - `In progress`
   - `Blocked`
   - `Implemented — not verified`
   - `Verified` (Only when executable evidence exists)
   - `Deferred`
   - `Out of scope`
3. **Honest Platform Support:**
   - Windows is the primary development environment.
   - Linux and macOS cannot be claimed as verified without real end-to-end execution on those systems.
   - If a platform is unverified or blocked, explicitly report it as such.

---

## 3. Architecture Boundaries & Dependencies

1. **Shared Rust Core:** Reusable input and language logic resides in the Rust core (`crates/borno-core` or similar), decoupled from any specific GUI or OS adapter.
2. **Separation of Concerns:** Keep platform adapters thin and engine interfaces clean.
3. **No Unjustified Dependencies:** Do not add external crates or npm packages without explaining the technical rationale.
4. **Offline & Privacy by Default:** Never introduce mandatory cloud dependencies, network calls for standard typing, or background keystroke logging.

---

## 4. Testing & Checkpoint Integrity

1. **Every Milestone Keeps the App Runnable:** A milestone is only complete if the sandbox application compiles, launches, and can be interactively checked.
2. **Automated Baseline:** Every milestone must include automated tests that run locally via reproducible commands and can run in CI.
3. **Regression Discipline:** All previous tests must continue to pass before declaring a milestone complete.
4. **Completion Report:** Every milestone must conclude with the standard completion report documenting files changed, architectural decisions, exact commands, test results, manual verification, known issues, acceptance criteria evidence, and next steps.
