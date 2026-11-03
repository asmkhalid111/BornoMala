# Revision Programme — R0 (2026-10-23)

> **Programme ID:** R0
> **Status:** Proposed — awaiting developer approval
> **Single-run conductor:** [`EXECUTE-ALL.md`](EXECUTE-ALL.md) — use this to drive every revision in order.
> **Source audit:** [`milestones/AUDIT-2026-10-23.md`](../AUDIT-2026-10-23.md)
> **Governing documents:** [`AGENTS.md`](../../AGENTS.md), [`ROADMAP.md`](../../ROADMAP.md)
> **Rule:** Do not execute any revision file until the developer approves that specific file. Do not start RV-N+1 while RV-N is unverified.

---

## 1. Purpose

This programme converts the 2026-10-23 audit into **executable, independently verifiable revisions**. Every defect that was found is assigned to exactly one revision file, with a named owner-level acceptance test and an evidence requirement.

It exists because the project is strong in architecture and weak in verification: eight release-blocking defects remain, several "Verified" labels are not backed by evidence, and the desktop shell will not survive feature growth in its current shape.

## 2. What this programme is not

- It is **not** a rewrite. The Rust core survives; the adapters survive; the lexicon pipeline survives.
- It is **not** permission to start M10, M12, or M13. §9 defines explicit no-go gates.
- It is **not** a substitute for `ROADMAP.md`. After each revision, the roadmap must be updated from evidence.
- It is **not** a batch job. One file, one work package at a time.

## 3. Document map

| File | Topic | Phase | Blocking? |
|---|---|:---:|:---:|
| [`RV-01-core-engine-correctness.md`](RV-01-core-engine-correctness.md) | Remove fabricated candidates, fix editing model, configuration semantics, dictionary load | 0 | **Yes** |
| [`RV-01.1-candidate-integrity.md`](RV-01.1-candidate-integrity.md) | Delete hard-coded candidate answers; re-baseline tests on rules | 0 | **Yes** |
| [`RV-01.2-cursor-editing-safety.md`](RV-01.2-cursor-editing-safety.md) | Byte/char unit model, property tests, paste correctness | 0 | **Yes** |
| [`RV-02-contract-and-ffi.md`](RV-02-contract-and-ffi.md) | C ABI v2, header single source, allocator safety, thread and panic contracts, codegen | 0 | **Yes** |
| [`RV-03-configuration-unification.md`](RV-03-configuration-unification.md) | One canonical config path, migration, safe reload, one parser | 0 | **Yes** |
| [`RV-04-windows-tsf-safety.md`](RV-04-windows-tsf-safety.md) | Composition lifecycle, key policy, registration hygiene, DPI, display attributes | 0 | **Yes** |
| [`RV-04.1-composition-lifecycle.md`](RV-04.1-composition-lifecycle.md) | End compositions correctly; remove use-after-free paths | 0 | **Yes** |
| [`RV-05-linux-fcitx-reproducibility.md`](RV-05-linux-fcitx-reproducibility.md) | Portable build, real tests, mode sync, key semantics, privacy, packaging | 0 | **Yes** |
| [`RV-06-verification-integrity.md`](RV-06-verification-integrity.md) | Remove self-certifying tests, honest benchmarks, CI matrix, evidence system, doc checks | 0 (continuous) | **Yes** |
| [`RV-07-desktop-architecture.md`](RV-07-desktop-architecture.md) | Engine modularisation, typed IPC, state layer, backend hardening | 1 | No |
| [`RV-08-ui-correctness-and-accessibility.md`](RV-08-ui-correctness-and-accessibility.md) | Design tokens, contrast, dialogs, CSP, reduced motion, ARIA | 1 | No |
| [`RV-08.1-ime-and-focus.md`](RV-08.1-ime-and-focus.md) | Browser/WebView composition, paste, focus retention | 1 | No |
| [`RV-09-governance-and-documentation.md`](RV-09-governance-and-documentation.md) | Roadmap truth, status machine, spec closure, missing docs, duplicate roadmap | 0 (continuous) | **Yes** |
| [`RV-10-licensing-and-release-hygiene.md`](RV-10-licensing-and-release-hygiene.md) | CC BY attribution, provenance, NOTICE, binary hygiene | 0 | **Yes** |
| [`RV-11-release-engineering.md`](RV-11-release-engineering.md) | Installers, signing, upgrade/uninstall, config migration, support matrix | 2 | No (gated) |
| [`RV-13-hardening-and-performance.md`](RV-13-hardening-and-performance.md) | Fuzzing, perf budgets, crash recovery, privacy gate, DPI, threat model | 2 | No (gated) |
| [`RV-12-future-horizon.md`](RV-12-future-horizon.md) | macOS decision, prediction gate, beta, updates, mobile, i18n | 3 | No |

## 4. Phases, entry and exit gates

### Phase 0 — Truth & Safety (blocking)

**Entry:** developer approves `RV-01` and `RV-01.1`.
**Exit:** every P0 in the audit is closed; `cargo test --workspace` and the persistence suite are green; the Windows E2E matrix is recorded; the Linux build is reproducible in CI; documentation numbers match reality.
**May run in parallel:** RV-01.1, RV-09, RV-10 (independent files); RV-06 starts day one and closes at the end of Phase 0.

### Phase 1 — Architecture & UX

**Entry:** Phase 0 exit gate signed.
**Exit:** engine split with unchanged public API; generated contract; typed IPC + state store; no UI component over 400 lines; accessibility and contrast evidence recorded; IME/paste/focus tests green.

### Phase 2 — Release engineering & hardening

**Entry:** Phase 1 exit gate + RV-04/RV-05 E2E evidence.
**Exit:** clean install, upgrade, uninstall on Windows; Linux package installs; signing; config migration; support matrix with evidence links; RV-13 fuzz, performance, privacy, crash-recovery and DPI gates green.

### Phase 3 — Horizon

**Entry:** Phase 2 exit gate.
**Exit:** explicit macOS decision, prediction go/no-go, beta results, update channel decision.

## 5. Global rules for AI agents

These are non-negotiable and override convenience:

1. **Read before writing.** Read `AGENTS.md`, `ROADMAP.md`, the audit, the parent RV, and every file you intend to touch. Do not edit from memory.
2. **One work package per commit.** Format: `fix(rv-01.1): remove literal candidate fixtures`. No mixed commits.
3. **Run the gate after each package.** If the gate fails, stop and fix; never continue on red.
4. **No behaviour change without a test.** Every bug fix gets a regression test that fails before the fix.
5. **No public API/ABI change without all three consumers.** Rust, C header, TypeScript, plus both native adapters. A rename that compiles but breaks the UI is a defect.
6. **No new dependency without a written rationale** (what it replaces, licence, maintenance status, binary size impact). `AGENTS.md` §3.3.
7. **Never delete a passing test to make a change look green.** If a test must change, state in the commit message which assertion was wrong and why.
8. **No decorative verification.** No unconditional `PASS` prints, no test that rewrites its own fixtures, no timing assertion in a debug build.
9. **Preserve the runnable checkpoint.** After every work package the sandbox must still build and launch.
10. **Stop conditions win.** If a required change needs a product decision (e.g. canonical identifier, default orientation), stop and ask; do not guess.
11. **Do not touch future revisions.** Implementing RV-07 work inside RV-01 is a scope violation.
12. **Record the environment.** Every evidence file states OS build, toolchain versions, and the exact commit.

## 6. Definition of Verified

A claim may be marked `Verified` only when all six exist:

- [ ] The exact command or manual procedure is recorded.
- [ ] The verbatim result (trimmed) is recorded.
- [ ] The environment (OS, toolchain, app version) is recorded.
- [ ] A negative check was performed (deliberately break it; the test must fail).
- [ ] Manual behaviour was observed where the claim is user-facing.
- [ ] Known limitations are listed.

Anything less is `Implemented — not verified`.

## 7. Evidence standard

Every revision writes `docs/evidence/<RV-ID>/<YYYY-MM-DD>-<topic>.md`:

```md
# Evidence — RV-XX.Y — <date>
- Commit:
- Environment:
- Commands:
- Output (verbatim, trimmed):
- Manual steps:
- Negative check:
- Result: PASS | FAIL
- Known limitations:
```

The roadmap row for the revision links to that file. A roadmap claim without a link is not a claim.

## 8. Status board

Update this table in the same commit as each completion report. Do not pre-fill.

| ID | Status | Commit | Evidence |
|---|---|---|---|
| RV-01 | Not started | — | — |
| RV-01.1 | Not started | — | — |
| RV-01.2 | Not started | — | — |
| RV-02 | Not started | — | — |
| RV-03 | Not started | — | — |
| RV-04 | Not started | — | — |
| RV-04.1 | Not started | — | — |
| RV-05 | Not started | — | — |
| RV-06 | Not started | — | — |
| RV-07 | Not started | — | — |
| RV-08 | Not started | — | — |
| RV-08.1 | Not started | — | — |
| RV-09 | Not started | — | — |
| RV-10 | Not started | — | — |
| RV-11 | Blocked by Phase 0/1 | — | — |
| RV-13 | Blocked by Phase 0/1 | — | — |
| RV-12 | Blocked by Phase 2 | — | — |

Allowed values: `Not started`, `In progress`, `Blocked`, `Implemented — not verified`, `Verified`, `Deferred`, `Out of scope`.

## 9. No-go decisions for the next phase

These are explicit refusals, not preferences. Revisit only with written evidence.

| Decision | No-go until | Reason |
|---|---|---|
| **M10 — Contextual prediction** | RV-07 and RV-08 complete; RV-01.2 property tests green | Prediction on an untested editing path multiplies risk and cannot be evaluated honestly. |
| **M12 — Packaging / release candidate** | RV-04 and RV-05 E2E evidence; RV-03 migration tests; RV-11 approved; NOTICE shipped | Shipping an installer around a broken config path and unsafe composition is a liability. |
| **M13 — v1.0.0** | RV-11 exit; support matrix with evidence links; macOS decision recorded | The release must not claim platforms it cannot prove. |
| **New feature work of any kind** | Phase 0 exit gate | Feature work on fabricated candidates and untested UI compounds debt. |
| **Second config path “for compatibility”** | Never | One canonical path is a hard invariant (RV-03). |
| **GUI tests as blocking CI before they are stable** | RV-06 exit | Flaky gates get disabled, which is worse than no gate. |

## 10. Execution protocol

1. Developer approves one RV file.
2. Agent inspects the repository and runs the baseline gate.
3. Agent implements work package 1, runs the gate, commits.
4. Repeat for each work package.
5. Agent writes the completion report and evidence files.
6. Agent updates `ROADMAP.md` status from evidence and links the evidence.
7. Developer reviews; only then is the next RV approved.

## 11. Rollback protocol

- Every work package must be revertible by a single `git revert`.
- If a revision cannot meet its acceptance criteria, revert to the last green checkpoint, mark the revision `Blocked`, and write the blocking condition in the report.
- Never leave `main` with a failing gate. Work on a branch named `rv/<id>-<topic>`.

## 12. Estimated effort (solo developer, part-time)

| Phase | Estimate |
|---|---|
| Phase 0 (RV-01 … RV-06, RV-09, RV-10) | 12–18 working days |
| Phase 1 (RV-07, RV-08) | 10–15 working days |
| Phase 2 (RV-11, RV-13) | 9–14 working days |
| Phase 3 (RV-12) | 3–5 working days + external dependencies |

Estimates are planning aids, not commitments.

---

*Proposed by the audit. Awaiting developer approval before execution.*
