#!/usr/bin/env bash
set -e

# 1. Setup profile with bornomala as the default and only active IM
mkdir -p "$HOME/.config/fcitx5"
cat << 'EOF' > "$HOME/.config/fcitx5/profile"
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=bornomala

[Groups/0/Items/0]
Name=bornomala
Layout=

[GroupOrder]
0=Default
EOF

# 2. Kill any stale daemon
pkill -9 fcitx5 2>/dev/null || true
sleep 0.5

# 3. Environment configuration
export DISPLAY="${DISPLAY:-:0}"
export FCITX_ADDON_DIRS="/usr/lib/x86_64-linux-gnu/fcitx5:$HOME/.local/lib/fcitx5"
export GTK_IM_MODULE=fcitx
export QT_IM_MODULE=fcitx
export XMODIFIERS="@im=fcitx"
export GDK_BACKEND=x11

# Ensure GNOME desktop interface points to fcitx
gsettings set org.gnome.desktop.interface gtk-im-module 'fcitx' 2>/dev/null || true

# 4. Start Fcitx 5 daemon in background
fcitx5 --disable waylandim -d >/dev/null 2>&1
sleep 1.5

# 5. Switch to bornomala
fcitx5-remote -s bornomala 2>/dev/null || true
fcitx5-remote -o 2>/dev/null || true

echo "=========================================================="
echo "  BornoMala Fcitx 5 Interactive Test Session"
echo "  Target: Gedit (GNOME Desktop Editor)"
echo ""
echo "  Features to Try:"
echo "    1. Phonetic: Type 'ami ' -> 'আমি '"
echo "    2. Candidates: Type 'k' -> see vertical popup suggestions"
echo "    3. Digits: Press '1'..'9' to pick candidate"
echo "    4. Cancel: Press 'Escape' to clear preedit"
echo "    5. Mode Toggle: Press 'F12' or 'F4' to cycle:"
echo "       Phonetic -> National -> Probhat -> Bijoy -> English"
echo "       (Tip: If F12 toggles Airplane Mode on your laptop, press F4!)"
echo "=========================================================="

# 6. Launch Gedit standalone
gedit -s "$@"
