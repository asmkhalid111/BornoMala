#!/usr/bin/env bash
set -euo pipefail

echo "=== BornoMala Fcitx 5 Addon Uninstaller ==="

rm -f "${HOME}/.local/lib/fcitx5/bornomala.so"
rm -f "${HOME}/.local/share/fcitx5/addon/bornomala.conf"
rm -f "${HOME}/.local/share/fcitx5/inputmethod/bornomala.conf"
rm -f "${HOME}/.local/share/icons/hicolor/64x64/apps/bornomala.png"

echo "Artifacts removed from ~/.local/."

# Reload daemon
export FCITX_ADDON_DIRS="${HOME}/.local/lib/fcitx5:${FCITX_ADDON_DIRS:-}"
fcitx5 -r -d >/dev/null 2>&1 || true

echo "=== BornoMala Fcitx 5 Addon successfully uninstalled! ==="
