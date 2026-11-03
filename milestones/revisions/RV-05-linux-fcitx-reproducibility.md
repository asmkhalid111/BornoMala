# RV-05 — Linux Fcitx 5 Reproducibility & Correctness

> **Revision ID:** RV-05
> **Status:** Proposed — awaiting developer approval
> **Priority:** P0 (blocking the Linux support claim)
> **Estimated effort:** 3–4 days
> **Dependencies:** RV-02 (ABI), RV-03 (config path)
> **Blocks:** RV-11, M8 re-verification, M13

---

## 1. Why this revision exists

The Fcitx adapter is the more idiomatic of the two native integrations, but its verification cannot be reproduced and it has real behaviour bugs:

| ID | Defect | Evidence |
|---|---|---|
| P0 | Build hardcodes `/home/asmkhalid/borno-linux-target/release/libbornomala_core.a`; fallback cannot exist because `install.sh` redirects `CARGO_TARGET_DIR` | [`CMakeLists.txt:15-22`](../../native/bornomala-fcitx5/CMakeLists.txt#L15-L22), [`install.sh:8,15`](../../native/bornomala-fcitx5/scripts/install.sh#L8-L15) |
| P0 | Release builds compile every `assert` out (no `-UNDEBUG`), so `ctest` passes vacuously | [`CMakeLists.txt:52-81`](../../native/bornomala-fcitx5/CMakeLists.txt#L52-L81), [`install.sh:20`](../../native/bornomala-fcitx5/scripts/install.sh#L20) |
| High | Per-context mode desync: property seeded from `m_globalMode` without `SWITCH_MODE`; second window gets wrong engine mode | [`bornomala.cpp:136-140`](../../native/bornomala-fcitx5/bornomala.cpp#L136-L140), `:463-476` |
| High | Escape/Return/Space/Backspace accepted even when the engine did not handle them | [`bornomala.cpp:241,255,275,289`](../../native/bornomala-fcitx5/bornomala.cpp#L241) |
| High | Mouse candidate selection never syncs property/engine | [`bornomala.cpp:89-106`](../../native/bornomala-fcitx5/bornomala.cpp#L89-L106) |
| High | Every keystroke's preedit and committed text logged at INFO | [`bornomala.cpp:184-186`](../../native/bornomala-fcitx5/bornomala.cpp#L184-L186), `:405-408` |
| Medium | Config path ignores `XDG_CONFIG_HOME` | [`bornomala.cpp:12-18`](../../native/bornomala-fcitx5/bornomala.cpp#L12-L18) |
| Medium | Scripts overwrite the user's Fcitx profile and `kill -9` the daemon | [`install.sh:52-76,88`](../../native/bornomala-fcitx5/scripts/install.sh#L52-L76) |
| Medium | CMake does not install the icon although the IM descriptor references it | [`CMakeLists.txt:88-90`](../../native/bornomala-fcitx5/CMakeLists.txt#L88-L90) |
| Contradicted | M8 says “Fully Verified”; M9 says “not verified on Linux host” | [`M8-completion-report.md:5`](../M8-completion-report.md#L5) vs [`M9-completion-report.md:217`](../M9-completion-report.md#L217) |

---

## 2. Current-state inspection

Read:

- `native/bornomala-fcitx5/CMakeLists.txt`, `bornomala.cpp`, `engine.h`, `key_mapper.cpp/.h`, `utf8_utils.h`
- `scripts/install.sh`, `uninstall.sh`, `interactive_tester.py`, `launch_test_editor.sh`
- `tests/*.cpp`, `tests/verify_gtk3.py`, `tests/verify_gedit.py`
- the M8 report's Layer 1–6 evidence

Record the current compiler/CMake/Fcitx versions used by the last “verified” run.

Baseline (in a Linux environment, WSL2 acceptable):

```bash
cmake -S native/bornomala-fcitx5 -B /tmp/borno-build -DCMAKE_BUILD_TYPE=Release
cmake --build /tmp/borno-build
ctest --test-dir /tmp/borno-build --output-on-failure
```

If the baseline fails for a new user, that is the first proof for this revision.

---

## 3. Scope

### Goals

1. A build that works for **any** user and in CI, with no hardcoded paths.
2. Tests that can actually fail: assertions effective in Release, or an explicit check macro.
3. Correct multi-window/multi-context behaviour.
4. Correct key acceptance semantics.
5. No typed content in logs.
6. Packaging that does not destroy the user's Fcitx configuration.
7. A reproducible evidence set for the Linux support claim.

### Non-goals

- No new Linux features.
- No IBus support.
- No Wayland-native redesign (record status, do not claim it).
- No installer (RV-11).

### Explicit no-go

- **No** `assert()`-only tests compiled with `NDEBUG`.
- **No** absolute paths from any developer machine.
- **No** `kill -9 fcitx5` in an install script.
- **No** claiming “verified on Wayland” from a `--disable waylandim` run.
- **No** logging of preedit/committed text at any level above debug.

---

## 4. Work packages

### WP-5.1 — Portable build

1. Remove the hardcoded path.
2. Resolve the static library in this order:
   - `-DBORNOMALA_CORE_LIB=<path>` if provided,
   - `cargo metadata --format-version 1 --no-deps` → `target_directory` → `release/libbornomala_core.a`,
   - fail with a message that includes the exact command to build it.
3. Make `install.sh` pass the resolved path instead of relying on `CARGO_TARGET_DIR` being discoverable.
4. Add `-DUNICODE`? Not applicable on Linux.
5. Ensure the Rust core is built with `-C relocation-model=pic` for the static archive used in a `.so` (document why).

Gate: a fresh clone in a clean container configures and builds.

### WP-5.2 — Effective tests

1. Add to every test target:
   ```cmake
   target_compile_options(test_x PRIVATE -UNDEBUG)
   ```
   or replace `assert` with a `CHECK(cond)` macro that prints and returns non-zero.
2. Add a “canary” test that deliberately fails when an env var is set, so CI can prove the harness detects failure (`BORNO_TEST_CANARY=1`).
3. CMake: add `enable_testing()` and register all tests (already partially present).
4. `test_engine_adapter` currently tests the core ABI, not the adapter; rename it to `test_core_abi` or make it instantiate the adapter. If instantiating needs a Fcitx instance, document that and keep the core test with an honest name.

Gate: `BORNO_TEST_CANARY=1 ctest` fails; normal `ctest` passes.

### WP-5.3 — Mode synchronisation

1. Make the engine the single source of truth for mode.
2. On `activate`, send `SWITCH_MODE` to match `m_globalMode` (or read the mode from the engine and set the property from it).
3. Add a two-context test: create context A, switch to Bijoy, create context B, assert B's engine mode and property agree.

Gate: two-window manual test produces correct characters in both.

### WP-5.4 — Key acceptance semantics

1. Accept only when `rc == BORNO_OK && res.handled`.
2. Escape/Return/Space/Backspace fall through when not handled.
3. Mouse selection: commit, reset the engine, sync the property (so `isComposing()` becomes false).
4. Ensure candidate digits require no modifier (already correct; keep and test).

Gate: Escape closes a dialog while composing; Backspace at idle reaches the app.

### WP-5.5 — Privacy

1. Remove preedit/commit logging; log only state transitions at debug level, gated by `FCITX_LOGGER` or an env var.
2. Grep the whole adapter for `preedit`, `commit`, `text` in log calls; assert none carry content.
3. Add a CI grep gate.

Gate: running the tester with logging enabled produces no Bangla text in the log.

### WP-5.6 — Packaging hygiene

1. CMake installs the icon, the addon descriptor, the IM descriptor, and a licence/notice file.
2. `install.sh` merges into `~/.config/fcitx5/profile` with a backup, or writes to a temporary profile for tests; it never overwrites a user's groups.
3. Replace `killall fcitx5` with `fcitx5-remote -r` (reload) or a graceful restart.
4. `interactive_tester.py` and the verify scripts must use a temporary `XDG_CONFIG_HOME` and restore it on exit.
5. Remove `scripts/__pycache__` from the tree.

Gate: install/uninstall on a clean user leaves the previous profile intact.

### WP-5.7 — CI and evidence

1. Add a GitHub Actions job on `ubuntu-24.04`:
   - install `fcitx5`, `libfcitx5core-dev`, `libfcitx5utils-dev`, `cmake`, `g++`, `pkg-config`, `python3-gi`,
   - build the Rust core with `relocation-model=pic`,
   - configure/build the addon,
   - run `ctest` (with `-UNDEBUG`),
   - run the canary negative check.
2. Optional, non-blocking: `xvfb-run` GTK smoke test. Mark it `continue-on-error` until stable; document why.
3. Record the WSL2 GUI verification separately, with the exact `FCITX_ADDON_DIRS` and `--disable waylandim` flag.

Gate: the job is green from a clean checkout; the canary run is red.

---

## 5. Support statement (replace the current M8 claim)

| Configuration | Status |
|---|---|
| Ubuntu 24.04/26.04 x86_64, Fcitx 5.1.x, X11/XWayland, WSLg | Verified (with evidence) |
| Same, native Wayland with `waylandim` | Unverified |
| Other distributions/desktops | Unverified |
| IBus | Out of scope |

Do not generalise beyond this table.

---

## 6. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | Fresh container builds without editing paths | CI log |
| AC-2 | Tests fail when a test is deliberately broken | canary run |
| AC-3 | Two-context mode test passes | test + manual |
| AC-4 | Unhandled keys fall through | manual |
| AC-5 | No typed content in logs | grep gate + log |
| AC-6 | Install preserves the user profile | before/after diff |
| AC-7 | CI job green | run link |
| AC-8 | Support table accurate | doc |

---

## 7. Testing plan

```bash
# In a clean Ubuntu container or WSL2
cargo build --release -p bornomala-core
cmake -S native/bornomala-fcitx5 -B /tmp/borno-build \
      -DCMAKE_BUILD_TYPE=Release \
      -DBORNOMALA_CORE_LIB="$(pwd)/target/release/libbornomala_core.a"
cmake --build /tmp/borno-build -j
ctest --test-dir /tmp/borno-build --output-on-failure
BORNO_TEST_CANARY=1 ctest --test-dir /tmp/borno-build --output-on-failure   # must fail
```

GUI (WSLg only, recorded): `verify_gtk3.py`, `verify_gedit.py` in a temporary `XDG_CONFIG_HOME`; capture the committed text.

---

## 8. Runnable checkpoint

- The addon builds and installs for a new user.
- The sandbox is unaffected.
- The user's Fcitx profile survives install/uninstall.

Rollback: revert the adapter commits; the previous build remains (with the same defects).

---

## 9. Evidence and reporting

`docs/evidence/RV-05/<date>-linux-repro.md`: CI link, container image, compiler versions, canary output, GUI run logs (no typed content except the test string), profile diff.

---

## 10. Roadmap update

- M8 status becomes `Partially verified (WSLg/X11 only)` until the CI job and GUI evidence exist.
- Add the support table to `docs/platform-support.md` (RV-09 creates the file).
- Record Wayland as unverified.

---

## 11. Stop conditions

- Fcitx 5 dev packages unavailable on the chosen runner → pin a container image, do not fake the job.
- The adapter cannot be tested without a real display → keep the GUI test non-blocking and state it.
- Mode truth requires an engine API that does not exist → route to RV-02 instead of duplicating state.

---

## 12. Handoff

RV-11 can package the addon. M8 may be re-signed only against this evidence.
