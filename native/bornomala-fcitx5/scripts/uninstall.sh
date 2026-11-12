#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
ICON_SRC="$PROJECT_ROOT/native/bornomala-fcitx5/data/bornomala.png"

# shellcheck source=native/bornomala-fcitx5/scripts/common.sh
source "$SCRIPT_DIR/common.sh"

usage() {
    cat << 'EOF'
Usage: uninstall.sh [OPTIONS]

Non-destructive user-mode uninstaller for BornoMala Fcitx 5 artifacts.
Removes ONLY BornoMala-owned files, never edits or removes your Fcitx profile,
and never stops or restarts running Fcitx 5 processes.

Options:
  --dry-run    Print planned removals without deleting any files
  --yes, -y    Confirm file removal in non-interactive execution
  -h, --help   Show this help message and exit
EOF
}

DRY_RUN=0
ASSUME_YES=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run|-n)
            DRY_RUN=1
            shift
            ;;
        --yes|-y)
            ASSUME_YES=1
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

echo "=== BornoMala Fcitx 5 Safe User-Mode Uninstaller ==="
echo "  Target lib dir : $BORNO_USER_LIB_DIR"
echo "  Target data dir: $BORNO_XDG_DATA_HOME"
echo "  Profile untouched: $BORNO_FCITX_PROFILE_PATH"

MANIFEST_ICON_STATE="UNKNOWN"
MANIFEST_ICON_SHA=""
MANIFEST_PATHS=()

if [[ -e "$BORNO_MANIFEST_DEST" ]]; then
    if [[ -L "$BORNO_MANIFEST_DEST" || ! -f "$BORNO_MANIFEST_DEST" ]]; then
        echo "Warning: ignoring non-regular/symlinked manifest at $BORNO_MANIFEST_DEST." >&2
    else
        while IFS= read -r line || [[ -n "$line" ]]; do
            [[ -z "$line" || "$line" == \#* ]] && continue
            IFS='|' read -r kind sha rec_path extra <<< "$line" || true
            if [[ -n "${extra:-}" || -z "${rec_path:-}" ]]; then
                echo "Warning: ignoring malformed manifest entry." >&2
                continue
            fi
            if ! borno_is_allowlisted_owned_path "$rec_path"; then
                echo "Warning: rejecting unowned or unsafe path in manifest: $rec_path" >&2
                continue
            fi
            case "$kind" in
                FILE)
                    MANIFEST_PATHS+=("$rec_path")
                    ;;
                ICON_OWNED)
                    MANIFEST_ICON_STATE="OWNED"
                    MANIFEST_ICON_SHA="$sha"
                    MANIFEST_PATHS+=("$rec_path")
                    ;;
                ICON_PRESERVED)
                    MANIFEST_ICON_STATE="PRESERVED"
                    ;;
                *)
                    echo "Warning: ignoring unrecognized manifest entry type '$kind'." >&2
                    ;;
            esac
        done < "$BORNO_MANIFEST_DEST"
    fi
fi

CANDIDATES=(
    "$BORNO_ADDON_SO_DEST"
    "$BORNO_ADDON_CONF_DEST"
    "$BORNO_IM_CONF_DEST"
    "$BORNO_NOTICES_DEST"
    "$BORNO_ICON_DEST"
    "$BORNO_MANIFEST_DEST"
)

PLANNED_REMOVALS=()

for path in "${CANDIDATES[@]}"; do
    if ! borno_is_allowlisted_owned_path "$path"; then
        echo "Error: internal allowlist violation for path '$path'." >&2
        exit 1
    fi

    if [[ ! -e "$path" && ! -L "$path" ]]; then
        continue
    fi

    if [[ -L "$path" || -d "$path" ]]; then
        echo "Warning: skipping non-regular/symlinked path: $path" >&2
        continue
    fi

    if [[ "$path" == "$BORNO_ICON_DEST" ]]; then
        if [[ "$MANIFEST_ICON_STATE" == "PRESERVED" ]]; then
            echo "  [preserve] $path (marked as pre-existing non-BornoMala icon)"
            continue
        elif [[ "$MANIFEST_ICON_STATE" == "OWNED" ]]; then
            actual_sha="$(borno_file_sha256 "$path")"
            if [[ -n "$MANIFEST_ICON_SHA" && "$actual_sha" != "$MANIFEST_ICON_SHA" ]]; then
                echo "  [preserve] $path (checksum changed since BornoMala install)"
                continue
            fi
        else
            # No valid manifest record: remove icon ONLY if it matches repository's bornomala.png
            if [[ -f "$ICON_SRC" ]] && cmp -s "$ICON_SRC" "$path"; then
                :
            else
                echo "  [preserve] $path (unverified ownership without matching manifest/source)"
                continue
            fi
        fi
    fi

    PLANNED_REMOVALS+=("$path")
done

if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "[DRY-RUN] Planned removals (no files deleted):"
    if [[ ${#PLANNED_REMOVALS[@]} -eq 0 ]]; then
        echo "  (no BornoMala-owned files present)"
    else
        for target in "${PLANNED_REMOVALS[@]}"; do
            echo "  [would-remove] $target"
        done
    fi
    if [[ -d "$BORNO_DOC_DIR" ]]; then
        echo "  [would-rmdir-if-empty] $BORNO_DOC_DIR"
    fi
    exit 0
fi

if [[ ${#PLANNED_REMOVALS[@]} -eq 0 && ! -d "$BORNO_DOC_DIR" ]]; then
    echo "No BornoMala user-mode files found; nothing to remove."
    exit 0
fi

if [[ "$ASSUME_YES" -ne 1 ]]; then
    if [[ -t 0 ]]; then
        echo "The following BornoMala-owned files will be removed:"
        for target in "${PLANNED_REMOVALS[@]}"; do
            echo "  - $target"
        done
        read -r -p "Proceed with uninstall? [y/N] " reply
        case "$reply" in
            y|Y|yes|YES)
                ;;
            *)
                echo "Uninstall cancelled; no files removed."
                exit 1
                ;;
        esac
    else
        echo "Error: non-interactive uninstall requires explicit confirmation (--yes) or --dry-run." >&2
        exit 1
    fi
fi

for target in "${PLANNED_REMOVALS[@]}"; do
    # Re-validate immediately before unlinking
    if borno_is_allowlisted_owned_path "$target" && [[ -f "$target" && ! -L "$target" ]]; then
        rm -f -- "$target"
        echo "  [removed] $target"
    fi
done

# Remove BornoMala-owned doc directory only if it is now completely empty
if [[ -d "$BORNO_DOC_DIR" && ! -L "$BORNO_DOC_DIR" ]]; then
    if rmdir -- "$BORNO_DOC_DIR" 2>/dev/null; then
        echo "  [rmdir]   $BORNO_DOC_DIR"
    else
        echo "  [kept]    $BORNO_DOC_DIR (not empty; preserved unrelated files)"
    fi
fi

echo "=== BornoMala Fcitx 5 uninstall finished cleanly ==="
echo "Your Fcitx profile (${BORNO_FCITX_PROFILE_PATH}) and running Fcitx daemon were not modified."
echo "If you had manually added 'bornomala' to your Fcitx configuration, remove it in fcitx5-configtool and run: fcitx5-remote -r"
