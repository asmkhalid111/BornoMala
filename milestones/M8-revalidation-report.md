# Milestone M8 Revalidation Report — Linux Fcitx 5 Integration (`de0149a`, WP1–WP3)

> **Report Role:** Current-Revision Evidence Reconciliation (distinct from historical [`M8-completion-report.md`](M8-completion-report.md))  
> **Date:** 2026-11-12  
> **Published Git Baseline:** `master` at `de0149af60e6aec767efea0f476043db85b31771` (`WP1`: `6bdb0ec7cc2c23eba6b1fc31e206c911a55e39ba`, `WP2`: `0e601aa5aef026145d3481040f4a2494bffb9172`, `WP3`: `de0149af60e6aec767efea0f476043db85b31771` above pre-M8 baseline `249bcbc`)  
> **Current M8 Status:** **Partially Verified — Release Build, Linux CI Workflow, Installer Hygiene & Isolated WSLg GTK3/Gedit Verified Locally; Hosted CI Blocked (Account Billing), Native Linux Desktop, Native Wayland & Live Pointer Candidate Click Unverified**

---

## 1. Purpose & Relationship to Historical M8 Evidence

The original [`milestones/M8-completion-report.md`](M8-completion-report.md) (`2026-10-19`) is preserved intact as the historical milestone record. Subsequent repository audits (`RV-05`, [`M9-completion-report.md` §7](M9-completion-report.md#L213-L219), and `FINAL-REMEDIATION-CLOSURE-REPORT.md`) identified contradictions and unverified/defective areas against current `master` (`249bcbc`), including:

- Unverified post-M8 adapter edits (`WP-01`, `WP-06`, `WP-C`).
- Release-mode `<cassert>` elision under `NDEBUG`.
- Per-context engine mode desync, shared `m_configMeta`, `$XDG_CONFIG_HOME` omission, unconditional `filterAndAccept()`, and pointer candidate selection state desync.
- Destructive `killall`/`pkill -9`/profile-overwrite scripts and absent Linux GitHub Actions CI.

`M8-WP1`, `M8-WP2`, and `M8-WP3` remediated those adapter, installer/harness, and CI workflow defects and re-collected executable evidence as recorded below.

---

## 2. Environments & Observed Versions

| Environment | Observed Details | Role |
|---|---|---|
| **WSL2 Ubuntu 26.04.1 LTS** | Kernel `6.18.54.1-microsoft-standard-WSL2`, `rustc 1.99.1`, `cargo 1.99.1`, `cmake 4.2.3`, `g++ 15.2.0`, `fcitx5 5.1.19-1`, `libfcitx5core-dev 5.1.19-1`, `libfcitx5config-dev 5.1.19-1`, `libfcitx5utils-dev 5.1.19-1`, `fcitx5-modules-dev 5.1.19-1`, `gedit 48.1-9build1`, `python3-gi 3.56.2-1`, `Gtk 3.24.52`, `xdotool 1:3.20160805.1-5.1build1`, WSLg `DISPLAY=:0` (`--disable waylandim`) | Local Release build, CTest, E2E installer/uninstaller, and isolated WSLg runtime checks |
| **GitHub Actions `ubuntu-24.04`** | Configured in [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) (`linux-fcitx5` job; `libfcitx5core-dev` `5.1.7-1build3` in Ubuntu 24.04 `universe`) | Push-triggered run [`#11`](https://github.com/asmkhalid111/BornoMala/actions/runs/43225585458) (`de0149a`, `2026-11-12`): all 3 jobs **not started** due to GitHub account billing lock (**hosted CI unverified**) |

---

## 3. Published M8 Work Packages (`249bcbc` → `de0149a`)

- **`M8-WP1` (`6bdb0ec7cc2c23eba6b1fc31e206c911a55e39ba`, `fix(linux): correct Fcitx adapter state and enforce Release tests`):**
  - Extracted shared [`adapter_helpers.h`](../native/bornomala-fcitx5/adapter_helpers.h) / [`adapter_helpers.cpp`](../native/bornomala-fcitx5/adapter_helpers.cpp), updated [`engine.h`](../native/bornomala-fcitx5/engine.h) and [`bornomala.cpp`](../native/bornomala-fcitx5/bornomala.cpp).
  - Fixed per-context Rust engine mode synchronization via explicit `BORNO_EVENT_SWITCH_MODE`, per-context `LinuxConfigMetadata`, `$XDG_CONFIG_HOME` resolution with relative-path rejection, `ShouldConsumeEngineResult()` (`rc == BORNO_OK && res.handled`) gating on all `filterAndAccept()` paths, and safe `TrackableObjectReference`-based `BornoCandidateWord::select()` context synchronization.
  - Added Release-active `BORNO_CHECK` ([`tests/test_check.h`](../native/bornomala-fcitx5/tests/test_check.h)) across C++ tests (`NDEBUG` preserved), designated negative canary [`tests/test_canary.cpp`](../native/bornomala-fcitx5/tests/test_canary.cpp) (`BORNO_TEST_CANARY=1`), and CMake installation of `data/bornomala.png` into `icons/hicolor/64x64/apps` ([`CMakeLists.txt`](../native/bornomala-fcitx5/CMakeLists.txt)).
- **`M8-WP2` (`0e601aa5aef026145d3481040f4a2494bffb9172`, `fix(linux): make Fcitx installation and test harness non-destructive`):**
  - Rewrote [`install.sh`](../native/bornomala-fcitx5/scripts/install.sh) and [`uninstall.sh`](../native/bornomala-fcitx5/scripts/uninstall.sh) sharing [`common.sh`](../native/bornomala-fcitx5/scripts/common.sh): XDG resolution, ELF validation, pre-write checks, atomic file installation, ownership manifest + allowlist validation, custom-icon preservation, dry-run/confirmation flags, zero `killall`/`pkill`, and default profile preservation (explicit opt-in `--register-profile` / `--reload-fcitx`).
  - Added [`fcitx_test_env.py`](../native/bornomala-fcitx5/scripts/fcitx_test_env.py) and updated [`verify_gtk3.py`](../native/bornomala-fcitx5/tests/verify_gtk3.py), [`verify_gedit.py`](../native/bornomala-fcitx5/tests/verify_gedit.py), [`interactive_tester.py`](../native/bornomala-fcitx5/scripts/interactive_tester.py), [`launch_test_editor.sh`](../native/bornomala-fcitx5/scripts/launch_test_editor.sh) (adding procfs `(comm, starttime)` child-process identity checks, `INT`/`TERM` traps, and bounded `$SESSION_TMP` cleanup), and the Windows `.bat` wrappers (`Launch-Linux-Fcitx5-Test.bat`, `Launch-Linux-Gedit-Test.bat`) to isolate `HOME`/`XDG_*` + private `dbus-daemon`, exit `77` (`RESULT=UNAVAILABLE`) when display/tools are absent, and omit `~/.config/fcitx5/profile` and `gsettings` mutations.
  - Added [`tests/test_scripts.py`](../native/bornomala-fcitx5/tests/test_scripts.py) (`16` tests total, including `4` `LaunchTestEditorCleanupTests` cases) and [`native/bornomala-fcitx5/README.md`](../native/bornomala-fcitx5/README.md).
- **`M8-WP3` (`de0149af60e6aec767efea0f476043db85b31771`, `ci(linux): add Fcitx 5 verification and reconcile M8 evidence`):**
  - Added `linux-fcitx5` job on `ubuntu-24.04` to [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) alongside the unchanged `windows-latest` jobs.
  - Published this revalidation report and [`docs/platform-support.md`](../docs/platform-support.md); reconciled [`ROADMAP.md`](../ROADMAP.md) and [`milestones/M8.md`](M8.md).

---

## 4. Executable Verification Evidence

### 4.1 Core, Build, CTest, Privacy & Packaging Checks

| Check | Exact Scope & Result |
|---|---|
| **Linux Rust Core Suite** (`cargo test -p bornomala-core`) | **Passed (`265 passed; 0 failed; 0 ignored`)** — `55` lib, `1` `c_abi_layout`, `8` `candidate_provenance`, `17` `contract`, `5` `dictionary_benchmarks`, `10` `dictionary`, `8` `engine_config`, `8` `ffi`, `57` `layout`, `7` `licensing`, `67` `phonetic`, `22` `unicode_golden`. |
| **Windows Core FFI Invariants** (`cargo test -p bornomala-core --test ffi_tests`) | **Passed (`8 passed; 0 failed`)** — includes `test_linux_adapter_privacy_and_portability_invariants` after WP1 & WP2 edits. |
| **Release PIC Staticlib + CMake Addon Build** (`-DCMAKE_BUILD_TYPE=Release`, `NDEBUG` active) | **Passed (`0` warnings/errors)** — built `libbornomala_core.a` (`26M` PIC) and ELF shared object `bornomala.so` (`7,774,696` bytes). |
| **Normal CTest** (`ctest --output-on-failure --verbose`) | **Passed (`4/4`)** — `CLinkageTest`, `KeyMapperTest`, `EngineAdapterTest` (covering WP1 Corrections A–D), and `CanaryTest` (explicitly printed `NDEBUG` active). |
| **Negative Canary CTest** (`BORNO_TEST_CANARY=1`) | **Verified Non-Zero Failure (`CANARY_EXIT_CODE=8`)** — `1..3` passed, `CanaryTest` failed with `[FAIL] Check failed: (false && "Intentional BORNO_TEST_CANARY=1 verification failure")`; subsequent normal CTest passed `4/4`. |
| **Staged `DESTDIR` Installation** | **Passed** — installed `bornomala.so`, `addon/bornomala.conf`, `inputmethod/bornomala.conf`, `icons/hicolor/64x64/apps/bornomala.png` (`3,263` bytes), and `doc/bornomala/THIRD-PARTY-NOTICES.md` (`91,679` bytes). |
| **Static Binary Privacy Scan** | **Passed (`0` matches)** on `bornomala.so` for `sym=0x`, `preedit=`, and `committed=`. |
| **WP2 Script Safety Suite** (`test_scripts.py`) | **Passed (`16/16` in `4.967s` final WP2 publication)** — initial WP2/WP3 snapshot passed `12/12` (`4.539s`); after adding `4` `LaunchTestEditorCleanupTests` procfs identity and bounded temp-cleanup regression cases before publication, full suite passed `16/16` (`4.967s`) covering temp-HOME/XDG paths, error exits before writes, idempotency, profile/custom-icon preservation, manifest allowlist, PID-identity cleanup, and `DISPLAY=""` -> exit `77`. |
| **Full Build-From-Source E2E Installer & Uninstaller (`$TMP_ROOT` fake HOME)** | **Passed** (`RUSTUP_HOME` & `CARGO_HOME` preserved for `rustup` toolchain) — built PIC staticlib + CMake addon, verified ELF `.so` & files, verified temp profile/unrelated sentinels byte-identical (`cmp -s`), verified 2nd run `[unchanged]`, verified `--dry-run`, confirmed `--yes` removal + empty `doc/bornomala` cleanup + 2nd `--yes` idempotency. |

### 4.2 Isolated WSLg Runtime Evidence (WSLg XWayland `--disable waylandim`)

All live runtime checks ran in private `dbus-daemon` + temporary `HOME`/`XDG_*` sessions, verifying exact `/proc/<fcitx_pid>/maps` mapping of the fresh WP1 `bornomala.so` (`~/.local/lib/fcitx5/bornomala.so` excluded; real `~/.config/fcitx5/profile` and stale `.so` SHA-256 checksums verified unchanged before and after):

| Scenario | Observed Result |
|---|---|
| **`verify_gtk3.py` (`Layer 6A`)** | **Passed** — active IM `bornomala`; Phonetic `ami ` -> `'আমি '`; compose-cancel `Escape` consumed and idle `Escape` delivered `['Escape']` to `Gtk.Entry`; WP-C `'1 '` literal numeric; multi-candidate digit `'2'` on `am` -> `'আমরা'`; `F12` National `ami ` -> `'ৃলহ '`; `0` runtime log privacy leaks. |
| **`verify_gedit.py` (`Layer 6B`)** | **Passed** — active IM `bornomala`; typed `ami ` + `Ctrl+S`; saved file content `'আমি \n'`; `0` runtime log privacy leaks. |
| **Dual-Process Mode Inheritance (`AppA` -> new `AppB`)** | **Passed** — switching `F12` to National in `AppA` (`'ৃলহ '`) caused newly launched `AppB` to inherit National and emit `'ৃলহ '` without pressing `F12`. |
| **Dual-Process Two-Context XDG Config Reload** | **Passed** — with sequential focus confirmation (`PRE_FOC_A=1` -> `'আমি. '`, `PRE_FOC_B=1` -> `'আমি. '`), enabling `auto_convert_period_to_dari: true` in isolated `$XDG_CONFIG_HOME/com.bornomala.desktop/config.json` triggered independent `bornomala.cpp:436` reload logs in both processes and produced `POST_OUT_A=[আমি. আমি। ]` (`POST_FOC_A=1`) and `POST_OUT_B=[আমি. আমি। ]` (`POST_FOC_B=1`). |

---

## 5. Explicitly Unverified & Remaining Gaps

1. **Pointer Candidate Popup Mouse Selection Live Check:** Unverified live (`POPUP_WINS: []` / `POINTER_CLICK_DID_NOT_COMMIT` under WSLg XWayland; `xdotool` did not reliably reach a clickable Fcitx ClassicUI candidate popup window). Headless `SelectCandidateInContext` state cleanup is verified in CTest `EngineAdapterTest`, and `BornoCandidateWord::select()` compiles cleanly.
2. **GitHub-Hosted `ubuntu-24.04` Execution (`Run #11`, Unverified / Account-Blocked):** Pushing `de0149af60e6aec767efea0f476043db85b31771` on `2026-11-12` triggered GitHub Actions run `#11` ([`runs/43225585458`](https://github.com/asmkhalid111/BornoMala/actions/runs/43225585458)). GitHub marked the run failed after ~5 seconds with annotations stating all three jobs were **not started because the GitHub account was locked due to a billing issue**. No `ubuntu-24.04` runner steps executed; hosted Linux CI remains unverified (external account/billing blocker, not a code-level test failure).
3. **Native Linux Desktop Sessions:** Physical / non-WSL native Ubuntu desktop X11 and native Wayland (`waylandim`) sessions remain **unverified**.
4. **Other Toolkits & Distros:** Qt5, Qt6, GTK4 live typing tests, interactive manual `interactive_tester.py` / `launch_test_editor.sh` desktop checks, non-Ubuntu/Debian distributions, IBus (`Out of Scope`), and macOS (`Unknown / Deferred`) remain unverified.

---

## 6. M8 Acceptance Status Assessment

M8 cannot yet be declared `100%` / `Fully Verified` across native Linux desktops for `de0149a` because live ClassicUI candidate-popup mouse clicking, native desktop Wayland/X11 sessions outside WSLg, Qt/GTK4 live typing, and hosted `ubuntu-24.04` GitHub Actions execution (blocked before job start by billing lock on run [`#43225585458`](https://github.com/asmkhalid111/BornoMala/actions/runs/43225585458)) remain unverified. Current honest classification: **Partially Verified (Linux core, Release CMake/CTest, non-destructive installer/uninstaller, `16/16` script suite, local CI mirror, and isolated WSL2/WSLg GTK3/Gedit verified; hosted CI blocked pre-execution, and native Linux desktop, Wayland, and pointer popup click unverified).**

