//! Failure-safe filesystem persistence, recovery, and backup management for Borno configuration.
//!
//! Owns all operating-system configuration directory paths, atomic safe replacement,
//! corrupted file backups, and protection of unsupported schema versions.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use bornomala_core::BornoConfig;
use semver::Version;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

pub const CANONICAL_APP_DIR_NAME: &str = "com.bornomala.desktop";
pub const PRIMARY_LEGACY_DIR_NAME: &str = "com.bornomala.sandbox";
pub const SECONDARY_LEGACY_DIR_NAME: &str = "com.borno.sandbox";
pub const CONFIG_FILE_NAME: &str = "config.json";

/// Abstraction for resolving the canonical configuration file path.
pub trait ConfigPathResolver {
    fn resolve_canonical_path(&self) -> Result<PathBuf, String>;
}

/// Standalone / memory resolver for testing.
pub struct StaticConfigPathResolver {
    pub path: Result<PathBuf, String>,
}

impl ConfigPathResolver for StaticConfigPathResolver {
    fn resolve_canonical_path(&self) -> Result<PathBuf, String> {
        self.path.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedLegacyPaths {
    pub primary_legacy_path: PathBuf,
    pub secondary_legacy_path: PathBuf,
}

/// Pure helper deriving legacy sibling paths from canonical path's shared parent.
/// Validates that canonical directory name matches CANONICAL_APP_DIR_NAME and file name matches CONFIG_FILE_NAME.
pub fn derive_sibling_paths(canonical_path: &Path) -> Option<DerivedLegacyPaths> {
    if canonical_path.file_name()? != CONFIG_FILE_NAME {
        return None;
    }
    let canonical_dir = canonical_path.parent()?;
    if canonical_dir.file_name()? != CANONICAL_APP_DIR_NAME {
        return None;
    }
    let shared_parent = canonical_dir.parent()?;
    let primary_legacy_path = shared_parent.join(PRIMARY_LEGACY_DIR_NAME).join(CONFIG_FILE_NAME);
    let secondary_legacy_path = shared_parent.join(SECONDARY_LEGACY_DIR_NAME).join(CONFIG_FILE_NAME);
    Some(DerivedLegacyPaths {
        primary_legacy_path,
        secondary_legacy_path,
    })
}

#[derive(Debug)]
pub enum PathStatus {
    Present,
    NotFound,
    Error(std::io::Error),
}

/// Inspects path using symlink_metadata, explicitly rejecting symlinks on both target file and parent directory.
pub fn inspect_path(path: &Path) -> PathStatus {
    // 1. Inspect parent application directory if present
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            match std::fs::symlink_metadata(parent) {
                Ok(meta) => {
                    let ft = meta.file_type();
                    if ft.is_symlink() {
                        return PathStatus::Error(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "Application directory {} is a symbolic link; symbolic links are rejected for configuration containers",
                                parent.display()
                            ),
                        ));
                    }
                    if !ft.is_dir() {
                        return PathStatus::Error(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Application container {} exists but is not a directory", parent.display()),
                        ));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // Parent does not exist yet (clean install scenario); safe to proceed to target check
                }
                Err(e) => return PathStatus::Error(e),
            }
        }
    }

    // 2. Inspect target configuration file
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            let file_type = meta.file_type();
            if file_type.is_symlink() {
                PathStatus::Error(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Path {} is a symbolic link; symbolic links are rejected for configuration files", path.display()),
                ))
            } else if file_type.is_file() {
                PathStatus::Present
            } else {
                PathStatus::Error(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Path {} exists but is not a regular file", path.display()),
                ))
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => PathStatus::NotFound,
        Err(e) => PathStatus::Error(e),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaClassification {
    SupportedCurrent,
    SupportedLegacyMigratable(&'static str),
    SupportedCurrentWithBuildMetadata(String),
    SupportedLegacyWithBuildMetadata(String),
    UnsupportedFutureStable(String),
    UnsupportedFuturePrerelease(String),
    UnsupportedDevelopmentPrerelease(String),
    UnsupportedObsolete(String),
    UnsupportedIntermediate(String),
    MalformedJson(String),
    InvalidMetadata,
    InvalidSemVerFormat(String),
}

/// Classifies configuration JSON string using strict SemVer 2.0.0 rules.
pub fn classify_schema_version(content: &str) -> SchemaClassification {
    let raw_val: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(e) => return SchemaClassification::MalformedJson(e.to_string()),
    };

    let s = match raw_val.get("schema_version").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return SchemaClassification::InvalidMetadata,
    };

    if s == "1.2.0" {
        return SchemaClassification::SupportedCurrent;
    } else if s == "1.1.0" {
        return SchemaClassification::SupportedLegacyMigratable("1.1.0");
    } else if s == "1.0.0" {
        return SchemaClassification::SupportedLegacyMigratable("1.0.0");
    }

    let ver = match Version::parse(s) {
        Ok(v) => v,
        Err(_) => return SchemaClassification::InvalidSemVerFormat(s.to_string()),
    };

    // Rule A: Prerelease Evaluation (Checked FIRST)
    if !ver.pre.is_empty() {
        let base = Version::new(ver.major, ver.minor, ver.patch);
        if base > Version::new(1, 2, 0) {
            return SchemaClassification::UnsupportedFuturePrerelease(s.to_string());
        } else {
            return SchemaClassification::UnsupportedDevelopmentPrerelease(s.to_string());
        }
    }

    // Rule B: Build Metadata Evaluation (Checked SECOND)
    if !ver.build.is_empty() {
        let base = Version::new(ver.major, ver.minor, ver.patch);
        if base == Version::new(1, 2, 0) {
            return SchemaClassification::SupportedCurrentWithBuildMetadata(s.to_string());
        } else if base == Version::new(1, 1, 0) || base == Version::new(1, 0, 0) {
            return SchemaClassification::SupportedLegacyWithBuildMetadata(s.to_string());
        } else if base > Version::new(1, 2, 0) {
            return SchemaClassification::UnsupportedFutureStable(s.to_string());
        } else if base < Version::new(1, 0, 0) {
            return SchemaClassification::UnsupportedObsolete(s.to_string());
        } else {
            return SchemaClassification::UnsupportedIntermediate(s.to_string());
        }
    }

    // Rule C: Numeric Version Ranges for Clean Releases
    if ver > Version::new(1, 2, 0) {
        SchemaClassification::UnsupportedFutureStable(s.to_string())
    } else if ver < Version::new(1, 0, 0) {
        SchemaClassification::UnsupportedObsolete(s.to_string())
    } else {
        SchemaClassification::UnsupportedIntermediate(s.to_string())
    }
}

/// Operational persistence status for the active session.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum PersistenceStatus {
    Active,
    ReadOnlyMissingWriteFailed(String),
    ReadOnlyPermissionDenied(String),
    LockedUnsupportedVersion { version: String },
}

/// Diagnostic outcome from loading configuration from disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadOutcome {
    LoadedOk,
    CreatedDefault,
    MissingFileWriteFailed(String),
    ReadFailed(String),
    UnsupportedVersionPreserved { version: String },
    RecoveredCorrupt { backup_path: String },
    RecoveredInvalid { backup_path: String },
    CorruptBackupFailed(String),
    InvalidBackupFailed(String),
    CorruptRecoveryWriteFailed { backup_path: String, error: String },
    InvalidRecoveryWriteFailed { backup_path: String, error: String },
}

/// Detailed diagnostic failure reasons for backup creation and cleanup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupFailureDetail {
    CreationError(String),
    WriteFailedCleanedUp {
        error: String,
        candidate_path: PathBuf,
    },
    WriteFailedCleanupFailed {
        write_error: String,
        cleanup_error: String,
        candidate_path: PathBuf,
    },
    CollisionExhaustion {
        attempts: usize,
    },
}

/// Detailed outcome of legacy backup creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupOutcome {
    Created(PathBuf),
    Failed(BackupFailureDetail),
}

/// Detailed outcome of the unified startup load-or-migrate workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupOutcome {
    CanonicalLoaded(LoadOutcome),
    CleanInstallCreated,
    CleanInstallWriteFailed(String),
    Migrated {
        source_path: PathBuf,
        backup_outcome: BackupOutcome,
    },
    MigrationWriteFailed {
        source_path: PathBuf,
        error: String,
    },
    LegacySourceReadFailed {
        source_path: PathBuf,
        error: String,
    },
    LegacyCorruptPreserved {
        source_path: PathBuf,
        error: String,
    },
    LegacyCorruptPreservedWriteFailed {
        source_path: PathBuf,
        error: String,
        write_error: String,
    },
    LegacyInvalidPreserved {
        source_path: PathBuf,
        error: String,
    },
    LegacyInvalidPreservedWriteFailed {
        source_path: PathBuf,
        error: String,
        write_error: String,
    },
    LegacyUnsupportedPreserved {
        source_path: PathBuf,
        version: String,
    },
    PathResolutionFailed(String),
    ParentResolutionFailed(String),
    CanonicalInspectionFailed(String),
    LegacyInspectionFailed {
        source_path: PathBuf,
        error: String,
    },
}

/// Fault injection seams for isolated, deterministic testing of the complete startup pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StartupFaultInjection {
    pub canonical_inspection_fault: bool,
    pub primary_legacy_inspection_fault: bool,
    pub secondary_legacy_inspection_fault: bool,
    pub primary_legacy_source_read_fault: bool,
    pub secondary_legacy_source_read_fault: bool,
    pub migration_write_fault: bool,
    pub clean_install_write_fault: bool,
    pub canonical_recovery_fault: Option<CanonicalFaultInjection>,
    pub migration_backup_fault: Option<BackupFaultInjection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BackupFaultInjection {
    pub fail_creation: bool,
    pub fail_write_midstream: bool,
    pub fail_cleanup: bool,
    pub exhaust_collisions: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CanonicalFaultInjection {
    pub fail_read: bool,
    pub fail_backup: bool,
    pub fail_default_write: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResetFaultInjection {
    pub fail_backup: bool,
    pub fail_default_write: bool,
    pub fail_engine_lock: bool,
    pub fail_effective_config_lock: bool,
    pub fail_persistence_status_lock: bool,
}

fn generate_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    now.to_string()
}

fn generate_temp_path(dest_path: &Path) -> PathBuf {
    let pid = std::process::id();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let file_name = format!("config.json.tmp.{pid}_{now}_{counter}");
    dest_path.with_file_name(file_name)
}

/// Executes platform-appropriate safe replacement of the configuration file.
///
/// 1. Ensures parent directory exists before creating any temporary file.
/// 2. Creates a temporary file in the EXACT SAME directory and volume as `dest_path`.
/// 3. Flushes and syncs all bytes to disk before closing the handle.
/// 4. Replaces `dest_path` via `std::fs::rename`.
/// 5. Cleans up temporary file on failure; preserves original destination file intact.
pub fn save_config_safe(dest_path: &Path, config: &BornoConfig) -> Result<(), String> {
    config.validate().map_err(|e| e.to_string())?;

    let json_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Serialization error: {e}"))?;

    if let Some(parent) = dest_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {e}", parent.display()))?;
        }
    }

    let tmp_path = generate_temp_path(dest_path);

    // Open and write to temp file
    let write_res = (|| -> std::io::Result<()> {
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        let mut file = options.open(&tmp_path)?;
        file.write_all(json_str.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
        drop(file);
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(format!("Failed to write temporary configuration file: {e}"));
    }

    // Replace destination file: destination is NEVER deleted prior to rename!
    if let Err(e) = std::fs::rename(&tmp_path, dest_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(format!(
            "Failed to replace configuration file at {}: {e}. Previous configuration preserved.",
            dest_path.display()
        ));
    }

    Ok(())
}

fn create_exclusive_backup(
    source_bytes: &[u8],
    source_file_path: &Path,
    suffix: &str,
    faults: Option<BackupFaultInjection>,
) -> BackupOutcome {
    use std::fs::OpenOptions;
    use std::io::Write;

    let ts = generate_timestamp();

    if faults.map_or(false, |f| f.exhaust_collisions) {
        return BackupOutcome::Failed(BackupFailureDetail::CollisionExhaustion { attempts: 1000 });
    }

    for attempt in 0..1000 {
        let candidate_filename = if attempt == 0 {
            format!("{}.{}.{}", CONFIG_FILE_NAME, suffix, ts)
        } else {
            format!("{}.{}.{}_{}", CONFIG_FILE_NAME, suffix, ts, attempt)
        };
        let candidate_path = source_file_path.with_file_name(candidate_filename);

        if faults.map_or(false, |f| f.fail_creation) {
            return BackupOutcome::Failed(BackupFailureDetail::CreationError(
                "Simulated backup creation failure".to_string(),
            ));
        }

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        let file_res = options.open(&candidate_path);
        let mut file = match file_res {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                continue;
            }
            Err(e) => {
                return BackupOutcome::Failed(BackupFailureDetail::CreationError(e.to_string()));
            }
        };

        if faults.map_or(false, |f| f.fail_write_midstream) {
            drop(file);
            if faults.map_or(false, |f| f.fail_cleanup) {
                return BackupOutcome::Failed(BackupFailureDetail::WriteFailedCleanupFailed {
                    write_error: "Simulated midstream write failure".to_string(),
                    cleanup_error: "Simulated cleanup failure".to_string(),
                    candidate_path,
                });
            } else {
                let _ = std::fs::remove_file(&candidate_path);
                return BackupOutcome::Failed(BackupFailureDetail::WriteFailedCleanedUp {
                    error: "Simulated midstream write failure".to_string(),
                    candidate_path,
                });
            }
        }

        let write_res = (|| -> std::io::Result<()> {
            file.write_all(source_bytes)?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_res {
            drop(file);
            if faults.map_or(false, |f| f.fail_cleanup) {
                return BackupOutcome::Failed(BackupFailureDetail::WriteFailedCleanupFailed {
                    write_error: e.to_string(),
                    cleanup_error: "Simulated cleanup failure".to_string(),
                    candidate_path,
                });
            } else {
                let _ = std::fs::remove_file(&candidate_path);
                return BackupOutcome::Failed(BackupFailureDetail::WriteFailedCleanedUp {
                    error: e.to_string(),
                    candidate_path,
                });
            }
        }

        return BackupOutcome::Created(candidate_path);
    }

    BackupOutcome::Failed(BackupFailureDetail::CollisionExhaustion { attempts: 1000 })
}

fn create_prerewrite_backup(dest_path: &Path, source_bytes: &[u8], prefix: &str) -> Result<String, String> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let ts = generate_timestamp();
    for attempt in 0..1000 {
        let candidate_filename = if attempt == 0 {
            format!("{}.{}.{}", CONFIG_FILE_NAME, prefix, ts)
        } else {
            format!("{}.{}.{}_{}", CONFIG_FILE_NAME, prefix, ts, attempt)
        };
        let candidate_path = dest_path.with_file_name(candidate_filename);

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        let file_res = options.open(&candidate_path);
        let mut file = match file_res {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                continue;
            }
            Err(e) => {
                return Err(format!("Failed to open pre-rewrite backup {}: {e}", candidate_path.display()));
            }
        };

        let write_res = (|| -> std::io::Result<()> {
            file.write_all(source_bytes)?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_res {
            drop(file);
            let _ = std::fs::remove_file(&candidate_path);
            return Err(format!("Failed to write pre-rewrite backup {}: {e}", candidate_path.display()));
        }

        return Ok(candidate_path.to_string_lossy().to_string());
    }

    Err("Pre-rewrite backup failed: exhausted 1000 candidate collisions".to_string())
}

fn create_recovery_backup(
    dest_path: &Path,
    source_bytes: &[u8],
    suffix: &str,
    fault: bool,
) -> Result<String, String> {
    if fault {
        return Err("Simulated recovery backup fault".to_string());
    }
    use std::fs::OpenOptions;
    use std::io::Write;

    let ts = generate_timestamp();
    for attempt in 0..1000 {
        let candidate_filename = if attempt == 0 {
            format!("{}.{}.{}", CONFIG_FILE_NAME, suffix, ts)
        } else {
            format!("{}.{}.{}_{}", CONFIG_FILE_NAME, suffix, ts, attempt)
        };
        let candidate_path = dest_path.with_file_name(candidate_filename);

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        let file_res = options.open(&candidate_path);
        let mut file = match file_res {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                continue;
            }
            Err(e) => {
                return Err(format!("Failed to open recovery backup {}: {e}", candidate_path.display()));
            }
        };

        let write_res = (|| -> std::io::Result<()> {
            file.write_all(source_bytes)?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_res {
            drop(file);
            let _ = std::fs::remove_file(&candidate_path);
            return Err(format!("Failed to write recovery backup {}: {e}", candidate_path.display()));
        }

        return Ok(candidate_path.to_string_lossy().to_string());
    }

    Err("Recovery backup failed: exhausted 1000 candidate collisions".to_string())
}

/// Loads configuration from disk following the strict verification pipeline:
///
/// - Missing file: writes default file if possible.
/// - Malformed JSON: creates `.corrupt.<timestamp>` backup, writes fresh defaults.
/// - Unsupported schema version: preserves file byte-for-byte untouched, returns in-memory defaults with Locked status.
/// - Structural or semantic invalidity: creates `.invalid.<timestamp>` backup, writes fresh defaults.
/// - Destructive recovery halts if backup creation fails.
pub fn load_config(dest_path: &Path) -> (BornoConfig, PersistenceStatus, LoadOutcome) {
    load_config_with_seam(dest_path, CanonicalFaultInjection::default())
}

pub fn load_config_with_seam(
    dest_path: &Path,
    fault: CanonicalFaultInjection,
) -> (BornoConfig, PersistenceStatus, LoadOutcome) {
    if !dest_path.exists() {
        if fault.fail_default_write {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyMissingWriteFailed("Simulated default write fault".to_string()),
                LoadOutcome::MissingFileWriteFailed("Simulated default write fault".to_string()),
            );
        }
        match save_config_safe(dest_path, &BornoConfig::default()) {
            Ok(()) => return (BornoConfig::default(), PersistenceStatus::Active, LoadOutcome::CreatedDefault),
            Err(e) => {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyMissingWriteFailed(e.clone()),
                    LoadOutcome::MissingFileWriteFailed(e),
                );
            }
        }
    }

    if fault.fail_read {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyPermissionDenied("Simulated canonical read failure".to_string()),
            LoadOutcome::ReadFailed("Simulated canonical read failure".to_string()),
        );
    }

    let source_bytes = match std::fs::read(dest_path) {
        Ok(b) => b,
        Err(e) => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Access denied at {}: {}", dest_path.display(), e)),
                LoadOutcome::ReadFailed(e.to_string()),
            );
        }
    };

    let content = match std::str::from_utf8(&source_bytes) {
        Ok(s) => s,
        Err(_) => {
            // Malformed UTF-8 syntax treated as corrupt
            return recover_canonical_corrupt(dest_path, &source_bytes, fault);
        }
    };

    let classification = classify_schema_version(content);
    match classification {
        SchemaClassification::SupportedCurrent => {
            let raw_val: serde_json::Value = match serde_json::from_str(content) {
                Ok(v) => v,
                Err(_) => return recover_canonical_corrupt(dest_path, &source_bytes, fault),
            };
            let config: BornoConfig = match serde_json::from_value(raw_val) {
                Ok(c) => c,
                Err(_) => return recover_canonical_invalid(dest_path, &source_bytes, fault),
            };
            if config.validate().is_err() {
                return recover_canonical_invalid(dest_path, &source_bytes, fault);
            }
            (config, PersistenceStatus::Active, LoadOutcome::LoadedOk)
        }
        SchemaClassification::SupportedCurrentWithBuildMetadata(ver_str) => {
            let raw_val: serde_json::Value = match serde_json::from_str(content) {
                Ok(v) => v,
                Err(_) => return recover_canonical_corrupt(dest_path, &source_bytes, fault),
            };
            let mut config: BornoConfig = match serde_json::from_value(raw_val) {
                Ok(c) => c,
                Err(_) => return recover_canonical_invalid(dest_path, &source_bytes, fault),
            };

            // Pre-rewrite backup mandatory
            let prefix = format!("bak_normalize_{}", ver_str);
            if fault.fail_backup {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyPermissionDenied(format!("Normalization backup failed: simulated fault at {}", dest_path.display())),
                    LoadOutcome::ReadFailed("Simulated normalization backup failure".to_string()),
                );
            }
            if let Err(e) = create_prerewrite_backup(dest_path, &source_bytes, &prefix) {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyPermissionDenied(format!("Normalization backup failed: {e}")),
                    LoadOutcome::ReadFailed(e),
                );
            }

            config.schema_version = "1.2.0".to_string();
            if config.validate().is_err() {
                return recover_canonical_invalid(dest_path, &source_bytes, fault);
            }
            let _ = save_config_safe(dest_path, &config);
            (config, PersistenceStatus::Active, LoadOutcome::LoadedOk)
        }
        SchemaClassification::SupportedLegacyMigratable(ver_str) => {
            let raw_val: serde_json::Value = match serde_json::from_str(content) {
                Ok(v) => v,
                Err(_) => return recover_canonical_corrupt(dest_path, &source_bytes, fault),
            };
            let mut config: BornoConfig = match serde_json::from_value(raw_val) {
                Ok(c) => c,
                Err(_) => return recover_canonical_invalid(dest_path, &source_bytes, fault),
            };

            // Pre-rewrite backup mandatory
            let prefix = format!("bak_upgrade_{}", ver_str);
            if fault.fail_backup {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyPermissionDenied(format!("Upgrade backup failed: simulated fault at {}", dest_path.display())),
                    LoadOutcome::ReadFailed("Simulated upgrade backup failure".to_string()),
                );
            }
            if let Err(e) = create_prerewrite_backup(dest_path, &source_bytes, &prefix) {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyPermissionDenied(format!("Upgrade backup failed: {e}")),
                    LoadOutcome::ReadFailed(e),
                );
            }

            config.schema_version = "1.2.0".to_string();
            if config.validate().is_err() {
                return recover_canonical_invalid(dest_path, &source_bytes, fault);
            }
            let _ = save_config_safe(dest_path, &config);
            (config, PersistenceStatus::Active, LoadOutcome::LoadedOk)
        }
        SchemaClassification::SupportedLegacyWithBuildMetadata(ver_str) => {
            let raw_val: serde_json::Value = match serde_json::from_str(content) {
                Ok(v) => v,
                Err(_) => return recover_canonical_corrupt(dest_path, &source_bytes, fault),
            };
            let mut config: BornoConfig = match serde_json::from_value(raw_val) {
                Ok(c) => c,
                Err(_) => return recover_canonical_invalid(dest_path, &source_bytes, fault),
            };

            // Pre-rewrite backup mandatory
            let prefix = format!("bak_upgrade_{}", ver_str);
            if fault.fail_backup {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyPermissionDenied(format!("Upgrade backup failed: simulated fault at {}", dest_path.display())),
                    LoadOutcome::ReadFailed("Simulated upgrade backup failure".to_string()),
                );
            }
            if let Err(e) = create_prerewrite_backup(dest_path, &source_bytes, &prefix) {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyPermissionDenied(format!("Upgrade backup failed: {e}")),
                    LoadOutcome::ReadFailed(e),
                );
            }

            config.schema_version = "1.2.0".to_string();
            if config.validate().is_err() {
                return recover_canonical_invalid(dest_path, &source_bytes, fault);
            }
            let _ = save_config_safe(dest_path, &config);
            (config, PersistenceStatus::Active, LoadOutcome::LoadedOk)
        }
        SchemaClassification::UnsupportedFutureStable(ver) |
        SchemaClassification::UnsupportedFuturePrerelease(ver) => {
            (
                BornoConfig::default(),
                PersistenceStatus::LockedUnsupportedVersion { version: ver.clone() },
                LoadOutcome::UnsupportedVersionPreserved { version: ver },
            )
        }
        SchemaClassification::MalformedJson(_) => {
            recover_canonical_corrupt(dest_path, &source_bytes, fault)
        }
        SchemaClassification::InvalidMetadata |
        SchemaClassification::InvalidSemVerFormat(_) |
        SchemaClassification::UnsupportedDevelopmentPrerelease(_) |
        SchemaClassification::UnsupportedObsolete(_) |
        SchemaClassification::UnsupportedIntermediate(_) => {
            recover_canonical_invalid(dest_path, &source_bytes, fault)
        }
    }
}

fn recover_canonical_corrupt(
    dest_path: &Path,
    source_bytes: &[u8],
    fault: CanonicalFaultInjection,
) -> (BornoConfig, PersistenceStatus, LoadOutcome) {
    let backup_path = match create_recovery_backup(dest_path, source_bytes, "corrupt", fault.fail_backup) {
        Ok(bp) => bp,
        Err(e) => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Corrupt file backup failed: {e}")),
                LoadOutcome::CorruptBackupFailed(e),
            );
        }
    };

    if fault.fail_default_write {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyMissingWriteFailed("Simulated default write fault".to_string()),
            LoadOutcome::CorruptRecoveryWriteFailed {
                backup_path,
                error: "Simulated default write fault".to_string(),
            },
        );
    }

    match save_config_safe(dest_path, &BornoConfig::default()) {
        Ok(()) => (
            BornoConfig::default(),
            PersistenceStatus::Active,
            LoadOutcome::RecoveredCorrupt { backup_path },
        ),
        Err(e) => (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyMissingWriteFailed(e.clone()),
            LoadOutcome::CorruptRecoveryWriteFailed {
                backup_path,
                error: e,
            },
        ),
    }
}

fn recover_canonical_invalid(
    dest_path: &Path,
    source_bytes: &[u8],
    fault: CanonicalFaultInjection,
) -> (BornoConfig, PersistenceStatus, LoadOutcome) {
    let backup_path = match create_recovery_backup(dest_path, source_bytes, "invalid", fault.fail_backup) {
        Ok(bp) => bp,
        Err(e) => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Invalid file backup failed: {e}")),
                LoadOutcome::InvalidBackupFailed(e),
            );
        }
    };

    if fault.fail_default_write {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyMissingWriteFailed("Simulated default write fault".to_string()),
            LoadOutcome::InvalidRecoveryWriteFailed {
                backup_path,
                error: "Simulated default write fault".to_string(),
            },
        );
    }

    match save_config_safe(dest_path, &BornoConfig::default()) {
        Ok(()) => (
            BornoConfig::default(),
            PersistenceStatus::Active,
            LoadOutcome::RecoveredInvalid { backup_path },
        ),
        Err(e) => (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyMissingWriteFailed(e.clone()),
            LoadOutcome::InvalidRecoveryWriteFailed {
                backup_path,
                error: e,
            },
        ),
    }
}

fn handle_legacy_corrupt(
    source_path: &Path,
    canonical_path: &Path,
    error: &str,
    migration_write_fault: bool,
) -> (BornoConfig, PersistenceStatus, StartupOutcome, PathBuf) {
    if migration_write_fault {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyMissingWriteFailed(format!("Simulated write failure at {}", canonical_path.display())),
            StartupOutcome::LegacyCorruptPreservedWriteFailed {
                source_path: source_path.to_path_buf(),
                error: error.to_string(),
                write_error: "Simulated write failure".to_string(),
            },
            canonical_path.to_path_buf(),
        );
    }
    match save_config_safe(canonical_path, &BornoConfig::default()) {
        Ok(()) => (
            BornoConfig::default(),
            PersistenceStatus::Active,
            StartupOutcome::LegacyCorruptPreserved {
                source_path: source_path.to_path_buf(),
                error: error.to_string(),
            },
            canonical_path.to_path_buf(),
        ),
        Err(we) => (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyMissingWriteFailed(we.clone()),
            StartupOutcome::LegacyCorruptPreservedWriteFailed {
                source_path: source_path.to_path_buf(),
                error: error.to_string(),
                write_error: we,
            },
            canonical_path.to_path_buf(),
        ),
    }
}

fn handle_legacy_invalid(
    source_path: &Path,
    canonical_path: &Path,
    error: &str,
    migration_write_fault: bool,
) -> (BornoConfig, PersistenceStatus, StartupOutcome, PathBuf) {
    if migration_write_fault {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyMissingWriteFailed(format!("Simulated write failure at {}", canonical_path.display())),
            StartupOutcome::LegacyInvalidPreservedWriteFailed {
                source_path: source_path.to_path_buf(),
                error: error.to_string(),
                write_error: "Simulated write failure".to_string(),
            },
            canonical_path.to_path_buf(),
        );
    }
    match save_config_safe(canonical_path, &BornoConfig::default()) {
        Ok(()) => (
            BornoConfig::default(),
            PersistenceStatus::Active,
            StartupOutcome::LegacyInvalidPreserved {
                source_path: source_path.to_path_buf(),
                error: error.to_string(),
            },
            canonical_path.to_path_buf(),
        ),
        Err(we) => (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyMissingWriteFailed(we.clone()),
            StartupOutcome::LegacyInvalidPreservedWriteFailed {
                source_path: source_path.to_path_buf(),
                error: error.to_string(),
                write_error: we,
            },
            canonical_path.to_path_buf(),
        ),
    }
}

fn evaluate_and_migrate_legacy_candidate(
    source_path: &Path,
    canonical_path: &Path,
    source_read_fault: bool,
    migration_write_fault: bool,
    backup_fault: Option<BackupFaultInjection>,
) -> (BornoConfig, PersistenceStatus, StartupOutcome, PathBuf) {
    if source_read_fault {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyPermissionDenied(format!("Failed to read legacy source file at {}: Simulated read error", source_path.display())),
            StartupOutcome::LegacySourceReadFailed {
                source_path: source_path.to_path_buf(),
                error: "Simulated read error".to_string(),
            },
            canonical_path.to_path_buf(),
        );
    }

    let source_bytes = match std::fs::read(source_path) {
        Ok(b) => b,
        Err(e) => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Access denied reading legacy source at {}: {e}", source_path.display())),
                StartupOutcome::LegacySourceReadFailed {
                    source_path: source_path.to_path_buf(),
                    error: e.to_string(),
                },
                canonical_path.to_path_buf(),
            );
        }
    };

    let content = match std::str::from_utf8(&source_bytes) {
        Ok(s) => s,
        Err(_) => {
            // Malformed UTF-8
            return handle_legacy_corrupt(source_path, canonical_path, "Malformed UTF-8", migration_write_fault);
        }
    };

    let classification = classify_schema_version(content);
    match classification {
        SchemaClassification::SupportedCurrent |
        SchemaClassification::SupportedLegacyMigratable(_) |
        SchemaClassification::SupportedCurrentWithBuildMetadata(_) |
        SchemaClassification::SupportedLegacyWithBuildMetadata(_) => {
            let raw_val: serde_json::Value = match serde_json::from_str(content) {
                Ok(v) => v,
                Err(e) => return handle_legacy_corrupt(source_path, canonical_path, &e.to_string(), migration_write_fault),
            };

            let mut config: BornoConfig = match serde_json::from_value(raw_val) {
                Ok(c) => c,
                Err(e) => return handle_legacy_invalid(source_path, canonical_path, &e.to_string(), migration_write_fault),
            };

            if let Err(e) = config.validate() {
                return handle_legacy_invalid(source_path, canonical_path, &e.to_string(), migration_write_fault);
            }

            // Normalize schema version to 1.2.0
            config.schema_version = "1.2.0".to_string();

            if migration_write_fault {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyMissingWriteFailed(format!("Simulated migration write failure at {}", canonical_path.display())),
                    StartupOutcome::MigrationWriteFailed {
                        source_path: source_path.to_path_buf(),
                        error: "Simulated migration write failure".to_string(),
                    },
                    canonical_path.to_path_buf(),
                );
            }

            // Step 1: Write to canonical destination
            if let Err(e) = save_config_safe(canonical_path, &config) {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyMissingWriteFailed(e.clone()),
                    StartupOutcome::MigrationWriteFailed {
                        source_path: source_path.to_path_buf(),
                        error: e,
                    },
                    canonical_path.to_path_buf(),
                );
            }

            // Step 2: Attempt legacy backup (decoupled)
            let backup_outcome = create_exclusive_backup(
                &source_bytes,
                source_path,
                "bak_migrated",
                backup_fault,
            );

            (
                config,
                PersistenceStatus::Active,
                StartupOutcome::Migrated {
                    source_path: source_path.to_path_buf(),
                    backup_outcome,
                },
                canonical_path.to_path_buf(),
            )
        }
        SchemaClassification::UnsupportedFutureStable(ver) |
        SchemaClassification::UnsupportedFuturePrerelease(ver) => {
            (
                BornoConfig::default(),
                PersistenceStatus::LockedUnsupportedVersion { version: ver.clone() },
                StartupOutcome::LegacyUnsupportedPreserved {
                    source_path: source_path.to_path_buf(),
                    version: ver,
                },
                canonical_path.to_path_buf(),
            )
        }
        SchemaClassification::MalformedJson(e) => {
            handle_legacy_corrupt(source_path, canonical_path, &e, migration_write_fault)
        }
        SchemaClassification::InvalidMetadata => {
            handle_legacy_invalid(source_path, canonical_path, "Missing or invalid schema_version", migration_write_fault)
        }
        SchemaClassification::InvalidSemVerFormat(s) => {
            handle_legacy_invalid(source_path, canonical_path, &format!("Invalid SemVer format: {s}"), migration_write_fault)
        }
        SchemaClassification::UnsupportedDevelopmentPrerelease(s) => {
            handle_legacy_invalid(source_path, canonical_path, &format!("Unsupported development prerelease: {s}"), migration_write_fault)
        }
        SchemaClassification::UnsupportedObsolete(s) => {
            handle_legacy_invalid(source_path, canonical_path, &format!("Unsupported obsolete schema: {s}"), migration_write_fault)
        }
        SchemaClassification::UnsupportedIntermediate(s) => {
            handle_legacy_invalid(source_path, canonical_path, &format!("Unsupported intermediate schema: {s}"), migration_write_fault)
        }
    }
}

pub fn startup_pipeline(
    resolver: &impl ConfigPathResolver,
) -> (BornoConfig, PersistenceStatus, StartupOutcome, PathBuf) {
    startup_pipeline_with_seams(resolver, None)
}

pub fn startup_pipeline_with_seams(
    resolver: &impl ConfigPathResolver,
    seams: Option<&StartupFaultInjection>,
) -> (BornoConfig, PersistenceStatus, StartupOutcome, PathBuf) {
    // 1. Resolve canonical path via resolver
    let canonical_path = match resolver.resolve_canonical_path() {
        Ok(p) => p,
        Err(e) => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Path resolution failed: {e}")),
                StartupOutcome::PathResolutionFailed(e),
                PathBuf::new(),
            );
        }
    };

    // 2. Canonical Path Inspection (symlink_metadata explicitly rejects symlinks)
    if seams.map_or(false, |s| s.canonical_inspection_fault) {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyPermissionDenied(format!("Simulated canonical inspection error at {}", canonical_path.display())),
            StartupOutcome::CanonicalInspectionFailed("Simulated canonical inspection error".to_string()),
            canonical_path,
        );
    }

    match inspect_path(&canonical_path) {
        PathStatus::Present => {
            // Authoritative canonical loader and recovery pipeline
            let (config, status, outcome) = if let Some(canon_fault) = seams.and_then(|s| s.canonical_recovery_fault) {
                load_config_with_seam(&canonical_path, canon_fault)
            } else {
                load_config(&canonical_path)
            };
            return (config, status, StartupOutcome::CanonicalLoaded(outcome), canonical_path);
        }
        PathStatus::Error(e) => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Filesystem I/O error ({:?}) inspecting canonical path {}: {e}", e.kind(), canonical_path.display())),
                StartupOutcome::CanonicalInspectionFailed(e.to_string()),
                canonical_path,
            );
        }
        PathStatus::NotFound => {
            // Proceed to derive legacy candidates
        }
    }

    // 3. Derive legacy sibling paths strictly from shared parent
    let legacy_paths = match derive_sibling_paths(&canonical_path) {
        Some(p) => p,
        None => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Cannot derive sibling paths from canonical path {} (unexpected directory name or no shared parent)", canonical_path.display())),
                StartupOutcome::ParentResolutionFailed(format!("Canonical path has invalid structure: {}", canonical_path.display())),
                canonical_path,
            );
        }
    };

    let primary_legacy_path = legacy_paths.primary_legacy_path;
    let secondary_legacy_path = legacy_paths.secondary_legacy_path;

    // 4. Primary Legacy Evaluation
    if seams.map_or(false, |s| s.primary_legacy_inspection_fault) {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyPermissionDenied(format!("Simulated primary legacy inspection error at {}", primary_legacy_path.display())),
            StartupOutcome::LegacyInspectionFailed { source_path: primary_legacy_path, error: "Simulated inspection error".to_string() },
            canonical_path,
        );
    }

    match inspect_path(&primary_legacy_path) {
        PathStatus::Present => {
            return evaluate_and_migrate_legacy_candidate(
                &primary_legacy_path,
                &canonical_path,
                seams.map_or(false, |s| s.primary_legacy_source_read_fault),
                seams.map_or(false, |s| s.migration_write_fault),
                seams.and_then(|s| s.migration_backup_fault),
            );
        }
        PathStatus::Error(e) => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Filesystem I/O error ({:?}) inspecting primary legacy path {}: {e}", e.kind(), primary_legacy_path.display())),
                StartupOutcome::LegacyInspectionFailed { source_path: primary_legacy_path, error: e.to_string() },
                canonical_path,
            );
        }
        PathStatus::NotFound => {
            // Fall through to secondary legacy candidate
        }
    }

    // 5. Secondary Legacy Evaluation
    if seams.map_or(false, |s| s.secondary_legacy_inspection_fault) {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyPermissionDenied(format!("Simulated secondary legacy inspection error at {}", secondary_legacy_path.display())),
            StartupOutcome::LegacyInspectionFailed { source_path: secondary_legacy_path, error: "Simulated inspection error".to_string() },
            canonical_path,
        );
    }

    match inspect_path(&secondary_legacy_path) {
        PathStatus::Present => {
            return evaluate_and_migrate_legacy_candidate(
                &secondary_legacy_path,
                &canonical_path,
                seams.map_or(false, |s| s.secondary_legacy_source_read_fault),
                seams.map_or(false, |s| s.migration_write_fault),
                seams.and_then(|s| s.migration_backup_fault),
            );
        }
        PathStatus::Error(e) => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Filesystem I/O error ({:?}) inspecting secondary legacy path {}: {e}", e.kind(), secondary_legacy_path.display())),
                StartupOutcome::LegacyInspectionFailed { source_path: secondary_legacy_path, error: e.to_string() },
                canonical_path,
            );
        }
        PathStatus::NotFound => {
            // Fall through to clean installation
        }
    }

    // 6. Clean Installation Path
    if seams.map_or(false, |s| s.clean_install_write_fault) {
        return (
            BornoConfig::default(),
            PersistenceStatus::ReadOnlyMissingWriteFailed(format!("Simulated clean install write fault at {}", canonical_path.display())),
            StartupOutcome::CleanInstallWriteFailed("Simulated clean install write fault".to_string()),
            canonical_path,
        );
    }

    let default_config = BornoConfig::default();
    match save_config_safe(&canonical_path, &default_config) {
        Ok(()) => (
            default_config,
            PersistenceStatus::Active,
            StartupOutcome::CleanInstallCreated,
            canonical_path,
        ),
        Err(e) => (
            default_config,
            PersistenceStatus::ReadOnlyMissingWriteFailed(e.clone()),
            StartupOutcome::CleanInstallWriteFailed(e),
            canonical_path,
        ),
    }
}

/// Resets the configuration file to defaults.
///
/// If `is_unsupported_version` is true, creates a `.bak_unsupported.<timestamp>` backup
/// prior to replacing with defaults.
pub fn reset_config_safe(
    dest_path: &Path,
    is_unsupported_version: bool,
) -> Result<(BornoConfig, Option<String>), String> {
    let raw_bytes = if is_unsupported_version && dest_path.exists() {
        std::fs::read(dest_path).ok()
    } else {
        None
    };

    reset_config_safe_with_faults(
        dest_path,
        raw_bytes.as_deref(),
        false,
        false,
    )
}

pub fn reset_config_safe_with_faults(
    dest_path: &Path,
    raw_disk_bytes: Option<&[u8]>,
    fail_backup: bool,
    fail_default_write: bool,
) -> Result<(BornoConfig, Option<String>), String> {
    if fail_backup {
        return Err("Failed to create backup: Simulated backup failure. Reset aborted.".to_string());
    }

    let backup_path = if let Some(bytes) = raw_disk_bytes {
        if dest_path.exists() {
            let bp = create_prerewrite_backup(dest_path, bytes, "bak_unsupported")
                .map_err(|e| format!("Failed to create backup of unsupported configuration: {e}. Reset aborted."))?;
            Some(bp)
        } else {
            None
        }
    } else {
        None
    };

    if fail_default_write {
        return Err("Failed to write default configuration: Simulated default write failure".to_string());
    }

    let default_config = BornoConfig::default();
    save_config_safe(dest_path, &default_config)?;
    Ok((default_config, backup_path))
}
