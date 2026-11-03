# RV-13 — Hardening, Performance & Privacy Assurance

> **Revision ID:** RV-13
> **Status:** Proposed — awaiting developer approval
> **Priority:** High (release gate)
> **Estimated effort:** 4–6 days
> **Dependencies:** RV-01, RV-02, RV-04, RV-05, RV-06, RV-07
> **Blocks:** M13

---

## 1. Why this revision exists

The roadmap’s M11 is currently a paragraph with no measurable gates. The audit found:

- no property/fuzz coverage for three parsers,
- timing assertions that do not measure what they claim,
- no crash-recovery test,
- per-keystroke content logging in the Linux adapter (fixed by RV-05, verified here),
- no privacy test that would fail if a network call or content log is introduced,
- no DPI verification,
- no long-session or memory-growth measurement.

This revision turns “hardening” into a set of automated gates that must be green before v1.0.0.

---

## 2. Current-state inspection

Read:

- `crates/bornomala-core/tests/dictionary_benchmarks.rs`, `tests/layout_tests.rs` (benchmark sections)
- `crates/bornomala-core/src/dictionary/trie.rs`, `config.rs`, `phonetic/tokenizer.rs`
- `native/bornomala-fcitx5/bornomala.cpp` (logging)
- `native/borno-tsf/candidate_window.cpp` (DPI)
- `.github/workflows/ci.yml`
- `docs/privacy.md`, `docs/testing.md` (from RV-09)

Baseline: run the RV-06 bench script and record the numbers, the machine, and the toolchain.

---

## 3. Scope

### Goals

1. Fuzz/property coverage for every parser and for the event stream.
2. A reproducible performance harness with budgets and percentiles.
3. Crash-recovery and long-session tests.
4. A privacy gate that fails on content logging or network calls.
5. DPI verification on Windows.
6. A threat-model review and a release-blocker triage.
7. All results recorded with hardware and versions.

### Non-goals

- No new features.
- No optimisation without a measured regression.
- No formal verification.

### Explicit no-go

- **No** performance claim without a committed benchmark run.
- **No** “hardened” label while a known P0 is open.
- **No** network access in the core or adapters.
- **No** logging of preedit/committed text at any level.
- **No** fuzz target that requires a network corpus download at test time.

---

## 4. Work packages

### WP-13.1 — Fuzzing

Add `cargo-fuzz` targets (or `proptest` where a fuzzer is impractical):

| Target | Input | Invariant |
|---|---|---|
| `trie_from_binary` | arbitrary bytes | never panics; returns `Err` or a trie; bounded depth |
| `config_from_json` | arbitrary UTF-8/JSON | never panics; `Err` or a valid config; unknown fields handled |
| `tokenizer` | arbitrary ASCII/Bangla | no panic; output is valid UTF-8; spans in bounds |
| `engine_events` | arbitrary event sequences | no panic; state invariants hold |

Run each for a fixed time (e.g. 5 minutes) in a nightly job; commit the corpus seeds.

Gate: 24-hour aggregate run with no crash; any crash becomes a regression test and a fix.

### WP-13.2 — Performance harness

1. Use criterion (or a hand-rolled harness) with:
   - warm-up,
   - `black_box`,
   - single-threaded measurement,
   - p50/p95/p99,
   - release profile.
2. Measure:
   - core keystroke latency (phonetic, three layouts),
   - candidate generation,
   - dictionary prefix query,
   - cold start (first query after load),
   - `InputString` of 200 characters,
   - FFI round trip (Rust caller),
   - memory (RSS before/after 100k keystrokes).
3. Budgets (proposed; confirm with the developer):
   - core keystroke p99 ≤ 1 ms,
   - FFI round trip p99 ≤ 2 ms,
   - cold dictionary load ≤ 150 ms,
   - memory growth after 100k keystrokes ≤ 5 MB.
4. CI runs a short bench; a full run is a manual release step.
5. Publish `docs/evidence/perf/<date>.md` with the machine, OS build, rustc version, and commit.

Gate: budgets met or explicitly waived with a written reason.

### WP-13.3 — Crash recovery and long sessions

1. Windows: kill the host process mid-composition; reopen; the IME must not be stuck.
2. Repeated focus switches (500 cycles) with composition; no crash, no handle growth.
3. 1,000,000 keystrokes in the sandbox: memory stable, no unbounded candidate history.
4. Engine panic injection (test hook): the next event must recover to idle.

Gate: all four recorded.

### WP-13.4 — Privacy gate

1. Add a CI check that fails if any of these appear outside tests/docs:
   - `reqwest`, `hyper`, `ureq`, `TcpStream`, `fetch(`, `XMLHttpRequest`, `https://` in runtime code (allow a documented allowlist for licence URLs in docs).
   - logging macros whose arguments include `preedit`, `committed`, `raw_buffer`, `text`.
2. Verify with a proxy/network capture that launch + typing performs no network I/O.
3. Verify logs during a 100-keystroke session contain no Bangla text.
4. Document the check and its limits in `docs/privacy.md`.

Gate: a deliberately added `fetch()` fails CI; the capture shows no requests.

### WP-13.5 — DPI and rendering

1. Windows: test 100 %, 125 %, 150 %, 200 % and mixed-DPI dual monitor.
2. Verify candidate window position, size, font, and no clipping.
3. Record screenshots per scale.
4. Fix or document each defect.

Gate: a table per scale with pass/fail and notes.

### WP-13.6 — Threat model and security review

Write `docs/security.md`:

- assets: typed text, config, dictionary, binaries;
- adversaries: malicious host app, malicious config file, malicious layout file, supply chain;
- mitigations: no network, local-only data, bounded parsers, signed binaries, least privilege (no admin at runtime), CSP in the WebView;
- residual risks: TSF runs in-process (a crash affects the host), third-party dependencies.

Review:

- FFI boundary (see RV-02),
- file parsers (fuzz results),
- installer elevation (RV-11),
- dependency audit (RV-10).

Gate: documented and reviewed; no unresolved critical item.

### WP-13.7 — Release-blocker triage

1. Create a triage log with severity rubric.
2. List every open issue with severity and evidence.
3. Define the release gate: no open P0; every P1 either fixed or accepted in writing.
4. Link the log from the roadmap.

Gate: the log exists and the gate is explicit.

---

## 5. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | Four fuzz targets run and are clean | corpus + run log |
| AC-2 | Bench harness with percentiles and budgets | report |
| AC-3 | Crash/long-session tests recorded | evidence |
| AC-4 | Privacy gate active and canary-failing | CI |
| AC-5 | No network traffic on launch/typing | capture |
| AC-6 | No typed content in logs | log inspection |
| AC-7 | DPI matrix recorded | screenshots |
| AC-8 | Threat model written | doc |
| AC-9 | Triage log with release gate | doc |

---

## 6. Testing plan

```powershell
cargo +nightly fuzz run trie_from_binary -- -max_total_time=300
cargo +nightly fuzz run config_from_json -- -max_total_time=300
pwsh -File scripts/bench.ps1
pwsh -File scripts/check_privacy.ps1
```

Manual: DPI matrix, 100k-keystroke session, host kill/reopen.

---

## 7. Runnable checkpoint

Hardening adds tests and tooling; the app must remain runnable after every package.

---

## 8. Evidence and reporting

`docs/evidence/RV-13/<date>-hardening.md` with fuzz logs, bench tables, memory graphs, captures, DPI screenshots.

---

## 9. Roadmap update

- Replace M11’s prose with the gates from this revision.
- M13 requires all RV-13 ACs green.

---

## 10. Stop conditions

- A fuzz target finds a crash → fix it before continuing; do not suppress.
- A performance budget cannot be met → report it and let the developer decide between optimisation and scope reduction.
- The privacy gate produces false positives on docs → narrow the path filter, never disable the gate.

---

## 11. Handoff

The release decision (M13) can now be made from evidence rather than intent.
