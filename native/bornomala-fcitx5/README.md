# BornoMala — Linux Fcitx 5 Native Addon (`bornomala-fcitx5`)

Native C++20 Fcitx 5 `InputMethodEngineV2` addon statically linking the BornoMala Rust core (`libbornomala_core.a` built with `-C relocation-model=pic`).

---

## Tested & Supported Scope

- **Tested Baseline Environment (WSL2/XWayland):** Ubuntu 26.04.1 LTS (x86_64), Fcitx 5 (`5.1.19`), `cmake` `4.2+`, `g++` `15.2+` (C++20), `cargo`/`rustc`, GTK3 under X11/XWayland (`DISPLAY=:0`, `fcitx5 --disable waylandim`).
- **Known Limitations:**
  - Native Wayland compositor IME (`waylandim`) and non-Ubuntu/non-Debian distributions remain unverified.
  - Candidate popup UI appearance is host-controlled by your desktop Fcitx 5 UI theme (ClassicUI / Kimpanel). Under WSLg XWayland, ClassicUI candidate popups are not exposed as normal queryable X11 windows to `xdotool`.
  - Automated Linux Fcitx 5 CI is configured in `.github/workflows/ci.yml` for Ubuntu 24.04. A successful GitHub-hosted run for this revision has not yet been verified.

---

## Prerequisites (Ubuntu / Debian)

```bash
sudo apt install cmake g++ pkg-config make \
  fcitx5 libfcitx5core-dev libfcitx5config-dev libfcitx5utils-dev fcitx5-modules-dev
```

Optional for isolated GUI verification:

```bash
sudo apt install python3-gi gir1.2-gtk-3.0 xdotool gedit
```

---

## Building Without Installing

From the repository root (ensure Rust is on `PATH`, e.g. `export PATH="$HOME/.cargo/bin:$PATH"`):

```bash
export CARGO_TARGET_DIR=/tmp/bornomala-m8-wp1-cargo
export RUSTFLAGS="-C relocation-model=pic"
cargo build --release -p bornomala-core

cmake -S native/bornomala-fcitx5 \
      -B /tmp/bornomala-m8-wp1-cmake \
      -DCMAKE_BUILD_TYPE=Release \
      -DBORNOMALA_CORE_STATIC="${CARGO_TARGET_DIR}/release/libbornomala_core.a"

cmake --build /tmp/bornomala-m8-wp1-cmake --parallel
ctest --test-dir /tmp/bornomala-m8-wp1-cmake --output-on-failure
```

Verify staged installation into a temporary root (`DESTDIR`):

```bash
DESTDIR=/tmp/bornomala-m8-wp1-install cmake --install /tmp/bornomala-m8-wp1-cmake
```

---

## Safe User-Mode Installation

[`scripts/install.sh`](scripts/install.sh) installs strictly into user-owned locations without running `sudo`, modifying `~/.profile`, editing your Fcitx profile, or killing/restarting running Fcitx 5 processes by default:

| Artifact | Destination |
|---|---|
| Addon module | `${HOME}/.local/lib/fcitx5/bornomala.so` |
| Addon metadata | `${XDG_DATA_HOME:-$HOME/.local/share}/fcitx5/addon/bornomala.conf` |
| Input method descriptor | `${XDG_DATA_HOME:-$HOME/.local/share}/fcitx5/inputmethod/bornomala.conf` |
| Icon (64×64 PNG) | `${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/64x64/apps/bornomala.png` |
| Notices & ownership manifest | `${XDG_DATA_HOME:-$HOME/.local/share}/doc/bornomala/{THIRD-PARTY-NOTICES.md,install-manifest.txt}` |

Run directly (or pass `--skip-build --build-dir /tmp/bornomala-m8-wp1-cmake` to install an existing build):

```bash
bash native/bornomala-fcitx5/scripts/install.sh
```

### Enabling and Reloading Manually

1. Ensure Fcitx 5 includes your user addon directory:
   ```bash
   export FCITX_ADDON_DIRS="/usr/lib/x86_64-linux-gnu/fcitx5:${HOME}/.local/lib/fcitx5"
   ```
2. Open **Fcitx 5 Configuration** (`fcitx5-configtool`) and add **BornoMala** (`বর্ণমালা`).
   *(Optional opt-in: `install.sh --register-profile` creates a `.bak` backup and safely appends `bornomala` to your first Fcitx profile group without discarding existing entries.)*
3. Reload Fcitx 5 manually when ready:
   ```bash
   fcitx5-remote -r
   ```
   *(Or restart in background with `fcitx5 -r -d`; never use `killall`/`pkill -9` on active desktop sessions.)*

Engine runtime settings are read from `${XDG_CONFIG_HOME:-$HOME/.config}/com.bornomala.desktop/config.json`.

---

## Safe Uninstallation

[`scripts/uninstall.sh`](scripts/uninstall.sh) removes **only** BornoMala-owned files validated against an explicit destination allowlist, never edits or removes your Fcitx profile, and never kills/restarts running Fcitx 5 processes.

```bash
# Preview every planned removal without changing files
bash native/bornomala-fcitx5/scripts/uninstall.sh --dry-run

# Execute confirmed non-interactive removal
bash native/bornomala-fcitx5/scripts/uninstall.sh --yes
```

---

## Isolated GUI Testing (Non-Destructive)

All GUI test harnesses create temporary `/tmp/bornomala-fcitx5-*` directories for `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, and `XDG_CACHE_HOME`, launch a private `dbus-daemon --session` and foreground `fcitx5` child tracked by PID, load the freshly built WP1 `bornomala.so` via `--build-dir` (or `/tmp/bornomala-m8-wp1-cmake`), verify `/proc/<pid>/maps`, never touch `~/.config/fcitx5/profile` or GNOME `gsettings`, and clean up only their own child PIDs and temporary directory on exit.

```bash
# Automated GTK3 verification
python3 -B native/bornomala-fcitx5/tests/verify_gtk3.py --build-dir /tmp/bornomala-m8-wp1-cmake

# Automated GNOME Gedit verification
python3 -B native/bornomala-fcitx5/tests/verify_gedit.py --build-dir /tmp/bornomala-m8-wp1-cmake

# Interactive GTK3 tester (or Launch-Linux-Fcitx5-Test.bat on Windows)
python3 -B native/bornomala-fcitx5/scripts/interactive_tester.py --build-dir /tmp/bornomala-m8-wp1-cmake

# Interactive standalone Gedit session (or Launch-Linux-Gedit-Test.bat on Windows)
bash native/bornomala-fcitx5/scripts/launch_test_editor.sh --build-dir /tmp/bornomala-m8-wp1-cmake

# Automated safety tests for installer, uninstaller, and GUI harness
python3 -B -m unittest discover -s native/bornomala-fcitx5/tests -p 'test_scripts.py' -v
```

Pass `--keep-temp` to retain temporary `/tmp/bornomala-fcitx5-*` logs for inspection; delete retained directories manually with `rm -rf /tmp/bornomala-fcitx5-*`. Missing display or GUI tools exit with code `77` (`RESULT=UNAVAILABLE`) rather than passing vacuously.

