#!/usr/bin/env python3
"""
Layer 6A: Isolated Real Graphical Application Integration Test (GTK3 under X11/WSLg)

Verifies end-to-end typing via Fcitx 5 + freshly built WP1 BornoMala into Gtk.Entry
inside a completely isolated private D-Bus + XDG temporary session.
"""

import argparse
import os
import subprocess
import sys
import threading
import time
from pathlib import Path

TESTS_DIR = Path(__file__).resolve().parent
SCRIPTS_DIR = TESTS_DIR.parent / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from fcitx_test_env import (  # noqa: E402
    EXIT_FAILED,
    EXIT_PASSED,
    EXIT_UNAVAILABLE,
    IsolatedFcitxSession,
    PrerequisiteUnavailableError,
    SessionFailureError,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Isolated BornoMala Fcitx 5 GTK3 verification")
    parser.add_argument("--build-dir", default=None, help="Directory containing built bornomala.so and .conf files")
    parser.add_argument("--keep-temp", action="store_true", help="Keep temporary isolated session directory on exit")
    return parser.parse_args()


def run_gtk3_suite(session: IsolatedFcitxSession) -> int:
    session.apply_env_to_current_process()
    loaded_so = session.verify_loaded_addon()
    print(f"[Layer 6A] Verified loaded addon binary: {loaded_so}")

    import gi
    gi.require_version("Gtk", "3.0")
    gi.require_version("Gdk", "3.0")
    from gi.repository import Gdk, GLib, Gtk

    key_events = []
    outcomes = {"passed": False, "error": "", "pointer_popup": "SKIPPED"}

    class BornoGtkTestWindow(Gtk.Window):
        def __init__(self):
            super().__init__(title="BornoMala-GTK3-Isolated-Verify")
            self.set_default_size(420, 120)
            self.entry = Gtk.Entry()
            self.entry.set_name("borno_entry")
            self.entry.connect("key-press-event", self.on_key_press)
            self.add(self.entry)
            self.connect("destroy", Gtk.main_quit)

        def on_key_press(self, _widget, event):
            key_events.append(Gdk.keyval_name(event.keyval))
            return False

        def clear_text(self):
            self.entry.set_text("")

    win = BornoGtkTestWindow()
    win.show_all()
    win.entry.grab_focus()

    def clear_entry():
        GLib.idle_add(win.clear_text)
        time.sleep(0.2)

    def send_keys(wid: str, keys):
        for k in keys:
            subprocess.run(["xdotool", "key", "--window", wid, "--delay", "220", k], env=session.env, check=True)
            time.sleep(0.28)
        time.sleep(0.3)

    def worker():
        try:
            win_id = None
            for _ in range(30):
                time.sleep(0.25)
                res_search = subprocess.run(
                    ["xdotool", "search", "--name", "BornoMala-GTK3-Isolated-Verify"],
                    env=session.env,
                    capture_output=True,
                    text=True,
                )
                ids = [tok for tok in res_search.stdout.strip().split() if tok.isdigit()]
                if ids:
                    win_id = ids[-1]
                    break
            if not win_id:
                raise RuntimeError("Timed out locating GTK3 test window via xdotool")

            subprocess.run(["xdotool", "windowactivate", "--sync", win_id], env=session.env, check=True)
            subprocess.run(["xdotool", "windowfocus", "--sync", win_id], env=session.env, check=True)
            subprocess.run(["xdotool", "mousemove", "--window", win_id, "200", "50", "click", "1"], env=session.env, check=True)
            time.sleep(0.6)

            subprocess.run(["fcitx5-remote", "-s", "bornomala"], env=session.env, check=False)
            subprocess.run(["fcitx5-remote", "-o"], env=session.env, check=False)
            time.sleep(0.4)

            im_name = subprocess.check_output(["fcitx5-remote", "-n"], env=session.env, text=True).strip()
            print(f"[Layer 6A] Active IM name: '{im_name}'")

            # 1. Phonetic typing ('a','m','i','space' -> 'আমি ')
            clear_entry()
            send_keys(win_id, ["a", "m", "i", "space"])
            phonetic_text = win.entry.get_text()
            print(f"[Layer 6A] Phonetic result: {phonetic_text!r}")
            assert "আমি " in phonetic_text, f"Expected 'আমি ' in {phonetic_text!r}"

            # 2. Composition cancel + idle Escape passthrough
            clear_entry()
            subprocess.run(["xdotool", "key", "--window", win_id, "k"], env=session.env, check=True)
            time.sleep(0.25)
            key_events.clear()
            subprocess.run(["xdotool", "key", "--window", win_id, "Escape"], env=session.env, check=True)
            time.sleep(0.25)
            assert win.entry.get_text() == "", f"Escape cancel did not clear preedit: {win.entry.get_text()!r}"
            assert "Escape" not in key_events, f"Composing Escape should be consumed, got {key_events}"

            key_events.clear()
            subprocess.run(["xdotool", "key", "--window", win_id, "Escape"], env=session.env, check=True)
            time.sleep(0.25)
            print(f"[Layer 6A] Idle Escape events delivered to app: {key_events}")
            assert "Escape" in key_events, f"Idle Escape was not passed through: {key_events}"

            # 3. Single-candidate numeric input (WP-C: '1' + 'space' -> '1 ')
            clear_entry()
            send_keys(win_id, ["1", "space"])
            digit_text = win.entry.get_text()
            print(f"[Layer 6A] Single-candidate digit result: {digit_text!r}")
            assert "1 " in digit_text, f"Expected literal '1 ' in {digit_text!r}"

            # 4. Candidate popup inspection & multi-candidate keyboard digit selection ('a','m' -> 4 cands, press '2')
            clear_entry()
            before_wins = set(
                subprocess.check_output(
                    ["xdotool", "search", "--onlyvisible", ""],
                    env=session.env,
                    stderr=subprocess.DEVNULL,
                ).decode().split()
            )
            send_keys(win_id, ["a", "m"])
            after_wins = set(
                subprocess.check_output(
                    ["xdotool", "search", "--onlyvisible", ""],
                    env=session.env,
                    stderr=subprocess.DEVNULL,
                ).decode().split()
            )
            popup_wins = [w for w in (after_wins - before_wins) if w.isdigit()]
            if not popup_wins:
                outcomes["pointer_popup"] = "SKIPPED (candidate popup window not exposed to X11 xdotool)"
                print(f"[Layer 6A] SUBCHECK_POINTER_CANDIDATE={outcomes['pointer_popup']}")
            else:
                outcomes["pointer_popup"] = f"EXPOSED ({popup_wins})"

            # Select candidate #2 via digit '2' when multiple candidates are active
            send_keys(win_id, ["2"])
            cand_text = win.entry.get_text()
            print(f"[Layer 6A] Multi-candidate digit '2' committed text: {cand_text!r}")
            assert cand_text and "2" not in cand_text, f"Multi-candidate digit '2' was not selected cleanly: {cand_text!r}"

            # 5. Fixed-layout switch via F12 (Phonetic -> National)
            clear_entry()
            send_keys(win_id, ["F12", "a", "m", "i", "space"])
            national_text = win.entry.get_text()
            print(f"[Layer 6A] Fixed National result: {national_text!r}")
            assert "ৃলহ " in national_text, f"Expected National output 'ৃলহ ' in {national_text!r}"

            outcomes["passed"] = True
        except Exception as exc:
            outcomes["error"] = str(exc)
        finally:
            GLib.idle_add(Gtk.main_quit)

    threading.Thread(target=worker, daemon=True).start()
    Gtk.main()

    leaks = session.scan_log_for_leaks()
    if leaks:
        print(f"RESULT=FAILED: runtime privacy leak in isolated log: {leaks[:3]}", file=sys.stderr)
        return EXIT_FAILED

    if outcomes["passed"]:
        print(f"[Layer 6A] Pointer candidate automation: {outcomes['pointer_popup']}")
        print("RESULT=PASSED")
        return EXIT_PASSED

    print(f"RESULT=FAILED: {outcomes['error']}", file=sys.stderr)
    return EXIT_FAILED


def main() -> int:
    args = parse_args()
    try:
        with IsolatedFcitxSession(
            build_dir=args.build_dir,
            keep_temp=args.keep_temp,
            require_xdotool=True,
            require_gedit=False,
            require_gtk3=True,
        ) as session:
            print(f"[Layer 6A] Isolated session dir: {session.root_dir}")
            print(f"[Layer 6A] Owned fcitx5 PID: {session.fcitx_proc.pid}, dbus PID: {session.dbus_pid}")
            return run_gtk3_suite(session)
    except PrerequisiteUnavailableError as exc:
        print(f"RESULT=UNAVAILABLE: {exc}", file=sys.stderr)
        return EXIT_UNAVAILABLE
    except SessionFailureError as exc:
        print(f"RESULT=FAILED: {exc}", file=sys.stderr)
        return EXIT_FAILED


if __name__ == "__main__":
    sys.exit(main())
