// Build script for BornoMala desktop binaries.
//
// Embeds a single unified Windows application manifest (Common Controls v6,
// asInvoker execution level, longPathAware=true) via `tauri-build` and, on
// Windows GNU GCC targets, emits a target-scoped GCC specs override so MinGW's
// implicit `default-manifest.o` does not emit a duplicate RT_MANIFEST resource.

const DEFAULT_MANIFEST_SPEC_TOKEN: &str = "%{!shared:%:if-exists(default-manifest.o%s)}";

// Omit `<?xml ...?>` because `tauri-winres` prepends a leading space to each
// trimmed line inside the `.rc` manifest block, which breaks XML parsing if an
// XML declaration is not at byte 0.
const WINDOWS_APP_MANIFEST: &str = r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
  <asmv3:application>
    <asmv3:windowsSettings xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">
      <longPathAware>true</longPathAware>
    </asmv3:windowsSettings>
  </asmv3:application>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetBuildEnv {
    target: String,
    host: String,
    target_os: String,
    target_env: String,
    target_abi: String,
    target_arch: String,
}

impl TargetBuildEnv {
    fn should_apply_mingw_specs(&self) -> bool {
        self.target_os == "windows"
            && self.target_env == "gnu"
            && self.target_abi != "llvm"
            && !self.target.ends_with("-gnullvm")
    }

    fn expected_mingw_arch(&self) -> Result<&'static str, String> {
        match self.target_arch.as_str() {
            "x86_64" => Ok("x86_64"),
            "x86" => Ok("i686"),
            "aarch64" => Ok("aarch64"),
            other => Err(format!(
                "unsupported Windows GNU target architecture `{other}` for target `{}`",
                self.target
            )),
        }
    }

    fn default_gcc_candidates(&self) -> Result<Vec<String>, String> {
        let arch = self.expected_mingw_arch()?;
        let mut candidates = vec![format!("{arch}-w64-mingw32-gcc")];
        if self.host.contains("windows") {
            candidates.push("gcc".to_string());
        }
        Ok(candidates)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct RustflagsLinkerOptions {
    linker: Option<String>,
    linker_flavor: Option<String>,
}

fn parse_rustflags_linker_options(encoded: &str) -> Result<RustflagsLinkerOptions, String> {
    let mut opts = RustflagsLinkerOptions::default();
    let args: Vec<&str> = encoded.split('\x1f').filter(|s| !s.is_empty()).collect();
    let mut i = 0;

    while i < args.len() {
        let arg = args[i];
        let codegen_val = if let Some(rest) = arg.strip_prefix("-C") {
            if rest.is_empty() {
                i += 1;
                if i >= args.len() {
                    break;
                }
                Some(args[i])
            } else {
                Some(rest)
            }
        } else {
            None
        };

        if let Some(val) = codegen_val {
            if let Some(linker) = val.strip_prefix("linker=") {
                if linker.trim().is_empty() {
                    return Err("empty `-C linker=` value in CARGO_ENCODED_RUSTFLAGS".into());
                }
                opts.linker = Some(linker.to_string());
            } else if let Some(flavor) = val.strip_prefix("linker-flavor=") {
                if flavor.trim().is_empty() {
                    return Err(
                        "empty `-C linker-flavor=` value in CARGO_ENCODED_RUSTFLAGS".into()
                    );
                }
                opts.linker_flavor = Some(flavor.to_string());
            }
        }

        i += 1;
    }

    Ok(opts)
}

fn cargo_target_linker_env_var(target: &str) -> String {
    format!(
        "CARGO_TARGET_{}_LINKER",
        target.replace(['-', '.'], "_").to_ascii_uppercase()
    )
}

fn validate_gcc_driver_name(driver: &str) -> Result<(), String> {
    let trimmed = driver.trim();
    if trimmed.is_empty() {
        return Err("target linker driver path/name is empty".into());
    }

    let file_name = std::path::Path::new(trimmed)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("target linker driver `{trimmed}` is not valid UTF-8"))?;
    let lower = file_name.to_ascii_lowercase();
    let base = lower.strip_suffix(".exe").unwrap_or(&lower);

    const NON_GCC_DRIVERS: &[&str] = &[
        "ld",
        "lld",
        "lld-link",
        "rust-lld",
        "ld.lld",
        "ld64.lld",
        "wasm-ld",
        "link",
        "clang",
        "clang++",
        "clang-cl",
        "zig",
    ];

    if NON_GCC_DRIVERS.contains(&base)
        || base.ends_with("-ld")
        || base.ends_with("-lld")
        || base.starts_with("clang")
        || base.contains("-clang")
    {
        return Err(format!(
            "active target linker `{trimmed}` is not a GCC driver; cannot extract/override `*endfile:` specs to suppress MinGW `default-manifest.o`"
        ));
    }

    Ok(())
}

fn matches_expected_mingw_machine(dumpmachine: &str, target_arch: &str) -> bool {
    let machine = dumpmachine.trim().to_ascii_lowercase();
    if !machine.contains("mingw") {
        return false;
    }

    match target_arch {
        "x86_64" => machine.starts_with("x86_64-"),
        "x86" => {
            machine.starts_with("i686-")
                || machine.starts_with("i586-")
                || machine.starts_with("i486-")
                || machine.starts_with("i386-")
        }
        "aarch64" => machine.starts_with("aarch64-"),
        _ => false,
    }
}

fn is_specs_section_header(line: &str) -> Option<&str> {
    let without_eol = line.trim_end_matches(['\r', '\n']);
    if without_eol.starts_with(char::is_whitespace) {
        return None;
    }

    let trimmed = without_eol.trim_end_matches([' ', '\t']);
    if !trimmed.starts_with('*') || !trimmed.ends_with(':') || trimmed.len() <= 2 {
        return None;
    }

    let name = &trimmed[1..trimmed.len() - 1];
    if name.chars().any(char::is_whitespace) {
        return None;
    }

    Some(name)
}

fn split_line_eol(line: &str) -> (&str, &str) {
    if let Some(prefix) = line.strip_suffix("\r\n") {
        (prefix, "\r\n")
    } else if let Some(prefix) = line.strip_suffix('\n') {
        (prefix, "\n")
    } else {
        (line, "")
    }
}

fn strip_default_manifest_token_from_line(line: &str) -> Option<String> {
    let (content, eol) = split_line_eol(line);
    let (before, after) = content.split_once(DEFAULT_MANIFEST_SPEC_TOKEN)?;

    let b = before.trim_end_matches([' ', '\t']);
    let a = after.trim_start_matches([' ', '\t']);

    if b.is_empty() && a.is_empty() {
        // Removing a standalone token line cleanly avoids creating an empty line
        // inside `*endfile:`, which GCC treats as a section terminator.
        return Some(String::new());
    }

    let joined = if b.is_empty() {
        a.to_string()
    } else if a.is_empty() {
        b.to_string()
    } else {
        format!("{b} {a}")
    };

    Some(format!("{joined}{eol}"))
}

/// Validates and rewrites a GCC `-dumpspecs` document so `*endfile:` no longer
/// links MinGW's implicit `default-manifest.o` while preserving all other sections.
fn transform_gcc_specs_for_no_default_manifest(specs: &str) -> Result<String, String> {
    if specs.trim().is_empty() {
        return Err("GCC specs input is empty".into());
    }

    let mut before_endfile = String::new();
    let mut after_endfile = String::new();
    let mut endfile_header: Option<&str> = None;
    let mut endfile_lines: Vec<&str> = Vec::new();
    let mut current_section: Option<&str> = None;
    let mut in_endfile = false;
    let mut endfile_count = 0usize;

    for line in specs.split_inclusive('\n') {
        if let Some(section_name) = is_specs_section_header(line) {
            current_section = Some(section_name);
            if section_name == "endfile" {
                endfile_count += 1;
                if endfile_count > 1 {
                    return Err("found multiple `*endfile:` sections in GCC specs".into());
                }
                endfile_header = Some(line);
                in_endfile = true;
            } else {
                in_endfile = false;
                let dest = if endfile_count == 0 {
                    &mut before_endfile
                } else {
                    &mut after_endfile
                };
                dest.push_str(line);
            }
            continue;
        }

        if in_endfile {
            endfile_lines.push(line);
        } else {
            if line.contains("default-manifest.o") {
                return Err(format!(
                    "unexpected `default-manifest.o` reference outside `*endfile:` section (in `{}`)",
                    current_section.unwrap_or("<preamble>")
                ));
            }
            let dest = if endfile_count == 0 {
                &mut before_endfile
            } else {
                &mut after_endfile
            };
            dest.push_str(line);
        }
    }

    let header = endfile_header
        .ok_or_else(|| "missing `*endfile:` section in GCC specs".to_string())?;

    let dm_refs: usize = endfile_lines
        .iter()
        .map(|l| l.matches("default-manifest.o").count())
        .sum();
    let token_matches: usize = endfile_lines
        .iter()
        .map(|l| l.matches(DEFAULT_MANIFEST_SPEC_TOKEN).count())
        .sum();

    if token_matches == 0 && dm_refs > 0 {
        return Err(format!(
            "unsupported `default-manifest.o` construct in `*endfile:`; expected exact `{DEFAULT_MANIFEST_SPEC_TOKEN}` token"
        ));
    }
    if token_matches == 0 {
        return Err(format!(
            "missing expected `{DEFAULT_MANIFEST_SPEC_TOKEN}` token in `*endfile:` section"
        ));
    }
    if token_matches > 1 || dm_refs != token_matches {
        return Err("found multiple `default-manifest.o` references in `*endfile:` section".into());
    }

    let input_had_crtend = endfile_lines.iter().any(|l| l.contains("crtend.o"));
    let mut new_endfile_body = String::new();
    for line in endfile_lines {
        if let Some(updated) = strip_default_manifest_token_from_line(line) {
            new_endfile_body.push_str(&updated);
        } else {
            new_endfile_body.push_str(line);
        }
    }

    if new_endfile_body.trim().is_empty() {
        return Err(
            "`*endfile:` section would be empty after removing `default-manifest.o` token".into(),
        );
    }
    if input_had_crtend && !new_endfile_body.contains("crtend.o") {
        return Err("`*endfile:` transformation lost required `crtend.o` reference".into());
    }

    let result = format!("{before_endfile}{header}{new_endfile_body}{after_endfile}");
    if result.contains("default-manifest.o") {
        return Err("transformed GCC specs still contain `default-manifest.o`".into());
    }

    Ok(result)
}

#[cfg(not(test))]
mod driver {
    use std::env;
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    use std::process::Command;

    use crate::{
        cargo_target_linker_env_var, matches_expected_mingw_machine,
        parse_rustflags_linker_options, transform_gcc_specs_for_no_default_manifest,
        validate_gcc_driver_name, TargetBuildEnv,
    };

    #[derive(Debug)]
    struct ProbedGccDriver {
        name: String,
        machine: String,
        transformed_specs: String,
    }

    enum ProbeError {
        NotFound,
        Other(String),
    }

    fn read_required_env(name: &str) -> Result<String, String> {
        env::var(name).map_err(|_| format!("missing Cargo build-script environment variable `{name}`"))
    }

    pub fn load_target_build_env() -> Result<TargetBuildEnv, String> {
        Ok(TargetBuildEnv {
            target: read_required_env("TARGET")?,
            host: read_required_env("HOST")?,
            target_os: read_required_env("CARGO_CFG_TARGET_OS")?,
            target_env: read_required_env("CARGO_CFG_TARGET_ENV")?,
            target_abi: env::var("CARGO_CFG_TARGET_ABI").unwrap_or_default(),
            target_arch: read_required_env("CARGO_CFG_TARGET_ARCH")?,
        })
    }

    fn probe_gcc_driver(driver: &str, target_arch: &str) -> Result<ProbedGccDriver, ProbeError> {
        validate_gcc_driver_name(driver).map_err(ProbeError::Other)?;

        let ver_out = Command::new(driver).arg("-v").output().map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                ProbeError::NotFound
            } else {
                ProbeError::Other(format!("failed to invoke `{driver} -v`: {err}"))
            }
        })?;
        if !ver_out.status.success() {
            return Err(ProbeError::Other(format!(
                "`{driver} -v` exited with status {}",
                ver_out.status
            )));
        }
        let ver_stderr = String::from_utf8_lossy(&ver_out.stderr);
        if ver_stderr.to_ascii_lowercase().contains("clang version") {
            return Err(ProbeError::Other(format!(
                "linker `{driver}` reports `clang version` rather than GCC"
            )));
        }

        let machine_out = Command::new(driver)
            .arg("-dumpmachine")
            .output()
            .map_err(|err| ProbeError::Other(format!("failed to invoke `{driver} -dumpmachine`: {err}")))?;
        if !machine_out.status.success() {
            return Err(ProbeError::Other(format!(
                "`{driver} -dumpmachine` exited with status {}",
                machine_out.status
            )));
        }
        let machine = String::from_utf8_lossy(&machine_out.stdout)
            .trim()
            .to_string();
        if !matches_expected_mingw_machine(&machine, target_arch) {
            return Err(ProbeError::Other(format!(
                "`{driver} -dumpmachine` reported `{machine}`, expected MinGW `{target_arch}`"
            )));
        }

        let specs_out = Command::new(driver)
            .arg("-dumpspecs")
            .output()
            .map_err(|err| ProbeError::Other(format!("failed to invoke `{driver} -dumpspecs`: {err}")))?;
        if !specs_out.status.success() {
            return Err(ProbeError::Other(format!(
                "`{driver} -dumpspecs` exited with status {}",
                specs_out.status
            )));
        }
        let raw_specs = String::from_utf8(specs_out.stdout).map_err(|err| {
            ProbeError::Other(format!("`{driver} -dumpspecs` emitted invalid UTF-8: {err}"))
        })?;
        let transformed_specs = transform_gcc_specs_for_no_default_manifest(&raw_specs)
            .map_err(|err| ProbeError::Other(format!("`{driver}` specs validation failed: {err}")))?;

        Ok(ProbedGccDriver {
            name: driver.to_string(),
            machine,
            transformed_specs,
        })
    }

    fn resolve_explicit_linker(target_env: &TargetBuildEnv) -> Result<Option<String>, String> {
        let target_linker_var = cargo_target_linker_env_var(&target_env.target);
        println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
        println!("cargo:rerun-if-env-changed=RUSTC_LINKER");
        println!("cargo:rerun-if-env-changed={target_linker_var}");
        println!("cargo:rerun-if-env-changed=PATH");

        let encoded_flags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
        let rustflags = parse_rustflags_linker_options(&encoded_flags)?;
        if let Some(flavor) = &rustflags.linker_flavor {
            if flavor != "gcc" {
                return Err(format!(
                    "explicit `-C linker-flavor={flavor}` is not `gcc`; cannot apply GCC `-specs=` override"
                ));
            }
        }

        if let Some(linker) = rustflags.linker {
            return Ok(Some(linker));
        }

        if let Ok(linker) = env::var(&target_linker_var) {
            if linker.trim().is_empty() {
                return Err(format!("environment variable `{target_linker_var}` is empty"));
            }
            return Ok(Some(linker));
        }

        if let Ok(linker) = env::var("RUSTC_LINKER") {
            if linker.trim().is_empty() {
                return Err("environment variable `RUSTC_LINKER` is empty".into());
            }
            return Ok(Some(linker));
        }

        Ok(None)
    }

    fn normalize_eol(text: &str) -> String {
        text.replace("\r\n", "\n")
    }

    fn resolve_mingw_specs(target_env: &TargetBuildEnv) -> Result<ProbedGccDriver, String> {
        if let Some(explicit) = resolve_explicit_linker(target_env)? {
            return match probe_gcc_driver(&explicit, &target_env.target_arch) {
                Ok(probed) => Ok(probed),
                Err(ProbeError::NotFound) => Err(format!(
                    "configured explicit target linker `{explicit}` was not found"
                )),
                Err(ProbeError::Other(msg)) => Err(msg),
            };
        }

        let candidates = target_env.default_gcc_candidates()?;
        let mut matched: Vec<ProbedGccDriver> = Vec::new();
        let mut diagnostics: Vec<String> = Vec::new();

        for candidate in &candidates {
            match probe_gcc_driver(candidate, &target_env.target_arch) {
                Ok(probed) => matched.push(probed),
                Err(ProbeError::NotFound) => diagnostics.push(format!("{candidate}: not found")),
                Err(ProbeError::Other(msg)) => diagnostics.push(format!("{candidate}: {msg}")),
            }
        }

        if matched.is_empty() {
            return Err(format!(
                "unable to locate a matching MinGW GCC linker driver for target `{}`; checked [{}]",
                target_env.target,
                diagnostics.join("; ")
            ));
        }

        let first = &matched[0];
        let first_norm = normalize_eol(&first.transformed_specs);
        for other in &matched[1..] {
            if other.machine != first.machine
                || normalize_eol(&other.transformed_specs) != first_norm
            {
                return Err(format!(
                    "ambiguous GCC drivers on PATH for target `{}` (`{}` [{}] vs `{}` [{}] differ in target/specs); set `{}` or `-C linker=...` explicitly",
                    target_env.target,
                    first.name,
                    first.machine,
                    other.name,
                    other.machine,
                    cargo_target_linker_env_var(&target_env.target),
                ));
            }
        }

        Ok(matched.into_iter().next().unwrap())
    }

    pub fn configure_mingw_no_default_manifest_specs(
        target_env: &TargetBuildEnv,
    ) -> Result<(), String> {
        let probed = resolve_mingw_specs(target_env)?;
        let out_dir = read_required_env("OUT_DIR")?;
        let specs_path = PathBuf::from(&out_dir).join("no-default-manifest.specs");

        fs::write(&specs_path, probed.transformed_specs.as_bytes()).map_err(|err| {
            format!("failed to write specs file `{}`: {err}", specs_path.display())
        })?;

        let meta = fs::metadata(&specs_path).map_err(|err| {
            format!("failed to verify written specs file `{}`: {err}", specs_path.display())
        })?;
        if meta.len() == 0 || meta.len() != probed.transformed_specs.len() as u64 {
            return Err(format!(
                "written specs file `{}` failed non-empty/size check (actual={}, expected={})",
                specs_path.display(),
                meta.len(),
                probed.transformed_specs.len()
            ));
        }

        let specs_str = specs_path.to_str().ok_or_else(|| {
            format!(
                "generated specs path `{}` is not valid UTF-8",
                specs_path.display()
            )
        })?;

        println!("cargo:rustc-link-arg-bins=-specs={specs_str}");
        Ok(())
    }
}

#[cfg(not(test))]
fn main() {
    let target_env = match driver::load_target_build_env() {
        Ok(env) => env,
        Err(err) => {
            eprintln!("bornomala-desktop build script environment error: {err}");
            std::process::exit(1);
        }
    };

    if target_env.should_apply_mingw_specs() {
        if let Err(err) = driver::configure_mingw_no_default_manifest_specs(&target_env) {
            eprintln!("bornomala-desktop Windows GNU manifest/specs setup failed: {err}");
            std::process::exit(1);
        }
    }

    let attributes = tauri_build::Attributes::new().windows_attributes(
        tauri_build::WindowsAttributes::new().app_manifest(WINDOWS_APP_MANIFEST),
    );
    if let Err(err) = tauri_build::try_build(attributes) {
        eprintln!("tauri_build failed: {err:#}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn windows_gnu_env() -> TargetBuildEnv {
        TargetBuildEnv {
            target: "x86_64-pc-windows-gnu".to_string(),
            host: "x86_64-pc-windows-gnu".to_string(),
            target_os: "windows".to_string(),
            target_env: "gnu".to_string(),
            target_abi: String::new(),
            target_arch: "x86_64".to_string(),
        }
    }

    #[test]
    fn unified_manifest_contains_exact_required_elements_and_no_xml_decl() {
        assert!(!WINDOWS_APP_MANIFEST.contains("<?xml"));
        assert!(WINDOWS_APP_MANIFEST.trim_start().starts_with("<assembly "));
        assert!(WINDOWS_APP_MANIFEST.trim_end().ends_with("</assembly>"));
        assert!(WINDOWS_APP_MANIFEST.contains("name=\"Microsoft.Windows.Common-Controls\""));
        assert!(WINDOWS_APP_MANIFEST.contains("version=\"6.0.0.0\""));
        assert!(WINDOWS_APP_MANIFEST.contains("publicKeyToken=\"6595b64144ccf1df\""));
        assert!(WINDOWS_APP_MANIFEST
            .contains("<requestedExecutionLevel level=\"asInvoker\" uiAccess=\"false\"/>"));
        assert!(WINDOWS_APP_MANIFEST.contains("<longPathAware>true</longPathAware>"));
        assert!(!WINDOWS_APP_MANIFEST.contains("dpiAware"));
        assert!(!WINDOWS_APP_MANIFEST.contains("<compatibility"));
    }

    #[test]
    fn transform_preserves_other_sections_and_removes_single_default_manifest_token() {
        let specs = "\
*startfile:
%{!shared:%{!municode:crt2%O%s}} crtbegin.o%s

*endfile:
%{mdaz-ftz:crtfastmath.o%s}    %{!shared:%:if-exists(default-manifest.o%s)}   %{fvtable-verify=none:%s; fvtable-verify=std:vtv_end.o%s}   crtend.o%s

*link:
%{mwindows:--subsystem windows}
";
        let transformed =
            transform_gcc_specs_for_no_default_manifest(specs).expect("valid specs should pass");

        assert!(!transformed.contains("default-manifest.o"));
        assert_eq!(transformed.matches("*endfile:").count(), 1);
        assert!(transformed.contains("*startfile:\n%{!shared:%{!municode:crt2%O%s}} crtbegin.o%s"));
        assert!(transformed.contains("%{mdaz-ftz:crtfastmath.o%s} %{fvtable-verify=none:%s; fvtable-verify=std:vtv_end.o%s}   crtend.o%s"));
        assert!(transformed.contains("*link:\n%{mwindows:--subsystem windows}"));
    }

    #[test]
    fn transform_handles_multiline_endfile_and_crlf() {
        let specs = "*startfile:\r\ncrt2%O%s\r\n\r\n*endfile:\r\n%{mdaz-ftz:crtfastmath.o%s}\r\n%{!shared:%:if-exists(default-manifest.o%s)}\r\ncrtend.o%s\r\n\r\n*link:\r\n-m i386pep\r\n";
        let transformed =
            transform_gcc_specs_for_no_default_manifest(specs).expect("multiline endfile should pass");

        assert!(!transformed.contains("default-manifest.o"));
        assert!(transformed.contains("*endfile:\r\n%{mdaz-ftz:crtfastmath.o%s}\r\ncrtend.o%s\r\n"));
        assert!(transformed.contains("*link:\r\n-m i386pep\r\n"));
    }

    #[test]
    fn transform_rejects_missing_endfile_section() {
        let specs = "*startfile:\ncrt2%O%s\n\n*link:\n--subsystem windows\n";
        let err = transform_gcc_specs_for_no_default_manifest(specs).unwrap_err();
        assert!(err.contains("*endfile:"));
    }

    #[test]
    fn transform_rejects_duplicate_endfile_sections() {
        let specs = "*endfile:\n%{!shared:%:if-exists(default-manifest.o%s)} crtend.o%s\n\n*endfile:\ncrtend.o%s\n";
        let err = transform_gcc_specs_for_no_default_manifest(specs).unwrap_err();
        assert!(err.contains("multiple `*endfile:`"));
    }

    #[test]
    fn transform_rejects_missing_default_manifest_token() {
        let specs = "*endfile:\n%{mdaz-ftz:crtfastmath.o%s} crtend.o%s\n";
        let err = transform_gcc_specs_for_no_default_manifest(specs).unwrap_err();
        assert!(err.contains("missing expected"));
    }

    #[test]
    fn transform_rejects_duplicate_default_manifest_tokens() {
        let specs = "*endfile:\n%{!shared:%:if-exists(default-manifest.o%s)} %{!shared:%:if-exists(default-manifest.o%s)} crtend.o%s\n";
        let err = transform_gcc_specs_for_no_default_manifest(specs).unwrap_err();
        assert!(err.contains("multiple `default-manifest.o`"));
    }

    #[test]
    fn transform_rejects_unrecognized_default_manifest_expression() {
        let specs = "*endfile:\n%{!shared:default-manifest.o%s} crtend.o%s\n";
        let err = transform_gcc_specs_for_no_default_manifest(specs).unwrap_err();
        assert!(err.contains("unsupported `default-manifest.o` construct"));
    }

    #[test]
    fn transform_rejects_default_manifest_outside_endfile() {
        let specs = "*startfile:\n%{!shared:%:if-exists(default-manifest.o%s)}\n\n*endfile:\ncrtend.o%s\n";
        let err = transform_gcc_specs_for_no_default_manifest(specs).unwrap_err();
        assert!(err.contains("outside `*endfile:`"));
    }

    #[test]
    fn transform_rejects_empty_endfile_after_removal() {
        let specs = "*endfile:\n%{!shared:%:if-exists(default-manifest.o%s)}\n\n*link:\n-m i386pep\n";
        let err = transform_gcc_specs_for_no_default_manifest(specs).unwrap_err();
        assert!(err.contains("would be empty"));
    }

    #[test]
    fn target_gating_applies_only_to_windows_gnu_gcc() {
        let win_gnu = windows_gnu_env();
        assert!(win_gnu.should_apply_mingw_specs());

        let linux_to_win_gnu = TargetBuildEnv {
            host: "x86_64-unknown-linux-gnu".into(),
            ..windows_gnu_env()
        };
        assert!(linux_to_win_gnu.should_apply_mingw_specs());
        assert_eq!(
            linux_to_win_gnu.default_gcc_candidates().unwrap(),
            vec!["x86_64-w64-mingw32-gcc"]
        );
        assert_eq!(
            win_gnu.default_gcc_candidates().unwrap(),
            vec!["x86_64-w64-mingw32-gcc", "gcc"]
        );

        let win_msvc = TargetBuildEnv {
            target: "x86_64-pc-windows-msvc".into(),
            target_env: "msvc".into(),
            ..windows_gnu_env()
        };
        assert!(!win_msvc.should_apply_mingw_specs());

        let win_gnullvm = TargetBuildEnv {
            target: "x86_64-pc-windows-gnullvm".into(),
            target_abi: "llvm".into(),
            ..windows_gnu_env()
        };
        assert!(!win_gnullvm.should_apply_mingw_specs());

        let linux_gnu = TargetBuildEnv {
            target: "x86_64-unknown-linux-gnu".into(),
            target_os: "linux".into(),
            ..windows_gnu_env()
        };
        assert!(!linux_gnu.should_apply_mingw_specs());

        let macos = TargetBuildEnv {
            target: "aarch64-apple-darwin".into(),
            target_os: "macos".into(),
            target_env: String::new(),
            target_arch: "aarch64".into(),
            ..windows_gnu_env()
        };
        assert!(!macos.should_apply_mingw_specs());
    }

    #[test]
    fn rustflags_and_driver_validation_reject_non_gcc_linkers() {
        let parsed = parse_rustflags_linker_options("-C\x1flinker=gcc.exe\x1f-Clinker-flavor=gcc")
            .expect("valid flags");
        assert_eq!(parsed.linker.as_deref(), Some("gcc.exe"));
        assert_eq!(parsed.linker_flavor.as_deref(), Some("gcc"));

        assert_eq!(
            cargo_target_linker_env_var("x86_64-pc-windows-gnu"),
            "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER"
        );

        assert!(validate_gcc_driver_name("gcc").is_ok());
        assert!(validate_gcc_driver_name("C:\\mingw64\\bin\\x86_64-w64-mingw32-gcc.exe").is_ok());
        assert!(validate_gcc_driver_name("lld-link.exe").is_err());
        assert!(validate_gcc_driver_name("rust-lld").is_err());
        assert!(validate_gcc_driver_name("clang").is_err());
        assert!(validate_gcc_driver_name("x86_64-w64-mingw32-ld.exe").is_err());

        assert!(matches_expected_mingw_machine("x86_64-w64-mingw32", "x86_64"));
        assert!(!matches_expected_mingw_machine("x86_64-pc-cygwin", "x86_64"));
        assert!(!matches_expected_mingw_machine("i686-w64-mingw32", "x86_64"));
        assert!(matches_expected_mingw_machine("i686-w64-mingw32", "x86"));
    }
}
