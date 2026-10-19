//! Failure-safe filesystem persistence, recovery, and backup management for Borno configuration.
//!
//! Owns all operating-system configuration directory paths, atomic safe replacement,
//! corrupted file backups, and protection of unsupported schema versions.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use bornomala_core::BornoConfig;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

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
/// 1. Creates a temporary file in the EXACT SAME directory and volume as `dest_path`.
/// 2. Flushes and syncs all bytes to disk before closing the handle.
/// 3. Replaces `dest_path` via `std::fs::rename`.
/// 4. Never deletes `dest_path` before replacement succeeds; preserves original file on failure.
pub fn save_config_safe(dest_path: &Path, config: &BornoConfig) -> Result<(), String> {
    config.validate().map_err(|e| e.to_string())?;

    let json_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Serialization error: {e}"))?;

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {e}", parent.display()))?;
    }

    let tmp_path = generate_temp_path(dest_path);

    // Open and write to temp file
    let write_res = (|| -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp_path)?;
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

/// Loads configuration from disk following the strict verification pipeline:
///
/// - Missing file: writes default file if possible.
/// - Malformed JSON: creates `.corrupt.<timestamp>` backup, writes fresh defaults.
/// - Unsupported schema version: preserves file byte-for-byte untouched, returns in-memory defaults with Locked status.
/// - Structural or semantic invalidity: creates `.invalid.<timestamp>` backup, writes fresh defaults.
/// - Destructive recovery halts if backup creation fails.
pub fn load_config(dest_path: &Path) -> (BornoConfig, PersistenceStatus, LoadOutcome) {
    if !dest_path.exists() {
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

    let content = match std::fs::read_to_string(dest_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(e.to_string()),
                LoadOutcome::ReadFailed(e.to_string()),
            );
        }
    };

    // Step 1: Parse generic JSON syntax
    let raw_val: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            // Malformed JSON syntax
            let backup_path = format!("{}.corrupt.{}", dest_path.display(), generate_timestamp());
            if let Err(e) = std::fs::copy(dest_path, &backup_path) {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyPermissionDenied(format!("Corrupt file backup failed: {e}")),
                    LoadOutcome::CorruptBackupFailed(e.to_string()),
                );
            }
            let _ = save_config_safe(dest_path, &BornoConfig::default());
            return (
                BornoConfig::default(),
                PersistenceStatus::Active,
                LoadOutcome::RecoveredCorrupt { backup_path },
            );
        }
    };

    // Step 2: Inspect schema_version
    let version_str = match raw_val.get("schema_version").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => {
            // Missing or non-string schema_version
            let backup_path = format!("{}.invalid.{}", dest_path.display(), generate_timestamp());
            if let Err(e) = std::fs::copy(dest_path, &backup_path) {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyPermissionDenied(format!("Invalid file backup failed: {e}")),
                    LoadOutcome::InvalidBackupFailed(e.to_string()),
                );
            }
            let _ = save_config_safe(dest_path, &BornoConfig::default());
            return (
                BornoConfig::default(),
                PersistenceStatus::Active,
                LoadOutcome::RecoveredInvalid { backup_path },
            );
        }
    };

    if version_str != "1.0.0" && version_str != "1.1.0" && version_str != "1.2.0" {
        // Unsupported schema version: PRESERVE BYTE-FOR-BYTE UNTOUCHED
        return (
            BornoConfig::default(),
            PersistenceStatus::LockedUnsupportedVersion {
                version: version_str.to_string(),
            },
            LoadOutcome::UnsupportedVersionPreserved {
                version: version_str.to_string(),
            },
        );
    }

    // Step 3: Typed deserialization with serde field defaults
    let mut config: BornoConfig = match serde_json::from_value(raw_val) {
        Ok(c) => c,
        Err(_) => {
            // Structural / type mismatch
            let backup_path = format!("{}.invalid.{}", dest_path.display(), generate_timestamp());
            if let Err(e) = std::fs::copy(dest_path, &backup_path) {
                return (
                    BornoConfig::default(),
                    PersistenceStatus::ReadOnlyPermissionDenied(format!("Invalid file backup failed: {e}")),
                    LoadOutcome::InvalidBackupFailed(e.to_string()),
                );
            }
            let _ = save_config_safe(dest_path, &BornoConfig::default());
            return (
                BornoConfig::default(),
                PersistenceStatus::Active,
                LoadOutcome::RecoveredInvalid { backup_path },
            );
        }
    };

    // If migrating from 1.0.0 or 1.1.0, upgrade schema_version to 1.2.0 and persist to disk
    if config.schema_version == "1.0.0" || config.schema_version == "1.1.0" {
        config.schema_version = "1.2.0".to_string();
        let _ = save_config_safe(dest_path, &config);
    }

    // Step 4: Semantic validation
    if config.validate().is_err() {
        let backup_path = format!("{}.invalid.{}", dest_path.display(), generate_timestamp());
        if let Err(e) = std::fs::copy(dest_path, &backup_path) {
            return (
                BornoConfig::default(),
                PersistenceStatus::ReadOnlyPermissionDenied(format!("Invalid file backup failed: {e}")),
                LoadOutcome::InvalidBackupFailed(e.to_string()),
            );
        }
        let _ = save_config_safe(dest_path, &BornoConfig::default());
        return (
            BornoConfig::default(),
            PersistenceStatus::Active,
            LoadOutcome::RecoveredInvalid { backup_path },
        );
    }

    (config, PersistenceStatus::Active, LoadOutcome::LoadedOk)
}

/// Resets the configuration file to defaults.
///
/// If `is_unsupported_version` is true, creates a `.bak_unsupported.<timestamp>` backup
/// prior to replacing with defaults.
pub fn reset_config_safe(
    dest_path: &Path,
    is_unsupported_version: bool,
) -> Result<(BornoConfig, Option<String>), String> {
    let backup_path = if is_unsupported_version && dest_path.exists() {
        let bp = format!("{}.bak_unsupported.{}", dest_path.display(), generate_timestamp());
        std::fs::copy(dest_path, &bp)
            .map_err(|e| format!("Failed to create backup of unsupported configuration: {e}. Reset aborted."))?;
        Some(bp)
    } else {
        None
    };

    let default_config = BornoConfig::default();
    save_config_safe(dest_path, &default_config)?;
    Ok((default_config, backup_path))
}

