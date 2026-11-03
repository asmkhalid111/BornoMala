# RV-10 — Licensing, Provenance & Release Hygiene

> **Revision ID:** RV-10
> **Status:** Proposed — awaiting developer approval
> **Priority:** P0 (legal blocker)
> **Estimated effort:** 1.5–2 days
> **Dependencies:** none (can start day one)
> **Blocks:** RV-11, M13

---

## 1. Why this revision exists

The product distributes a CC BY 4.0 derived lexicon inside both native binaries and does not ship the attribution the licence requires.

| ID | Defect | Evidence |
|---|---|---|
| P0-7 | Leipzig CC BY 4.0 data is embedded via `include_bytes!` and ships inside `bornomala_tsf.dll` and `bornomala.so`; no NOTICE/THIRD-PARTY file exists | [`dictionary/mod.rs:16`](../../crates/bornomala-core/src/dictionary/mod.rs#L16) |
| P0-7 | `docs/resources-and-licenses.md` claims attribution is “in the application LICENSE”; neither LICENSE file mentions Leipzig or CC BY | [`resources-and-licenses.md:29`](../../docs/resources-and-licenses.md#L29) |
| Provenance | A word (`বর্ণমালা`) was added by hand while the document says the artifact is exclusively corpus-derived | [`M9-completion-report.md:36`](../M9-completion-report.md#L36) |
| Provenance | `lexicon_provenance.json` still points at `crates/borno-core/...` | [`lexicon_provenance.json`](../../docs/lexicon_provenance.json) |
| Hygiene | Two byte-identical 7.9 MB DLLs and a 9.9 MB harness sit in the source tree; `*.dll`/`*.exe` are git-ignored but shipped in the folder | [`build.ps1:57,60-66`](../../native/borno-tsf/build.ps1#L57) |
| Hygiene | `scripts/__pycache__` committed to the working tree | tree |
| Hygiene | `data/bornomala.png` has no recorded provenance/licence | tree |
| Dependency policy | No `cargo-deny`/`npm audit`; licences of dependencies are not inventoried | no config |

---

## 2. Current-state inspection

Read:

- `LICENSE-MIT`, `LICENSE-APACHE`, `docs/resources-and-licenses.md`, `docs/lexicon_provenance.json`
- `scripts/prepare_lexicon.py`
- `Cargo.toml`, `crates/bornomala-core/Cargo.toml`, `src-tauri/Cargo.toml`, `package.json`
- `native/bornomala-fcitx5/data/bornomala.png`
- the build scripts and `.gitignore`

Inventory every third-party component:

| Component | Where it ships | Licence | Attribution required? |
|---|---|---|---|
| Leipzig lexicon (derived) | Rust core → both binaries | CC BY 4.0 | **Yes** |
| Rust crates | static binary | per crate | Yes for some |
| npm packages | bundled JS | per package | Yes for some |
| Fcitx 5 headers | linked | LGPL-2.1+ | Dynamic linking; notices |
| MinGW runtime (static) | DLL | permissive + exception | Usually no, verify |
| Fonts (after RV-08) | bundled | SIL OFL | **Yes** |
| `bornomala.png` | Fcitx icon | unknown | Resolve |

---

## 3. Scope

### Goals

1. Ship a real `THIRD-PARTY-NOTICES.md` with the Leipzig attribution and every dependency that requires one.
2. Correct the provenance documents (including the hand-added word).
3. Make the app expose the notice (About/Diagnostics).
4. Remove build artifacts from the source tree; keep one artifact name.
5. Add automated licence checks.
6. Record the icon and font provenance.

### Non-goals

- No relicensing of the project.
- No change to the lexicon contents beyond documenting them.
- No legal advice; when in doubt, attribute more.

### Explicit no-go

- **No** shipping a binary without the NOTICE.
- **No** claiming “attribution is in the LICENSE” unless it literally is.
- **No** silently dropping a hand-curated lexicon entry; document it or remove it with a test update.
- **No** committing build outputs.

---

## 4. Work packages

### WP-10.1 — Third-party notices

Create `THIRD-PARTY-NOTICES.md` at the repo root:

1. **Leipzig Corpora Collection** — dataset name, URL, CC BY 4.0 URL, the required attribution string, the transformation performed (NFC, Bengali-only filter, frequency aggregation, top-40,001, binary trie), and the date.
2. **Hunspell validator** — state that it was used only as an offline oracle and no data is redistributed.
3. **Rust crates** — generated section (see WP-10.4).
4. **npm packages** — generated section.
5. **Fcitx 5** — LGPL notice and link.
6. **Fonts** — OFL notice when bundled.
7. **Icon** — provenance or replace with an original/CC0 asset.

Rules: one file, human-readable, shipped with both binaries and shown in the app.

### WP-10.2 — Correct the provenance documents

1. Fix `crates/borno-core` → `crates/bornomala-core` everywhere.
2. Record the hand-added `বর্ণমালা` entry: frequency, rank, reason, date, and that it is not corpus-derived. Update the “exclusively corpus-derived” sentence.
3. Update `binary_sha256` if the artifact changes; verify the current hash matches.
4. Add a “How to regenerate” section pointing at `scripts/prepare_lexicon.py`.
5. Add the source archive hash and the extraction date.

Gate: hashing the artifact matches the recorded hash; a fresh regeneration reproduces the same count.

### WP-10.3 — App-facing notice

Add a “Licences & Attribution” section in the sandbox settings (About/Diagnostics tab), reading the same content (embedded at build time or fetched from a bundled file).

Gate: visible in the app, screenshotted.

### WP-10.4 — Automated dependency licences

1. Add `cargo-deny` config (`deny.toml`) with allowed licences and a ban on unknown.
2. Add `npm audit --omit=dev` and `license-checker` (or `pnpm licenses`) to CI.
3. Generate a dependency licence section into `THIRD-PARTY-NOTICES.md` via a script; CI fails if it is stale.
4. Record the licence of every direct dependency in a table.

Gate: CI fails when a dependency with a disallowed licence is added (simulate with a fake entry or a dry run).

### WP-10.5 — Binary hygiene

1. Build outputs go to `out/` (git-ignored); delete `borno_tsf.dll` and `test_harness.exe` from the source folder.
2. One DLL name: `bornomala_tsf.dll`; remove the alias and the fallback copy.
3. Update `register.bat`/`unregister.bat` to the single name.
4. Remove `scripts/__pycache__`; add `__pycache__/` is already ignored — verify it is not shipped.
5. Add a CI check that no tracked file matches `*.dll`, `*.exe`, `*.a`, `*.so`, `*.pyc`.

Gate: a clean clone contains no binaries.

### WP-10.6 — Privacy statement alignment

`docs/privacy.md` (created in RV-09) must state:

- no telemetry, no accounts, no network for typing;
- the only network use is none (and after RV-08, fonts are local);
- diagnostics are opt-in and never include typed text;
- the Fcitx INFO logging issue is fixed (RV-05.5) and verified.

---

## 5. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | `THIRD-PARTY-NOTICES.md` exists and includes Leipzig | file |
| AC-2 | The notice ships in the app | screenshot |
| AC-3 | Provenance hash matches the artifact | command output |
| AC-4 | Hand-added word documented | doc diff |
| AC-5 | No `crates/borno-core` path remains | grep |
| AC-6 | No tracked binaries | `git ls-files` |
| AC-7 | One DLL name | tree + scripts |
| AC-8 | `cargo-deny` + npm audit in CI | workflow |
| AC-9 | Icon provenance recorded or replaced | doc |

---

## 6. Testing plan

```powershell
Get-FileHash crates\bornomala-core\data\lexicon.bin -Algorithm SHA256
cargo deny check licenses
npm audit --omit=dev
git ls-files | Select-String -Pattern '\.(dll|exe|so|a|pyc)$'
pwsh -File scripts/check_docs.ps1
```

Manual: open the app, confirm the notice renders.

Negative check: add a fake GPL dependency to `Cargo.toml` in a branch → `cargo-deny` fails.

---

## 7. Runnable checkpoint

Documentation and build-hygiene changes only; the app must still build and launch from `out/` artifacts.

---

## 8. Evidence and reporting

`docs/evidence/RV-10/<date>-licensing.md`: notices file link, hash output, dependency audit output, screenshot.

---

## 9. Roadmap update

- Mark the licensing dimension `Verified` only after the notice ships.
- Add “NOTICE present” to the M12 release checklist.
- Correct `docs/resources-and-licenses.md`.

---

## 10. Stop conditions

- The icon provenance cannot be established → replace it with an original/CC0 asset rather than guessing.
- `cargo-deny` flags a dependency the project cannot replace → record a documented exception with the developer’s approval.
- The lexicon cannot be regenerated deterministically → stop; that is a reproducibility defect for RV-06.

---

## 11. Handoff

RV-11 may package the binaries with the notice included.
