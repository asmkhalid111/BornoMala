#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

SESSION_TMP="${SESSION_TMP:-}"
KEEP_TEMP="${KEEP_TEMP:-0}"
DBUS_PID="${DBUS_PID:-}"
DBUS_COMM="${DBUS_COMM:-}"
DBUS_STARTTIME="${DBUS_STARTTIME:-}"
FCITX_PID="${FCITX_PID:-}"
FCITX_COMM="${FCITX_COMM:-}"
FCITX_STARTTIME="${FCITX_STARTTIME:-}"
GEDIT_PID="${GEDIT_PID:-}"
GEDIT_COMM="${GEDIT_COMM:-}"
GEDIT_STARTTIME="${GEDIT_STARTTIME:-}"

BORNO_PROC_STATE=""
BORNO_PROC_COMM=""
BORNO_PROC_STARTTIME=""
BORNO_CAPTURED_COMM=""
BORNO_CAPTURED_STARTTIME=""

borno_editor_read_proc_identity() {
    local pid="$1"
    BORNO_PROC_STATE=""
    BORNO_PROC_COMM=""
    BORNO_PROC_STARTTIME=""

    [[ -n "$pid" && "$pid" =~ ^[0-9]+$ && "$pid" -gt 1 ]] || return 1
    [[ -r "/proc/${pid}/stat" && -r "/proc/${pid}/comm" ]] || return 1

    local stat_line=""
    local comm_val=""
    IFS= read -r stat_line < "/proc/${pid}/stat" || return 1
    IFS= read -r comm_val < "/proc/${pid}/comm" || return 1

    # /proc/<pid>/stat field 2 is (comm), which may contain spaces; slice after the last ") ".
    local rest="${stat_line##*) }"
    local -a fields=()
    read -ra fields <<< "$rest" || return 1
    if [[ "${#fields[@]}" -lt 20 ]]; then
        return 1
    fi

    # After "(comm) ", fields[0] is field 3 (state) and fields[19] is field 22 (starttime).
    local state_val="${fields[0]}"
    local start_val="${fields[19]}"
    [[ -n "$comm_val" && "$state_val" =~ ^[A-Za-z]$ && "$start_val" =~ ^[0-9]+$ ]] || return 1

    BORNO_PROC_STATE="$state_val"
    BORNO_PROC_COMM="$comm_val"
    BORNO_PROC_STARTTIME="$start_val"
    return 0
}

borno_editor_capture_pid_identity() {
    local pid="$1"
    local expected_prefix="${2:-}"
    BORNO_CAPTURED_COMM=""
    BORNO_CAPTURED_STARTTIME=""

    local tries=0
    while [[ "$tries" -lt 25 ]]; do
        if ! borno_editor_read_proc_identity "$pid"; then
            return 1
        fi
        if [[ "$BORNO_PROC_STATE" == "Z" ]]; then
            return 1
        fi
        if [[ -z "$expected_prefix" || "$BORNO_PROC_COMM" == "${expected_prefix}"* ]]; then
            BORNO_CAPTURED_COMM="$BORNO_PROC_COMM"
            BORNO_CAPTURED_STARTTIME="$BORNO_PROC_STARTTIME"
            return 0
        fi
        # Wait briefly for a newly backgrounded subshell to complete execve().
        if [[ "$BORNO_PROC_COMM" != "bash" && "$BORNO_PROC_COMM" != "sh" ]]; then
            echo "[Editor] Warning: PID ${pid} has unexpected process name '${BORNO_PROC_COMM}' (expected '${expected_prefix}*')." >&2
            return 1
        fi
        sleep 0.01
        tries=$((tries + 1))
    done

    echo "[Editor] Warning: timed out verifying process identity for PID ${pid} (comm='${BORNO_PROC_COMM}', expected '${expected_prefix}*')." >&2
    return 1
}

# Verify that PID is still alive (non-zombie) and matches the recorded (comm, starttime) instance.
# Returns:
#   0 -> exact live owned process verified
#   1 -> process has already exited (missing from /proc or zombie)
#   2 -> identity unestablished or mismatched (must NOT signal)
borno_editor_check_verified_target() {
    local pid="$1"
    local expected_comm="$2"
    local expected_starttime="$3"
    local expected_prefix="${4:-}"

    if [[ -z "$pid" ]]; then
        return 1
    fi

    if ! borno_editor_read_proc_identity "$pid"; then
        return 1
    fi

    if [[ "$BORNO_PROC_STATE" == "Z" ]]; then
        if [[ -n "$expected_starttime" && "$BORNO_PROC_STARTTIME" == "$expected_starttime" ]]; then
            wait "$pid" 2>/dev/null || true
        fi
        return 1
    fi

    if [[ -z "$expected_comm" || -z "$expected_starttime" || ! "$expected_starttime" =~ ^[0-9]+$ ]]; then
        echo "[Editor] Warning: skipping cleanup signal for PID ${pid}; process identity was not recorded." >&2
        return 2
    fi

    if [[ "$BORNO_PROC_COMM" != "$expected_comm" || "$BORNO_PROC_STARTTIME" != "$expected_starttime" ]]; then
        echo "[Editor] Warning: skipping cleanup signal for PID ${pid}; process identity mismatch (expected comm='${expected_comm}' starttime='${expected_starttime}', found comm='${BORNO_PROC_COMM}' starttime='${BORNO_PROC_STARTTIME}')." >&2
        return 2
    fi

    if [[ -n "$expected_prefix" && "$BORNO_PROC_COMM" != "${expected_prefix}"* ]]; then
        echo "[Editor] Warning: skipping cleanup signal for PID ${pid}; comm '${BORNO_PROC_COMM}' does not match expected prefix '${expected_prefix}'." >&2
        return 2
    fi

    return 0
}

borno_editor_terminate_verified_pid() {
    local pid="$1"
    local expected_comm="$2"
    local expected_starttime="$3"
    local expected_prefix="${4:-}"
    local term_wait_steps="${5:-30}"

    local check_rc=0
    borno_editor_check_verified_target "$pid" "$expected_comm" "$expected_starttime" "$expected_prefix" || check_rc=$?
    if [[ "$check_rc" -ne 0 ]]; then
        return 0
    fi

    kill -TERM "$pid" 2>/dev/null || true

    local tries=0
    while [[ "$tries" -lt "$term_wait_steps" ]]; do
        check_rc=0
        borno_editor_check_verified_target "$pid" "$expected_comm" "$expected_starttime" "$expected_prefix" || check_rc=$?
        if [[ "$check_rc" -ne 0 ]]; then
            return 0
        fi
        sleep 0.1
        tries=$((tries + 1))
    done

    # Re-verify identity immediately before escalating SIGKILL to this exact process instance.
    check_rc=0
    borno_editor_check_verified_target "$pid" "$expected_comm" "$expected_starttime" "$expected_prefix" || check_rc=$?
    if [[ "$check_rc" -eq 0 ]]; then
        echo "[Editor] Child PID ${pid} (${expected_comm}) ignored SIGTERM; escalating SIGKILL on verified owned instance." >&2
        kill -KILL "$pid" 2>/dev/null || true
        sleep 0.05
        borno_editor_check_verified_target "$pid" "$expected_comm" "$expected_starttime" "$expected_prefix" || true
    fi
    return 0
}

borno_editor_cleanup_temp_dir() {
    local dir="${1:-}"
    local keep_temp="${2:-0}"

    if [[ -z "$dir" ]]; then
        return 0
    fi
    if [[ "$keep_temp" -ne 0 ]]; then
        echo "[Editor] Kept isolated session directory: $dir"
        return 0
    fi
    if [[ "$dir" != /* || "$dir" == "/" || "$dir" == *"/../"* || "$dir" == *"/.." || -L "$dir" ]]; then
        echo "[Editor] Refusing to remove unsafe temporary path: $dir" >&2
        return 1
    fi
    if [[ "$(basename -- "$dir")" != bornomala-fcitx5-editor.* ]]; then
        echo "[Editor] Refusing to remove non-launcher temporary directory: $dir" >&2
        return 1
    fi
    if [[ -d "$dir" ]]; then
        rm -rf -- "$dir"
    fi
    return 0
}

borno_editor_cleanup() {
    local rc=$?
    borno_editor_terminate_verified_pid "$GEDIT_PID" "$GEDIT_COMM" "$GEDIT_STARTTIME" "${GEDIT_EXPECTED_PREFIX:-}"
    borno_editor_terminate_verified_pid "$FCITX_PID" "$FCITX_COMM" "$FCITX_STARTTIME" "${FCITX_EXPECTED_PREFIX:-}"
    borno_editor_terminate_verified_pid "$DBUS_PID" "$DBUS_COMM" "$DBUS_STARTTIME" "${DBUS_EXPECTED_PREFIX:-}"
    borno_editor_cleanup_temp_dir "$SESSION_TMP" "$KEEP_TEMP" || true
    return "$rc"
}

borno_editor_resolve_build_dir() {
    local build_dir="$1"
    local -a candidates=()
    if [[ -n "$build_dir" ]]; then
        candidates=("$build_dir")
    else
        candidates=("/tmp/bornomala-m8-wp1-cmake" "${REPO_ROOT}/build")
    fi
    local cand
    for cand in "${candidates[@]}"; do
        if [[ -f "${cand}/bornomala.so" && -f "${cand}/bornomala.conf" && -f "${cand}/bornomala-im.conf" ]]; then
            (cd "$cand" && pwd)
            return 0
        fi
    done
    return 1
}

borno_editor_main() {
    local BUILD_DIR="${BORNO_FCITX5_BUILD_DIR:-}"
    local -a EDITOR_ARGS=()
    KEEP_TEMP=0

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
            --keep-temp)
                KEEP_TEMP=1
                shift
                ;;
            --help|-h)
                cat << 'EOF'
Usage: launch_test_editor.sh [--build-dir DIR] [--keep-temp] [--] [gedit args...]

Launches standalone Gedit inside a completely isolated temporary Fcitx 5 + D-Bus + XDG session.
Never touches ~/.config/fcitx5/profile, ~/.local/lib/fcitx5, or GNOME gsettings.
EOF
                exit 0
                ;;
            --)
                shift
                EDITOR_ARGS+=("$@")
                break
                ;;
            *)
                EDITOR_ARGS+=("$1")
                shift
                ;;
        esac
    done

    local RESOLVED_BUILD=""
    if ! RESOLVED_BUILD="$(borno_editor_resolve_build_dir "$BUILD_DIR")"; then
        echo "RESULT=UNAVAILABLE: fresh WP1 build (bornomala.so, bornomala.conf, bornomala-im.conf) not found." >&2
        exit 77
    fi

    if [[ -z "${DISPLAY:-}" ]]; then
        echo "RESULT=UNAVAILABLE: DISPLAY is not set; GUI editor requires WSLg/X11." >&2
        exit 77
    fi

    local req
    for req in fcitx5 fcitx5-remote dbus-daemon gedit; do
        if ! command -v "$req" >/dev/null 2>&1; then
            echo "RESULT=UNAVAILABLE: required command '$req' not found." >&2
            exit 77
        fi
    done

    SESSION_TMP="$(mktemp -d "${TMPDIR:-/tmp}/bornomala-fcitx5-editor.XXXXXX")"
    DBUS_PID=""
    DBUS_COMM=""
    DBUS_STARTTIME=""
    FCITX_PID=""
    FCITX_COMM=""
    FCITX_STARTTIME=""
    GEDIT_PID=""
    GEDIT_COMM=""
    GEDIT_STARTTIME=""
    GEDIT_EXPECTED_PREFIX="gedit"
    FCITX_EXPECTED_PREFIX="fcitx5"
    DBUS_EXPECTED_PREFIX="dbus-daemon"

    trap 'exit 130' INT
    trap 'exit 143' TERM
    trap borno_editor_cleanup EXIT

    mkdir -p "$SESSION_TMP/home" "$SESSION_TMP/config/fcitx5" "$SESSION_TMP/config/com.bornomala.desktop" \
             "$SESSION_TMP/data/fcitx5/addon" "$SESSION_TMP/data/fcitx5/inputmethod" "$SESSION_TMP/cache"

    cp -- "$RESOLVED_BUILD/bornomala.conf" "$SESSION_TMP/data/fcitx5/addon/bornomala.conf"
    cp -- "$RESOLVED_BUILD/bornomala-im.conf" "$SESSION_TMP/data/fcitx5/inputmethod/bornomala.conf"

    cat > "$SESSION_TMP/config/fcitx5/profile" << 'EOF'
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

    export HOME="$SESSION_TMP/home"
    export XDG_CONFIG_HOME="$SESSION_TMP/config"
    export XDG_DATA_HOME="$SESSION_TMP/data"
    export XDG_CACHE_HOME="$SESSION_TMP/cache"
    export XDG_DATA_DIRS="$SESSION_TMP/data:/usr/local/share:/usr/share"
    export FCITX_ADDON_DIRS="/usr/lib/x86_64-linux-gnu/fcitx5:$RESOLVED_BUILD"
    export GTK_IM_MODULE=fcitx
    export QT_IM_MODULE=fcitx
    export XMODIFIERS="@im=fcitx"
    export GDK_BACKEND=x11

    local DBUS_INFO=""
    DBUS_INFO="$(dbus-daemon --session --fork --nopidfile --print-address=1 --print-pid=1)"
    export DBUS_SESSION_BUS_ADDRESS="$(printf '%s\n' "$DBUS_INFO" | sed -n '1p')"
    DBUS_PID="$(printf '%s\n' "$DBUS_INFO" | sed -n '2p')"
    if borno_editor_capture_pid_identity "$DBUS_PID" "$DBUS_EXPECTED_PREFIX"; then
        DBUS_COMM="$BORNO_CAPTURED_COMM"
        DBUS_STARTTIME="$BORNO_CAPTURED_STARTTIME"
    else
        echo "Error: failed to verify isolated dbus-daemon child identity (PID='${DBUS_PID}')." >&2
        exit 1
    fi

    fcitx5 --verbose "default=4,bornomala=5" --disable waylandim -D >"$SESSION_TMP/fcitx5.log" 2>&1 &
    FCITX_PID=$!
    if borno_editor_capture_pid_identity "$FCITX_PID" "$FCITX_EXPECTED_PREFIX"; then
        FCITX_COMM="$BORNO_CAPTURED_COMM"
        FCITX_STARTTIME="$BORNO_CAPTURED_STARTTIME"
    fi

    local _
    for _ in $(seq 1 50); do
        if ! borno_editor_read_proc_identity "$FCITX_PID" || [[ "$BORNO_PROC_STATE" == "Z" ]]; then
            echo "Error: isolated fcitx5 exited unexpectedly (see $SESSION_TMP/fcitx5.log)." >&2
            exit 1
        fi
        if fcitx5-remote --check >/dev/null 2>&1; then
            break
        fi
        sleep 0.15
    done

    if ! fcitx5-remote --check >/dev/null 2>&1; then
        echo "Error: timed out waiting for isolated fcitx5." >&2
        exit 1
    fi

    fcitx5-remote -s bornomala >/dev/null 2>&1 || true
    fcitx5-remote -o >/dev/null 2>&1 || true

    echo "=========================================================="
    echo "  BornoMala Fcitx 5 Isolated Interactive Test Session"
    echo "  Target: Gedit (GNOME Desktop Editor)"
    echo "  Addon : $RESOLVED_BUILD/bornomala.so"
    echo "  Temp  : $SESSION_TMP"
    echo ""
    echo "  Features to Try:"
    echo "    1. Phonetic: Type 'ami ' -> 'আমি '"
    echo "    2. Candidates: Type 'k' or 'am' -> suggestions; '1'..'9' picks when >1 candidates"
    echo "    3. Cancel: Press 'Escape' to clear preedit (idle Escape reaches app)"
    echo "    4. Mode Toggle: Press 'F12' or 'F4' to cycle modes"
    echo "=========================================================="

    if [[ ${#EDITOR_ARGS[@]} -gt 0 ]]; then
        gedit -s "${EDITOR_ARGS[@]}" &
    else
        gedit -s &
    fi
    GEDIT_PID=$!
    if borno_editor_capture_pid_identity "$GEDIT_PID" "$GEDIT_EXPECTED_PREFIX"; then
        GEDIT_COMM="$BORNO_CAPTURED_COMM"
        GEDIT_STARTTIME="$BORNO_CAPTURED_STARTTIME"
    fi
    wait "$GEDIT_PID"
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    borno_editor_main "$@"
fi
