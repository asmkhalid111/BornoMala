# RV-12 — Future Horizon (macOS, Prediction, Beta, Update, Mobile)

> **Revision ID:** RV-12
> **Status:** Proposed — awaiting developer approval; **planning only** until Phase 2 exits
> **Priority:** Low now, decisive later
> **Estimated effort:** 3–5 days of planning/feasibility + external dependencies
> **Dependencies:** RV-11 (release process), RV-06 (evidence), RV-09 (governance)
> **Blocks:** M10, M13

---

## 1. Why this revision exists

The roadmap still reads as if v1.0.0 will ship on Windows, Linux, and macOS, while:

- macOS has never been built, installed, or tested; there is no Mac.
- M10 (contextual prediction) is listed as a milestone but has no gate.
- There is no beta programme, no update channel, and no plan for a mobile keyboard.
- The support matrix is aspirational.

This revision is **decision work**: it turns vague future intentions into explicit go/no-go gates with thresholds and evidence. It does not implement features.

---

## 2. Current-state inspection

Read:

- `ROADMAP.md` §1.2, §4, §9, §M10, §M12, §M13
- `milestones/AUDIT-2026-10-23.md` §10
- `docs/platform-support.md` (created in RV-09)
- `docs/design-system-and-usability.md` §1 (three version domains)

Record the actual state: no Mac hardware, no tester pool, no update infrastructure, no prediction code.

---

## 3. Scope

### Goals

1. A recorded **macOS decision** with a concrete path or an explicit deferral.
2. A **prediction gate** with numeric thresholds and a reproducible evaluation plan.
3. A **beta programme** definition (recruitment, feedback, triage).
4. An **update channel** decision (manual download vs signed auto-update).
5. A **mobile** decision (out of scope for v1, future Android keyboard).
6. An **i18n** plan for the app UI (English/Bangla).
7. A **layout editor / personalization** decision with privacy constraints.

### Non-goals

- No implementation of prediction, macOS, mobile, or auto-update.
- No new product scope for v1.0.0.

### Explicit no-go

- **No** macOS feature in v1.0.0 without a real Mac and an InputMethodKit E2E test.
- **No** prediction integration before RV-07/RV-08 and an evaluation that beats a documented baseline.
- **No** auto-update before signing and a rollback plan.
- **No** telemetry as a condition of using the product.
- **No** cloud personalization.

---

## 4. Work packages

### WP-12.1 — macOS decision

Produce `docs/decisions/0006-macos-scope.md` with one of:

**Option A — defer (recommended until hardware exists):**
- v1.0.0 support matrix lists macOS as `Deferred — requires real Mac access`.
- README and release notes say so plainly.
- A feasibility note describes InputMethodKit requirements, Xcode toolchain, code signing/notarization, and the estimated 4–6 week effort.
- Revisit at M13.

**Option B — implement:**
- Requires: a real Mac (Apple Silicon preferred), Xcode, a developer account for notarization, and an InputMethodKit spike.
- Accept criteria: build, install, type in TextEdit and one browser, focus/cancel/backspace, candidate window, plus a signed/notarized package.
- Add `M8.6 — macOS InputMethodKit` as a real milestone with its own spec, not a promise.

Gate: a written decision, committed, with the evidence (or the lack of hardware) stated.

### WP-12.2 — Prediction gate (M10)

Define before any model work:

**Task:** next-word suggestion for Bangla, offline, optional.

**Baseline:** unigram frequency from the existing lexicon + the current candidate list.

**Evaluation set:** ≥5,000 held-out sentences from a licence-clean corpus; document split, preprocessing, and provenance. No training on the test split.

**Thresholds (proposed; developer may adjust before the run):**

| Metric | Threshold |
|---|---|
| Top-1 accuracy on held-out set | ≥ 25 % |
| Top-3 accuracy | ≥ 45 % |
| Keystroke→suggestion p99 | ≤ 5 ms (core) |
| Memory (model + index) | ≤ 20 MB |
| Cold start impact | ≤ 100 ms |
| Works offline | mandatory |
| No typed text leaves the machine | mandatory |

**Method:**

1. Python pipeline in `research/prediction/` with pinned dependencies and a lockfile.
2. Reproducible commands (`make eval`), dataset hashes, and a results file.
3. Ablation vs baseline.
4. Privacy review: no network, no logs.

**Decision:** if thresholds fail → mark prediction `Experimental` or `Deferred`; do **not** block v1.0.0.

Gate: `docs/decisions/0007-prediction-scope.md` with the measured numbers and the decision.

### WP-12.3 — Beta programme

Define:

- **Recruitment:** 5–10 Bangla typists across Windows 10/11 and Ubuntu; at least two using Bijoy/Probhat daily.
- **Build:** a signed (or clearly marked unsigned) beta with a visible version and a “Report a problem” link.
- **Feedback:** a template capturing app, OS, input method, steps, expected/actual, and **no typed content unless the tester chooses to include it**.
- **Triage:** severity rubric (P0 data loss/crash, P1 wrong text, P2 usability, P3 cosmetic); weekly review; every P0 blocks release.
- **Privacy:** no automatic telemetry; feedback is user-submitted.

Gate: `docs/beta-programme.md` + a triage log template.

### WP-12.4 — Update channel

Decide:

- **Manual download (recommended for v1):** release page + checksums + in-app “check for updates” that opens the page (no network call unless the user clicks).
- **Signed auto-update (future):** requires signing, a hosted manifest, delta or full package, rollback, and a kill switch. Add as `M12.5` only after signing exists.

Rules: no background network call by default; any check is explicit and documented in `docs/privacy.md`.

Gate: `docs/decisions/0008-update-channel.md`.

### WP-12.5 — Mobile decision

Record:

- Android keyboard is **out of scope for v1.0.0**.
- The Rust core is reusable, but the IME lifecycle (InputMethodService), candidate UI, and Play Store policy are a separate product effort.
- If pursued, it becomes a new milestone after v1.0.0 with its own threat model (keyboard apps are privacy-sensitive).

Gate: one paragraph in `docs/decisions/0009-mobile-scope.md`.

### WP-12.6 — i18n

1. Decide UI languages: English + Bangla at minimum.
2. Extract user-facing strings into a catalogue (`src/i18n/en.json`, `bn.json`).
3. No string concatenation for grammar; use ICU-style messages if plurals appear.
4. Language follows the OS by default, overridable in settings.
5. The Bangla UI must use the bundled Bengali font and correct `lang` attributes.

Gate: a plan and a first extraction; implementation scheduled after v1.0.0 if it risks the release.

### WP-12.7 — Layout editor / personalization

Decision:

- **v1.0.0:** no layout editor. Provide layout visualization (already exists) and config import/export.
- **Future:** a constrained editor with schema validation, diffable output, and no executable plug-ins. Any custom layout must pass the same validation as built-ins.
- **Personalization:** local only, opt-in, no cloud, no keystroke log.

Gate: `docs/decisions/0010-customization-scope.md`.

---

## 5. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | macOS decision recorded with hardware reality | ADR |
| AC-2 | Prediction thresholds + evaluation plan recorded | ADR |
| AC-3 | No prediction code merged before the plan | repo |
| AC-4 | Beta programme documented | doc |
| AC-5 | Update channel decided with privacy rules | ADR |
| AC-6 | Mobile scope recorded | ADR |
| AC-7 | i18n plan exists | doc |
| AC-8 | Customization scope recorded | ADR |
| AC-9 | Support matrix matches decisions | doc |

---

## 6. Testing plan

Planning revision: no code tests. Validate by review:

- Does every decision name a concrete gate and an owner?
- Does every threshold have a measurement method?
- Does every privacy claim have a verification step?

If any decision is implemented later, it must arrive with its own RV file and evidence.

---

## 7. Runnable checkpoint

No code change; the app must still build. Documentation-only.

---

## 8. Evidence and reporting

`docs/decisions/0006…0010` plus `docs/beta-programme.md` and `docs/evidence/RV-12/<date>-horizon.md`.

---

## 9. Roadmap update

- M10 becomes `Deferred until the prediction gate passes`.
- Add `M8.6 macOS` only if Option B is chosen.
- Add `M12.5 update channel` only if auto-update is chosen.
- M13 records the final macOS and mobile scope.

---

## 10. Stop conditions

- No Mac access → Option A only. Do not write macOS implementation files.
- Prediction evaluation cannot be licence-clean → defer prediction.
- Beta recruitment unavailable → do not claim external validation; state “developer-only testing”.

---

## 11. Handoff

After this revision, the roadmap’s future is explicit: what will be built, what will not, and what evidence would change the decision.
