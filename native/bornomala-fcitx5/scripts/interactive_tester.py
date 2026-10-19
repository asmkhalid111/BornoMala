#!/usr/bin/env python3
"""
Interactive Tester for BornoMala Fcitx 5 Linux Addon (WSLg & Native Linux)
Provides a dedicated graphical interface for manual typing, candidate verification,
and mode switching without requiring command-line or hotkey acrobatics.
"""

import os
import sys
import time
import subprocess
import threading

# Configure environment for Fcitx 5 under WSLg/X11
uid = os.getuid()
bus_path = f"/run/user/{uid}/bus"
if os.path.exists(bus_path):
    os.environ["DBUS_SESSION_BUS_ADDRESS"] = f"unix:path={bus_path}"

os.environ["DISPLAY"] = os.environ.get("DISPLAY", ":0")
os.environ["GTK_IM_MODULE"] = "fcitx"
os.environ["QT_IM_MODULE"] = "fcitx"
os.environ["XMODIFIERS"] = "@im=fcitx"
os.environ["GDK_BACKEND"] = "x11"
os.environ["FCITX_ADDON_DIRS"] = "/usr/lib/x86_64-linux-gnu/fcitx5:" + os.path.expanduser("~/.local/lib/fcitx5")

# Ensure profile makes bornomala the default and only active IM
def ensure_profile():
    profile_dir = os.path.expanduser("~/.config/fcitx5")
    os.makedirs(profile_dir, exist_ok=True)
    profile_path = os.path.join(profile_dir, "profile")
    content = """[Groups/0]
Name=Default
Default Layout=us
DefaultIM=bornomala

[Groups/0/Items/0]
Name=bornomala
Layout=

[GroupOrder]
0=Default
"""
    with open(profile_path, "w", encoding="utf-8") as f:
        f.write(content)

    subprocess.run(["pkill", "-9", "fcitx5"], capture_output=True)
    time.sleep(0.5)

    env = os.environ.copy()
    subprocess.Popen(
        ["fcitx5", "--disable", "waylandim", "-d"],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True
    )
    time.sleep(1.5)
    subprocess.run(["fcitx5-remote", "-s", "bornomala"], env=env, capture_output=True)
    subprocess.run(["fcitx5-remote", "-o"], env=env, capture_output=True)

ensure_profile()

import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
from gi.repository import Gtk, Gdk, Pango

class BornoInteractiveApp(Gtk.Window):
    def __init__(self):
        super().__init__(title="BornoMala Linux (Fcitx 5) — Interactive Tester")
        self.set_default_size(700, 500)
        self.set_position(Gtk.WindowPosition.CENTER)
        self.connect("destroy", Gtk.main_quit)

        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        vbox.set_margin_start(20)
        vbox.set_margin_end(20)
        vbox.set_margin_top(20)
        vbox.set_margin_bottom(20)
        self.add(vbox)

        # Title and description
        title_label = Gtk.Label()
        title_label.set_markup("<span size='x-large' weight='bold'>বর্ণমালা (BornoMala) — Linux Fcitx 5 Engine</span>")
        vbox.pack_start(title_label, False, False, 0)

        subtitle = Gtk.Label()
        subtitle.set_markup(
            "<b>Ready to type!</b> BornoMala is active.\n"
            "• Type <b>ami</b> + Space → <b>আমি</b>  • Press <b>F12</b> or <b>F4</b> to switch layouts\n"
            "• Type <b>k</b> → see candidate popup list  • Press <b>1..9</b> to choose candidates"
        )
        subtitle.set_justify(Gtk.Justification.CENTER)
        vbox.pack_start(subtitle, False, False, 0)

        # Mode Buttons
        mode_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        vbox.pack_start(mode_box, False, False, 0)

        # Apply CSS for Bengali font and modern typography
        css_provider = Gtk.CssProvider()
        css_provider.load_from_data(b"""
        entry, textview {
            font-family: 'Nirmala UI', 'Lohit Bengali', Sans;
            font-size: 16pt;
        }
        """)
        Gtk.StyleContext.add_provider_for_screen(
            Gdk.Screen.get_default(),
            css_provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        lbl = Gtk.Label(label="Switch Mode (or press F12 / F4):")
        mode_box.pack_start(lbl, False, False, 0)

        self.btn_cycle = Gtk.Button(label="🔄 Cycle Mode (F12 / F4)")
        self.btn_cycle.connect("clicked", self.on_cycle_mode)
        mode_box.pack_start(self.btn_cycle, False, False, 0)

        btn_clear = Gtk.Button(label="🗑️ Clear Text")
        btn_clear.connect("clicked", self.on_clear_text)
        mode_box.pack_end(btn_clear, False, False, 0)

        # Single Line Input Box
        entry_lbl = Gtk.Label(label="Single-line Input (Gtk.Entry):", xalign=0)
        vbox.pack_start(entry_lbl, False, False, 0)

        self.entry = Gtk.Entry()
        self.entry.set_placeholder_text("Click here and type: ami amar sonar bangla...")
        vbox.pack_start(self.entry, False, False, 0)

        # Multi Line Text View
        area_lbl = Gtk.Label(label="Multi-line Document Editor (Gtk.TextView):", xalign=0)
        vbox.pack_start(area_lbl, False, False, 0)

        scrolled = Gtk.ScrolledWindow()
        scrolled.set_hexpand(True)
        scrolled.set_vexpand(True)
        self.text_view = Gtk.TextView()
        self.text_view.set_wrap_mode(Gtk.WrapMode.WORD)
        scrolled.add(self.text_view)
        vbox.pack_start(scrolled, True, True, 0)

        # Instructions Footer
        footer = Gtk.Label()
        footer.set_markup(
            "<span size='small' color='#666666'>"
            "Layout Sequence: <b>Phonetic → জাতীয় (National) → প্রভাত (Probhat) → বিজয় (Bijoy) → English</b>\n"
            "Tip: If F12 toggles Airplane Mode on your laptop, press <b>F4</b> or use the button above."
            "</span>"
        )
        vbox.pack_start(footer, False, False, 0)

    def on_cycle_mode(self, widget):
        # Inject F4 to cycle layout modes
        subprocess.run(["xdotool", "key", "F4"], capture_output=True)

    def on_clear_text(self, widget):
        self.entry.set_text("")
        buf = self.text_view.get_buffer()
        buf.set_text("")
        self.entry.grab_focus()

def main():
    win = BornoInteractiveApp()
    win.show_all()
    # Ensure entry grabs initial focus
    win.entry.grab_focus()
    Gtk.main()

if __name__ == "__main__":
    main()
