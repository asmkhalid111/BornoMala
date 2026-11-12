#!/usr/bin/env python3
"""
Automated safety and isolation tests for M8-WP2 Linux scripts and GUI test harnesses.

Uses temporary fake HOME/XDG directories, stub executables on PATH, and real subprocess
invocations (never merely grepping source text).
"""

import os
import shutil
import stat
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

TESTS_DIR = Path(__file__).resolve().parent
FCITX_ROOT = TESTS_DIR.parent
REPO_ROOT = FCITX_ROOT.parents[1]
SCRIPTS_DIR = FCITX_ROOT / "scripts"
INSTALL_SH = SCRIPTS_DIR / "install.sh"
UNINSTALL_SH = SCRIPTS_DIR / "uninstall.sh"
LAUNCH_EDITOR_SH = SCRIPTS_DIR / "launch_test_editor.sh"
INTERACTIVE_PY = SCRIPTS_DIR / "interactive_tester.py"
VERIFY_GTK3_PY = TESTS_DIR / "verify_gtk3.py"
VERIFY_GEDIT_PY = TESTS_DIR / "verify_gedit.py"

if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import fcitx_test_env  # noqa: E402


def make_executable(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    path.chmod(0o755)


def make_fake_build_dir(build_dir: Path, variant: bytes = b"v1") -> None:
    build_dir.mkdir(parents=True, exist_ok=True)
    # Minimal valid ELF magic header for borno_is_valid_elf_module check
    (build_dir / "bornomala.so").write_bytes(b"\x7fELF\x02\x01\x01\x00" + variant)
    (build_dir / "bornomala.conf").write_text("[Addon]\nName=bornomala\n", encoding="utf-8")
    (build_dir / "bornomala-im.conf").write_text("[InputMethod]\nName=bornomala\n", encoding="utf-8")


def create_poison_stubs(bin_dir: Path, log_file: Path) -> None:
    """Place poison stubs for dangerous process-killing commands on PATH."""
    bin_dir.mkdir(parents=True, exist_ok=True)
    for cmd in ("killall", "pkill", "systemctl", "gsettings"):
        stub = bin_dir / cmd
        make_executable(
            stub,
            f"#!/usr/bin/env bash\n"
            f"printf '%s %s\\n' '{cmd}' \"$*\" >> '{log_file}'\n"
            f"exit 99\n",
        )


def build_clean_subprocess_env(fake_home: Path, poison_bin: Path, extra: dict = None) -> dict:
    # Remove XDG_* unless explicitly supplied by extra
    base_path = os.environ.get("PATH", "/usr/bin:/bin")
    env = {
        "HOME": str(fake_home),
        "PATH": f"{poison_bin}:{base_path}",
        "TERM": "xterm-256color",
        "PYTHONDONTWRITEBYTECODE": "1",
    }
    if extra:
        env.update(extra)
    return env


class InstallerSafetyTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="borno-wp2-inst-test-"))
        self.fake_home = self.tmp / "home"
        self.fake_home.mkdir()
        self.build_dir = self.tmp / "build"
        make_fake_build_dir(self.build_dir)
        self.poison_bin = self.tmp / "poison-bin"
        self.poison_log = self.tmp / "poison.log"
        create_poison_stubs(self.poison_bin, self.poison_log)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_default_destinations_and_profile_preservation(self):
        cfg_dir = self.fake_home / ".config" / "fcitx5"
        cfg_dir.mkdir(parents=True)
        profile_path = cfg_dir / "profile"
        original_profile = "[Groups/0]\nName=CustomUser\nDefaultIM=unikey\n"
        profile_path.write_text(original_profile, encoding="utf-8")

        shell_profile = self.fake_home / ".profile"
        shell_profile.write_text("# user shell rc\nexport FOO=bar\n", encoding="utf-8")

        unrelated_addon = self.fake_home / ".local" / "share" / "fcitx5" / "addon" / "unrelated.conf"
        unrelated_addon.parent.mkdir(parents=True)
        unrelated_addon.write_text("keep=true\n", encoding="utf-8")

        env = build_clean_subprocess_env(self.fake_home, self.poison_bin)
        res = subprocess.run(
            ["bash", str(INSTALL_SH), "--skip-build", "--build-dir", str(self.build_dir)],
            env=env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)
        self.assertFalse(self.poison_log.exists(), "Installer invoked forbidden process/settings command!")

        expected_so = self.fake_home / ".local" / "lib" / "fcitx5" / "bornomala.so"
        expected_addon = self.fake_home / ".local" / "share" / "fcitx5" / "addon" / "bornomala.conf"
        expected_im = self.fake_home / ".local" / "share" / "fcitx5" / "inputmethod" / "bornomala.conf"
        expected_icon = self.fake_home / ".local" / "share" / "icons" / "hicolor" / "64x64" / "apps" / "bornomala.png"
        expected_notices = self.fake_home / ".local" / "share" / "doc" / "bornomala" / "THIRD-PARTY-NOTICES.md"
        expected_manifest = self.fake_home / ".local" / "share" / "doc" / "bornomala" / "install-manifest.txt"

        for p in (expected_so, expected_addon, expected_im, expected_icon, expected_notices, expected_manifest):
            self.assertTrue(p.is_file(), f"Missing expected installed artifact: {p}")

        # Existing profile, shell rc, and unrelated addon config must remain byte-for-byte identical
        self.assertEqual(profile_path.read_text(encoding="utf-8"), original_profile)
        self.assertEqual(shell_profile.read_text(encoding="utf-8"), "# user shell rc\nexport FOO=bar\n")
        self.assertEqual(unrelated_addon.read_text(encoding="utf-8"), "keep=true\n")

        # Repeated install must be idempotent and report [unchanged]
        res2 = subprocess.run(
            ["bash", str(INSTALL_SH), "--skip-build", "--build-dir", str(self.build_dir)],
            env=env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res2.returncode, 0, res2.stderr + res2.stdout)
        self.assertIn("[unchanged]", res2.stdout)

    def test_xdg_data_home_explicit_empty_and_relative_fallbacks(self):
        explicit_xdg = self.tmp / "custom-xdg-data"
        env_explicit = build_clean_subprocess_env(
            self.fake_home,
            self.poison_bin,
            {"XDG_DATA_HOME": str(explicit_xdg)},
        )
        res = subprocess.run(
            ["bash", str(INSTALL_SH), "--skip-build", "--build-dir", str(self.build_dir)],
            env=env_explicit,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)
        self.assertTrue((explicit_xdg / "fcitx5" / "addon" / "bornomala.conf").is_file())
        self.assertFalse((self.fake_home / ".local" / "share" / "fcitx5" / "addon" / "bornomala.conf").exists())

        # Empty XDG_DATA_HOME -> fallback to $HOME/.local/share
        home2 = self.tmp / "home2"
        home2.mkdir()
        env_empty = build_clean_subprocess_env(home2, self.poison_bin, {"XDG_DATA_HOME": ""})
        res2 = subprocess.run(
            ["bash", str(INSTALL_SH), "--skip-build", "--build-dir", str(self.build_dir)],
            env=env_empty,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res2.returncode, 0, res2.stderr + res2.stdout)
        self.assertTrue((home2 / ".local" / "share" / "fcitx5" / "addon" / "bornomala.conf").is_file())

        # Relative XDG_DATA_HOME -> must be rejected with warning and fall back to $HOME/.local/share
        home3 = self.tmp / "home3"
        home3.mkdir()
        env_rel = build_clean_subprocess_env(home3, self.poison_bin, {"XDG_DATA_HOME": "relative/xdg-data"})
        res3 = subprocess.run(
            ["bash", str(INSTALL_SH), "--skip-build", "--build-dir", str(self.build_dir)],
            env=env_rel,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res3.returncode, 0, res3.stderr + res3.stdout)
        self.assertIn("ignoring invalid relative/malformed XDG_DATA_HOME", res3.stderr)
        self.assertTrue((home3 / ".local" / "share" / "fcitx5" / "addon" / "bornomala.conf").is_file())
        self.assertFalse((Path("relative") / "xdg-data").exists())

    def test_missing_build_artifact_fails_before_creating_directories(self):
        bad_build = self.tmp / "empty-build"
        bad_build.mkdir()
        home_clean = self.tmp / "home-clean"
        home_clean.mkdir()
        env = build_clean_subprocess_env(home_clean, self.poison_bin)
        res = subprocess.run(
            ["bash", str(INSTALL_SH), "--skip-build", "--build-dir", str(bad_build)],
            env=env,
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(res.returncode, 0)
        self.assertIn("Required ELF addon module not found", res.stderr)
        self.assertFalse((home_clean / ".local").exists(), "Installer wrote directories before validating artifacts!")

    def test_missing_notices_file_fails_before_writing_anything(self):
        # Assemble a staged copy of the installer without THIRD-PARTY-NOTICES.md at root
        staged_repo = self.tmp / "staged-repo"
        staged_scripts = staged_repo / "native" / "bornomala-fcitx5" / "scripts"
        staged_data = staged_repo / "native" / "bornomala-fcitx5" / "data"
        staged_scripts.mkdir(parents=True)
        staged_data.mkdir(parents=True)
        shutil.copy2(SCRIPTS_DIR / "common.sh", staged_scripts / "common.sh")
        shutil.copy2(INSTALL_SH, staged_scripts / "install.sh")
        shutil.copy2(FCITX_ROOT / "data" / "bornomala.png", staged_data / "bornomala.png")

        home_clean = self.tmp / "home-no-notices"
        home_clean.mkdir()
        env = build_clean_subprocess_env(home_clean, self.poison_bin)
        res = subprocess.run(
            ["bash", str(staged_scripts / "install.sh"), "--skip-build", "--build-dir", str(self.build_dir)],
            env=env,
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(res.returncode, 0)
        self.assertIn("third-party notices", res.stderr.lower())
        self.assertFalse((home_clean / ".local").exists(), "Installer wrote directories when THIRD-PARTY-NOTICES.md was missing!")

    def test_preserves_custom_preexisting_icon_and_opt_in_profile_merge(self):
        custom_icon = self.fake_home / ".local" / "share" / "icons" / "hicolor" / "64x64" / "apps" / "bornomala.png"
        custom_icon.parent.mkdir(parents=True)
        custom_bytes = b"CUSTOM_USER_ICON_DO_NOT_OVERWRITE"
        custom_icon.write_bytes(custom_bytes)

        cfg_dir = self.fake_home / ".config" / "fcitx5"
        cfg_dir.mkdir(parents=True)
        profile_path = cfg_dir / "profile"
        existing_profile = (
            "[Groups/0]\n"
            "Name=WorkGroup\n"
            "Default Layout=us\n"
            "DefaultIM=keyboard-us\n\n"
            "[Groups/0/Items/0]\n"
            "Name=keyboard-us\n"
            "Layout=\n\n"
            "[Groups/1]\n"
            "Name=SecondaryGroup\n\n"
            "[GroupOrder]\n"
            "0=WorkGroup\n"
            "1=SecondaryGroup\n"
        )
        profile_path.write_text(existing_profile, encoding="utf-8")

        env = build_clean_subprocess_env(self.fake_home, self.poison_bin)
        res = subprocess.run(
            ["bash", str(INSTALL_SH), "--skip-build", "--build-dir", str(self.build_dir), "--register-profile"],
            env=env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)
        # Custom icon untouched
        self.assertEqual(custom_icon.read_bytes(), custom_bytes)
        # Backup created + original IM/groups preserved + bornomala appended
        backups = list(cfg_dir.glob("profile.bak.*"))
        self.assertEqual(len(backups), 1)
        self.assertEqual(backups[0].read_text(encoding="utf-8"), existing_profile)
        updated = profile_path.read_text(encoding="utf-8")
        self.assertIn("Name=WorkGroup", updated)
        self.assertIn("Name=SecondaryGroup", updated)
        self.assertIn("Name=keyboard-us", updated)
        self.assertEqual(updated.count("Name=bornomala"), 1)


class UninstallerSafetyTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="borno-wp2-uninst-test-"))
        self.fake_home = self.tmp / "home"
        self.fake_home.mkdir()
        self.build_dir = self.tmp / "build"
        make_fake_build_dir(self.build_dir)
        self.poison_bin = self.tmp / "poison-bin"
        self.poison_log = self.tmp / "poison.log"
        create_poison_stubs(self.poison_bin, self.poison_log)
        self.env = build_clean_subprocess_env(self.fake_home, self.poison_bin)

        res = subprocess.run(
            ["bash", str(INSTALL_SH), "--skip-build", "--build-dir", str(self.build_dir)],
            env=self.env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_dry_run_and_noninteractive_confirmation_preserve_files(self):
        so_path = self.fake_home / ".local" / "lib" / "fcitx5" / "bornomala.so"
        self.assertTrue(so_path.is_file())

        # Non-interactive without --yes must fail cleanly without deleting anything
        res_refused = subprocess.run(
            ["bash", str(UNINSTALL_SH)],
            env=self.env,
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(res_refused.returncode, 0)
        self.assertTrue(so_path.is_file())

        # Dry-run must exit 0 and delete nothing
        res_dry = subprocess.run(
            ["bash", str(UNINSTALL_SH), "--dry-run"],
            env=self.env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res_dry.returncode, 0, res_dry.stderr + res_dry.stdout)
        self.assertIn("[would-remove]", res_dry.stdout)
        self.assertTrue(so_path.is_file())

    def test_confirmed_uninstall_preserves_unrelated_files_profile_and_is_idempotent(self):
        cfg_dir = self.fake_home / ".config" / "fcitx5"
        cfg_dir.mkdir(parents=True)
        profile = cfg_dir / "profile"
        profile_text = "[Groups/0]\nName=Default\nDefaultIM=keyboard-us\n"
        profile.write_text(profile_text, encoding="utf-8")

        unrelated_lib = self.fake_home / ".local" / "lib" / "fcitx5" / "other-im.so"
        unrelated_lib.write_bytes(b"other-so")
        unrelated_addon = self.fake_home / ".local" / "share" / "fcitx5" / "addon" / "other.conf"
        unrelated_addon.write_text("other=1\n", encoding="utf-8")

        res = subprocess.run(
            ["bash", str(UNINSTALL_SH), "--yes"],
            env=self.env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)
        self.assertFalse(self.poison_log.exists())

        # BornoMala-owned files and empty doc/bornomala dir removed
        self.assertFalse((self.fake_home / ".local" / "lib" / "fcitx5" / "bornomala.so").exists())
        self.assertFalse((self.fake_home / ".local" / "share" / "fcitx5" / "addon" / "bornomala.conf").exists())
        self.assertFalse((self.fake_home / ".local" / "share" / "fcitx5" / "inputmethod" / "bornomala.conf").exists())
        self.assertFalse((self.fake_home / ".local" / "share" / "icons" / "hicolor" / "64x64" / "apps" / "bornomala.png").exists())
        self.assertFalse((self.fake_home / ".local" / "share" / "doc" / "bornomala").exists())

        # Profile and unrelated files preserved
        self.assertEqual(profile.read_text(encoding="utf-8"), profile_text)
        self.assertEqual(unrelated_lib.read_bytes(), b"other-so")
        self.assertEqual(unrelated_addon.read_text(encoding="utf-8"), "other=1\n")

        # Second uninstall succeeds safely
        res2 = subprocess.run(
            ["bash", str(UNINSTALL_SH), "--yes"],
            env=self.env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res2.returncode, 0, res2.stderr + res2.stdout)

    def test_malformed_manifest_cannot_delete_arbitrary_paths(self):
        decoy_file = self.tmp / "precious-outside-file.txt"
        decoy_file.write_text("DO_NOT_DELETE", encoding="utf-8")

        profile = self.fake_home / ".config" / "fcitx5" / "profile"
        profile.parent.mkdir(parents=True)
        profile.write_text("KEEP_PROFILE=1\n", encoding="utf-8")

        manifest = self.fake_home / ".local" / "share" / "doc" / "bornomala" / "install-manifest.txt"
        manifest.write_text(
            f"FILE|abc|{decoy_file}\n"
            f"FILE|abc|/etc/passwd\n"
            f"FILE|abc|{self.fake_home}/.local/lib/fcitx5/../../precious-outside-file.txt\n"
            f"FILE|abc|{profile}\n"
            "MALFORMED LINE\n",
            encoding="utf-8",
        )

        res = subprocess.run(
            ["bash", str(UNINSTALL_SH), "--yes"],
            env=self.env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)
        self.assertIn("rejecting unowned or unsafe path", res.stderr)
        self.assertEqual(decoy_file.read_text(encoding="utf-8"), "DO_NOT_DELETE")
        self.assertEqual(profile.read_text(encoding="utf-8"), "KEEP_PROFILE=1\n")


class GuiHarnessIsolationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="borno-wp2-gui-test-"))
        self.fake_home = self.tmp / "real-home"
        self.fake_home.mkdir()
        self.real_profile = self.fake_home / ".config" / "fcitx5" / "profile"
        self.real_profile.parent.mkdir(parents=True)
        self.real_profile.write_text("REAL_USER_PROFILE=UNCHANGED\n", encoding="utf-8")

        self.stale_local_so = self.fake_home / ".local" / "lib" / "fcitx5" / "bornomala.so"
        self.stale_local_so.parent.mkdir(parents=True)
        self.stale_local_so.write_bytes(b"\x7fELFstale")

        self.wp1_build = self.tmp / "wp1-build"
        make_fake_build_dir(self.wp1_build, variant=b"wp1-fresh")

        self.poison_bin = self.tmp / "poison-bin"
        self.poison_log = self.tmp / "poison.log"
        create_poison_stubs(self.poison_bin, self.poison_log)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_isolated_session_dirs_never_point_at_real_home_or_stale_local_so(self):
        sess = fcitx_test_env.IsolatedFcitxSession(build_dir=str(self.wp1_build))
        sess.artifacts = fcitx_test_env.resolve_build_artifacts(str(self.wp1_build))
        sess._prepare_temp_tree()
        try:
            self.assertNotEqual(Path(sess.env["HOME"]), self.fake_home)
            self.assertNotEqual(Path(sess.env["XDG_CONFIG_HOME"]), self.fake_home / ".config")
            self.assertNotEqual(Path(sess.env["XDG_DATA_HOME"]), self.fake_home / ".local" / "share")
            self.assertIn(str(self.wp1_build.resolve()), sess.env["FCITX_ADDON_DIRS"])
            self.assertNotIn(".local/lib/fcitx5", sess.env["FCITX_ADDON_DIRS"])
            self.assertTrue((Path(sess.env["XDG_CONFIG_HOME"]) / "fcitx5" / "profile").is_file())
            self.assertEqual(self.real_profile.read_text(encoding="utf-8"), "REAL_USER_PROFILE=UNCHANGED\n")
            self.assertFalse(self.poison_log.exists())
        finally:
            sess.cleanup()
        self.assertIsNone(sess.root_dir)

    def test_missing_build_artifacts_never_falls_back_to_stale_local_so(self):
        old_home = os.environ.get("HOME")
        try:
            os.environ["HOME"] = str(self.fake_home)
            missing_dir = self.tmp / "does-not-exist"
            with self.assertRaises(fcitx_test_env.PrerequisiteUnavailableError):
                fcitx_test_env.resolve_build_artifacts(
                    explicit_build_dir=str(missing_dir),
                    env={"BORNO_FCITX5_BUILD_DIR": str(missing_dir)},
                )
        finally:
            if old_home is not None:
                os.environ["HOME"] = old_home

    def test_terminate_owned_pid_respects_comm_and_stops_only_owned_child(self):
        # Launch an inert sleep child and verify mismatch comm does NOT terminate it,
        # while matching comm ("sleep") DOES terminate that exact child cleanly.
        child = subprocess.Popen(["sleep", "30"])
        try:
            fcitx_test_env.terminate_owned_pid(child.pid, expected_comms=("fcitx5",), timeout_sec=0.3)
            time.sleep(0.1)
            self.assertIsNone(child.poll(), "Mismatched comm must not kill unrelated PID!")

            fcitx_test_env.terminate_owned_pid(child.pid, expected_comms=("sleep",), timeout_sec=2.0)
            child.wait(timeout=2.0)
            self.assertIsNotNone(child.poll(), "Owned child PID was not terminated cleanly")
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()

    def test_unavailable_display_returns_exit_77_not_pass_and_runs_no_kills(self):
        env = build_clean_subprocess_env(
            self.fake_home,
            self.poison_bin,
            {"DISPLAY": ""},
        )

        for script_cmd in (
            [sys.executable, "-B", str(VERIFY_GTK3_PY), "--build-dir", str(self.wp1_build)],
            [sys.executable, "-B", str(VERIFY_GEDIT_PY), "--build-dir", str(self.wp1_build)],
            [sys.executable, "-B", str(INTERACTIVE_PY), "--build-dir", str(self.wp1_build)],
            ["bash", str(LAUNCH_EDITOR_SH), "--build-dir", str(self.wp1_build)],
        ):
            res = subprocess.run(script_cmd, env=env, capture_output=True, text=True)
            self.assertEqual(
                res.returncode,
                77,
                f"Expected exit code 77 (UNAVAILABLE) for {script_cmd}, got {res.returncode}: {res.stderr}\n{res.stdout}",
            )
            self.assertIn("RESULT=UNAVAILABLE", res.stderr)
            self.assertEqual(self.real_profile.read_text(encoding="utf-8"), "REAL_USER_PROFILE=UNCHANGED\n")
            self.assertFalse(self.poison_log.exists(), f"Forbidden command invoked by {script_cmd}")


class LaunchTestEditorCleanupTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="borno-wp2-editor-test-"))
        self.fake_home = self.tmp / "home"
        self.fake_home.mkdir()
        self.poison_bin = self.tmp / "poison-bin"
        self.poison_log = self.tmp / "poison.log"
        create_poison_stubs(self.poison_bin, self.poison_log)
        self.env = build_clean_subprocess_env(self.fake_home, self.poison_bin)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _run_sourced_editor_bash(self, bash_body: str) -> subprocess.CompletedProcess:
        script = f'source "{LAUNCH_EDITOR_SH}"\n{bash_body}\n'
        return subprocess.run(
            ["bash", "-c", script],
            env=self.env,
            capture_output=True,
            text=True,
        )

    def test_valid_owned_child_cleaned_up_and_temp_dir_bounded(self):
        session_dir = self.tmp / "bornomala-fcitx5-editor.valid1"
        session_dir.mkdir()
        (session_dir / "fcitx5.log").write_text("temp log\n", encoding="utf-8")

        outside_dir = self.tmp / "outside-precious"
        outside_dir.mkdir()
        outside_file = outside_dir / "keep.txt"
        outside_file.write_text("KEEP", encoding="utf-8")

        res = self._run_sourced_editor_bash(
            f"""
sleep 30 &
child_pid=$!
borno_editor_capture_pid_identity "$child_pid" "sleep"
FCITX_PID="$child_pid"
FCITX_COMM="$BORNO_CAPTURED_COMM"
FCITX_STARTTIME="$BORNO_CAPTURED_STARTTIME"
FCITX_EXPECTED_PREFIX="sleep"
SESSION_TMP="{session_dir}"
KEEP_TEMP=0
borno_editor_cleanup
if kill -0 "$child_pid" 2>/dev/null; then
    echo "STILL_ALIVE"
else
    echo "TERMINATED_OK"
fi
"""
        )
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)
        self.assertIn("TERMINATED_OK", res.stdout)
        self.assertFalse(session_dir.exists(), "Launcher-owned SESSION_TMP was not removed")
        self.assertEqual(outside_file.read_text(encoding="utf-8"), "KEEP")
        self.assertFalse(self.poison_log.exists(), "Cleanup invoked forbidden process-killing command!")

    def test_already_exited_and_zombie_children_handled_without_signaling(self):
        res = self._run_sourced_editor_bash(
            """
sleep 0.01 &
reaped_pid=$!
wait "$reaped_pid"
DBUS_PID="$reaped_pid"
DBUS_COMM="sleep"
DBUS_STARTTIME="1"
DBUS_EXPECTED_PREFIX="sleep"

bash -c 'exit 0' &
zombie_pid=$!
sleep 0.05
if borno_editor_read_proc_identity "$zombie_pid"; then
    FCITX_PID="$zombie_pid"
    FCITX_COMM="$BORNO_PROC_COMM"
    FCITX_STARTTIME="$BORNO_PROC_STARTTIME"
    FCITX_EXPECTED_PREFIX="$BORNO_PROC_COMM"
else
    FCITX_PID="$zombie_pid"
    FCITX_COMM="bash"
    FCITX_STARTTIME="1"
    FCITX_EXPECTED_PREFIX="bash"
fi

borno_editor_cleanup
echo "EXITED_CLEANLY"
"""
        )
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)
        self.assertIn("EXITED_CLEANLY", res.stdout)
        self.assertEqual(res.stderr.strip(), "")
        self.assertFalse(self.poison_log.exists())

    def test_mismatched_comm_starttime_or_missing_identity_never_signals_pid(self):
        child = subprocess.Popen(["sleep", "30"])
        try:
            stat_text = Path(f"/proc/{child.pid}/stat").read_text(encoding="utf-8")
            actual_starttime = stat_text.rsplit(") ", 1)[-1].split()[19]

            cases = [
                ("mismatched_comm", "fcitx5", actual_starttime, "fcitx5", "identity mismatch"),
                ("reused_pid_starttime", "sleep", "999999999", "sleep", "identity mismatch"),
                ("missing_identity", "", "", "fcitx5", "process identity was not recorded"),
            ]
            for label, fake_comm, fake_start, expected_prefix, expected_msg in cases:
                res = self._run_sourced_editor_bash(
                    f"""
FCITX_PID="{child.pid}"
FCITX_COMM="{fake_comm}"
FCITX_STARTTIME="{fake_start}"
FCITX_EXPECTED_PREFIX="{expected_prefix}"
borno_editor_cleanup
"""
                )
                self.assertEqual(res.returncode, 0, f"{label}: {res.stderr + res.stdout}")
                self.assertIn(expected_msg, res.stderr, f"{label}: missing diagnostic in {res.stderr!r}")
                time.sleep(0.05)
                self.assertIsNone(child.poll(), f"{label}: unrelated PID {child.pid} was signaled!")
                self.assertFalse(self.poison_log.exists(), f"{label}: forbidden command invoked!")
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()

    def test_non_launcher_temp_dir_is_not_removed(self):
        outside_dir = self.tmp / "precious-unrelated-dir"
        outside_dir.mkdir()
        precious_file = outside_dir / "important.txt"
        precious_file.write_text("DO_NOT_DELETE", encoding="utf-8")

        res = self._run_sourced_editor_bash(
            f"""
SESSION_TMP="{outside_dir}"
KEEP_TEMP=0
borno_editor_cleanup
"""
        )
        self.assertEqual(res.returncode, 0, res.stderr + res.stdout)
        self.assertIn("Refusing to remove non-launcher temporary directory", res.stderr)
        self.assertEqual(precious_file.read_text(encoding="utf-8"), "DO_NOT_DELETE")
        self.assertFalse(self.poison_log.exists())


if __name__ == "__main__":
    unittest.main()

