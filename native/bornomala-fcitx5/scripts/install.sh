#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
FCITX_SRC_DIR="$PROJECT_ROOT/native/bornomala-fcitx5"
NOTICES_FILE="$PROJECT_ROOT/THIRD-PARTY-NOTICES.md"
ICON_SRC="$FCITX_SRC_DIR/data/bornomala.png"

# shellcheck source=native/bornomala-fcitx5/scripts/common.sh
source "$SCRIPT_DIR/common.sh"

usage() {
    cat << 'EOF'
Usage: install.sh [OPTIONS]

Non-destructive user-mode installer for the BornoMala Fcitx 5 addon.
By default, this script does NOT modify your Fcitx profile, shell rc files,
or stop/restart any running Fcitx 5 daemon.

Options:
  --build-dir DIR       Use DIR for CMake build artifacts (default: $XDG_CACHE_HOME/bornomala/fcitx5-build)
  --skip-build          Skip cargo/cmake build steps and install prebuilt artifacts from --build-dir
  --register-profile    Opt-in: safely append bornomala to Fcitx profile after creating a backup
  --reload-fcitx        Opt-in: run 'fcitx5-remote --check -r' if Fcitx is running on this session bus
  -h, --help            Show this help message and exit
EOF
}

BUILD_DIR=""
SKIP_BUILD=0
REGISTER_PROFILE=0
RELOAD_FCITX=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --build-dir)
            if [[ $# -lt 2 || -z "${2:-}" ]]; then
                echo "Error: --build-dir requires a directory argument." >&2
                exit 2
            fi
            BUILD_DIR="$2"
            shift 2
            ;;
        --skip-build|--no-build)
            SKIP_BUILD=1
            shift
            ;;
        --register-profile)
            REGISTER_PROFILE=1
            shift
            ;;
        --reload-fcitx)
            RELOAD_FCITX=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Error: unknown option '$1'." >&2
            usage >&2
            exit 2
            ;;
    esac
done

borno_init_user_paths

if [[ -z "$BUILD_DIR" ]]; then
    BUILD_DIR="${BORNO_FCITX5_BUILD_DIR:-${BORNO_XDG_CACHE_HOME}/bornomala/fcitx5-build}"
fi

# 1. Pre-flight validation BEFORE creating directories or modifying anything
if [[ ! -f "$NOTICES_FILE" || ! -r "$NOTICES_FILE" || ! -s "$NOTICES_FILE" ]]; then
    echo "Error: Required readable third-party notices file not found at: $NOTICES_FILE" >&2
    exit 1
fi

if [[ ! -f "$ICON_SRC" || ! -r "$ICON_SRC" || ! -s "$ICON_SRC" ]]; then
    echo "Error: Required application icon not found at: $ICON_SRC" >&2
    exit 1
fi

if [[ "$SKIP_BUILD" -eq 1 ]]; then
    if ! borno_is_valid_elf_module "$BUILD_DIR/bornomala.so"; then
        echo "Error: Required ELF addon module not found or invalid at: $BUILD_DIR/bornomala.so" >&2
        exit 1
    fi
    for req_file in "$BUILD_DIR/bornomala.conf" "$BUILD_DIR/bornomala-im.conf"; do
        if [[ ! -f "$req_file" || ! -r "$req_file" || ! -s "$req_file" ]]; then
            echo "Error: Required build descriptor missing or empty: $req_file" >&2
            exit 1
        fi
    done
else
    for tool in cargo cmake make; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            echo "Error: Required build tool '$tool' not found in PATH." >&2
            exit 1
        fi
    done
fi

echo "=== BornoMala Fcitx 5 Safe User-Mode Installer ==="
echo "  User addon lib : $BORNO_ADDON_SO_DEST"
echo "  XDG data home  : $BORNO_XDG_DATA_HOME"
echo "  XDG config home: $BORNO_XDG_CONFIG_HOME"

# 2. Build (unless --skip-build)
if [[ "$SKIP_BUILD" -eq 0 ]]; then
    TARGET_DIR="${CARGO_TARGET_DIR:-${PROJECT_ROOT}/target}"
    echo "[1/3] Building bornomala-core static library (PIC)..."
    mkdir -p "$TARGET_DIR"
    (cd "$PROJECT_ROOT" && CARGO_TARGET_DIR="$TARGET_DIR" RUSTFLAGS="-C relocation-model=pic" cargo build --release -p bornomala-core)

    echo "[2/3] Building Fcitx 5 addon in $BUILD_DIR..."
    mkdir -p "$BUILD_DIR"
    (cd "$BUILD_DIR" && cmake -DCMAKE_BUILD_TYPE=Release -DBORNOMALA_CORE_STATIC="${TARGET_DIR}/release/libbornomala_core.a" "$FCITX_SRC_DIR" && make -j"$(nproc)")

    if ! borno_is_valid_elf_module "$BUILD_DIR/bornomala.so"; then
        echo "Error: Built addon module missing or invalid: $BUILD_DIR/bornomala.so" >&2
        exit 1
    fi
    for req_file in "$BUILD_DIR/bornomala.conf" "$BUILD_DIR/bornomala-im.conf"; do
        if [[ ! -f "$req_file" || ! -r "$req_file" || ! -s "$req_file" ]]; then
            echo "Error: Built descriptor missing or empty: $req_file" >&2
            exit 1
        fi
    done
else
    echo "[1/3] Using validated prebuilt artifacts from: $BUILD_DIR"
fi

# Helper: atomic idempotent install using temporary file in target directory + cmp check
atomic_install_file() {
    local src="$1"
    local dest="$2"
    local mode="$3"
    local dest_dir
    dest_dir="$(dirname "$dest")"

    if [[ -L "$dest" ]]; then
        echo "Error: refusing to overwrite symbolic link destination: $dest" >&2
        return 1
    fi

    mkdir -p "$dest_dir"
    if [[ -f "$dest" ]] && cmp -s "$src" "$dest"; then
        chmod "$mode" "$dest"
        echo "  [unchanged] $dest"
        return 0
    fi

    local tmp
    tmp="$(mktemp "${dest_dir}/.bornomala-install.XXXXXX")"
    cp -- "$src" "$tmp"
    chmod "$mode" "$tmp"
    mv -f -- "$tmp" "$dest"
    echo "  [installed] $dest"
}

echo "[3/3] Installing BornoMala-owned user-mode artifacts..."
atomic_install_file "$BUILD_DIR/bornomala.so" "$BORNO_ADDON_SO_DEST" 0755
atomic_install_file "$BUILD_DIR/bornomala.conf" "$BORNO_ADDON_CONF_DEST" 0644
atomic_install_file "$BUILD_DIR/bornomala-im.conf" "$BORNO_IM_CONF_DEST" 0644
atomic_install_file "$NOTICES_FILE" "$BORNO_NOTICES_DEST" 0644

# Preserve an existing custom non-BornoMala icon unless we already own it
ICON_SRC_SHA="$(borno_file_sha256 "$ICON_SRC")"
ICON_OWNED=1
if [[ -e "$BORNO_ICON_DEST" ]]; then
    if [[ -L "$BORNO_ICON_DEST" ]]; then
        echo "Warning: preserving symlinked icon at $BORNO_ICON_DEST (not claimed by BornoMala)." >&2
        ICON_OWNED=0
    else
        EXISTING_ICON_SHA="$(borno_file_sha256 "$BORNO_ICON_DEST")"
        PREV_RECORDED_SHA=""
        if [[ -f "$BORNO_MANIFEST_DEST" ]]; then
            PREV_RECORDED_SHA="$(awk -F'|' -v p="$BORNO_ICON_DEST" '$1=="ICON_OWNED" && $3==p {print $2}' "$BORNO_MANIFEST_DEST" | head -n 1 || true)"
        fi
        if [[ "$EXISTING_ICON_SHA" != "$ICON_SRC_SHA" && "$EXISTING_ICON_SHA" != "$PREV_RECORDED_SHA" ]]; then
            echo "Warning: preserving unrelated pre-existing icon at $BORNO_ICON_DEST (different checksum)." >&2
            ICON_OWNED=0
        fi
    fi
fi

if [[ "$ICON_OWNED" -eq 1 ]]; then
    atomic_install_file "$ICON_SRC" "$BORNO_ICON_DEST" 0644
fi

# Record exact ownership manifest atomically
MANIFEST_TMP="$(mktemp "${BORNO_DOC_DIR}/.bornomala-manifest.XXXXXX")"
{
    printf '# BornoMala Fcitx 5 User-Mode Ownership Manifest v1\n'
    printf 'FILE|%s|%s\n' "$(borno_file_sha256 "$BORNO_ADDON_SO_DEST")" "$BORNO_ADDON_SO_DEST"
    printf 'FILE|%s|%s\n' "$(borno_file_sha256 "$BORNO_ADDON_CONF_DEST")" "$BORNO_ADDON_CONF_DEST"
    printf 'FILE|%s|%s\n' "$(borno_file_sha256 "$BORNO_IM_CONF_DEST")" "$BORNO_IM_CONF_DEST"
    printf 'FILE|%s|%s\n' "$(borno_file_sha256 "$BORNO_NOTICES_DEST")" "$BORNO_NOTICES_DEST"
    if [[ "$ICON_OWNED" -eq 1 ]]; then
        printf 'ICON_OWNED|%s|%s\n' "$(borno_file_sha256 "$BORNO_ICON_DEST")" "$BORNO_ICON_DEST"
    else
        printf 'ICON_PRESERVED|-|%s\n' "$BORNO_ICON_DEST"
    fi
} > "$MANIFEST_TMP"
atomic_install_file "$MANIFEST_TMP" "$BORNO_MANIFEST_DEST" 0644
rm -f -- "$MANIFEST_TMP"

# Opt-in profile registration (never touches profile by default)
if [[ "$REGISTER_PROFILE" -eq 1 ]]; then
    echo "Applying opt-in non-destructive Fcitx 5 profile update..."
    python3 -B - "$BORNO_FCITX_PROFILE_PATH" << 'PYEOF'
import os
import re
import shutil
import sys
import tempfile
import time

profile_path = sys.argv[1]
profile_dir = os.path.dirname(profile_path)
os.makedirs(profile_dir, exist_ok=True)

if not os.path.exists(profile_path):
    content = (
        "[Groups/0]\n"
        "Name=Default\n"
        "Default Layout=us\n"
        "DefaultIM=keyboard-us\n\n"
        "[Groups/0/Items/0]\n"
        "Name=keyboard-us\n"
        "Layout=\n\n"
        "[Groups/0/Items/1]\n"
        "Name=bornomala\n"
        "Layout=\n\n"
        "[GroupOrder]\n"
        "0=Default\n"
    )
    fd, tmp_path = tempfile.mkstemp(prefix=".profile-tmp-", dir=profile_dir)
    with os.fdopen(fd, "w", encoding="utf-8") as f:
        f.write(content)
    os.chmod(tmp_path, 0o600)
    os.replace(tmp_path, profile_path)
    print(f"  [profile] created new profile with bornomala at {profile_path}")
    sys.exit(0)

if os.path.islink(profile_path):
    print(f"Error: refusing to modify symlinked profile: {profile_path}", file=sys.stderr)
    sys.exit(1)

with open(profile_path, "r", encoding="utf-8") as f:
    raw = f.read()

# Check if bornomala is already registered in any group item
if re.search(r"(?m)^\s*Name\s*=\s*bornomala\s*$", raw):
    print(f"  [profile] bornomala already registered in {profile_path}; left unchanged.")
    sys.exit(0)

backup_path = f"{profile_path}.bak.{time.strftime('%Y%m%d-%H%M%S')}"
shutil.copy2(profile_path, backup_path)
print(f"  [profile] backup created at {backup_path}")

# Discover target group index: prefer Group 0 if present, otherwise first [Groups/<id>]
group_ids = re.findall(r"(?m)^\[Groups/([^\]/]+)\]\s*$", raw)
target_group = "0" if "0" in group_ids else (group_ids[0] if group_ids else "0")

item_pattern = re.compile(rf"(?m)^\[Groups/{re.escape(target_group)}/Items/(\d+)\]\s*$")
existing_indices = [int(m.group(1)) for m in item_pattern.finditer(raw)]
next_idx = (max(existing_indices) + 1) if existing_indices else 0

block = f"\n[Groups/{target_group}/Items/{next_idx}]\nName=bornomala\nLayout=\n"
group_order_match = re.search(r"(?m)^\[GroupOrder\]\s*$", raw)
if group_order_match:
    pos = group_order_match.start()
    updated = raw[:pos].rstrip() + "\n" + block + "\n" + raw[pos:]
else:
    if not group_ids:
        updated = raw.rstrip() + f"\n\n[Groups/{target_group}]\nName=Default\nDefault Layout=us\n" + block
    else:
        updated = raw.rstrip() + "\n" + block

fd, tmp_path = tempfile.mkstemp(prefix=".profile-tmp-", dir=profile_dir)
with os.fdopen(fd, "w", encoding="utf-8") as f:
    f.write(updated)
st_mode = os.stat(profile_path).st_mode & 0o777
os.chmod(tmp_path, st_mode)
os.replace(tmp_path, profile_path)
print(f"  [profile] added [Groups/{target_group}/Items/{next_idx}] Name=bornomala to {profile_path}")
PYEOF
else
    echo "  [profile] Left ${BORNO_FCITX_PROFILE_PATH} untouched."
fi

# Opt-in graceful Fcitx 5 reload (never kills processes)
if [[ "$RELOAD_FCITX" -eq 1 ]]; then
    if command -v fcitx5-remote >/dev/null 2>&1; then
        if fcitx5-remote --check >/dev/null 2>&1; then
            fcitx5-remote -r
            echo "  [fcitx5] Reloaded running Fcitx 5 instance via fcitx5-remote -r."
        else
            echo "  [fcitx5] No running Fcitx 5 instance detected on current D-Bus session."
        fi
    else
        echo "Warning: --reload-fcitx requested, but fcitx5-remote was not found." >&2
    fi
fi

cat << EOF

=== BornoMala Fcitx 5 User-Mode Installation Complete ===
Next manual steps to use BornoMala in your desktop session:
  1. Ensure Fcitx 5 can locate user addons by exporting:
       export FCITX_ADDON_DIRS="/usr/lib/x86_64-linux-gnu/fcitx5:${BORNO_USER_LIB_DIR}"
  2. Enable "BornoMala" (বর্ণমালা) via fcitx5-configtool (or re-run with --register-profile).
  3. Reload Fcitx 5 cleanly when ready: fcitx5-remote -r  (or fcitx5 -r -d).
EOF
