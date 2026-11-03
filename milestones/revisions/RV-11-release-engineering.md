# RV-11 — Release Engineering (Installers, Signing, Migration)

> **Revision ID:** RV-11
> **Status:** Proposed — awaiting developer approval
> **Priority:** High (Phase 2; gated)
> **Estimated effort:** 5–8 days
> **Dependencies:** RV-03 (canonical path), RV-04 (TSF verified), RV-05 (Linux verified), RV-10 (NOTICE), RV-07/08 (app architecture)
> **Blocks:** M12, M13

---

## 1. Why this revision exists

There is no install story. The current “installation” is:

- `bornomala_tsf.dll` registered from the **build directory** by a `.bat` file run as administrator ([`borno_tsf.cpp:946-950`](../../native/borno-tsf/borno_tsf.cpp#L946-L950)).
- A portable `Borno-Sandbox.exe` in the repo root.
- A Linux `install.sh` that copies into `~/.local` and edits the user’s Fcitx profile.
- No uninstaller, no upgrade path, no signature, no checksums, no support matrix.

The audit’s release-readiness score is 2/10 for this reason.

**Hard rule:** do not start this revision until RV-04 and RV-05 have recorded end-to-end evidence. Packaging a broken composition lifecycle or an unreproducible Linux build is a liability.

---

## 2. Current-state inspection

Read:

- `native/borno-tsf/scripts/register.bat`, `unregister.bat`, `src-tauri/tauri.conf.json` (`bundle.active: false`)
- `native/bornomala-fcitx5/CMakeLists.txt`, `scripts/install.sh`, `uninstall.sh`
- `ROADMAP.md` §M12, §9
- `docs/windows-tsf-architecture.md` (registration/elevation notes)
- `docs/design-system-and-usability.md` §2.2 (config lifecycle)

Inventory what a user needs:

| Platform | Component | Where it must live | Who registers |
|---|---|---|---|
| Windows | `bornomala_tsf.dll` | `%ProgramFiles%\BornoMala\` | installer (elevated) |
| Windows | sandbox exe | same folder | installer |
| Windows | config | `%APPDATA%\com.bornomala.desktop\` | app on first run |
| Linux | `bornomala.so` | `/usr/lib/<triplet>/fcitx5/` or user `~/.local/lib/fcitx5/` | package or user |
| Linux | descriptors, icon, notice | `/usr/share/fcitx5/...`, `/usr/share/icons/...` | package |

---

## 3. Scope

### Goals

1. A Windows installer that installs to Program Files, registers the installed DLL, adds an uninstall entry, and supports upgrade.
2. A Linux package that installs the addon, descriptors, icon, licence, and integrates with Fcitx without destroying user config.
3. Code signing (or a documented, honest unsigned state with SmartScreen guidance).
4. Clean install / upgrade / uninstall verified on a clean VM.
5. Config migration verified between the sandbox and desktop identifiers.
6. A support matrix and release checklist with evidence links.

### Non-goals

- No auto-update service (design only, implement later).
- No Microsoft Store packaging.
- No macOS packaging (RV-12 decides).

### Explicit no-go

- **No** installer that registers a path outside its install directory.
- **No** registration during build or test (RV-04.3).
- **No** uninstaller that deletes user config without asking.
- **No** release artifact without a checksum and a NOTICE.
- **No** “release candidate” label before the clean-VM matrix is recorded.
- **No** shipping with `bundle.active: false`.

---

## 4. Work packages

### WP-11.1 — Windows installer

Choose one:

- **WiX Toolset v4** (MSI): best for enterprise, supports per-machine install and custom actions.
- **NSIS**: simpler, smaller, faster to iterate.

Recommended for a solo developer: **NSIS** for the first release, with a documented upgrade path; revisit WiX if MSI is required.

Installer requirements:

1. Install to `%ProgramFiles%\BornoMala\` (per-machine) or `%LocalAppData%\Programs\BornoMala\` (per-user). Pick one and document.
2. Copy `bornomala_tsf.dll`, the sandbox exe, `THIRD-PARTY-NOTICES.md`, and a `LICENSE` copy.
3. Register the TSF DLL **from the installed path** (`regsvr32 /s` or `DllRegisterServer` via a custom action with rollback).
4. Add an uninstall entry (Add/Remove Programs) with version, publisher, icon.
5. On upgrade: stop nothing (the IME is in-process), replace files after deregistering the old DLL if locked, re-register, preserve config.
6. On uninstall: unregister, remove files, **ask** before deleting `%APPDATA%` config.
7. Log to `%TEMP%\bornomala-install.log`; never log typed content.

Gate: install on a clean Windows VM; `Win+Space` lists Borno; typing works in Notepad.

### WP-11.2 — Configuration lifecycle migration

1. First run of the installed app: if `com.bornomala.sandbox` exists and `com.bornomala.desktop` does not, migrate (RV-03.2).
2. Coexistence: portable sandbox and installed app both use `com.bornomala.desktop`; the last writer wins; document it.
3. Corrupt config: preserve, back up, start with defaults, show status.
4. Test: install over an existing dev setup; verify settings survive.

Gate: migration test matrix recorded (4 cases from RV-03).

### WP-11.3 — Code signing

1. Obtain a code-signing certificate (OV or EV). If unavailable, document the unsigned state and the SmartScreen warning honestly.
2. Sign the DLL, the sandbox exe, and the installer with `signtool` (SHA-256, timestamp).
3. Verify with `signtool verify /pa`.
4. Add the thumbprint/public key to the release checklist (never the private key).

Gate: `signtool verify` passes on the installed artifacts, or the unsigned state is documented with a mitigation note.

### WP-11.4 — Linux packaging

1. Prefer a `.deb` built with `cpack` or `nfpm`, installing to system paths.
2. Alternative: a portable tarball with `install.sh` that uses `XDG_DATA_HOME` and does not edit the user profile destructively.
3. Include: `.so`, addon/IM descriptors, icon, licence, NOTICE, and a `postinst` that runs `fcitx5-remote -r` (not `killall`).
4. `prerm`/`postrm` must not delete user config.
5. Verify on a clean Ubuntu VM/container.

Gate: `dpkg -i`, restart Fcitx, type in a GTK app, `dpkg -r`, profile intact.

### WP-11.5 — Checksums and release artifacts

1. Produce `SHA256SUMS` for every artifact.
2. Sign the checksum file (GPG or the code-signing key) if possible.
3. Name artifacts with version and architecture: `BornoMala-0.2.0-x64-setup.exe`, `bornomala-fcitx5_0.2.0_amd64.deb`.
4. Attach the NOTICE and a short install guide.

Gate: a fresh download verifies against the checksum.

### WP-11.6 — Support matrix and release checklist

Create `docs/platform-support.md` (RV-09) with:

| Platform | Builds | Installs | Types in real apps | Core features | Status |
|---|---|---|---|---|---|
| Windows 11 24H2/25H2 x64 | | | | | |
| Windows 10 22H2 x64 | | | | | |
| Ubuntu 24.04 x64 (X11/XWayland) | | | | | |
| Ubuntu 24.04 Wayland | | | | | Unverified |
| macOS | | | | | Deferred/Blocked |

Rules: every cell has an evidence link; `Unknown` is never replaced by a guess.

Release checklist (`docs/release-process.md`):

- [ ] Version bumped in all manifests (`package.json`, `Cargo.toml`, `tauri.conf.json`).
- [ ] `cargo test --workspace` green.
- [ ] persistence suite green.
- [ ] frontend tests + build green.
- [ ] Windows E2E matrix green.
- [ ] Linux CI green + GUI evidence.
- [ ] NOTICE present and shipped.
- [ ] Installer signed / checksums generated.
- [ ] Clean install / upgrade / uninstall recorded.
- [ ] Support matrix updated.
- [ ] Release notes written.
- [ ] Tag created and artifacts attached.

### WP-11.7 — Versioning

1. Adopt SemVer for the product; the config schema has its own version; the C ABI has `BORNO_ABI_VERSION`.
2. Tag format `v0.2.0`.
3. Record a compatibility table: app version ↔ config schema ↔ ABI.
4. Bump rules: ABI break = major product bump until 1.0; schema change = minor + migration.

---

## 5. Acceptance criteria

| # | Criterion | Evidence |
|---|---|---|
| AC-1 | Installer installs to Program Files and registers installed path | VM log |
| AC-2 | Uninstall removes files and asks about config | VM log |
| AC-3 | Upgrade preserves settings | VM log |
| AC-4 | Linux package installs/uninstalls without profile loss | container log |
| AC-5 | Signatures verified or unsigned state documented | `signtool` output |
| AC-6 | `SHA256SUMS` verifies | command |
| AC-7 | Support matrix has evidence links | doc |
| AC-8 | Release checklist complete | doc |
| AC-9 | `bundle.active` true | config |

---

## 6. Testing plan

Windows:

```powershell
# Clean VM snapshot
.\BornoMala-x.y.z-x64-setup.exe /S
# verify
reg query "HKLM\SOFTWARE\Microsoft\CTF\TIP\{8A57F89C-...}"
# type in Notepad
# upgrade
.\BornoMala-x.y.z+1-x64-setup.exe /S
# uninstall
& "C:\Program Files\BornoMala\uninstall.exe"
```

Linux:

```bash
sudo dpkg -i bornomala-fcitx5_x.y.z_amd64.deb
fcitx5-remote -r
# type in gedit
sudo dpkg -r bornomala-fcitx5
```

Negative checks:

- Install with the IME active (DLL locked) → installer handles replace/reboot prompt.
- Corrupt config before upgrade → app starts with defaults, backup created.
- Uninstall with user config present → prompt, not silent delete.

---

## 7. Runnable checkpoint

Before and after each package, the sandbox must run from the dev tree. Release artifacts are produced only at the end.

---

## 8. Evidence and reporting

`docs/evidence/RV-11/<date>-release.md`: VM snapshots, install logs, `signtool` output, checksum verification, support matrix, release notes draft.

---

## 9. Roadmap update

- M12 `Verified` only after AC-1…AC-9.
- M13 remains blocked until macOS is decided and the matrix is complete.

---

## 10. Stop conditions

- No signing certificate → ship unsigned only with explicit developer acceptance and a SmartScreen warning in the release notes.
- Fcitx package cannot be built reproducibly in CI → ship the tarball with install.sh and mark the `.deb` as future work.
- The installer cannot register the TSF DLL reliably → stop; the product is not installable yet.

---

## 11. Handoff

RV-12 can make the macOS decision against a real release process, not a hypothetical one.
