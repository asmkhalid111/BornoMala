#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
FCITX_SRC_DIR="$PROJECT_ROOT/native/bornomala-fcitx5"
BUILD_DIR="${HOME}/bornomala-fcitx5-build"
TARGET_DIR="${HOME}/borno-linux-target"

echo "=== BornoMala Fcitx 5 User-Mode Installer ==="

# 1. Ensure Rust core is built with PIC
echo "[1/5] Building bornomala-core static library (PIC)..."
mkdir -p "$TARGET_DIR"
(cd "$PROJECT_ROOT" && CARGO_TARGET_DIR="$TARGET_DIR" RUSTFLAGS="-C relocation-model=pic" cargo build --release -p bornomala-core)

# 2. Build Fcitx 5 addon using CMake
echo "[2/5] Building Fcitx 5 addon..."
mkdir -p "$BUILD_DIR"
(cd "$BUILD_DIR" && cmake -DCMAKE_BUILD_TYPE=Release "$FCITX_SRC_DIR" && make -j"$(nproc)")

# 3. Create destination user directories
echo "[3/5] Setting up user directories in ~/.local/..."
mkdir -p "${HOME}/.local/lib/fcitx5"
mkdir -p "${HOME}/.local/share/fcitx5/addon"
mkdir -p "${HOME}/.local/share/fcitx5/inputmethod"
mkdir -p "${HOME}/.local/share/icons/hicolor/64x64/apps"

# 4. Copy addon artifacts
echo "[4/5] Installing addon artifacts..."
cp -f "$BUILD_DIR/bornomala.so" "${HOME}/.local/lib/fcitx5/bornomala.so"
cp -f "$BUILD_DIR/bornomala.conf" "${HOME}/.local/share/fcitx5/addon/bornomala.conf"
cp -f "$BUILD_DIR/bornomala-im.conf" "${HOME}/.local/share/fcitx5/inputmethod/bornomala.conf"
if [[ -f "$FCITX_SRC_DIR/data/bornomala.png" ]]; then
    cp -f "$FCITX_SRC_DIR/data/bornomala.png" "${HOME}/.local/share/icons/hicolor/64x64/apps/bornomala.png"
fi

# 5. Environment & Daemon reload
echo "[5/5] Configuring environment and reloading Fcitx 5..."
SYS_ADDON_DIR="/usr/lib/x86_64-linux-gnu/fcitx5"
USER_ADDON_DIR="${HOME}/.local/lib/fcitx5"
export FCITX_ADDON_DIRS="${SYS_ADDON_DIR}:${USER_ADDON_DIR}:${FCITX_ADDON_DIRS:-}"

# Export FCITX_ADDON_DIRS in ~/.profile if not present
if ! grep -q "FCITX_ADDON_DIRS.*\.local/lib/fcitx5" "${HOME}/.profile" 2>/dev/null; then
    echo "export FCITX_ADDON_DIRS=\"${SYS_ADDON_DIR}:${USER_ADDON_DIR}:\${FCITX_ADDON_DIRS:-}\"" >> "${HOME}/.profile"
fi

# Ensure Fcitx 5 profile includes bornomala
mkdir -p "${HOME}/.config/fcitx5"
PROFILE_PATH="${HOME}/.config/fcitx5/profile"
if [[ -f "$PROFILE_PATH" ]]; then
    if ! grep -q "bornomala" "$PROFILE_PATH"; then
        echo "Adding bornomala to existing fcitx5 profile..."
        # Add to Group 0
        sed -i '/\[Groups\/0\/Items\/0\]/i [Groups/0/Items/99]\nName=bornomala\nLayout=\n' "$PROFILE_PATH" || true
    fi
else
    cat << 'EOF' > "$PROFILE_PATH"
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=bornomala

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[Groups/0/Items/1]
Name=bornomala
Layout=

[GroupOrder]
0=Default
EOF
fi

# Restart daemon with addon path
export DISPLAY="${DISPLAY:-:0}"
export DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-unix:path=/run/user/$(id -u)/bus}"
if [[ -d "/mnt/wslg/runtime-dir" ]]; then
    export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/mnt/wslg/runtime-dir}"
    FCITX_EXTRA_ARGS="--disable waylandim"
else
    FCITX_EXTRA_ARGS=""
fi

killall fcitx5 2>/dev/null || true
sleep 1
fcitx5 ${FCITX_EXTRA_ARGS} -d >"${HOME}/fcitx.log" 2>&1 || true

echo "=== BornoMala Fcitx 5 Addon successfully installed! ==="
