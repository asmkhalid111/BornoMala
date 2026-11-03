# EXECUTE-ALL — Revision Programme Conductor

> **File role:** Single entry point that drives [`RV-01`](RV-01-core-engine-correctness.md) through [`RV-13`](RV-13-hardening-and-performance.md) in dependency order.
> **Status:** Proposed — awaiting developer approval.
> **Read first:** [`00-PROGRAMME.md`](00-PROGRAMME.md), [`AGENTS.md`](../../AGENTS.md), [`ROADMAP.md`](../../ROADMAP.md), [`AUDIT-2026-10-23.md`](../AUDIT-2026-10-23.md).
> **This file does not replace any revision file.** It only sequences them, records state, and decides when to continue, park, or stop.

---

## 0. The honest contract

### 0.1 What “run once” means here

A single run starts at the first unfinished work package and continues **without asking questions** until one of these is true:

1. every revision is `Verified`, or
2. no eligible work remains because all remaining gates need a human, external hardware, or an unanswered decision.

It must then write the blocker reports and `PROGRAMME-REPORT.md`, and stop. Re-running resumes from `EXECUTION-STATE.json` without redoing completed work.

### 0.2 What a single run cannot honestly do

These gates are **human-only or hardware-only** and must never be simulated:

| Gate | Why it cannot be automated | Where |
|---|---|---|
| Real typing in Word / Chrome / VS Code / Explorer | Requires interactive host apps and visual confirmation | RV-04 §5 |
| Windows focus-switch / crash stress | Requires a human or a fragile UI-automation rig | RV-04.1 §6 |
| Linux GUI typing (GTK/Gedit) | Requires a Linux desktop session (WSLg or VM) | RV-05 §7 |
| Screen-reader / keyboard-only pass | Requires Narrator or a human | RV-08 §6 |
| 100/125/150/200 % DPI | Requires a display or a VM with scaling | RV-13.5 |
| Clean-VM install/upgrade/uninstall + elevation | Requires a VM and administrator approval | RV-11 §6 |
| Code signing | Requires a certificate and private key | RV-11.3 |
| macOS build/install | Requires a real Mac | RV-12.1 |
| Product decisions | Requires the developer’s choice | §4 |

If the agent cannot satisfy a gate, it **parks** it (see §11). It must never mark it `Verified`, never write an evidence file claiming a run that did not happen, and never weaken the acceptance criterion to make it pass.

### 0.3 Definition of programme completion

The programme is complete only when **all** of the following are true:

- [ ] Every RV in §6 is `Verified` with a linked evidence file.
- [ ] `cargo test --workspace` is green; the persistence suite is green.
- [ ] `npm test` and `npm run build` are green.
- [ ] The Windows E2E matrix (RV-04 §5) is recorded.
- [ ] The Linux CI job (RV-05.7) is green, with GUI evidence where claimed.
- [ ] `scripts/check_docs.ps1` passes.
- [ ] `THIRD-PARTY-NOTICES.md` ships and is visible in the app.
- [ ] Windows installer and Linux package pass clean install/upgrade/uninstall.
- [ ] `PROGRAMME-REPORT.md` exists with the final support matrix and every evidence link.
- [ ] No revision is `In progress` or `Implemented — not verified`.

Anything short of this is a **parked run**, not a completed programme. Say so plainly in the report.

---

## 1. Invocation

### 1.1 The single prompt

Copy this into the coding agent verbatim:

> Execute the Borno revision programme in **AUTOPILOT** mode.
> Read `AGENTS.md`, `ROADMAP.md`, `milestones/AUDIT-2026-10-23.md`, `milestones/revisions/00-PROGRAMME.md`, `milestones/revisions/EXECUTE-ALL.md`, and the current `milestones/revisions/EXECUTION-STATE.json` (create it from §17 if missing).
> Follow §6 order and §8 run loop exactly. Apply the pre-answered decisions in §4; do not ask about anything already answered. Continue until no eligible work remains. Park human-only gates per §11 with the exact blocker template. Never mark a gate `Verified` without evidence. Commit one work package at a time on branch `revision-programme`. At the end, write `PROGRAMME-REPORT.md` and stop.

### 1.2 Modes

| Mode | Behaviour | Use |
|---|---|---|
| `AUTOPILOT` (default) | Parks human gates and continues with independent work; stops when nothing eligible remains | “Run once” |
| `ASSISTED` | Stops and asks at the first human gate or unanswered decision | Supervised |
| `SINGLE` | Executes exactly one RV passed as an argument, then stops | Fine control |
| `DRY-RUN` | Produces a plan and the preflight report only; no edits | First inspection |
| `RECHECK` | Re-runs every verification gate and evidence check without changing code | Before release |

Mode is stored in `EXECUTION-STATE.json` and may be changed by the developer between runs.

### 1.3 Hard rules that apply in every mode

- One work package = one commit.
- Run the gate after every work package; red means stop the package, not skip it.
- Never edit a file another active revision owns.
- Never rewrite a revision file to make it easier.
- Never fabricate evidence, test output, screenshots, or hardware access.

---

## 2. Pre-flight checklist (human, ~15 minutes)

Do this once before the first run.

- [ ] Commit or stash all current work; `git status` clean.
- [ ] Create the branch: `git switch -c revision-programme`.
- [ ] Back up `%APPDATA%\com.bornomala.sandbox\config.json` (migration will touch it).
- [ ] Confirm toolchain: `cargo --version`, `rustc --version`, `node -v`, `npm -v`.
- [ ] Confirm MinGW `g++.exe` is on `PATH` (for the TSF build).
- [ ] Confirm WSL2 with a Linux distro is installed (for RV-05), if you want it in the first run.
- [ ] Confirm access to a Windows VM for RV-11, if available.
- [ ] Fill the decisions in §4 (or accept the defaults).
- [ ] Decide whether the agent may push. Default: **no pushes**.
- [ ] Snapshot the machine or at least the registry key:
  `reg export "HKLM\SOFTWARE\Classes\CLSID\{8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}" ime-backup.reg`

If you skip the checklist, the first run may park early — that is safer than guessing.

---

## 3. Environment probe (agent, before any edit)

Run and record in `EXECUTION-STATE.json` under `environment`:

```powershell
git status --short
git branch --show-current
cargo --version
rustc --version
node --version
npm --version
(Get-Command g++.exe -ErrorAction SilentlyContinue).Source
wsl --status
Test-Path "$env:APPDATA\com.bornomala.sandbox\config.json"
Test-Path "$env:APPDATA\com.bornomala.desktop\config.json"
reg query "HKLM\SOFTWARE\Classes\CLSID\{8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}" /s
(Get-Command signtool.exe -ErrorAction SilentlyContinue).Source
npm ls --depth=0
```

Set capability flags:

```json
{
  "windows": true,
  "mingw": true,
  "wsl_linux": false,
  "clean_vm": false,
  "signing_cert": false,
  "mac": false,
  "ci_remote": true,
  "network_for_deps": true
}
```

Flags drive what can be completed now and what must be parked. Do not guess a flag.

---

## 4. Decision register

Answer these once, in `EXECUTION-STATE.json` under `decisions`. Defaults are proposed by the audit; change any before the run.

| ID | Decision | Default | Used by | Pre-answer field |
|---|---|---|---|---|
| D1 | Cursor model | **A — newtypes**; fall back to B if >15 files | RV-01.2 | `cursor_model` |
| D2 | Foreign IME in sandbox | **A — verbatim passthrough + notice** | RV-08.1 | `foreign_ime` |
| D3 | Windows toggle key default | **F4** (F12 remains selectable) | RV-04.2 | `toggle_key` |
| D4 | Windows installer format | **NSIS, per-machine** | RV-11.1 | `installer` |
| D5 | Code signing | **Deferred — unsigned, documented** unless a cert path is provided | RV-11.3 | `signing` |
| D6 | Icon provenance | **Replace with original or CC0 asset** if provenance cannot be proven | RV-10.1 | `icon_policy` |
| D7 | macOS scope | **Defer**; record `Deferred — no Mac access` | RV-12.1 | `macos` |
| D8 | Prediction | **Defer implementation**; accept proposed thresholds, run nothing | RV-12.2 | `prediction` |
| D9 | Release version for this programme | **0.2.0** | RV-11.7 | `release_version` |
| D10 | Artifact location | **`out/`** (git-ignored) | RV-10.5 | `artifact_dir` |
| D11 | Commit/push policy | **Local commits only; no push; no tag push** | §10 | `push` |
| D12 | Elevation | **Never auto-elevate**; installer tests only in a VM | RV-11 | `elevate` |
| D13 | Dev-dependency additions | **Approve the list in §5 only** | RV-01.2, RV-06, RV-07, RV-08 | `deps` |
| D14 | CI changes | **Allowed to edit workflow files; no secrets** | RV-06.5 | `ci` |

If a decision is missing at the moment it is needed, the agent **parks** the revision and asks once. It must not choose silently.

---

## 5. Pre-approved dependencies

Only these may be added without a new approval. Record each in the commit message and in `THIRD-PARTY-NOTICES.md` if it ships.

**Rust (dev/build only unless noted):**

| Crate | Purpose | Licence |
|---|---|---|
| `proptest` | property tests (RV-01.2, RV-02) | MIT/Apache-2.0 |
| `criterion` | benchmarks (RV-06.2, RV-13.2) | Apache-2.0/MIT |
| `tempfile` | config tests (RV-01, RV-07) | MIT/Apache-2.0 |
| `cbindgen` | C header generation (RV-02.5) | MPL-2.0 |
| `ts-rs` | TS type generation (RV-02.5, RV-07) | MIT |
| `cargo-fuzz` | fuzzing (RV-13.1, nightly) | Apache-2.0/MIT |
| `cargo-deny` | licence audit (RV-10.4) | MIT/Apache-2.0 |

**npm (dev only):**

| Package | Purpose | Licence |
|---|---|---|
| `vitest` | unit tests | MIT |
| `@testing-library/react` | component tests | MIT |
| `@testing-library/user-event` | interaction tests | MIT |
| `jsdom` | DOM environment | MIT |
| `eslint`, `typescript-eslint` | lint | MIT |
| `eslint-plugin-react-hooks` | hooks lint | MIT |
| `eslint-plugin-jsx-a11y` | a11y lint | MPL-2.0 |
| `prettier` | format | MIT |

**Fonts:** one bundled OFL Bengali + one OFL UI font (record names and licences).

Anything else: stop and ask. “It would be convenient” is not a rationale.

---

## 6. Execution order

Linear default. Parallel only with subagents that own disjoint files (§12).

| Step | Revision | Hard dependencies | Human/external gate | Parkable? |
|:--:|---|---|---|:--:|
| 1 | RV-01.1 candidate integrity | — | — | — |
| 2 | RV-01.2 cursor safety | RV-01.1 | D1 | No |
| 3 | RV-01 core correctness | RV-01.1, RV-01.2 | — | — |
| 4 | RV-10 licensing & hygiene | — | D6, app screenshot | Yes |
| 5 | RV-02 contract & ABI | RV-01 | adapter builds | Yes (build only) |
| 6 | RV-03 config unification | RV-02 | native observation | Yes (Windows manual) |
| 7 | RV-04.1 composition lifecycle | RV-02, RV-03 | focus/crash stress | Yes |
| 8 | RV-04 TSF safety | RV-04.1 | full E2E matrix | Yes |
| 9 | RV-05 Linux reproducibility | RV-02, RV-03 | Linux host / CI + GUI | Yes |
| 10 | RV-06 verification integrity | RV-01.1, RV-01.2 | CI run | Yes |
| 11 | RV-09 governance | RV-06, RV-10 | — | — |
| 12 | RV-07 desktop architecture | RV-02, RV-06 | — | — |
| 13 | RV-08.1 IME & focus | RV-07 | D2, manual IME matrix | Yes (manual part) |
| 14 | RV-08 UI & a11y | RV-07 | screen reader, keyboard pass | Yes |
| 15 | RV-13 hardening | RV-01, RV-02, RV-06, RV-07 | fuzz time, DPI, crash stress | Yes |
| 16 | RV-11 release engineering | RV-03, RV-04, RV-05, RV-07, RV-08, RV-10 | VM, elevation, signing | Yes |
| 17 | RV-12 future horizon | RV-11, RV-09 | Mac, developer decisions | Planning only |

**Parking rule:** if step N is parked, the agent may continue to step N+1 **only if** N+1's hard dependencies are code-complete and its own gate does not require N's evidence. When in doubt, stop and record the dependency.

**Never** reorder RV-01.x or start RV-07 before RV-02.

---

## 7. State file

Location: `milestones/revisions/EXECUTION-STATE.json` (committed).
Narrative log: `milestones/revisions/EXECUTION-LOG.md` (append-only).
Final report: `milestones/revisions/PROGRAMME-REPORT.md`.

### 7.1 Schema

```json
{
  "programme_version": "R0",
  "mode": "AUTOPILOT",
  "started_at": "2026-10-23T00:00:00+06:00",
  "last_updated": "2026-10-23T00:00:00+06:00",
  "branch": "revision-programme",
  "environment": {
    "windows": true, "mingw": true, "wsl_linux": false,
    "clean_vm": false, "signing_cert": false, "mac": false,
    "ci_remote": true, "network_for_deps": true
  },
  "decisions": {
    "cursor_model": "A", "foreign_ime": "A", "toggle_key": "F4",
    "installer": "NSIS", "signing": "deferred", "icon_policy": "replace",
    "macos": "defer", "prediction": "defer", "release_version": "0.2.0",
    "artifact_dir": "out", "push": false, "elevate": false,
    "deps": "approved-list", "ci": true
  },
  "revisions": {
    "RV-01.1": {
      "status": "Not started",
      "code_complete": false,
      "verified": false,
      "work_packages": { "WP-1.1.1": "todo", "WP-1.1.2": "todo" },
      "evidence": null,
      "commit": null,
      "blocker": null
    }
  },
  "blockers": []
}
```

Status values: `Not started`, `In progress`, `Implemented — not verified`, `Verified`, `Blocked`, `Deferred`.

### 7.2 Update rules

- Update after **every** work package, in the same commit.
- Never set `verified: true` without the evidence file path.
- `blocker` is null unless the revision is parked; then it holds the gate ID and a one-line reason.
- The state file is the resume mechanism. If it is stale, the run is not resumable.

---

## 8. The run loop

Execute exactly this sequence.

```
1. LOAD
   Read EXECUTE-ALL.md, 00-PROGRAMME.md, EXECUTION-STATE.json.
   If state is missing: run §3 probe, write state with defaults, commit "chore: init revision state".

2. SELECT
   Walk §6 in order. Pick the first revision whose status is not Verified
   and whose hard dependencies are at least code_complete.
   If none: go to 12.

3. PREFLIGHT THE REVISION
   Read the RV file and every file it lists.
   Run its baseline commands.
   If a stop condition is already true: park (11) and continue.

4. PICK THE NEXT WORK PACKAGE
   First WP not marked done in state.
   If none: go to 9 (revision acceptance).

5. IMPLEMENT
   Make the smallest change that satisfies the WP.
   Do not touch files owned by other revisions.
   Do not refactor beyond the WP.

6. GATE
   Run the WP gate command.
   If green: continue.
   If red: fix up to 3 attempts. If still red: rollback, park with the failing output.

7. EVIDENCE
   Append the command, trimmed output, and environment to the revision evidence file.
   Record the negative check if the WP requires one.

8. COMMIT + STATE
   Commit exactly one WP: "fix(rv-xx.y): <wp summary>".
   Update EXECUTION-STATE.json and EXECUTION-LOG.md in the same commit.
   Return to 4.

9. REVISION ACCEPTANCE
   Run every acceptance criterion in the RV.
   If all satisfied: run the full regression, write the completion report.
   If a human/external gate remains: mark "Implemented — not verified", park (11), continue.
   If all verified: set status Verified, link evidence, update ROADMAP.md, continue.

10. DOCS
    Update the revision's row in ROADMAP.md and the programme status board.

11. PARK
    Write a blocker record using Appendix B.
    Set revision status Blocked (or Implemented — not verified for evidence-only gates).
    Add to state.blockers.
    Return to 2 (do not stop the whole run).

12. FINALISE
    If every revision Verified: run §14 final verification,
    write PROGRAMME-REPORT.md, stop.
    Else: write PROGRAMME-REPORT.md listing parked gates and next actions, stop.
```

**Retry budget:** 3 attempts per gate. A fourth attempt requires a written note in the log explaining why the first three were wrong. Never loop indefinitely.

**Time budget:** if one work package exceeds ~2 hours of agent work, stop, record what is blocked, and split it in the RV (with the developer’s approval) rather than thrashing.

---

## 9. Gate types

| Type | Example | Satisfied by |
|---|---|---|
| **Command** | `cargo test -p bornomala-core` | Green exit code, recorded output |
| **Negative** | Deliberately break a test; it must fail | Recorded failing run, then restored |
| **Evidence** | Contrast table, E2E matrix | File in `docs/evidence/` with dates |
| **Manual** | Type in Word, screen reader | Human observation recorded by the human; the agent prepares the script |
| **External** | Linux CI, clean VM, Mac | Run link / VM log / Mac log |
| **Decision** | Cursor Option A/B | Developer answer in state.decisions |

The agent may satisfy Command, Negative, Evidence (when it can run the command), and Decision (from state). It must **not** satisfy Manual or External on the developer’s behalf.

---

## 10. Commit and evidence protocol

### 10.1 Branch and commits

- Branch: `revision-programme`.
- One work package per commit:
  `fix(rv-01.1): delete literal candidate fixtures`
  `feat(rv-02): add abi v2 state layout`
  `chore(rv-06): move fixtures to generator`
- Commit body: files changed, gate command, result, evidence path.
- No pushes unless `decisions.push` is true.
- No tags until the programme completes.

### 10.2 Evidence file

`docs/evidence/<RV-ID>/<YYYY-MM-DD>-<topic>.md`:

```md
# Evidence — RV-XX.Y — <date>
- Commit:
- Environment:
- Commands:
- Output (verbatim, trimmed):
- Negative check:
- Manual steps:
- Result: PASS | PARTIAL | FAIL
- Known limitations:
```

### 10.3 Definition of Verified (same as programme §6)

Six points; if any is missing, the status is `Implemented — not verified`.

---

## 11. Park and resume

### 11.1 When to park

Park when the remaining work needs:

- a human decision not in §4,
- a manual observation the agent cannot make,
- hardware (Mac, VM, DPI display, Linux desktop),
- a certificate or administrator approval,
- a CI run the agent cannot trigger.

### 11.2 What to write

Use Appendix B. Include:

- the exact gate ID and revision,
- what is needed,
- why it cannot be automated,
- exact steps for the human (commands, app names, expected result),
- what the agent already prepared,
- how to resume.

### 11.3 Resume

On the next run:

1. Read `blockers`.
2. If the human has recorded the evidence file, mark the gate satisfied and continue.
3. If not, keep it parked and continue with the next eligible revision.
4. Never re-park the same gate without adding new information.

---

## 12. Context and subagent management

A single context cannot hold 18 revisions. Use this strategy:

1. **One revision per context.** After completing or parking a revision, start the next from `EXECUTE-ALL.md` + the RV + the state file only.
2. **If the runtime supports subagents:** the parent orchestrator reads the state, spawns one subagent per revision with:
   - this file,
   - the state excerpt,
   - the target RV path,
   - the repo path and branch,
   - an explicit “do not edit other revisions” rule.
   The parent validates the gate output and makes the commits.
3. **File ownership:** never let two agents edit the same file concurrently. Partition by revision.
4. **Handoff note:** every subagent ends with `DONE / PARKED / FAILED` plus the commit SHA and gate output.
5. **No context bleed:** do not paste the whole audit into a subagent; give it the RV and the relevant evidence.

---

## 13. Never-do list

1. Never mark a manual or external gate `Verified`.
2. Never write a test that asserts the current buggy behaviour to make a gate green.
3. Never delete a failing test without a replacement.
4. Never disable a CI job to pass.
5. Never add a dependency outside §5 without approval.
6. Never change the ABI without updating both adapters and the desktop in the same revision.
7. Never leave the branch with a red gate.
8. Never rewrite `ROADMAP.md` history; correct current claims.
9. Never commit build artifacts (`*.dll`, `*.exe`, `*.so`, `*.a`, `*.pyc`).
10. Never log typed text.
11. Never auto-elevate, auto-register, or auto-install.
12. Never push or tag unless allowed.
13. Never “fix” a flaky test by adding a sleep.
14. Never copy a candidate/state/contract definition into a third place.
15. Never skip RV-01.1 or RV-01.2.
16. Never start RV-07 before RV-02.
17. Never start RV-11 before RV-04 and RV-05 are verified.
18. Never claim macOS support.
19. Never mark the programme complete with a parked revision.
20. Never ask the developer a question already answered in §4.

---

## 14. Final verification and report

Run in this order:

```powershell
cargo test --workspace
cargo run -p bornomala-desktop --bin persistence_suite
npm test
npm run build
pwsh -File scripts/check_docs.ps1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
pwsh -File native/borno-tsf/build.ps1 -NoRunHarness
```

Then write `milestones/revisions/PROGRAMME-REPORT.md`:

```md
# Revision Programme Report — R0
## Result: COMPLETE | PARTIAL (n parked)
## Revisions
| RV | Status | Commit | Evidence |
## Verification
- commands + results
## Parked gates
| Gate | Revision | What is needed | Prepared artifacts |
## Deferred decisions
## Release readiness
- support matrix link
- known limitations
## Next action for the developer
```

Do not use the word “complete” if any revision is parked.

---

## 15. Appendix A — per-revision quick reference

| RV | First command | Exit evidence |
|---|---|---|
| RV-01.1 | `cargo test -p bornomala-core candidates` | provenance test + sandbox screenshot |
| RV-01.2 | `cargo test -p bornomala-core editing` | property tests + paste manual |
| RV-01 | `cargo test --workspace` | sub-evidence + config tests |
| RV-02 | `cargo test -p bornomala-core --test ffi_tests` | ABI layout + adapter builds |
| RV-03 | `cargo run -p bornomala-desktop --bin persistence_suite` | migration + native observation |
| RV-04.1 | `pwsh -File native/borno-tsf/build.ps1 -NoRunHarness` | stress log |
| RV-04 | build + E2E | matrix of 12 rows |
| RV-05 | `ctest` in Linux container | CI link + GUI log |
| RV-06 | `pwsh -File scripts/check_docs.ps1` | canaries + CI matrix |
| RV-09 | `pwsh -File scripts/check_docs.ps1` | claim register |
| RV-07 | `cargo test --workspace && npm test` | line counts + tests |
| RV-08.1 | `npm test` | IME matrix |
| RV-08 | `npm test && npm run build` | contrast table + keyboard pass |
| RV-13 | `pwsh -File scripts/bench.ps1` | fuzz + perf + DPI |
| RV-11 | installer build | VM install/upgrade/uninstall |
| RV-12 | none | ADRs |

## 16. Appendix B — blocker template

```md
## BLOCKED — <RV-ID> / <gate-id>
- Status: PARKED
- What is needed:
- Why the agent cannot do it:
- Exact steps for the developer:
  1.
  2.
- Expected result:
- Artifacts prepared by the agent:
- Resume command/prompt:
- Date:
```

## 17. Appendix C — initial state template

Save as `milestones/revisions/EXECUTION-STATE.json`:

```json
{
  "programme_version": "R0",
  "mode": "AUTOPILOT",
  "started_at": "",
  "last_updated": "",
  "branch": "revision-programme",
  "environment": {},
  "decisions": {},
  "revisions": {},
  "blockers": []
}
```

The agent fills `environment`, `decisions`, and `revisions` on the first run. Do not pre-fill statuses.

---

## 18. What to do when the run stops

1. Read `PROGRAMME-REPORT.md` §Parked gates.
2. Do the human action (type in Word, run the VM, plug in the Mac).
3. Record the evidence in the specified file.
4. Re-run the same prompt. The conductor resumes at the parked gate.
5. Repeat until `Result: COMPLETE`.

If the run stopped for a red gate rather than a parked gate, read the failing output first; do not just re-run.

---

*End of conductor.*
