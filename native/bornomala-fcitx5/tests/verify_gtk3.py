#!/usr/bin/env python3
"""
Layer 6: Real Graphical Application Integration Test (GTK3 under WSLg)
Tests end-to-end typing via Fcitx 5 + BornoMala into a real GTK3 entry widget.
"""

import os
import sys
import time
import subprocess
import threading

# Set environment before loading Gtk
uid = os.getuid()
bus_path = f"/run/user/{uid}/bus"
if os.path.exists(bus_path):
    os.environ["DBUS_SESSION_BUS_ADDRESS"] = f"unix:path={bus_path}"

os.environ["GTK_IM_MODULE"] = "fcitx"
os.environ["QT_IM_MODULE"] = "fcitx"
os.environ["XMODIFIERS"] = "@im=fcitx"
os.environ["GDK_BACKEND"] = "x11"
os.environ["DISPLAY"] = os.environ.get("DISPLAY", ":0")

import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
from gi.repository import Gtk, Gdk, GLib

class BornoGtkTestWindow(Gtk.Window):
    def __init__(self):
        super().__init__(title="BornoMala GTK3 Integration Test")
        self.set_default_size(400, 100)
        self.entry = Gtk.Entry()
        self.entry.set_name("borno_entry")
        self.add(self.entry)
        self.connect("destroy", Gtk.main_quit)
        self.test_success = False

    def get_text(self):
        return self.entry.get_text()

def ensure_fcitx5_profile():
    profile_dir = os.path.expanduser("~/.config/fcitx5")
    os.makedirs(profile_dir, exist_ok=True)
    profile_path = os.path.join(profile_dir, "profile")
    content = """[Groups/0]
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
"""
    with open(profile_path, "w", encoding="utf-8") as f:
        f.write(content)

def ensure_fcitx5():
    ensure_fcitx5_profile()
    subprocess.run(["pkill", "-9", "fcitx5"], capture_output=True)
    time.sleep(0.5)
    print("[Layer 6] Starting fcitx5 daemon with logging...")
    env = os.environ.copy()
    env["FCITX_ADDON_DIRS"] = "/usr/lib/x86_64-linux-gnu/fcitx5:" + os.path.expanduser("~/.local/lib/fcitx5")
    log_file = open("/tmp/fcitx5_test.log", "w", encoding="utf-8")
    subprocess.Popen(
        ["fcitx5", "--verbose", "default=4", "--disable", "waylandim"],
        env=env,
        stdout=log_file,
        stderr=log_file,
        start_new_session=True
    )
    time.sleep(2.0)

def run_typing_worker(window):
    time.sleep(1.5) # wait for window mapping

    try:
        env = os.environ.copy()

        # Find window via xdotool
        out = subprocess.check_output(
            ["xdotool", "search", "--name", "BornoMala GTK3 Integration Test"],
            env=env
        ).decode().strip().split()
        if not out:
            print("[Layer 6] Could not find window by title via xdotool")
            return
        win_id = out[-1]
        print(f"[Layer 6] Found X11 window id: {win_id}")

        subprocess.run(["xdotool", "windowactivate", "--sync", win_id], env=env, check=True)
        subprocess.run(["xdotool", "windowfocus", "--sync", win_id], env=env, check=True)
        time.sleep(1.0)

        # 2. Activate BornoMala via fcitx5-remote
        subprocess.run(["fcitx5-remote", "-s", "bornomala"], env=env)
        time.sleep(0.5)
        subprocess.run(["fcitx5-remote", "-o"], env=env)
        time.sleep(0.5)

        im_name = subprocess.check_output(["fcitx5-remote", "-n"], env=env).decode().strip()
        print(f"[Layer 6] Active IM name: '{im_name}'")

        # Click inside entry widget to ensure X11/GTK focus is fully established
        subprocess.run(["xdotool", "windowactivate", "--sync", win_id], env=env, check=True)
        subprocess.run(["xdotool", "click", "1"], env=env, check=True)
        time.sleep(1.0)

        # 3. Type "ami " via xdotool key by key
        print("[Layer 6] Injecting key events 'a', 'm', 'i', 'space' via xdotool...")
        for k in ["a", "m", "i", "space"]:
            subprocess.run(["xdotool", "key", "--window", win_id, "--delay", "250", k], env=env, check=True)
            time.sleep(0.4)
            print(f"  After '{k}': entry text = '{window.get_text()}'")

        time.sleep(0.5)

        # Check entry text
        text = window.get_text()
        print(f"[Layer 6] Entry text after typing: '{text}'")

        if "আমি" in text:
            print("[Layer 6] SUCCESS: Bangla Unicode text 'আমি' received by GTK widget!")
            window.test_success = True
        else:
            print(f"[Layer 6] Text was: '{text}' (expected to contain 'আমি')")

    except Exception as e:
        print(f"[Layer 6] Worker exception: {e}")
    finally:
        GLib.idle_add(Gtk.main_quit)

def main():
    print("[Layer 6] Starting GTK3 GUI Integration Test...")
    ensure_fcitx5()
    win = BornoGtkTestWindow()
    win.show_all()

    # Start typing in background thread
    t = threading.Thread(target=run_typing_worker, args=(win,), daemon=True)
    t.start()

    Gtk.main()

    if win.test_success:
        print("[Layer 6] Real Graphical Application Integration Test PASSED!")
        sys.exit(0)
    else:
        print("[Layer 6] Real Graphical Application Integration Test FAILED.")
        sys.exit(1)

if __name__ == "__main__":
    main()

