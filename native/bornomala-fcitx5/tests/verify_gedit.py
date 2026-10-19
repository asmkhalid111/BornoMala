#!/usr/bin/env python3
"""
Layer 6B: Verification of Live Bangla Typing in Gedit (GNOME Text Editor under WSLg)
Tests end-to-end typing via Fcitx 5 + BornoMala into a standard desktop text editor.
"""

import os
import sys
import time
import subprocess

def get_visible_windows(env):
    try:
        out = subprocess.check_output(
            ["xdotool", "search", "--onlyvisible", ""],
            env=env,
            stderr=subprocess.DEVNULL
        ).decode().strip().split()
        return [w for w in out if w.isdigit()]
    except Exception:
        return []

def ensure_fcitx5():
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

    subprocess.run(["pkill", "-9", "fcitx5"], capture_output=True)
    time.sleep(0.5)
    print("[Layer 6B] Starting fcitx5 daemon with logging...")
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

def main():
    print("[Layer 6B] Starting Gedit End-to-End Typing Test...")
    ensure_fcitx5()

    test_file = "/tmp/borno_test_gedit.txt"
    if os.path.exists(test_file):
        os.remove(test_file)

    env = os.environ.copy()
    uid = os.getuid()
    bus_path = f"/run/user/{uid}/bus"
    if os.path.exists(bus_path):
        env["DBUS_SESSION_BUS_ADDRESS"] = f"unix:path={bus_path}"

    env["GTK_IM_MODULE"] = "fcitx"
    env["QT_IM_MODULE"] = "fcitx"
    env["XMODIFIERS"] = "@im=fcitx"
    env["GDK_BACKEND"] = "x11"
    env["DISPLAY"] = env.get("DISPLAY", ":0")
    env["FCITX_ADDON_DIRS"] = "/usr/lib/x86_64-linux-gnu/fcitx5:" + os.path.expanduser("~/.local/lib/fcitx5")

    # Ensure gsettings points to fcitx
    try:
        subprocess.run(["gsettings", "set", "org.gnome.desktop.interface", "gtk-im-module", "fcitx"], env=env, check=False)
    except Exception:
        pass

    # Get baseline windows
    initial_windows = set(get_visible_windows(env))

    # 1. Launch Gedit standalone (-s)
    print("[Layer 6B] Launching Gedit (standalone)...")
    proc = subprocess.Popen(["gedit", "-s", test_file], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    win_id = None
    for _ in range(20):
        time.sleep(0.5)
        current_windows = set(get_visible_windows(env))
        new_windows = current_windows - initial_windows
        if new_windows:
            win_id = list(new_windows)[-1]
            break

    if not win_id:
        print("[Layer 6B] Could not detect new Gedit window")
        proc.kill()
        sys.exit(1)

    print(f"[Layer 6B] Found Gedit window ID: {win_id}")

    try:
        # 2. Activate and focus window
        subprocess.run(["xdotool", "windowactivate", "--sync", win_id], env=env, check=True)
        subprocess.run(["xdotool", "windowfocus", "--sync", win_id], env=env, check=True)
        time.sleep(1.0)

        # 3. Switch to BornoMala input method
        subprocess.run(["fcitx5-remote", "-s", "bornomala"], env=env)
        time.sleep(0.5)
        subprocess.run(["fcitx5-remote", "-o"], env=env)
        time.sleep(0.5)

        im_name = subprocess.check_output(["fcitx5-remote", "-n"], env=env).decode().strip()
        print(f"[Layer 6B] Active IM name: '{im_name}'")

        # Click inside the editor text view to focus
        subprocess.run(["xdotool", "windowactivate", "--sync", win_id], env=env, check=True)
        subprocess.run(["xdotool", "mousemove", "--window", win_id, "200", "200", "click", "1"], env=env, check=True)
        time.sleep(1.0)

        # 4. Type "ami "
        print("[Layer 6B] Injecting keys 'a', 'm', 'i', 'space' into Gedit...")
        for k in ["a", "m", "i", "space"]:
            subprocess.run(["xdotool", "key", "--window", win_id, "--delay", "250", k], env=env, check=True)
            time.sleep(0.4)

        time.sleep(1.0)

        # 5. Save via Ctrl+S
        print("[Layer 6B] Saving document via Ctrl+S...")
        subprocess.run(["xdotool", "key", "--window", win_id, "ctrl+s"], env=env, check=True)
        time.sleep(2.0)

    finally:
        # Terminate Gedit
        proc.terminate()
        try:
            proc.wait(timeout=2.0)
        except subprocess.TimeoutExpired:
            proc.kill()

    # 6. Verify saved file content
    if not os.path.exists(test_file):
        print(f"[Layer 6B] FAILED: Output file {test_file} was not written")
        sys.exit(1)

    with open(test_file, "r", encoding="utf-8") as f:
        content = f.read()

    print(f"[Layer 6B] Read content from Gedit: '{content}'")
    if "আমি" in content:
        print("[Layer 6B] SUCCESS: Real Gedit application successfully received and saved Bangla Unicode text 'আমি'!")
        sys.exit(0)
    else:
        print(f"[Layer 6B] FAILED: Text in gedit was '{content}' (expected to contain 'আমি')")
        sys.exit(1)

if __name__ == "__main__":
    main()
