# Contributing to BornoMala (বর্ণমালা)

Thank you for your interest in **BornoMala**!

BornoMala is an open-source effort to build a unified, lightning-fast, offline-first, and privacy-respecting Bangla input platform for desktop systems.

---

## 🧭 Project Status: Focused Roadmap Phase

BornoMala is currently in an **active, disciplined foundational development phase** led by the project founder ([Ashrafuzzaman Khalid](https://github.com/asmkhalid111)). 

Development strictly follows an engineering blueprint defined in [`ROADMAP.md`](./ROADMAP.md) and governed by [`AGENTS.md`](./AGENTS.md). Every milestone is backed by automated tests, verification reports, and strict platform contracts to ensure architectural coherence and zero regressions across our test suite.

To allow the author to finish building out this core architecture and see the foundational vision through to completion without diverging directions, we ask contributors to observe the collaboration guidelines below.

---

## 🤝 How to Participate

### 1. Forking & Independent Exploration (Fully Encouraged!)
You are free and warmly encouraged to **fork this repository**, experiment with your own ideas, add layouts, or build custom extensions under the [MIT / Apache 2.0](./README.md#license) dual licenses. 

If you discover exciting optimizations or alternative approaches in your fork, feel free to share your learnings with us!

### 2. Issues & Bug Reports
- **Bug Reports:** If you discover a rendering glitch, transliteration edge-case, or keyboard navigation defect, please open a GitHub Issue with clear reproduction steps (including your Windows build, host application, and typed keystrokes).
- **Feature Requests / Discussions:** If you have ideas for future milestones, open a discussion or issue so it can be cataloged for upcoming planning passes.

### 3. Direct Pull Requests & Code Contributions
To ensure your time is respected and efforts are not wasted on unmergeable changes:

1. **Discuss Before You Build:**
   - **Please do not submit large, unsolicited pull requests** or major architectural refactors without prior discussion.
   - If you want to contribute, please reach out first by opening a GitHub Issue/Discussion or emailing **`asm.khalid111@gmail.com`**.
2. **Align with the Master Roadmap:**
   - Review [`ROADMAP.md`](./ROADMAP.md) and explain how your proposal aligns with the current or upcoming milestone.
   - Show how your change respects BornoMala’s core principles:
     - **Offline & Private by Default:** Zero telemetry, zero network dependencies, zero background keystroke logging.
     - **Zero Non-System Runtime Dependencies:** Keep the Rust core pure and native adapters thin.
     - **Sub-Millisecond Keystroke Latency:** Maintain zero-overhead state transactions.
     - **Strict Canonical Normalization:** Every emitted character must adhere to Unicode NFC and joiner safety.
3. **Keep the Test Suite 100% Green:**
   - Every pull request must compile cleanly, introduce automated tests for new functionality, and pass the existing 229 automated tests:
     ```powershell
     # Core Rust engine test suite (221 tests)
     cargo test -p bornomala-core

     # Desktop persistence suite (8 tests)
     cargo run -p bornomala-desktop --bin persistence_suite

     # Native Windows TSF prototype build & test harness
     powershell -ExecutionPolicy Bypass -File "native/borno-tsf/build.ps1"
     ```

---

## 🛡️ Direct Commit & Collaborator Policy

Direct write/commit access to the primary repository is granted on an **invitation-only basis**. 

Collaborators are invited based on:
1. Prior communication and alignment with the lead author via email or discussion.
2. Demonstrated familiarity with `ROADMAP.md` and the existing Rust C-ABI / Windows TSF architecture.
3. A track record of high-quality, verified contributions that adhere to the project's evidence-based standards.

---

## 🌟 Future Horizon

Once the foundational milestones (through v1.0.0) are fully realized and stabilized, BornoMala will transition into a broader community-driven governance model with open RFCs, platform maintainers (e.g., Linux IBus/Fcitx5 and macOS), and multi-contributor working groups.

Until then, thank you for respecting this build phase and supporting the creation of an exceptional Bangla input platform!
