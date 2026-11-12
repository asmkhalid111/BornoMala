#!/usr/bin/env python3
"""
Isolated Fcitx 5 + D-Bus + XDG session manager for non-destructive Linux GUI tests.

Guarantees:
- Never reads or modifies the real user's ~/.config/fcitx5/profile, ~/.local/lib/fcitx5,
  or GNOME gsettings.
- Allocates private temporary directories for HOME, XDG_CONFIG_HOME, XDG_DATA_HOME,
  and XDG_CACHE_HOME.
- Starts a private D-Bus session daemon and foreground fcitx5 child process, tracking
  their exact PIDs and terminating only verified owned children on exit.
"""

from __future__ import annotations

import dataclasses
import os
import re
import shutil
import signal
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Dict, List, Optional, Sequence


EXIT_PASSED = 0
EXIT_FAILED = 1
EXIT_UNAVAILABLE = 77


class PrerequisiteUnavailableError(RuntimeError):
    """Raised when required display, D-Bus, or GUI tools are unavailable."""


class SessionFailureError(RuntimeError):
    """Raised when the isolated Fcitx session or addon load check fails."""


@dataclasses.dataclass(frozen=True)
class BuildArtifacts:
    build_dir: str
    addon_so: str
    addon_conf: str
    im_conf: str


def repo_root_from_scripts() -> Path:
    return Path(__file__).resolve().parents[3]


def resolve_build_artifacts(explicit_build_dir: Optional[str] = None, env: Optional[Dict[str, str]] = None) -> BuildArtifacts:
    """Resolve freshly built WP1 Fcitx 5 addon artifacts without falling back to ~/.local."""
    env_map = env if env is not None else os.environ
    env_dir = env_map.get("BORNO_FCITX5_BUILD_DIR", "").strip()

    if explicit_build_dir:
        candidates = [Path(explicit_build_dir)]
    elif env_dir:
        candidates = [Path(env_dir)]
    else:
        candidates = [
            Path("/tmp/bornomala-m8-wp1-cmake"),
            repo_root_from_scripts() / "build",
        ]

    checked: List[str] = []
    for base in candidates:
        so_path = base / "bornomala.so"
        addon_conf = base / "bornomala.conf"
        im_conf = base / "bornomala-im.conf"
        checked.append(str(base))
        if so_path.is_file() and addon_conf.is_file() and im_conf.is_file():
            return BuildArtifacts(
                build_dir=str(base.resolve()),
                addon_so=str(so_path.resolve()),
                addon_conf=str(addon_conf.resolve()),
                im_conf=str(im_conf.resolve()),
            )

    raise PrerequisiteUnavailableError(
        "Fresh Fcitx 5 build artifacts (bornomala.so, bornomala.conf, bornomala-im.conf) "
        f"not found in: {', '.join(checked)}. Build WP1 first via CMake before running GUI tests."
    )


def _read_proc_comm(pid: int) -> Optional[str]:
    comm_path = Path(f"/proc/{pid}/comm")
    try:
        return comm_path.read_text(encoding="utf-8").strip()
    except OSError:
        return None


def _pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False
    except PermissionError:
        return True


def terminate_owned_pid(pid: Optional[int], expected_comms: Sequence[str], timeout_sec: float = 3.0) -> None:
    """Verify child process identity and terminate only that exact owned PID cleanly."""
    if not pid or pid <= 1:
        return
    if not _pid_alive(pid):
        return

    comm = _read_proc_comm(pid)
    if expected_comms and comm is not None and not any(comm.startswith(prefix) for prefix in expected_comms):
        print(f"[Harness] Skipping kill for PID {pid}: expected {expected_comms}, found '{comm}'.")
        return

    try:
        os.kill(pid, signal.SIGTERM)
    except ProcessLookupError:
        return

    deadline = time.monotonic() + timeout_sec
    while time.monotonic() < deadline:
        if not _pid_alive(pid):
            return
        time.sleep(0.05)

    # Re-verify identity before final escalation on this owned PID only
    comm_after = _read_proc_comm(pid)
    if expected_comms and comm_after is not None and not any(comm_after.startswith(prefix) for prefix in expected_comms):
        return

    print(f"[Harness] Child PID {pid} ({comm_after or 'unknown'}) ignored SIGTERM; escalating SIGKILL on owned child.")
    try:
        os.kill(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def terminate_owned_popen(proc: Optional[subprocess.Popen], expected_comms: Sequence[str], timeout_sec: float = 3.0) -> None:
    if proc is None:
        return
    if proc.poll() is not None:
        return
    terminate_owned_pid(proc.pid, expected_comms, timeout_sec=timeout_sec)
    try:
        proc.wait(timeout=1.0)
    except subprocess.TimeoutExpired:
        pass


class IsolatedFcitxSession:
    """Context manager launching a fully isolated Fcitx 5 + private D-Bus test environment."""

    def __init__(
        self,
        build_dir: Optional[str] = None,
        keep_temp: bool = False,
        require_xdotool: bool = True,
        require_gedit: bool = False,
        require_gtk3: bool = True,
    ) -> None:
        self.explicit_build_dir = build_dir
        self.keep_temp = keep_temp
        self.require_xdotool = require_xdotool
        self.require_gedit = require_gedit
        self.require_gtk3 = require_gtk3

        self.root_dir: Optional[Path] = None
        self.artifacts: Optional[BuildArtifacts] = None
        self.env: Dict[str, str] = {}
        self.dbus_pid: Optional[int] = None
        self.fcitx_proc: Optional[subprocess.Popen] = None
        self.owned_apps: List[subprocess.Popen] = []
        self._log_handle = None
        self.log_path: Optional[Path] = None

    def check_prerequisites(self) -> None:
        self.artifacts = resolve_build_artifacts(self.explicit_build_dir)

        display = os.environ.get("DISPLAY", "").strip()
        if not display:
            raise PrerequisiteUnavailableError("DISPLAY environment variable is unset; no X11/XWayland server available.")

        if display.startswith(":"):
            display_num = display[1:].split(".", 1)[0]
            x_sock = Path(f"/tmp/.X11-unix/X{display_num}")
            if not x_sock.exists():
                raise PrerequisiteUnavailableError(f"X11 socket {x_sock} does not exist for DISPLAY={display}.")

        required_bins = ["fcitx5", "fcitx5-remote", "dbus-daemon"]
        if self.require_xdotool:
            required_bins.append("xdotool")
        if self.require_gedit:
            required_bins.append("gedit")

        missing = [tool for tool in required_bins if shutil.which(tool) is None]
        if missing:
            raise PrerequisiteUnavailableError(f"Required runtime tools unavailable: {', '.join(missing)}")

        if self.require_gtk3:
            try:
                import gi  # noqa: F401
                gi.require_version("Gtk", "3.0")
                gi.require_version("Gdk", "3.0")
                # Check GIR typelib presence without importing Gtk/Gdk early so GDK_BACKEND=x11 applies cleanly
                repo = gi.Repository.get_default()
                repo.require("Gtk", "3.0", 0)
            except Exception as exc:
                raise PrerequisiteUnavailableError(f"PyGObject GTK3 unavailable: {exc}") from exc

    def _prepare_temp_tree(self) -> None:
        self.root_dir = Path(tempfile.mkdtemp(prefix="bornomala-fcitx5-test-"))
        home_dir = self.root_dir / "home"
        cfg_dir = self.root_dir / "config"
        data_dir = self.root_dir / "data"
        cache_dir = self.root_dir / "cache"

        (cfg_dir / "fcitx5").mkdir(parents=True, exist_ok=True)
        (cfg_dir / "com.bornomala.desktop").mkdir(parents=True, exist_ok=True)
        (data_dir / "fcitx5" / "addon").mkdir(parents=True, exist_ok=True)
        (data_dir / "fcitx5" / "inputmethod").mkdir(parents=True, exist_ok=True)
        (data_dir / "icons" / "hicolor" / "64x64" / "apps").mkdir(parents=True, exist_ok=True)
        home_dir.mkdir(parents=True, exist_ok=True)
        cache_dir.mkdir(parents=True, exist_ok=True)

        assert self.artifacts is not None
        shutil.copy2(self.artifacts.addon_conf, data_dir / "fcitx5" / "addon" / "bornomala.conf")
        shutil.copy2(self.artifacts.im_conf, data_dir / "fcitx5" / "inputmethod" / "bornomala.conf")

        icon_src = repo_root_from_scripts() / "native" / "bornomala-fcitx5" / "data" / "bornomala.png"
        if icon_src.is_file():
            shutil.copy2(icon_src, data_dir / "icons" / "hicolor" / "64x64" / "apps" / "bornomala.png")

        profile_text = (
            "[Groups/0]\n"
            "Name=Default\n"
            "Default Layout=us\n"
            "DefaultIM=bornomala\n\n"
            "[Groups/0/Items/0]\n"
            "Name=keyboard-us\n"
            "Layout=\n\n"
            "[Groups/0/Items/1]\n"
            "Name=bornomala\n"
            "Layout=\n\n"
            "[GroupOrder]\n"
            "0=Default\n"
        )
        (cfg_dir / "fcitx5" / "profile").write_text(profile_text, encoding="utf-8")

        env = os.environ.copy()
        env["HOME"] = str(home_dir)
        env["XDG_CONFIG_HOME"] = str(cfg_dir)
        env["XDG_DATA_HOME"] = str(data_dir)
        env["XDG_CACHE_HOME"] = str(cache_dir)
        env["XDG_DATA_DIRS"] = f"{data_dir}:/usr/local/share:/usr/share"
        env["FCITX_ADDON_DIRS"] = f"/usr/lib/x86_64-linux-gnu/fcitx5:{self.artifacts.build_dir}"
        env["GTK_IM_MODULE"] = "fcitx"
        env["QT_IM_MODULE"] = "fcitx"
        env["XMODIFIERS"] = "@im=fcitx"
        env["GDK_BACKEND"] = "x11"
        env["PYTHONDONTWRITEBYTECODE"] = "1"
        self.env = env

    def _start_private_dbus(self) -> None:
        dbus_cmd = [
            "dbus-daemon",
            "--session",
            "--fork",
            "--nopidfile",
            "--print-address=1",
            "--print-pid=1",
        ]
        try:
            out = subprocess.check_output(dbus_cmd, env=self.env, text=True)
        except subprocess.CalledProcessError as exc:
            raise SessionFailureError(f"Failed to start private dbus-daemon: {exc}") from exc

        lines = [ln.strip() for ln in out.splitlines() if ln.strip()]
        if len(lines) < 2:
            raise SessionFailureError(f"Unexpected dbus-daemon output: {out!r}")

        self.env["DBUS_SESSION_BUS_ADDRESS"] = lines[0]
        self.dbus_pid = int(lines[1])

    def _start_fcitx5(self) -> None:
        assert self.root_dir is not None
        self.log_path = self.root_dir / "fcitx5.log"
        self._log_handle = open(self.log_path, "w", encoding="utf-8")
        self.fcitx_proc = subprocess.Popen(
            ["fcitx5", "--verbose", "default=4,bornomala=5", "--disable", "waylandim", "-D"],
            env=self.env,
            stdout=self._log_handle,
            stderr=self._log_handle,
        )

        # Wait for the isolated Fcitx 5 instance to register on our private D-Bus bus
        deadline = time.monotonic() + 8.0
        while time.monotonic() < deadline:
            if self.fcitx_proc.poll() is not None:
                raise SessionFailureError(
                    f"Isolated fcitx5 exited unexpectedly with code {self.fcitx_proc.returncode}"
                )
            rc = subprocess.run(
                ["fcitx5-remote", "--check"],
                env=self.env,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            ).returncode
            if rc == 0:
                return
            time.sleep(0.15)

        raise SessionFailureError("Timed out waiting for isolated fcitx5 daemon on private D-Bus.")

    def start(self) -> "IsolatedFcitxSession":
        self.check_prerequisites()
        self._prepare_temp_tree()
        try:
            self._start_private_dbus()
            self._start_fcitx5()
        except Exception:
            self.cleanup()
            raise
        return self

    def verify_loaded_addon(self) -> str:
        """Confirm via /proc/<pid>/maps that our exact built bornomala.so is loaded."""
        assert self.fcitx_proc is not None and self.artifacts is not None
        maps_file = Path(f"/proc/{self.fcitx_proc.pid}/maps")
        try:
            content = maps_file.read_text(encoding="utf-8")
        except OSError as exc:
            raise SessionFailureError(f"Cannot inspect {maps_file}: {exc}") from exc

        expected_real = os.path.realpath(self.artifacts.addon_so)
        matched_paths = sorted(
            {
                ln.split()[-1]
                for ln in content.splitlines()
                if "bornomala.so" in ln and ln.split()
            }
        )
        if not matched_paths:
            raise SessionFailureError("bornomala.so was not mapped into isolated fcitx5 process.")
        if expected_real not in [os.path.realpath(p) for p in matched_paths]:
            raise SessionFailureError(
                f"Wrong bornomala.so loaded: expected {expected_real}, found {matched_paths}"
            )
        return expected_real

    def track_app(self, proc: subprocess.Popen) -> subprocess.Popen:
        self.owned_apps.append(proc)
        return proc

    def apply_env_to_current_process(self) -> None:
        """Apply isolated XDG/IM/D-Bus vars to current Python process before Gtk import."""
        for key in (
            "HOME",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "XDG_DATA_DIRS",
            "FCITX_ADDON_DIRS",
            "DBUS_SESSION_BUS_ADDRESS",
            "GTK_IM_MODULE",
            "QT_IM_MODULE",
            "XMODIFIERS",
            "GDK_BACKEND",
            "DISPLAY",
        ):
            if key in self.env:
                os.environ[key] = self.env[key]

    def scan_log_for_leaks(self) -> List[str]:
        if not self.log_path or not self.log_path.is_file():
            return []
        text = self.log_path.read_text(encoding="utf-8", errors="replace")
        pattern = re.compile(r"(sym=0x|preedit=|committed=)")
        return [ln for ln in text.splitlines() if pattern.search(ln)]

    def cleanup(self) -> None:
        for app in reversed(self.owned_apps):
            terminate_owned_popen(app, expected_comms=())
        self.owned_apps.clear()

        if self.fcitx_proc is not None:
            if self.fcitx_proc.poll() is None:
                subprocess.run(
                    ["fcitx5-remote", "-e"],
                    env=self.env,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
            terminate_owned_popen(self.fcitx_proc, expected_comms=("fcitx5",))
            self.fcitx_proc = None

        if self.dbus_pid is not None:
            terminate_owned_pid(self.dbus_pid, expected_comms=("dbus-daemon",))
            self.dbus_pid = None

        if self._log_handle is not None:
            try:
                self._log_handle.close()
            except OSError:
                pass
            self._log_handle = None

        if self.root_dir is not None and not self.keep_temp:
            shutil.rmtree(self.root_dir, ignore_errors=True)
            self.root_dir = None

    def __enter__(self) -> "IsolatedFcitxSession":
        return self.start()

    def __exit__(self, exc_type, exc, tb) -> None:
        self.cleanup()

