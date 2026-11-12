#!/usr/bin/env bash
# Shared XDG resolution, ELF validation, hashing, and allowlist helpers for BornoMala Fcitx 5 scripts.

borno_trim_trailing_slashes() {
    local p="$1"
    while [[ "${#p}" -gt 1 && "${p: -1}" == "/" ]]; do
        p="${p%/}"
    done
    printf '%s' "$p"
}

borno_is_valid_abs_dir() {
    local p="$1"
    [[ -n "$p" && "$p" == /* && "$p" != *$'\n'* && "$p" != *$'\r'* ]]
}

borno_resolve_xdg_dir() {
    local var_name="$1"
    local raw_val="$2"
    local fallback="$3"

    if [[ -n "$raw_val" ]]; then
        if borno_is_valid_abs_dir "$raw_val"; then
            borno_trim_trailing_slashes "$raw_val"
            return 0
        fi
        echo "Warning: ignoring invalid relative/malformed ${var_name}='${raw_val}'; falling back to '${fallback}'." >&2
    fi

    borno_trim_trailing_slashes "$fallback"
}

borno_init_user_paths() {
    if ! borno_is_valid_abs_dir "${HOME:-}"; then
        echo "Error: HOME must be set to a valid absolute directory (got '${HOME:-}')." >&2
        return 1
    fi

    BORNO_HOME_DIR="$(borno_trim_trailing_slashes "$HOME")"
    if [[ "$BORNO_HOME_DIR" == "/" ]]; then
        BORNO_XDG_DATA_DEFAULT="/.local/share"
        BORNO_XDG_CONFIG_DEFAULT="/.config"
        BORNO_XDG_CACHE_DEFAULT="/.cache"
        BORNO_USER_LIB_DIR="/.local/lib/fcitx5"
    else
        BORNO_XDG_DATA_DEFAULT="${BORNO_HOME_DIR}/.local/share"
        BORNO_XDG_CONFIG_DEFAULT="${BORNO_HOME_DIR}/.config"
        BORNO_XDG_CACHE_DEFAULT="${BORNO_HOME_DIR}/.cache"
        BORNO_USER_LIB_DIR="${BORNO_HOME_DIR}/.local/lib/fcitx5"
    fi

    BORNO_XDG_DATA_HOME="$(borno_resolve_xdg_dir "XDG_DATA_HOME" "${XDG_DATA_HOME:-}" "$BORNO_XDG_DATA_DEFAULT")"
    BORNO_XDG_CONFIG_HOME="$(borno_resolve_xdg_dir "XDG_CONFIG_HOME" "${XDG_CONFIG_HOME:-}" "$BORNO_XDG_CONFIG_DEFAULT")"
    BORNO_XDG_CACHE_HOME="$(borno_resolve_xdg_dir "XDG_CACHE_HOME" "${XDG_CACHE_HOME:-}" "$BORNO_XDG_CACHE_DEFAULT")"

    BORNO_ADDON_SO_DEST="${BORNO_USER_LIB_DIR}/bornomala.so"
    BORNO_ADDON_CONF_DEST="${BORNO_XDG_DATA_HOME}/fcitx5/addon/bornomala.conf"
    BORNO_IM_CONF_DEST="${BORNO_XDG_DATA_HOME}/fcitx5/inputmethod/bornomala.conf"
    BORNO_ICON_DEST="${BORNO_XDG_DATA_HOME}/icons/hicolor/64x64/apps/bornomala.png"
    BORNO_DOC_DIR="${BORNO_XDG_DATA_HOME}/doc/bornomala"
    BORNO_NOTICES_DEST="${BORNO_DOC_DIR}/THIRD-PARTY-NOTICES.md"
    BORNO_MANIFEST_DEST="${BORNO_DOC_DIR}/install-manifest.txt"
    BORNO_APP_CONFIG_DIR="${BORNO_XDG_CONFIG_HOME}/com.bornomala.desktop"
    BORNO_FCITX_PROFILE_PATH="${BORNO_XDG_CONFIG_HOME}/fcitx5/profile"
}

borno_file_sha256() {
    local file_path="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum -- "$file_path" | awk '{print $1}'
        return 0
    fi
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 -- "$file_path" | awk '{print $1}'
        return 0
    fi
    python3 -c 'import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], "rb").read()).hexdigest())' "$file_path"
}

borno_is_valid_elf_module() {
    local file_path="$1"
    [[ -f "$file_path" && ! -L "$file_path" && -r "$file_path" && -s "$file_path" ]] || return 1
    local magic_hex
    if command -v od >/dev/null 2>&1; then
        magic_hex="$(od -An -tx1 -N4 -- "$file_path" 2>/dev/null | tr -d ' \t\r\n')"
        [[ "$magic_hex" == "7f454c46" ]]
        return $?
    fi
    python3 -c 'import sys; d=open(sys.argv[1],"rb").read(4); sys.exit(0 if d == b"\x7fELF" else 1)' "$file_path"
}

borno_is_allowlisted_owned_path() {
    local candidate="$1"
    [[ -n "$candidate" && "$candidate" == /* ]] || return 1
    [[ "$candidate" != *"/../"* && "$candidate" != *"/.." && "$candidate" != "$BORNO_FCITX_PROFILE_PATH" ]] || return 1

    case "$candidate" in
        "$BORNO_ADDON_SO_DEST" | \
        "$BORNO_ADDON_CONF_DEST" | \
        "$BORNO_IM_CONF_DEST" | \
        "$BORNO_ICON_DEST" | \
        "$BORNO_NOTICES_DEST" | \
        "$BORNO_MANIFEST_DEST")
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

