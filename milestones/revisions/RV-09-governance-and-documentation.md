# RV-09 — Governance, Documentation & Evidence Discipline

> **Revision ID:** RV-09
> **Status:** Proposed — awaiting developer approval
> **Priority:** P0 (trust infrastructure)
> **Estimated effort:** 2 days + 15 minutes per later revision
> **Dependencies:** none (starts day one)
> **Blocks:** every future `Verified` claim

---

## 1. Why this revision exists

The audit found the project’s documents contradict each other and the code:

- Test counts: **142 / 217 / 229 / 238 / 246** across five documents; actual **232 + 8**.
- `ROADMAP.md` says M9 is “In progress (Next), 0 %” while `M9-completion-report.md` says “Complete & Formally Verified”.
- `ROADMAP.md` says M7 is Verified and tested in four applications; the M7 report says Levels 5–6 are `PENDING`.
- Eight milestone specs still say “awaiting developer approval” although the milestones are marked Verified.
- Two documents both declare themselves the master roadmap (`ROADMAP.md`, `Bangla_Input_Platform_Master_Roadmap.md`).
- Required docs from roadmap §6.5 are missing: `testing.md`, `platform-support.md`, `privacy.md`, `release-process.md`.
- `docs/architecture.md` is 43 lines and names the pre-rename crate.

This is not cosmetic. `AGENTS.md` names `ROADMAP.md` as the single source of truth; when it disagrees with reality, every downstream agent inherits the error.

---

## 2. Current-state inspection

Read every document that makes a factual claim about status:

- `ROADMAP.md` (sections 4, 4.1, 4.2, 6.5, 9)
- `README.md`, `CONTRIBUTING.md`
- `milestones/M0…M9*.md`, `milestones/AUDIT-2026-10-23.md`
- `docs/*.md`, `docs/lexicon_provenance.json`
- `AGENTS.md`

Build a **claim register**: every sentence with a number, a status, or a “verified” word, with its source line and the evidence (if any).

---

## 3. Scope

### Goals

1. One roadmap, one status per milestone, one evidence link per claim.
2. A formal status transition machine that cannot silently skip states.
3. Milestone specs closed when their milestone closes.
4. The five missing documents created (or explicitly deferred with a reason).
5. `docs/architecture.md` rewritten to match the code.
6. A decision log (ADR-lite) so future agents do not re-litigate settled choices.
7. Automated consistency checks (shared with RV-06.6).

### Non-goals

- Not a rewrite of the roadmap’s product direction.
- Not a marketing README.
- Not a changelog of every commit.

### Explicit no-go

- **No** “Verified” without a linked evidence file.
- **No** two master documents.
- **No** status percentages without a defined calculation.
- **No** spec file left at “Proposed” after its milestone closes.
- **No** historical numbers silently rewritten; correct the current claim and mark the old one as historical.

---

## 4. Work packages

### WP-9.1 — Archive the duplicate roadmap

1. Move `Bangla_Input_Platform_Master_Roadmap.md` to `docs/archive/master-roadmap-2026-10-17.md`.
2. Add a header: “Archived — superseded by ROADMAP.md. Do not edit.”
3. Add a note in `ROADMAP.md` pointing to the archive.
4. Add a CI check that fails if a second file at repo root contains “master planning document”.

Gate: exactly one root roadmap.

### WP-9.2 — Status machine

Define in `AGENTS.md` (and reference from the roadmap):

```
Not started → In progress → Implemented — not verified → Verified
                     ↘ Blocked            ↘ Deferred
```

Rules:

- `Verified` requires: evidence file, green gate command, manual observation where user-facing, known limitations.
- A later change that invalidates a claim must move the status back to `Implemented — not verified` in the same commit.
- `Blocked` requires a written blocking condition and a date.
- Percentages are computed as “acceptance criteria met / total”, never estimated.

Gate: `AGENTS.md` contains the machine; the roadmap uses only its values.

### WP-9.3 — Roadmap truth pass

For every milestone M0–M9:

1. Set the status from evidence, not from intention.
2. Link the evidence file (or mark the claim unverified).
3. Correct M7 → `Implemented — not verified`; M8 → `Partially verified (WSLg/X11 only)`; M9 → `In progress` until its backlog items are closed or deferred.
4. Replace the M11/M12/M13 percentages (10/15/5) with computed values or “Not started”.
5. Fix stale paths (`crates/borno-core` → `crates/bornomala-core`).
6. Add RV-01…RV-12 rows.

Gate: the docs consistency script (RV-06.6) passes.

### WP-9.4 — Close the specs

For each `milestones/Mx.md`:

- Add a final `## Status` section: `Closed — superseded by <report>` or `Closed — implemented in commits <range>`.
- Change the header status line from “Proposed / awaiting approval” to the historical truth (e.g. “Approved and implemented 2026-10-18”).
- Do not delete the original content; preserve the audit trail.

Gate: no milestone spec reads as “not yet approved” while its milestone is complete.

### WP-9.5 — Missing documents

Create, with real content (not placeholders):

| File | Minimum content |
|---|---|
| `docs/testing.md` | commands per layer, expected counts, debug vs release, manual matrices, how to add a test |
| `docs/platform-support.md` | Windows/Linux/macOS matrix with evidence links and limitations |
| `docs/privacy.md` | no telemetry, no keystroke logs, what diagnostics contain, network behaviour, how to verify |
| `docs/release-process.md` | versioning, building, signing, packaging, migration, rollback, support policy |
| `docs/definition-of-verified.md` | the six-point checklist from the programme |
| `docs/architecture.md` | rewrite: crate boundaries, ABI, adapters, config, data flow, dependency rules |

Gate: every file exists, is linked from README/roadmap, and contains no TBD markers.

### WP-9.6 — Decision log

Create `docs/decisions/` with one file per settled decision:

- `0001-canonical-config-path.md` (com.bornomala.desktop)
- `0002-candidate-slot-0-immutability.md`
- `0003-avro-compatibility-scope.md`
- `0004-abi-versioning.md`
- `0005-default-candidate-orientation.md`
- `0006-macos-scope.md` (pending RV-12)

Each: context, decision, consequences, alternatives rejected, date.

Gate: every recurring question in the audit has a decision record.

### WP-9.7 — Doc-code consistency automation

Reuse the script from RV-06.6. Add:

- milestone status match,
- evidence link existence,
- test-count match,
- link check,
- archived-roadmap check,
- `crates/borno-core` check.

Gate: deliberately break one → script fails.

### WP-9.8 — Per-revision ritual

Add to the programme: after each revision, update the status board, the roadmap row, the evidence link, and the changelog section of the completion report. Fifteen minutes, every time.

---

## 5. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | One root roadmap | tree + CI |
| AC-2 | Status machine documented and used | AGENTS + roadmap |
| AC-3 | Every milestone status matches its report | script |
| AC-4 | Every `Verified` has an evidence link | script |
| AC-5 | No spec says “awaiting approval” after closure | review |
| AC-6 | Five missing docs exist with real content | tree |
| AC-7 | Architecture doc matches the code | review |
| AC-8 | Six ADRs exist | tree |
| AC-9 | Consistency script fails on a deliberate break | canary |

---

## 6. Testing plan

```powershell
pwsh -File scripts/check_docs.ps1
```

Manual review: read `ROADMAP.md` top to bottom and check every status against the evidence folder.

Negative checks:

- Change a test count in `README.md` → script fails.
- Change `M7` to `Verified` without an evidence link → script fails.
- Add a second root “master roadmap” → script fails.

---

## 7. Runnable checkpoint

Documentation-only revision: the app must still build and launch, and no code file changes.

---

## 8. Evidence and reporting

`docs/evidence/RV-09/<date>-governance.md`: the claim register, the before/after status table, the script canary output.

---

## 9. Roadmap update

Self-referential: this revision updates the roadmap it audits. Record the change explicitly.

---

## 10. Stop conditions

- A status cannot be determined from evidence → mark `Implemented — not verified` and list what is missing; do not guess.
- The developer disagrees with a status → record the decision in the ADR log, do not silently override.
- The consistency script would require rewriting historical reports → limit it to current claims and mark old numbers historical.

---

## 11. Handoff

Every later revision now has a place to record truth. RV-06 owns the automation; RV-09 owns the content.
