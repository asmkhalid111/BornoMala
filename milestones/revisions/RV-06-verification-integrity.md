# RV-06 — Verification & Evidence Integrity

> **Revision ID:** RV-06
> **Status:** Proposed — awaiting developer approval
> **Priority:** P0 (blocking trust)
> **Estimated effort:** 3–4 days (continuous through Phase 0)
> **Dependencies:** RV-01.1 (test re-baseline), RV-01.2 (property tests)
> **Blocks:** RV-07, RV-11, M13

---

## 1. Why this revision exists

The repository currently contains several forms of self-certifying verification. Until they are removed, **no** “Verified” claim can be trusted, including future ones.

| ID | Defect | Evidence |
|---|---|---|
| V-SELF-1 | A `#[test]` rewrites the layout “reference” fixtures from the same builders the golden tests read | [`builtin.rs:363-376`](../../crates/bornomala-core/src/layout/builtin.rs#L363-L376), [`layout_tests.rs:29-49`](../../crates/bornomala-core/tests/layout_tests.rs#L29-L49) |
| V-SELF-2 | Benchmark gates are compiled out in debug; the published numbers have no warm-up/`black_box`/isolation | [`dictionary_benchmarks.rs:54-59`](../../crates/bornomala-core/tests/dictionary_benchmarks.rs#L54-L59) |
| V-SELF-3 | The TSF harness prints `PASS`/`VERIFIED` unconditionally, including a fake rollback verification | [`test_harness.cpp:269`](../../native/borno-tsf/test_harness.cpp#L269), `:282-288` |
| V-COUNT-1 | Test counts disagree across five documents (142 / 217 / 229 / 238 / 246 vs actual 232 + 8) | audit §6.1 |
| V-CI-1 | CI runs only core tests and a frontend build; no desktop, native, formatting, lint, or doc checks | [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) |

This revision makes verification **honest, reproducible, and automatic**.

---

## 2. Current-state inspection

Read:

- `.github/workflows/ci.yml`
- `crates/bornomala-core/src/layout/builtin.rs` (the fixture test)
- `crates/bornomala-core/tests/layout_tests.rs`, `dictionary_benchmarks.rs`
- `native/borno-tsf/build.ps1`, `test_harness.cpp`
- `native/bornomala-fcitx5/CMakeLists.txt`
- every document that states a test count: `README.md`, `CONTRIBUTING.md`, `ROADMAP.md`, `milestones/M*.md`

Baseline:

```powershell
cargo test --workspace 2>&1 | Select-String "test result"
```

Compute the authoritative count per suite and record it. It is the number every document must use.

---

## 3. Scope

### Goals

1. Remove every self-certifying test and every unconditional `PASS` print.
2. Separate correctness tests from performance measurements.
3. Make CI run every layer the project claims to verify.
4. Establish one authoritative evidence format and one source of truth for counts.
5. Automate a documentation consistency check.

### Non-goals

- Not a rewrite of the test suite; keep the 232 behavioural tests.
- No new product features.
- No UI tests here (RV-08 owns those).

### Explicit no-go

- **No** test that mutates tracked files during `cargo test`.
- **No** timing assertion in a debug test run.
- **No** `PASS` printed without a computed condition.
- **No** hand-maintained test counts in prose.
- **No** CI gate that is known flaky and permanently red.

---

## 4. Work packages

### WP-6.1 — Fixture generation out of `cargo test`

1. Create a small binary or script: `crates/bornomala-core/src/bin/gen_fixtures.rs` or `scripts/gen_layout_fixtures.ps1`.
2. Move the body of `test_export_fixtures` into it.
3. Delete the `#[test]` that rewrites fixtures.
4. Keep fixtures checked in; add a CI step that regenerates to a temp dir and compares with `git diff --exit-code`.
5. Add a header comment to each fixture: generated file, do not edit, source of truth is the builder.

**Acceptance:** `cargo test` cannot modify `tests/fixtures/`; deliberately changing a builder mapping makes the CI fixture check fail.

### WP-6.2 — Honest benchmarks

1. Move timing measurements out of `tests/` into `benches/` (criterion) or into `#[ignore]` tests that require release mode.
2. Add warm-up (≥1,000 iterations), `std::hint::black_box`, and a single-threaded run (`--test-threads=1` or criterion).
3. Report p50/p95/p99, not a single mean.
4. Add a `scripts/bench.ps1` that builds `--release`, runs the bench, and writes `docs/evidence/perf/<date>.md` with the machine description.
5. Keep a **debug-mode smoke test** that only asserts correctness, never timing.
6. Remove or mark the arithmetic “memory gate” as an estimate; if a memory budget is required, measure with an allocator counter.

**Acceptance:** `cargo test` runs fast and asserts no timing; `scripts/bench.ps1` produces a report with percentiles and hardware.

### WP-6.3 — TSF harness honesty

1. Replace every unconditional `PASS` with a condition; track a `failures` counter.
2. Return a non-zero exit code if any check fails.
3. Remove the unconditional “Automatic Registration Rollback: VERIFIED” line; only print it if a rollback was actually observed.
4. Gate registration behind `--register-test` (RV-04.3).
5. Label Levels 5–6 as `NOT TESTED` with the missing prerequisite, not `PENDING` (which reads like a promise).

**Acceptance:** force one check to fail (`--fail-check 3`) and confirm the harness exits non-zero.

### WP-6.4 — Assertion-effective Linux tests

Handled in RV-05.2; this revision verifies it is wired into CI and records the canary output.

### WP-6.5 — CI matrix

Create jobs:

| Job | Runner | Steps | Blocking |
|---|---|---|---|
| `rust-core` | windows-latest | `cargo test --workspace`; `cargo fmt --check`; `cargo clippy -D warnings` | yes |
| `desktop` | windows-latest | `cargo test -p bornomala-desktop`; `cargo run --bin persistence_suite`; `npm ci`; `npm run build` | yes |
| `frontend-tests` | windows-latest | `npm test` (after RV-08 adds Vitest) | starts non-blocking until tests exist |
| `native-windows` | windows-latest | build DLL + harness without registration; assert registry unchanged | yes |
| `native-linux` | ubuntu-24.04 container | build addon + `ctest` + canary | yes |
| `docs-consistency` | ubuntu-latest | run `scripts/check_docs.ps1` (WP-6.6) | yes |

Rules:

- `npm ci`, not `npm install`.
- Cache cargo/npm.
- No job may require a GUI except the opt-in WSL job.
- A failing negative check (canary) is itself a job step that must **fail**; wrap it so the job asserts the failure.

### WP-6.6 — Documentation consistency script

Create `scripts/check_docs.ps1` (or Python) that fails when:

1. Any document states a test count that differs from the authoritative value (search for `\b\d+\s+(core\s+)?tests?\b` in `README.md`, `CONTRIBUTING.md`, `ROADMAP.md`, `milestones/*.md`).
2. A milestone status in `ROADMAP.md` differs from the status line in its report.
3. A relative markdown link points to a missing file.
4. `docs/evidence/` references in the roadmap point to missing files.
5. `crates/borno-core` (old path) appears anywhere.

To make (1) maintainable, define the count in one place: `docs/test-inventory.md` generated by `scripts/gen_test_inventory.ps1` from `cargo test -- --list`. Documents link to it instead of restating numbers; where a number is restated, the script enforces it.

**Acceptance:** deliberately change a count in `README.md` → script fails.

### WP-6.7 — Evidence system

1. Create `docs/evidence/README.md` defining the template and the rule.
2. Add `docs/evidence/<RV>/` folders as revisions complete.
3. Add a `Definition of Verified` section to `AGENTS.md` (or link to the programme).
4. Every roadmap row that says `Verified` must link an evidence file. The docs script enforces it.

### WP-6.8 — Test inventory and classification

1. Generate a table: suite, file, count, type (unit/integration/golden/property/bench), runtime (debug), owner.
2. Identify tests that assert nothing meaningful (grep for tests with no `assert`/`panic!`), list them, and fix or delete with justification.
3. Identify duplicate coverage (the `"ka"` scenario in three files) and consolidate only where it reduces maintenance without losing coverage.

**Acceptance:** the inventory exists; every test has at least one assertion; duplicates are documented, not silently removed.

---

## 5. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | `cargo test` cannot modify tracked files | `git status` after run |
| AC-2 | Fixture drift fails CI | canary |
| AC-3 | No timing assertions in the default test run | source + run time |
| AC-4 | Bench script produces percentiles + machine info | report |
| AC-5 | TSF harness exits non-zero on a forced failure | run |
| AC-6 | CI runs core, desktop, native Windows, native Linux, docs | workflow |
| AC-7 | Docs script fails on a wrong count/href/status | canary |
| AC-8 | Every `Verified` row links evidence | script |
| AC-9 | Test inventory generated and accurate | file |

---

## 6. Testing plan

```powershell
cargo test --workspace
git status --short                     # must be clean
pwsh -File scripts/check_docs.ps1
pwsh -File scripts/bench.ps1 -Quick    # quick smoke
pwsh -File native/borno-tsf/build.ps1 -NoRunHarness
native\borno-tsf\test_harness.exe --fail-check 3   # must exit non-zero
```

Plus the CI workflow run itself.

---

## 7. Runnable checkpoint

- All local commands pass.
- CI is green.
- The sandbox still launches.

Rollback: workflow changes are isolated in `.github/`; fixture generation can be reverted independently.

---

## 8. Evidence and reporting

`docs/evidence/RV-06/<date>-verification.md` with each canary output (the deliberate failure) and the CI run link.

---

## 9. Roadmap update

- Replace all test counts with a link to `docs/test-inventory.md` or the verified number.
- Add the CI matrix to the roadmap's cross-cutting requirements.
- Mark M6 “229 tests” entries as historical; do not rewrite history, correct the current claims.

---

## 10. Stop conditions

- The docs script becomes more complex than the docs → simplify the rule (link-only counts).
- A CI job cannot run without a paid runner or a GUI → document and keep it opt-in.
- Removing a “test” reveals it was the only coverage for a behaviour → add a real test first.

---

## 11. Handoff

With honest verification in place, every later revision can prove itself. RV-07 and RV-08 add tests into this framework, not beside it.
