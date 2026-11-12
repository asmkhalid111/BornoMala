#!/usr/bin/env python3
"""
Interactive Tester for BornoMala Fcitx 5 Linux Addon (WSLg & Native X11)
Runs strictly inside an isolated temporary Fcitx 5 + D-Bus + XDG session.
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from fcitx_test_env import (  # noqa: E402
    EXIT_FAILED,
    EXIT_PASSED,
    EXIT_UNAVAILABLE,
    IsolatedFcitxSession,
    PrerequisiteUnavailableError,
    SessionFailureError,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="BornoMala Fcitx 5 isolated interactive GTK3 tester")
    parser.add_argument("--build-dir", default=None, help="Directory containing built bornomala.so and .conf files")
    parser.add_argument("--keep-temp", action="store_true", help="Keep temporary isolated XDG/Fcitx session dir on exit")
    return parser.parse_args()


def run_gui(session: IsolatedFcitxSession) -> int:
    # Apply isolated XDG/IM/D-Bus vars BEFORE importing Gtk
    session.apply_env_to_current_process()

    loaded_so = session.verify_loaded_addon()
    subprocess.run(["fcitx5-remote", "-s", "bornomala"], env=session.env, capture_output=True)
    subprocess.run(["fcitx5-remote", "-o"], env=session.env, capture_output=True)

    import gi
    gi.require_version("Gtk", "3.0")
    gi.require_version("Gdk", "3.0")
    from gi.repository import Gdk, Gtk

    class BornoInteractiveApp(Gtk.Window):
        def __init__(self):
            super().__init__(title="BornoMala Linux (Fcitx 5) — Isolated Interactive Tester")
            self.set_default_size(700, 500)
            self.set_position(Gtk.WindowPosition.CENTER)
            self.connect("destroy", Gtk.main_quit)

            vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
            vbox.set_margin_start(20)
            vbox.set_margin_end(20)
            vbox.set_margin_top(20)
            vbox.set_margin_bottom(20)
            self.add(vbox)

            title_label = Gtk.Label()
            title_label.set_markup("<span size='x-large' weight='bold'>বর্ণমালা (BornoMala) — Isolated Fcitx 5 Tester</span>")
            vbox.pack_start(title_label, False, False, 0)

            subtitle = Gtk.Label()
            subtitle.set_markup(
                f"<b>Loaded Addon:</b> <tt>{loaded_so}</tt>\n"
                "• Type <b>ami</b> + Space → <b>আমি</b>  • Press <b>F12</b> or <b>F4</b> to switch layouts\n"
                "• Type <b>k</b> or <b>am</b> → suggestions  • Press <b>1..9</b> when multiple candidates exist"
            )
            subtitle.set_justify(Gtk.Justification.CENTER)
            vbox.pack_start(subtitle, False, False, 0)

            mode_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            vbox.pack_start(mode_box, False, False, 0)

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
                Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
            )

            lbl = Gtk.Label(label="Switch Mode (or press F12 / F4):")
            mode_box.pack_start(lbl, False, False, 0)

            self.btn_cycle = Gtk.Button(label="Cycle Mode (F12 / F4)")
            self.btn_cycle.connect("clicked", self.on_cycle_mode)
            mode_box.pack_start(self.btn_cycle, False, False, 0)

            btn_clear = Gtk.Button(label="Clear Text")
            btn_clear.connect("clicked", self.on_clear_text)
            mode_box.pack_end(btn_clear, False, False, 0)

            entry_lbl = Gtk.Label(label="Single-line Input (Gtk.Entry):", xalign=0)
            vbox.pack_start(entry_lbl, False, False, 0)

            self.entry = Gtk.Entry()
            self.entry.set_placeholder_text("Click here and type: ami amar sonar bangla...")
            vbox.pack_start(self.entry, False, False, 0)

            area_lbl = Gtk.Label(label="Multi-line Document Editor (Gtk.TextView):", xalign=0)
            vbox.pack_start(area_lbl, False, False, 0)

            scrolled = Gtk.ScrolledWindow()
            scrolled.set_hexpand(True)
            scrolled.set_vexpand(True)
            self.text_view = Gtk.TextView()
            self.text_view.set_wrap_mode(Gtk.WrapMode.WORD)
            scrolled.add(self.text_view)
            vbox.pack_start(scrolled, True, True, 0)

            footer = Gtk.Label()
            footer.set_markup(
                "<span size='small' color='#666666'>"
                "Layout Sequence: <b>Phonetic → জাতীয় (National) → প্রভাত (Probhat) → বিজয় (Bijoy) → English</b>\n"
                "Isolated session: your ~/.config/fcitx5/profile and ~/.local/lib/fcitx5 remain untouched."
                "</span>"
            )
            vbox.pack_start(footer, False, False, 0)

        def on_cycle_mode(self, _widget):
            subprocess.run(["xdotool", "key", "F4"], env=session.env, capture_output=True)

        def on_clear_text(self, _widget):
            self.entry.set_text("")
            self.text_view.get_buffer().set_text("")
            self.entry.grab_focus()

    win = BornoInteractiveApp()
    win.show_all()
    win.entry.grab_focus()
    Gtk.main()
    return EXIT_PASSED


def main() -> int:
    args = parse_args()
    try:
        with IsolatedFcitxSession(
            build_dir=args.build_dir,
            keep_temp=args.keep_temp,
            require_xdotool=False,
            require_gedit=False,
            require_gtk3=True,
        ) as session:
            print(f"[Tester] Isolated session dir: {session.root_dir}")
            print(f"[Tester] Owned fcitx5 PID: {session.fcitx_proc.pid}, dbus PID: {session.dbus_pid}")
            return run_gui(session)
    except PrerequisiteUnavailableError as exc:
        print(f"RESULT=UNAVAILABLE: {exc}", file=sys.stderr)
        return EXIT_UNAVAILABLE
    except SessionFailureError as exc:
        print(f"RESULT=FAILED: {exc}", file=sys.stderr)
        return EXIT_FAILED


if __name__ == "__main__":
    sys.exit(main())
