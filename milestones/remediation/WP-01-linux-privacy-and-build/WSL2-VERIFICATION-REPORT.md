# WSL2 Linux Build and Fcitx 5 Verification Report

> **Work Package:** WP-01 Follow-up  
> **Environment:** Windows 11 + WSL2 (Ubuntu 26.04.1 LTS x86_64, Kernel 6.18.54.1-microsoft-standard-WSL2, WSLg active)  
> **Verification Date:** 2026-10-26  
> **Auditor / Agent:** Antigravity AI  

---

## 1. Executive Summary

In accordance with user authorization and safeguards, Steps 1–4 of the Linux verification plan were executed in an isolated WSL2 Linux environment. The Rust core was successfully compiled with PIC for Linux, CMake cleanly auto-discovered the newly built library via the intended Cargo workspace root path without hardcoded user directories, the Fcitx 5 C++ addon compiled and linked cleanly, all 3 CTest native unit tests passed (100%), binary privacy string scans confirmed complete removal of sensitive logging, and the transient Fcitx 5 daemon successfully discovered and loaded the addon.

In accordance with **Safeguard #8**, interactive GUI testing (Step 5) was not forced because the repository's test harness scripts (`verify_gtk3.py` and `interactive_tester.py`) hardcode addon paths to `~/.local/lib/fcitx5/` and overwrite user profile configuration (`~/.config/fcitx5/profile`). Concrete options for non-destructive GUI verification are documented for user decision.

---

## 2. Repository State & Safeguards Compliance

### A. Pre-Build & Post-Build Git Status
- **Pre-Build Status:** 4 modified tracked files (`ffi_tests.rs`, `CMakeLists.txt`, `bornomala.cpp`, `install.sh`) and untracked remediation/revision directories.
- **Post-Build Status:** Exactly identical. All user changes, audit documentation, and remediation plans are fully preserved. No working code or master documents were modified or deleted.

```
Changes not staged for commit:
	modified:   crates/bornomala-core/tests/ffi_tests.rs
	modified:   native/bornomala-fcitx5/CMakeLists.txt
	modified:   native/bornomala-fcitx5/bornomala.cpp
	modified:   native/bornomala-fcitx5/scripts/install.sh

Untracked files:
	milestones/AUDIT-2026-10-23.md
	milestones/remediation/
	milestones/revisions/
```

### B. Build Directory Isolation
- Existing target folders and user directories were preserved.
- A completely fresh, isolated Linux-side CMake build directory was utilized:  
  `/home/asmkhalid/bornomala-fcitx5-clean-build`

---

## 3. Step-by-Step Verification Evidence

### Step 1: Rust Core Linux PIC Compilation
- **Command:**
  ```bash
  CARGO_TARGET_DIR="/mnt/e/Codes/Bangla Input Platform/Borno V1/target" \
  RUSTFLAGS="-C relocation-model=pic" \
  cargo build --release -p bornomala-core
  ```
- **Exit Code:** `0`
- **Output Artifact:**
  - Path: `/mnt/e/Codes/Bangla Input Platform/Borno V1/target/release/libbornomala_core.a`
  - Size: 26,238,354 bytes
  - Type: `current ar archive` containing 64-bit ELF objects (`x86-64`, relocatable, position-independent)
- **Technical Note:**
  The repository root contains `.cargo/config.toml` specifying `[build] target-dir = "C:/Users/AsmKhalid-HP15/.cargo-target/borno"`. When invoked under Linux Cargo without explicit target direction, Windows drive-letter colons (`:`) conflict with Linux path delimiters. Passing `CARGO_TARGET_DIR` explicitly cleanly overrides this configuration.

---

### Step 2: CMake Configuration & Static Library Discovery
- **Command:**
  ```bash
  cd /home/asmkhalid/bornomala-fcitx5-clean-build
  cmake "/mnt/e/Codes/Bangla Input Platform/Borno V1/native/bornomala-fcitx5"
  ```
- **Exit Code:** `0`
- **Resolution Output:**
  ```
  -- Using Cargo workspace target static library: /mnt/e/Codes/Bangla Input Platform/Borno V1/target/release/libbornomala_core.a
  -- Found libbornomala_core.a: /mnt/e/Codes/Bangla Input Platform/Borno V1/target/release/libbornomala_core.a
  -- Configuring done (0.2s)
  -- Generating done (0.0s)
  -- Build files have been written to: /home/asmkhalid/bornomala-fcitx5-clean-build
  ```
- **Discovery Branch Verification:**
  - Intended branch exercised: **Precedence 3** (`WORKSPACE_TARGET = ${CMAKE_CURRENT_SOURCE_DIR}/../../target/release/libbornomala_core.a`).
  - Confirmed: Selected artifact matches the newly compiled Linux PIC static archive.
  - No CMake files were altered to force this resolution.

---

### Step 3: Fcitx 5 C++ Addon Compilation & Dynamic Linkage
- **Command:**
  ```bash
  cd /home/asmkhalid/bornomala-fcitx5-clean-build
  make -j$(nproc)
  ```
- **Exit Code:** `0`
- **Output Artifact:**
  - Path: `/home/asmkhalid/bornomala-fcitx5-clean-build/bornomala.so`
  - Size: 7,903,440 bytes
  - Type: `ELF 64-bit LSB shared object, x86-64, version 1 (GNU/Linux), dynamically linked`
- **Dynamic Dependency Check (`ldd bornomala.so`):**
  - `libFcitx5Core.so.7 => /lib/x86_64-linux-gnu/libFcitx5Core.so.7`
  - `libFcitx5Config.so.6 => /lib/x86_64-linux-gnu/libFcitx5Config.so.6`
  - `libFcitx5Utils.so.2 => /lib/x86_64-linux-gnu/libFcitx5Utils.so.2`
  - `libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6`
  - Zero unresolved dynamic symbols or missing shared objects.

---

### Step 4: Native CTest Unit Suite Execution
- **Command:**
  ```bash
  cd /home/asmkhalid/bornomala-fcitx5-clean-build
  ctest --output-on-failure
  ```
- **Exit Code:** `0`
- **Results:**
  ```
  Test project /home/asmkhalid/bornomala-fcitx5-clean-build
      Start 1: CLinkageTest
  1/3 Test #1: CLinkageTest .....................   Passed    0.03 sec
      Start 2: KeyMapperTest
  2/3 Test #2: KeyMapperTest ....................   Passed    0.01 sec
      Start 3: EngineAdapterTest
  3/3 Test #3: EngineAdapterTest ................   Passed    0.03 sec

  100% tests passed, 0 tests failed out of 3
  Total Test time (real) =   0.08 sec
  ```
- **Scope Boundary:**  
  These native test binaries verify C ABI linkage, keysym mapping, and engine event processing in isolation. Per Safeguard #6, passing CTest does not constitute proof of active daemon session input handling.

---

### Step 5: Privacy Checks (Static Binary Scan & Daemon Loading)
- **Binary Content Scan:**
  ```bash
  strings /home/asmkhalid/bornomala-fcitx5-clean-build/bornomala.so | grep -E '(sym=0x|preedit=|committed=)'
  ```
  - **Exit Code:** `1` (Zero occurrences found).
  - Confirmed: Format strings containing raw keysyms (`sym=0x`), preedit text (`preedit='...'`), and committed text (`committed='...'`) have been completely purged from the compiled binary.
  - Remaining operational logs in binary:
    - `[Borno] Synchronized engine configuration from `
    - `[Borno] syncContextAndUI: handled=` (with `isComposing` and `candidates` counts only)
    - `[Borno] reset called`
- **Transient Daemon Loading Test:**
  ```bash
  FCITX_ADDON_DIRS="/home/asmkhalid/bornomala-fcitx5-clean-build:/usr/lib/x86_64-linux-gnu/fcitx5" \
  timeout 3s fcitx5 --verbose default=4 --disable waylandim
  ```
  - **Observed Daemon Output:**
    ```
    Loaded addon bornomala
    Found 1 input method(s) in addon bornomala
    Created classicui for x11 display::0
    ```
  - **Evidence:** Fcitx 5 successfully parsed the addon metadata (`bornomala.conf`), dynamically loaded `bornomala.so`, and registered the `bornomala` input method into the session.

---

## 4. Non-Destructive Isolated GUI Verification (Option A Execution)

Per user approval, **Option A: Non-Destructive GUI Verification** was executed using an isolated test harness (`borno_gui_test_runner.py`).

### A. Session & Configuration Isolation Architecture
- **Configuration Directory (`XDG_CONFIG_HOME`):** Directed to `/home/asmkhalid/borno-gui-verification-isolated/config`. The test profile was written here exclusively. The real user profile at `~/.config/fcitx5/profile` was completely untouched (timestamp remained `Oct 26 15:51`).
- **Data Directory (`XDG_DATA_HOME` & `XDG_DATA_DIRS`):** Pointed to `/home/asmkhalid/borno-gui-verification-isolated/data`, containing only copies of `bornomala.conf` and `bornomala-im.conf` from the clean build directory.
- **Addon Library Directory (`FCITX_ADDON_DIRS`):** Pointed exclusively to `/home/asmkhalid/bornomala-fcitx5-clean-build:/usr/lib/x86_64-linux-gnu/fcitx5`. The stale binary at `~/.local/lib/fcitx5/bornomala.so` was completely excluded and was not touched (timestamp remained `Oct 19 20:01`).
- **Active Binary Verification:**  
  Inspected `/proc/<pid>/maps` for the running test Fcitx 5 process:
  ```
  7e8072200000-7e807226a000 r--p /home/asmkhalid/bornomala-fcitx5-clean-build/bornomala.so
  7e807226a000-7e807236e000 r-xp /home/asmkhalid/bornomala-fcitx5-clean-build/bornomala.so
  7e807236e000-7e807250a000 r--p /home/asmkhalid/bornomala-fcitx5-clean-build/bornomala.so
  7e807250a000-7e8072516000 r--p /home/asmkhalid/bornomala-fcitx5-clean-build/bornomala.so
  7e8072516000-7e8072518000 rw-p /home/asmkhalid/bornomala-fcitx5-clean-build/bornomala.so
  ```
  **Confirmed:** The test daemon loaded the freshly compiled Linux artifact, not the legacy user-mode binary.

### B. Interactive GUI Test Results

A native GTK3 test window (`Gtk.Window` containing `Gtk.Entry`, backend X11 under WSLg `DISPLAY=:0`) was spawned. Keystroke injection was performed via `xdotool` with intermediate entry text sampled after every event.

#### Test Case 1: Standard Phonetic Input & Commit
- **Input sequence:** `a` `m` `i` `space`
- **Preedit trace:**
  - After `'a'`: entry text = `""` (character held in preedit composition)
  - After `'m'`: entry text = `""` (character held in preedit composition)
  - After `'i'`: entry text = `""` (character held in preedit composition)
  - After `'space'`: entry text = `"আমি "` (committed by engine)
- **Result:** **Passed** (`output = "আমি "`).

#### Test Case 2: Composition & Backspace Editing
- **Input sequence:** `k` `o` `BackSpace` `space`
- **Preedit trace:**
  - After `'k'`: entry text = `""`
  - After `'o'`: entry text = `""`
  - After `'BackSpace'`: entry text = `""` (removes trailing 'o' from preedit buffer)
  - After `'space'`: entry text = `"ক "` (commits remaining 'k')
- **Result:** **Passed** (`output = "ক "`).

#### Test Case 3A: Mode Switch to National Layout (1x F12)
- **Mechanism:** In `bornomala.cpp:L199`, F12 cycles `BORNO_MODE_PHONETIC` -> `BORNO_MODE_FIXED_NATIONAL`.
- **Input sequence:** `F12`, then `a` `m` `i` `space`
- **Result:** **Passed** (`output = "ৃলহ "`, exactly matching National/Jatiya character positions: 'a'='ৃ', 'm'='ল', 'i'='হ').

#### Test Case 3B: Cycle to English Passthrough Mode (3x F12)
- **Mechanism:** F12 cycles National -> Probhat -> Bijoy -> Passthrough.
- **Input sequence:** 3x `F12`, then `a` `m` `i` `space`
- **Trace:**
  - After `'a'`: entry text = `"a"`
  - After `'m'`: entry text = `"am"`
  - After `'i'`: entry text = `"ami"`
  - After `'space'`: entry text = `"ami "`
- **Result:** **Passed** (unmodified Latin characters pass through directly without composition).

---

## 5. Runtime Privacy Verification

During active interactive typing, preedit composition, backspace editing, mode switches, and string commits, Fcitx 5 logs were captured with verbose logging (`default=4,bornomala=5`) into `/home/asmkhalid/borno-gui-verification-isolated/fcitx5_gui.log`.

### A. Privacy Scan
- **Command:**
  ```bash
  grep -E '(sym=0x|preedit=|committed=)' /home/asmkhalid/borno-gui-verification-isolated/fcitx5_gui.log
  ```
- **Exit Code:** `1` (Zero matches).
- **Confirmation:** Zero raw keysyms (`sym=0x`), zero preedit buffer strings (`preedit='...'`), and zero committed string payloads (`committed='...'`) were emitted during the entire GUI typing session.

### B. Observed Operational Logs
Logs confirmed normal operation using only privacy-preserving metrics:
```
[Borno] syncContextAndUI: handled=1, isComposing=1, candidates=1
[Borno] syncContextAndUI: handled=1, isComposing=1, candidates=4
[Borno] syncContextAndUI: handled=1, isComposing=1, candidates=4
[Borno] syncContextAndUI: handled=1, isComposing=0, candidates=0
[Borno] reset called
```

---

## 6. Final Verification Status Matrix

| Verification Layer | Status | Executable Evidence |
| :--- | :--- | :--- |
| **Rust Core Linux PIC Build** | **Verified** | Built `target/release/libbornomala_core.a` (26 MB, ELF 64-bit ar archive, exit code 0) |
| **CMake Discovery & Configuration** | **Verified** | Precedence 3 cleanly auto-discovered workspace root target without hardcoded user paths (exit code 0) |
| **Fcitx 5 C++ Addon Compilation** | **Verified** | Built `bornomala.so` (7.9 MB shared object), `ldd` resolved all symbols (exit code 0) |
| **Addon Unit & Linkage Tests** | **Verified** | All 3 CTest tests passed (100%, 0.08s total runtime, exit code 0) |
| **Fcitx 5 Daemon Addon Loading** | **Verified** | Daemon logged `Loaded addon bornomala`, registered 1 input method, verified via `/proc/<pid>/maps` |
| **Static / Binary Privacy Scan** | **Verified** | `strings bornomala.so` confirmed 0 matches for `sym=0x`, `preedit=`, `committed=` |
| **GUI Live Keystroke & Typing** | **Verified** | Phonetic input (`"আমি "`), Backspace editing (`"ক "`), National layout (`"ৃলহ "`), and English passthrough (`"ami "`) verified |
| **Runtime Keystroke Privacy Log** | **Verified** | Zero occurrences of keysyms, preedit text, or committed text in live runtime logs |
| **Overall Linux Integration** | **Verified** | Full end-to-end integration verified under WSL2/WSLg with zero system/session pollution |

---

## 7. Session & Repository Preservation Audit

- **User Profile (`~/.config/fcitx5/profile`):** Untouched (timestamp `Oct 26 15:51`).
- **User Library (`~/.local/lib/fcitx5/bornomala.so`):** Untouched (timestamp `Oct 19 20:01`).
- **System Processes:** No lingering Fcitx processes. Daemon terminated cleanly on test exit.
- **Git Repository:** Zero unintended modifications. The 4 approved WP-01 files remain identical to the pre-test baseline.
