#!/usr/bin/env python3
"""
Layer 6B: Isolated Real Desktop Application Integration Test (GNOME Gedit under X11/WSLg)

Launches standalone Gedit inside an isolated private D-Bus + XDG temporary session,
loads the freshly built WP1 bornomala.so, verifies Phonetic typing + file save, and
cleans up only tracked owned child PIDs without modifying user gsettings or profiles.
"""

import argparse
import os
import subprocess
import sys
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
    terminate_owned_popen,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Isolated BornoMala Fcitx 5 Gedit verification")
    parser.add_argument("--build-dir", default=None, help="Directory containing built bornomala.so and .conf files")
    parser.add_argument("--keep-temp", action="store_true", help="Keep temporary isolated session directory on exit")
    return parser.parse_args()


def get_visible_windows(env) -> set:
    try:
        out = subprocess.check_output(
            ["xdotool", "search", "--onlyvisible", ""],
            env=env,
            stderr=subprocess.DEVNULL,
        ).decode().strip().split()
        return {w for w in out if w.isdigit()}
    except Exception:
        return set()


def run_gedit_suite(session: IsolatedFcitxSession) -> int:
    loaded_so = session.verify_loaded_addon()
    print(f"[Layer 6B] Verified loaded addon binary: {loaded_so}")

    assert session.root_dir is not None
    test_file = session.root_dir / "gedit_output.txt"

    initial_windows = get_visible_windows(session.env)
    proc = session.track_app(
        subprocess.Popen(
            ["gedit", "-s", str(test_file)],
            env=session.env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    )

    win_id = None
    try:
        for _ in range(30):
            if proc.poll() is not None:
                print(f"RESULT=FAILED: gedit exited early with code {proc.returncode}", file=sys.stderr)
                return EXIT_FAILED
            time.sleep(0.35)
            new_windows = get_visible_windows(session.env) - initial_windows
            if new_windows:
                win_id = sorted(new_windows)[-1]
                break

        if not win_id:
            print("RESULT=FAILED: timed out detecting Gedit window", file=sys.stderr)
            return EXIT_FAILED

        print(f"[Layer 6B] Found Gedit window ID: {win_id} (PID {proc.pid})")

        subprocess.run(["xdotool", "windowactivate", "--sync", win_id], env=session.env, check=True)
        subprocess.run(["xdotool", "windowfocus", "--sync", win_id], env=session.env, check=True)
        time.sleep(0.8)

        subprocess.run(["fcitx5-remote", "-s", "bornomala"], env=session.env, check=False)
        subprocess.run(["fcitx5-remote", "-o"], env=session.env, check=False)
        time.sleep(0.4)

        im_name = subprocess.check_output(["fcitx5-remote", "-n"], env=session.env, text=True).strip()
        print(f"[Layer 6B] Active IM name: '{im_name}'")

        subprocess.run(["xdotool", "mousemove", "--window", win_id, "200", "200", "click", "1"], env=session.env, check=True)
        time.sleep(0.8)

        for k in ["a", "m", "i", "space"]:
            subprocess.run(["xdotool", "key", "--window", win_id, "--delay", "250", k], env=session.env, check=True)
            time.sleep(0.35)

        time.sleep(0.8)
        subprocess.run(["xdotool", "key", "--window", win_id, "ctrl+s"], env=session.env, check=True)
        time.sleep(1.5)
    finally:
        terminate_owned_popen(proc, expected_comms=("gedit",))

    if not test_file.is_file():
        print(f"RESULT=FAILED: output file was not created at {test_file}", file=sys.stderr)
        return EXIT_FAILED

    content = test_file.read_text(encoding="utf-8")
    print(f"[Layer 6B] Read content from Gedit: {content!r}")

    leaks = session.scan_log_for_leaks()
    if leaks:
        print(f"RESULT=FAILED: runtime privacy leak in isolated log: {leaks[:3]}", file=sys.stderr)
        return EXIT_FAILED

    if "আমি" in content:
        print("RESULT=PASSED")
        return EXIT_PASSED

    print(f"RESULT=FAILED: expected 'আমি' in Gedit output, got {content!r}", file=sys.stderr)
    return EXIT_FAILED


def main() -> int:
    args = parse_args()
    try:
        with IsolatedFcitxSession(
            build_dir=args.build_dir,
            keep_temp=args.keep_temp,
            require_xdotool=True,
            require_gedit=True,
            require_gtk3=False,
        ) as session:
            print(f"[Layer 6B] Isolated session dir: {session.root_dir}")
            print(f"[Layer 6B] Owned fcitx5 PID: {session.fcitx_proc.pid}, dbus PID: {session.dbus_pid}")
            return run_gedit_suite(session)
    except PrerequisiteUnavailableError as exc:
        print(f"RESULT=UNAVAILABLE: {exc}", file=sys.stderr)
        return EXIT_UNAVAILABLE
    except SessionFailureError as exc:
        print(f"RESULT=FAILED: {exc}", file=sys.stderr)
        return EXIT_FAILED


if __name__ == "__main__":
    sys.exit(main())
