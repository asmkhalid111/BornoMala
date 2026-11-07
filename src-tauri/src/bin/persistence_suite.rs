//! Layer 3 Application Persistence, Migration & Recovery Test Suite
//!
//! Validates atomic safe replacement, recovery from malformed files,
//! pre-upgrade/normalization backups, unsupported schema protection,
//! sibling legacy migration, startup fault injection, and degraded session recovery.

use bornomala_core::{BornoConfig, BornoEngine};
use bornomala_desktop_lib::config_store::*;
use bornomala_desktop_lib::{
    apply_and_persist_config, execute_reset_transaction, import_config_internal, AppState,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("borno_persist_test_{}_{}", std::process::id(), now));
        std::fs::create_dir_all(&path).expect("create test temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn create_test_state(canonical_path: PathBuf, status: PersistenceStatus) -> AppState {
    AppState {
        engine: Mutex::new(BornoEngine::new()),
        config_mutex: Mutex::new(()),
        config_path: canonical_path,
        persistence_status: Mutex::new(status),
        effective_config: Mutex::new(BornoConfig::default()),
        session_degraded: AtomicBool::new(false),
    }
}

// ============================================================================
// 8 Baseline Tests Retained from Prior Milestones
// ============================================================================

fn test_persistence_initializes_default_on_missing_file() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");

    assert!(!path.exists());
    let (config, status, outcome) = load_config(&path);

    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::CreatedDefault);
    assert!(path.exists());

    let disk_content = std::fs::read_to_string(&path).expect("read");
    let parsed: BornoConfig = serde_json::from_str(&disk_content).expect("parse");
    assert_eq!(parsed, BornoConfig::default());
}

fn test_persistence_save_and_reload_roundtrip() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");

    let mut custom = BornoConfig::default();
    custom.candidate_display_limit = 2;
    custom.auto_commit_on_mode_switch = false;
    custom.preferred_fixed_layout = bornomala_core::layout::FixedLayoutId::Probhat;
    custom.default_startup_mode = bornomala_core::StartupMode::Fixed;

    save_config_safe(&path, &custom).expect("save");

    let (loaded, status, outcome) = load_config(&path);
    assert_eq!(loaded, custom);
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::LoadedOk);
}

fn test_persistence_atomic_replacement_of_existing_file() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");

    let initial = BornoConfig::default();
    save_config_safe(&path, &initial).expect("initial save");
    assert_eq!(load_config(&path).0.candidate_display_limit, 5);

    let mut updated = initial.clone();
    updated.candidate_display_limit = 3;
    save_config_safe(&path, &updated).expect("replacement save");

    let (reloaded, status, outcome) = load_config(&path);
    assert_eq!(reloaded.candidate_display_limit, 3);
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::LoadedOk);

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read_dir")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_name(), "config.json");
}

fn test_persistence_replacement_failure_preserves_existing_config() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");

    let initial = BornoConfig::default();
    save_config_safe(&path, &initial).expect("initial save");

    let mut invalid = initial.clone();
    invalid.candidate_display_limit = 0;
    let res = save_config_safe(&path, &invalid);
    assert!(res.is_err());

    let (reloaded, status, outcome) = load_config(&path);
    assert_eq!(reloaded.candidate_display_limit, 5);
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::LoadedOk);
}

fn test_persistence_corrupted_json_recovers_and_backs_up() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");

    std::fs::write(&path, "{ corrupt json syntax !!").expect("write");

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);

    match outcome {
        LoadOutcome::RecoveredCorrupt { backup_path } => {
            assert!(Path::new(&backup_path).exists());
            let backup_content = std::fs::read_to_string(&backup_path).expect("read backup");
            assert_eq!(backup_content, "{ corrupt json syntax !!");
        }
        other => panic!("Expected RecoveredCorrupt, got {:?}", other),
    }

    let (reloaded, _, _) = load_config(&path);
    assert_eq!(reloaded, BornoConfig::default());
}

fn test_persistence_unsupported_schema_version_is_preserved_byte_for_byte() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");

    let future_json = r#"{
  "schema_version": "2.0.0",
  "default_startup_mode": "Phonetic",
  "candidate_display_limit": 8,
  "future_super_feature": true
}"#;
    std::fs::write(&path, future_json).expect("write future json");

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(
        status,
        PersistenceStatus::LockedUnsupportedVersion {
            version: "2.0.0".to_string()
        }
    );
    assert_eq!(
        outcome,
        LoadOutcome::UnsupportedVersionPreserved {
            version: "2.0.0".to_string()
        }
    );

    let content_after = std::fs::read_to_string(&path).expect("read after");
    assert_eq!(content_after, future_json);
}

fn test_persistence_unsupported_schema_reset_creates_backup_and_restores_defaults() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");

    let future_json = r#"{"schema_version": "9.9.9"}"#;
    std::fs::write(&path, future_json).expect("write future json");

    let (reset_config, backup_path) = reset_config_safe(&path, true).expect("reset");
    assert_eq!(reset_config, BornoConfig::default());
    assert!(backup_path.is_some());

    let bp = backup_path.unwrap();
    assert!(Path::new(&bp).exists());
    assert_eq!(std::fs::read_to_string(&bp).expect("read bp"), future_json);

    let (reloaded, status, outcome) = load_config(&path);
    assert_eq!(reloaded, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::LoadedOk);
}

fn test_persistence_migration_upgrades_to_v1_2_0_and_adds_defaults() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");

    let legacy_1_0_0_json = r#"{
  "schema_version": "1.0.0",
  "default_startup_mode": "Phonetic",
  "preferred_fixed_layout": "National",
  "candidate_display_limit": 4,
  "auto_commit_on_mode_switch": false
}"#;
    std::fs::write(&path, legacy_1_0_0_json).expect("write legacy 1.0.0 json");

    let (loaded, status, outcome) = load_config(&path);
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::LoadedOk);
    assert_eq!(loaded.schema_version, "1.2.0");
    assert_eq!(loaded.candidate_display_limit, 4);
    assert!(!loaded.auto_commit_on_mode_switch);
    assert!(loaded.enable_dictionary_assistance);
    assert!(loaded.enable_word_completion);
    assert_eq!(loaded.max_dictionary_suggestions, 3);
    assert!(!loaded.auto_convert_period_to_dari);
    assert_eq!(loaded.candidate_orientation, bornomala_core::CandidateOrientation::Vertical);
    assert_eq!(loaded.candidate_font_size, 14);
    assert_eq!(loaded.active_theme, bornomala_core::ThemeId::DarkGlass);
    assert_eq!(loaded.mode_toggle_key, bornomala_core::ModeToggleKey::F12);

    let disk_content = std::fs::read_to_string(&path).expect("read upgraded json");
    let disk_config: BornoConfig = serde_json::from_str(&disk_content).expect("parse upgraded disk config");
    assert_eq!(disk_config.schema_version, "1.2.0");
    assert_eq!(disk_config.candidate_display_limit, 4);
    assert!(!disk_config.auto_commit_on_mode_switch);

    let (reloaded, re_status, re_outcome) = load_config(&path);
    assert_eq!(reloaded, loaded);
    assert_eq!(re_status, PersistenceStatus::Active);
    assert_eq!(re_outcome, LoadOutcome::LoadedOk);
}

// ============================================================================
// Proposed Migration, Recovery, and Safety Tests (TEST-MIG-01 to TEST-MIG-64)
// ============================================================================

// TEST-MIG-01: Malformed JSON syntax in canonical file
fn test_schema_canonical_malformed_json_recovers_and_backs_up() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let corrupt_content = "{ corrupt syntax !!";
    std::fs::write(&path, corrupt_content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    match outcome {
        LoadOutcome::RecoveredCorrupt { backup_path } => {
            assert!(Path::new(&backup_path).exists());
            assert_eq!(std::fs::read_to_string(&backup_path).unwrap(), corrupt_content);
        }
        other => panic!("Expected RecoveredCorrupt, got {:?}", other),
    }
    let reloaded: BornoConfig = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(reloaded, BornoConfig::default());
}

// TEST-MIG-02: Canonical JSON parses but lacks schema_version key
fn test_schema_canonical_missing_schema_version_recovers_and_backs_up() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let missing_version_json = r#"{"candidate_display_limit": 3}"#;
    std::fs::write(&path, missing_version_json).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    match outcome {
        LoadOutcome::RecoveredInvalid { backup_path } => {
            assert!(Path::new(&backup_path).exists());
            assert_eq!(std::fs::read_to_string(&backup_path).unwrap(), missing_version_json);
        }
        other => panic!("Expected RecoveredInvalid, got {:?}", other),
    }
}

// TEST-MIG-03: Canonical has non-string schema_version
fn test_schema_canonical_non_string_schema_version_recovers() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": 123, "candidate_display_limit": 3}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, LoadOutcome::RecoveredInvalid { .. }));
}

// TEST-MIG-04: Canonical has malformed semver string (not future)
fn test_schema_canonical_malformed_semver_string_not_future() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "abc", "candidate_display_limit": 3}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, LoadOutcome::RecoveredInvalid { .. }));
}

// TEST-MIG-05: Canonical has structural type mismatch
fn test_schema_canonical_structural_type_mismatch_recovers() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.2.0", "candidate_display_limit": "invalid_string"}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, LoadOutcome::RecoveredInvalid { .. }));
}

// TEST-MIG-06: Canonical semantic validation failure
fn test_schema_canonical_semantic_validation_failure_recovers() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.2.0", "candidate_display_limit": 99}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, LoadOutcome::RecoveredInvalid { .. }));
}

// TEST-MIG-07: Canonical has obsolete semver (< 1.0.0)
fn test_schema_canonical_obsolete_semver_recovers() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "0.9.0", "candidate_display_limit": 3}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, LoadOutcome::RecoveredInvalid { .. }));
}

// TEST-MIG-08: Canonical has intermediate semver (1.0.1)
fn test_schema_canonical_intermediate_semver_recovers() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.0.1", "candidate_display_limit": 3}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, LoadOutcome::RecoveredInvalid { .. }));
}

// TEST-MIG-09: Canonical has development prerelease semver (1.2.0-rc.1)
fn test_schema_canonical_prerelease_semver_recovers() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.2.0-rc.1", "candidate_display_limit": 3}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, LoadOutcome::RecoveredInvalid { .. }));
}

// TEST-MIG-10: Build metadata normalization preserves unknown properties in backup
fn test_schema_canonical_build_metadata_normalization_preserves_unknown_properties_in_backup() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{
  "schema_version": "1.2.0+build.5",
  "candidate_display_limit": 4,
  "custom_key": 42
}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config.schema_version, "1.2.0");
    assert_eq!(config.candidate_display_limit, 4);
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::LoadedOk);

    // Verify backup created with custom_key
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let backup = entries.iter().find(|e| e.file_name().to_string_lossy().contains(".bak_normalize"));
    assert!(backup.is_some(), "Expected .bak_normalize backup");
    let backup_str = std::fs::read_to_string(backup.unwrap().path()).unwrap();
    assert!(backup_str.contains("\"custom_key\": 42"));
    assert!(backup_str.contains("1.2.0+build.5"));
}

// TEST-MIG-11: Legacy build metadata upgrade preserves unknown properties in backup
fn test_schema_canonical_legacy_build_metadata_upgrade_preserves_unknown_properties_in_backup() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{
  "schema_version": "1.1.0+build.5",
  "candidate_display_limit": 4,
  "plugin_data": true
}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config.schema_version, "1.2.0");
    assert_eq!(config.candidate_display_limit, 4);
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::LoadedOk);

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let backup = entries.iter().find(|e| e.file_name().to_string_lossy().contains(".bak_upgrade_1.1.0"));
    assert!(backup.is_some(), "Expected .bak_upgrade_1.1.0 backup");
    let backup_str = std::fs::read_to_string(backup.unwrap().path()).unwrap();
    assert!(backup_str.contains("\"plugin_data\": true"));
}

// TEST-MIG-12: Supported v1.0.0 creates pre-upgrade backup
fn test_schema_canonical_supported_v1_0_0_creates_preupgrade_backup() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.0.0", "candidate_display_limit": 4}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config.schema_version, "1.2.0");
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::LoadedOk);

    let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().filter_map(|e| e.ok()).collect();
    let backup = entries.iter().find(|e| e.file_name().to_string_lossy().contains(".bak_upgrade_1.0.0"));
    assert!(backup.is_some(), "Expected .bak_upgrade_1.0.0 backup");
}

// TEST-MIG-13: Supported v1.1.0 creates pre-upgrade backup
fn test_schema_canonical_supported_v1_1_0_creates_preupgrade_backup() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.1.0", "candidate_display_limit": 4}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config.schema_version, "1.2.0");
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(outcome, LoadOutcome::LoadedOk);

    let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().filter_map(|e| e.ok()).collect();
    let backup = entries.iter().find(|e| e.file_name().to_string_lossy().contains(".bak_upgrade_1.1.0"));
    assert!(backup.is_some(), "Expected .bak_upgrade_1.1.0 backup");
}

// TEST-MIG-14: v1.0.0 upgrade backup failure prevents replacement
fn test_canonical_v1_0_0_upgrade_backup_failure_prevents_replacement() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.0.0", "candidate_display_limit": 4}"#;
    std::fs::write(&path, content).unwrap();

    let seam = CanonicalFaultInjection {
        fail_backup: true,
        ..Default::default()
    };
    let (_, status, _) = load_config_with_seam(&path, seam);
    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
}

// TEST-MIG-15: v1.1.0 upgrade backup failure prevents replacement
fn test_canonical_v1_1_0_upgrade_backup_failure_prevents_replacement() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.1.0", "candidate_display_limit": 4}"#;
    std::fs::write(&path, content).unwrap();

    let seam = CanonicalFaultInjection {
        fail_backup: true,
        ..Default::default()
    };
    let (_, status, _) = load_config_with_seam(&path, seam);
    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
}

// TEST-MIG-16: Canonical has future semver (2.0.0) -> locked
fn test_schema_canonical_future_semver_preserves_and_locks() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "2.0.0", "feature": "ultra"}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::LockedUnsupportedVersion { version: "2.0.0".to_string() });
    assert_eq!(outcome, LoadOutcome::UnsupportedVersionPreserved { version: "2.0.0".to_string() });
    assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
}

// TEST-MIG-17: Canonical has future prerelease (1.3.0-alpha.1) -> locked
fn test_schema_canonical_future_prerelease_preserves_and_locks() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.3.0-alpha.1"}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::LockedUnsupportedVersion { version: "1.3.0-alpha.1".to_string() });
    assert_eq!(outcome, LoadOutcome::UnsupportedVersionPreserved { version: "1.3.0-alpha.1".to_string() });
    assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
}

// TEST-MIG-18: Canonical has future patch prerelease (1.2.1-alpha.1) -> locked
fn test_schema_canonical_future_prerelease_patch_preserves_and_locks() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "1.2.1-alpha.1"}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::LockedUnsupportedVersion { version: "1.2.1-alpha.1".to_string() });
    assert_eq!(outcome, LoadOutcome::UnsupportedVersionPreserved { version: "1.2.1-alpha.1".to_string() });
    assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
}

// TEST-MIG-19: Canonical has future semver with build metadata (2.0.0+build.1) -> locked
fn test_schema_canonical_future_semver_with_build_metadata_locked() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let content = r#"{"schema_version": "2.0.0+build.1"}"#;
    std::fs::write(&path, content).unwrap();

    let (config, status, outcome) = load_config(&path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::LockedUnsupportedVersion { version: "2.0.0+build.1".to_string() });
    assert_eq!(outcome, LoadOutcome::UnsupportedVersionPreserved { version: "2.0.0+build.1".to_string() });
    assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
}

// TEST-MIG-20: Canonical precedence over legacy
fn test_migration_canonical_precedence_over_legacy() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let primary_dir = dir.path().join("com.bornomala.sandbox");
    let secondary_dir = dir.path().join("com.borno.sandbox");
    std::fs::create_dir_all(&canon_dir).unwrap();
    std::fs::create_dir_all(&primary_dir).unwrap();
    std::fs::create_dir_all(&secondary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let primary_path = primary_dir.join("config.json");
    let secondary_path = secondary_dir.join("config.json");

    let mut canon_conf = BornoConfig::default();
    canon_conf.candidate_display_limit = 4;
    save_config_safe(&canon_path, &canon_conf).unwrap();

    let mut pri_conf = BornoConfig::default();
    pri_conf.candidate_display_limit = 8;
    save_config_safe(&primary_path, &pri_conf).unwrap();

    let mut sec_conf = BornoConfig::default();
    sec_conf.candidate_display_limit = 9;
    save_config_safe(&secondary_path, &sec_conf).unwrap();

    let pri_bytes = std::fs::read(&primary_path).unwrap();
    let sec_bytes = std::fs::read(&secondary_path).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, resolved) = startup_pipeline(&resolver);

    assert_eq!(resolved, canon_path);
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(config.candidate_display_limit, 4);
    assert_eq!(outcome, StartupOutcome::CanonicalLoaded(LoadOutcome::LoadedOk));
    assert_eq!(std::fs::read(&primary_path).unwrap(), pri_bytes);
    assert_eq!(std::fs::read(&secondary_path).unwrap(), sec_bytes);
}

// TEST-MIG-21: Primary legacy precedence over secondary
fn test_migration_primary_precedence_over_secondary() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let primary_dir = dir.path().join("com.bornomala.sandbox");
    let secondary_dir = dir.path().join("com.borno.sandbox");
    std::fs::create_dir_all(&primary_dir).unwrap();
    std::fs::create_dir_all(&secondary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let primary_path = primary_dir.join("config.json");
    let secondary_path = secondary_dir.join("config.json");

    let mut pri_conf = BornoConfig::default();
    pri_conf.candidate_display_limit = 7;
    save_config_safe(&primary_path, &pri_conf).unwrap();

    let mut sec_conf = BornoConfig::default();
    sec_conf.candidate_display_limit = 9;
    save_config_safe(&secondary_path, &sec_conf).unwrap();

    let sec_bytes = std::fs::read(&secondary_path).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, resolved) = startup_pipeline(&resolver);

    assert_eq!(resolved, canon_path);
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(config.candidate_display_limit, 7);
    assert!(matches!(outcome, StartupOutcome::Migrated { source_path, .. } if source_path == primary_path));
    assert!(canon_path.exists());
    assert_eq!(std::fs::read(&secondary_path).unwrap(), sec_bytes);
}

// TEST-MIG-22: Secondary legacy migrated when primary absent
fn test_migration_secondary_migrated_when_primary_absent() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let secondary_dir = dir.path().join("com.borno.sandbox");
    std::fs::create_dir_all(&secondary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let secondary_path = secondary_dir.join("config.json");

    let mut sec_conf = BornoConfig::default();
    sec_conf.candidate_display_limit = 8;
    save_config_safe(&secondary_path, &sec_conf).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, resolved) = startup_pipeline(&resolver);

    assert_eq!(resolved, canon_path);
    assert_eq!(status, PersistenceStatus::Active);
    assert_eq!(config.candidate_display_limit, 8);
    assert!(matches!(outcome, StartupOutcome::Migrated { source_path, .. } if source_path == secondary_path));
    assert!(canon_path.exists());
    assert!(secondary_path.exists());
}

// TEST-MIG-23: Resolve legacy sibling paths
fn test_migration_resolve_legacy_sibling_paths() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let derived = derive_sibling_paths(&canon_path).expect("derive");

    assert_eq!(derived.primary_legacy_path, dir.path().join("com.bornomala.sandbox").join("config.json"));
    assert_eq!(derived.secondary_legacy_path, dir.path().join("com.borno.sandbox").join("config.json"));
}

// TEST-MIG-24: derive_sibling_paths rejects invalid canonical directory name
fn test_derive_sibling_paths_rejects_invalid_canonical_directory_name() {
    let dir = TestTempDir::new();
    let wrong_path = dir.path().join("wrong_name").join("config.json");
    assert!(derive_sibling_paths(&wrong_path).is_none());

    let resolver = StaticConfigPathResolver { path: Ok(wrong_path) };
    let (_, status, outcome, _) = startup_pipeline(&resolver);
    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert!(matches!(outcome, StartupOutcome::ParentResolutionFailed(_)));
}

// TEST-MIG-25: Canonical application container junction rejected
fn test_filesystem_canonical_container_junction_rejected() {
    let dir = TestTempDir::new();
    let target_dir = dir.path().join("target");
    std::fs::create_dir_all(&target_dir).unwrap();
    let junction_dir = dir.path().join("com.bornomala.desktop");

    // Create directory junction (works unprivileged on Windows)
    let output = std::process::Command::new("cmd")
        .args(["/c", "mklink", "/J", &junction_dir.to_string_lossy(), &target_dir.to_string_lossy()])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let canon_path = junction_dir.join("config.json");
            std::fs::write(target_dir.join("config.json"), "{}").unwrap();
            match inspect_path(&canon_path) {
                PathStatus::Error(e) => {
                    assert!(e.to_string().contains("rejected") || e.to_string().contains("symbolic link") || e.to_string().contains("Symbolic"));
                }
                other => panic!("Expected PathStatus::Error for canonical container junction, got {:?}", other),
            }
        }
    }
}

// TEST-MIG-26A: Primary legacy application container junction rejected
fn test_filesystem_primary_legacy_container_junction_rejected() {
    let dir = TestTempDir::new();
    let target_dir = dir.path().join("target_primary");
    std::fs::create_dir_all(&target_dir).unwrap();
    let junction_dir = dir.path().join("com.bornomala.sandbox");

    let output = std::process::Command::new("cmd")
        .args(["/c", "mklink", "/J", &junction_dir.to_string_lossy(), &target_dir.to_string_lossy()])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let legacy_path = junction_dir.join("config.json");
            std::fs::write(target_dir.join("config.json"), "{}").unwrap();
            match inspect_path(&legacy_path) {
                PathStatus::Error(e) => {
                    assert!(e.to_string().contains("rejected") || e.to_string().contains("symbolic link") || e.to_string().contains("Symbolic"));
                }
                other => panic!("Expected PathStatus::Error for primary legacy junction, got {:?}", other),
            }
        }
    }
}

// TEST-MIG-26B: Secondary legacy application container junction rejected
fn test_filesystem_secondary_legacy_container_junction_rejected() {
    let dir = TestTempDir::new();
    let target_dir = dir.path().join("target_secondary");
    std::fs::create_dir_all(&target_dir).unwrap();
    let junction_dir = dir.path().join("com.borno.sandbox");

    let output = std::process::Command::new("cmd")
        .args(["/c", "mklink", "/J", &junction_dir.to_string_lossy(), &target_dir.to_string_lossy()])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let legacy_path = junction_dir.join("config.json");
            std::fs::write(target_dir.join("config.json"), "{}").unwrap();
            match inspect_path(&legacy_path) {
                PathStatus::Error(e) => {
                    assert!(e.to_string().contains("rejected") || e.to_string().contains("symbolic link") || e.to_string().contains("Symbolic"));
                }
                other => panic!("Expected PathStatus::Error for secondary legacy junction, got {:?}", other),
            }
        }
    }
}

// TEST-MIG-26C: Canonical config path being a directory instead of a regular file is rejected
fn test_filesystem_canonical_config_path_is_directory_rejected() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    std::fs::create_dir_all(&canon_dir).unwrap();
    let canon_config_dir = canon_dir.join("config.json");
    // Create config.json as a directory
    std::fs::create_dir_all(&canon_config_dir).unwrap();

    match inspect_path(&canon_config_dir) {
        PathStatus::Error(e) => {
            assert!(e.to_string().contains("not a regular file"), "Expected 'not a regular file', got: {}", e);
        }
        other => panic!("Expected PathStatus::Error for config path being a directory, got {:?}", other),
    }
}

// TEST-MIG-26D: Application container being a regular file instead of a directory is rejected
fn test_filesystem_application_container_is_regular_file_rejected() {
    let dir = TestTempDir::new();
    let container_file = dir.path().join("com.bornomala.desktop");
    // Create the container as a regular file
    std::fs::write(&container_file, "regular file content").unwrap();
    let canon_path = container_file.join("config.json");

    match inspect_path(&canon_path) {
        PathStatus::Error(e) => {
            assert!(e.to_string().contains("not a directory"), "Expected 'not a directory', got: {}", e);
        }
        other => panic!("Expected PathStatus::Error for application container being a regular file, got {:?}", other),
    }
}

// TEST-MIG-26E: Distinct symbolic link file privilege enforcement and rejection
fn test_filesystem_symbolic_link_file_distinct_privilege_and_rejection() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    std::fs::create_dir_all(&canon_dir).unwrap();
    let target_file = dir.path().join("target_config.json");
    std::fs::write(&target_file, "{}").unwrap();
    let symlink_path = canon_dir.join("config.json");

    #[cfg(windows)]
    {
        match std::os::windows::fs::symlink_file(&target_file, &symlink_path) {
            Ok(()) => {
                // If Developer Mode or Admin privilege is active:
                match inspect_path(&symlink_path) {
                    PathStatus::Error(e) => {
                        assert!(
                            e.to_string().contains("symbolic link") || e.to_string().contains("rejected"),
                            "Expected symlink rejection, got: {}",
                            e
                        );
                    }
                    other => panic!("Expected PathStatus::Error for file symlink, got {:?}", other),
                }
            }
            Err(e) => {
                // Unprivileged execution without SeCreateSymbolicLinkPrivilege:
                // Windows error 1314 is ERROR_PRIVILEGE_NOT_HELD
                assert!(
                    e.raw_os_error() == Some(1314) || e.kind() == std::io::ErrorKind::PermissionDenied,
                    "Expected unprivileged symlink error (privilege not held), got {:?}",
                    e
                );
            }
        }
    }
}

// TEST-MIG-26F: Target configuration file reparse points (junctions) rejected across all containers
fn test_filesystem_target_config_file_reparse_rejected_all_containers() {
    let dir = TestTempDir::new();
    let target_dir = dir.path().join("target");
    std::fs::create_dir_all(&target_dir).unwrap();

    let containers = [
        "com.bornomala.desktop",
        "com.bornomala.sandbox",
        "com.borno.sandbox",
    ];

    for container_name in containers {
        let container_dir = dir.path().join(container_name);
        std::fs::create_dir_all(&container_dir).unwrap();
        let target_file_junction = container_dir.join("config.json");

        // Create junction at config.json pointing to target_dir (unprivileged on Windows)
        let output = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J", &target_file_junction.to_string_lossy(), &target_dir.to_string_lossy()])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                match inspect_path(&target_file_junction) {
                    PathStatus::Error(e) => {
                        assert!(
                            e.to_string().contains("is a symbolic link") || e.to_string().contains("rejected"),
                            "Container {}: expected symlink rejection for target file junction, got: {}",
                            container_name,
                            e
                        );
                    }
                    other => panic!("Container {}: expected PathStatus::Error for target file junction, got {:?}", container_name, other),
                }
            }
        }
    }
}

// TEST-MIG-27: Corrupt primary legacy falls through to defaults (no secondary migration)
fn test_migration_corrupt_primary_legacy_falls_through_to_defaults() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let primary_dir = dir.path().join("com.bornomala.sandbox");
    let secondary_dir = dir.path().join("com.borno.sandbox");
    std::fs::create_dir_all(&primary_dir).unwrap();
    std::fs::create_dir_all(&secondary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let primary_path = primary_dir.join("config.json");
    let secondary_path = secondary_dir.join("config.json");

    let corrupt = "{ corrupt json !!";
    std::fs::write(&primary_path, corrupt).unwrap();

    let mut sec_conf = BornoConfig::default();
    sec_conf.candidate_display_limit = 8;
    save_config_safe(&secondary_path, &sec_conf).unwrap();
    let sec_bytes = std::fs::read(&secondary_path).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, resolved) = startup_pipeline(&resolver);

    assert_eq!(resolved, canon_path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, StartupOutcome::LegacyCorruptPreserved { .. }));
    assert_eq!(std::fs::read_to_string(&primary_path).unwrap(), corrupt);
    assert_eq!(std::fs::read(&secondary_path).unwrap(), sec_bytes);
    assert!(canon_path.exists());
}

// TEST-MIG-28: Corrupt secondary legacy falls through to defaults
fn test_migration_corrupt_secondary_legacy_falls_through_to_defaults() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let secondary_dir = dir.path().join("com.borno.sandbox");
    std::fs::create_dir_all(&secondary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let secondary_path = secondary_dir.join("config.json");

    let corrupt = "{ corrupt json !!";
    std::fs::write(&secondary_path, corrupt).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, resolved) = startup_pipeline(&resolver);

    assert_eq!(resolved, canon_path);
    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, StartupOutcome::LegacyCorruptPreserved { .. }));
    assert_eq!(std::fs::read_to_string(&secondary_path).unwrap(), corrupt);
    assert!(canon_path.exists());
}

// TEST-MIG-29: Invalid primary missing schema_version preserved
fn test_migration_invalid_primary_missing_schema_preserved() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let primary_dir = dir.path().join("com.bornomala.sandbox");
    let secondary_dir = dir.path().join("com.borno.sandbox");
    std::fs::create_dir_all(&primary_dir).unwrap();
    std::fs::create_dir_all(&secondary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let primary_path = primary_dir.join("config.json");
    let secondary_path = secondary_dir.join("config.json");

    let invalid = r#"{"candidate_display_limit": 3}"#;
    std::fs::write(&primary_path, invalid).unwrap();

    let mut sec_conf = BornoConfig::default();
    sec_conf.candidate_display_limit = 8;
    save_config_safe(&secondary_path, &sec_conf).unwrap();
    let sec_bytes = std::fs::read(&secondary_path).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, _) = startup_pipeline(&resolver);

    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, StartupOutcome::LegacyInvalidPreserved { .. }));
    assert_eq!(std::fs::read_to_string(&primary_path).unwrap(), invalid);
    assert_eq!(std::fs::read(&secondary_path).unwrap(), sec_bytes);
    assert!(canon_path.exists());
}

// TEST-MIG-30: Invalid secondary non-string schema preserved
fn test_migration_invalid_secondary_non_string_schema_preserved() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let secondary_dir = dir.path().join("com.borno.sandbox");
    std::fs::create_dir_all(&secondary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let secondary_path = secondary_dir.join("config.json");

    let invalid = r#"{"schema_version": 99, "candidate_display_limit": 3}"#;
    std::fs::write(&secondary_path, invalid).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, _) = startup_pipeline(&resolver);

    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, StartupOutcome::LegacyInvalidPreserved { .. }));
    assert_eq!(std::fs::read_to_string(&secondary_path).unwrap(), invalid);
    assert!(canon_path.exists());
}

// TEST-MIG-31: Invalid secondary malformed semver preserved
fn test_migration_invalid_secondary_malformed_semver_preserved() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let secondary_dir = dir.path().join("com.borno.sandbox");
    std::fs::create_dir_all(&secondary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let secondary_path = secondary_dir.join("config.json");

    let invalid = r#"{"schema_version": "invalid", "candidate_display_limit": 3}"#;
    std::fs::write(&secondary_path, invalid).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, _) = startup_pipeline(&resolver);

    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, StartupOutcome::LegacyInvalidPreserved { .. }));
    assert_eq!(std::fs::read_to_string(&secondary_path).unwrap(), invalid);
    assert!(canon_path.exists());
}

// TEST-MIG-32: Obsolete primary legacy preserved
fn test_migration_obsolete_primary_legacy_preserved() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let primary_dir = dir.path().join("com.bornomala.sandbox");
    std::fs::create_dir_all(&primary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let primary_path = primary_dir.join("config.json");

    let obsolete = r#"{"schema_version": "0.9.0", "candidate_display_limit": 3}"#;
    std::fs::write(&primary_path, obsolete).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, _) = startup_pipeline(&resolver);

    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, StartupOutcome::LegacyInvalidPreserved { .. }));
    assert_eq!(std::fs::read_to_string(&primary_path).unwrap(), obsolete);
    assert!(canon_path.exists());
}

// TEST-MIG-33: Future primary legacy preserved and locked
fn test_migration_future_primary_legacy_preserved_and_locked() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let primary_dir = dir.path().join("com.bornomala.sandbox");
    std::fs::create_dir_all(&primary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let primary_path = primary_dir.join("config.json");

    let future = r#"{"schema_version": "2.0.0", "super_feature": true}"#;
    std::fs::write(&primary_path, future).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, _) = startup_pipeline(&resolver);

    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::LockedUnsupportedVersion { version: "2.0.0".to_string() });
    assert!(matches!(outcome, StartupOutcome::LegacyUnsupportedPreserved { version, .. } if version == "2.0.0"));
    assert_eq!(std::fs::read_to_string(&primary_path).unwrap(), future);
    assert!(!canon_path.exists());
}

// TEST-MIG-34: Future secondary legacy preserved and locked
fn test_migration_future_secondary_legacy_preserved_and_locked() {
    let dir = TestTempDir::new();
    let canon_dir = dir.path().join("com.bornomala.desktop");
    let secondary_dir = dir.path().join("com.borno.sandbox");
    std::fs::create_dir_all(&secondary_dir).unwrap();

    let canon_path = canon_dir.join("config.json");
    let secondary_path = secondary_dir.join("config.json");

    let future = r#"{"schema_version": "2.0.0", "super_feature": true}"#;
    std::fs::write(&secondary_path, future).unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (config, status, outcome, _) = startup_pipeline(&resolver);

    assert_eq!(config, BornoConfig::default());
    assert_eq!(status, PersistenceStatus::LockedUnsupportedVersion { version: "2.0.0".to_string() });
    assert!(matches!(outcome, StartupOutcome::LegacyUnsupportedPreserved { version, .. } if version == "2.0.0"));
    assert_eq!(std::fs::read_to_string(&secondary_path).unwrap(), future);
    assert!(!canon_path.exists());
}

// TEST-MIG-35: Startup primary legacy source read failure aborts
fn test_startup_primary_legacy_source_read_failure_aborts() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let primary_path = dir.path().join("com.bornomala.sandbox").join("config.json");
    std::fs::create_dir_all(primary_path.parent().unwrap()).unwrap();
    std::fs::write(&primary_path, "{}").unwrap();

    let seams = StartupFaultInjection {
        primary_legacy_source_read_fault: true,
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert!(matches!(outcome, StartupOutcome::LegacySourceReadFailed { .. }));
    assert!(!canon_path.exists());
}

// TEST-MIG-36: Startup secondary legacy source read failure aborts
fn test_startup_secondary_legacy_source_read_failure_aborts() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let secondary_path = dir.path().join("com.borno.sandbox").join("config.json");
    std::fs::create_dir_all(secondary_path.parent().unwrap()).unwrap();
    std::fs::write(&secondary_path, "{}").unwrap();

    let seams = StartupFaultInjection {
        secondary_legacy_source_read_fault: true,
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert!(matches!(outcome, StartupOutcome::LegacySourceReadFailed { .. }));
    assert!(!canon_path.exists());
}

// TEST-MIG-37: Startup canonical inspection error aborts
fn test_startup_canonical_inspection_error_aborts() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");

    let seams = StartupFaultInjection {
        canonical_inspection_fault: true,
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert!(matches!(outcome, StartupOutcome::CanonicalInspectionFailed(_)));
    assert!(!canon_path.exists());
}

// TEST-MIG-38: Canonical read failure after inspection reports readonly
fn test_canonical_read_failure_after_inspection_reports_readonly() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    std::fs::write(&canon_path, "{}").unwrap();

    let seam = CanonicalFaultInjection {
        fail_read: true,
        ..Default::default()
    };
    let (_, status, outcome) = load_config_with_seam(&canon_path, seam);

    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert!(matches!(outcome, LoadOutcome::ReadFailed(_)));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), "{}");
}

// TEST-MIG-39: Startup primary legacy inspection error aborts
fn test_startup_primary_legacy_inspection_error_aborts() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");

    let seams = StartupFaultInjection {
        primary_legacy_inspection_fault: true,
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert!(matches!(outcome, StartupOutcome::LegacyInspectionFailed { .. }));
    assert!(!canon_path.exists());
}

// TEST-MIG-40: Startup secondary legacy inspection error aborts
fn test_startup_secondary_legacy_inspection_error_aborts() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");

    let seams = StartupFaultInjection {
        secondary_legacy_inspection_fault: true,
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert!(matches!(outcome, StartupOutcome::LegacyInspectionFailed { .. }));
    assert!(!canon_path.exists());
}

// TEST-MIG-41: Startup clean install parent dir creation failure
fn test_startup_clean_install_parent_dir_creation_failure() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let seams = StartupFaultInjection {
        clean_install_write_fault: true,
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert!(matches!(status, PersistenceStatus::ReadOnlyMissingWriteFailed(_)));
    assert!(matches!(outcome, StartupOutcome::CleanInstallWriteFailed(_)));
}

// TEST-MIG-42: Startup migration write failure preserves legacy
fn test_startup_migration_write_failure_preserves_legacy() {
    let dir = TestTempDir::new();
    let primary_dir = dir.path().join("com.bornomala.sandbox");
    std::fs::create_dir_all(&primary_dir).unwrap();
    let primary_path = primary_dir.join("config.json");
    let legacy_content = r#"{"schema_version": "1.0.0", "candidate_display_limit": 4}"#;
    std::fs::write(&primary_path, legacy_content).unwrap();

    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let seams = StartupFaultInjection {
        migration_write_fault: true,
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert!(matches!(status, PersistenceStatus::ReadOnlyMissingWriteFailed(_)));
    assert!(matches!(outcome, StartupOutcome::MigrationWriteFailed { .. }));
    assert_eq!(std::fs::read_to_string(&primary_path).unwrap(), legacy_content);
}

// TEST-MIG-43: Temp file cleanup on write failure
fn test_persistence_temp_file_cleanup_on_write_failure() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");

    let mut invalid = BornoConfig::default();
    invalid.candidate_display_limit = 0; // Validation error
    let res = save_config_safe(&path, &invalid);
    assert!(res.is_err());

    let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().filter_map(|e| e.ok()).collect();
    let tmp_files: Vec<_> = entries.iter().filter(|e| e.file_name().to_string_lossy().contains(".tmp.")).collect();
    assert_eq!(tmp_files.len(), 0, "No orphaned temporary files should remain");
}

// TEST-MIG-44: Temp file cleanup on replacement failure
fn test_persistence_temp_file_cleanup_on_rename_failure() {
    let dir = TestTempDir::new();
    let path = dir.path().join("config.json");
    let initial = BornoConfig::default();
    save_config_safe(&path, &initial).unwrap();

    let mut invalid = initial.clone();
    invalid.candidate_display_limit = 99;
    let res = save_config_safe(&path, &invalid);
    assert!(res.is_err());

    let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().filter_map(|e| e.ok()).collect();
    let tmp_files: Vec<_> = entries.iter().filter(|e| e.file_name().to_string_lossy().contains(".tmp.")).collect();
    assert_eq!(tmp_files.len(), 0);
}

// TEST-MIG-45: Startup migration backup creation failure decoupled
fn test_startup_migration_backup_creation_failure_decoupled() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let primary_path = dir.path().join("com.bornomala.sandbox").join("config.json");
    std::fs::create_dir_all(primary_path.parent().unwrap()).unwrap();
    let mut config = BornoConfig::default();
    config.candidate_display_limit = 6;
    save_config_safe(&primary_path, &config).unwrap();

    let seams = StartupFaultInjection {
        migration_backup_fault: Some(BackupFaultInjection {
            fail_creation: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (loaded, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert_eq!(loaded.candidate_display_limit, 6);
    assert_eq!(status, PersistenceStatus::Active);
    assert!(canon_path.exists());
    assert!(primary_path.exists());
    assert!(matches!(outcome, StartupOutcome::Migrated { backup_outcome: BackupOutcome::Failed(BackupFailureDetail::CreationError(_)), .. }));
}

// TEST-MIG-46: Startup migration backup midstream write failure cleaned up
fn test_startup_migration_backup_midstream_write_failure_cleaned_up() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let primary_path = dir.path().join("com.bornomala.sandbox").join("config.json");
    std::fs::create_dir_all(primary_path.parent().unwrap()).unwrap();
    save_config_safe(&primary_path, &BornoConfig::default()).unwrap();

    let seams = StartupFaultInjection {
        migration_backup_fault: Some(BackupFaultInjection {
            fail_write_midstream: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert_eq!(status, PersistenceStatus::Active);
    match outcome {
        StartupOutcome::Migrated { backup_outcome: BackupOutcome::Failed(BackupFailureDetail::WriteFailedCleanedUp { candidate_path, .. }), .. } => {
            assert!(!candidate_path.exists(), "Partial backup file must be removed");
        }
        other => panic!("Expected WriteFailedCleanedUp, got {:?}", other),
    }
}

// TEST-MIG-47: Startup migration backup cleanup failure reports accurately
fn test_startup_migration_backup_cleanup_failure_reports_accurately() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let primary_path = dir.path().join("com.bornomala.sandbox").join("config.json");
    std::fs::create_dir_all(primary_path.parent().unwrap()).unwrap();
    save_config_safe(&primary_path, &BornoConfig::default()).unwrap();

    let seams = StartupFaultInjection {
        migration_backup_fault: Some(BackupFaultInjection {
            fail_write_midstream: true,
            fail_cleanup: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, StartupOutcome::Migrated { backup_outcome: BackupOutcome::Failed(BackupFailureDetail::WriteFailedCleanupFailed { .. }), .. }));
}

// TEST-MIG-48: Backup collision loop preserves preexisting backup files
fn test_backup_collision_loop_preserves_preexisting_backup_files() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let primary_path = dir.path().join("com.bornomala.sandbox").join("config.json");
    std::fs::create_dir_all(primary_path.parent().unwrap()).unwrap();
    save_config_safe(&primary_path, &BornoConfig::default()).unwrap();

    // Pre-create a collision candidate in primary legacy dir
    let collision_file = primary_path.parent().unwrap().join("config.json.bak_migration.0");
    std::fs::write(&collision_file, "original preexisting backup").unwrap();

    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (_, status, outcome, _) = startup_pipeline(&resolver);

    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, StartupOutcome::Migrated { backup_outcome: BackupOutcome::Created(_), .. }));
    assert_eq!(std::fs::read_to_string(&collision_file).unwrap(), "original preexisting backup");
}

// TEST-MIG-49: Startup migration backup collision exhaustion
fn test_startup_migration_backup_collision_exhaustion() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("com.bornomala.desktop").join("config.json");
    let primary_path = dir.path().join("com.bornomala.sandbox").join("config.json");
    std::fs::create_dir_all(primary_path.parent().unwrap()).unwrap();
    save_config_safe(&primary_path, &BornoConfig::default()).unwrap();

    let seams = StartupFaultInjection {
        migration_backup_fault: Some(BackupFaultInjection {
            exhaust_collisions: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    let resolver = StaticConfigPathResolver { path: Ok(canon_path.clone()) };
    let (_, status, outcome, _) = startup_pipeline_with_seams(&resolver, Some(&seams));

    assert_eq!(status, PersistenceStatus::Active);
    assert!(matches!(outcome, StartupOutcome::Migrated { backup_outcome: BackupOutcome::Failed(BackupFailureDetail::CollisionExhaustion { attempts: 1000 }), .. }));
}

// TEST-MIG-50: Canonical corrupt recovery backup failure preserves corrupt file
fn test_canonical_corrupt_recovery_backup_failure_preserves_corrupt_file() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let corrupt = "{ corrupt syntax !!";
    std::fs::write(&canon_path, corrupt).unwrap();

    let seam = CanonicalFaultInjection {
        fail_backup: true,
        ..Default::default()
    };
    let (_, status, outcome) = load_config_with_seam(&canon_path, seam);

    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert!(matches!(outcome, LoadOutcome::CorruptBackupFailed(_)));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), corrupt);
}

// TEST-MIG-51: Canonical invalid recovery backup failure preserves invalid file
fn test_canonical_invalid_recovery_backup_failure_preserves_invalid_file() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let invalid = r#"{"schema_version": "0.9.0"}"#;
    std::fs::write(&canon_path, invalid).unwrap();

    let seam = CanonicalFaultInjection {
        fail_backup: true,
        ..Default::default()
    };
    let (_, status, outcome) = load_config_with_seam(&canon_path, seam);

    assert!(matches!(status, PersistenceStatus::ReadOnlyPermissionDenied(_)));
    assert!(matches!(outcome, LoadOutcome::InvalidBackupFailed(_)));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), invalid);
}

// TEST-MIG-52: Canonical corrupt recovery default write failure reports readonly
fn test_canonical_corrupt_recovery_default_write_failure_reports_readonly() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let corrupt = "{ corrupt syntax !!";
    std::fs::write(&canon_path, corrupt).unwrap();

    let seam = CanonicalFaultInjection {
        fail_default_write: true,
        ..Default::default()
    };
    let (_, status, outcome) = load_config_with_seam(&canon_path, seam);

    assert!(matches!(status, PersistenceStatus::ReadOnlyMissingWriteFailed(_)));
    assert!(matches!(outcome, LoadOutcome::CorruptRecoveryWriteFailed { .. }));
}

// TEST-MIG-53: Canonical invalid recovery default write failure reports readonly
fn test_canonical_invalid_recovery_default_write_failure_reports_readonly() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let invalid = r#"{"schema_version": "0.9.0"}"#;
    std::fs::write(&canon_path, invalid).unwrap();

    let seam = CanonicalFaultInjection {
        fail_default_write: true,
        ..Default::default()
    };
    let (_, status, outcome) = load_config_with_seam(&canon_path, seam);

    assert!(matches!(status, PersistenceStatus::ReadOnlyMissingWriteFailed(_)));
    assert!(matches!(outcome, LoadOutcome::InvalidRecoveryWriteFailed { .. }));
}

// TEST-MIG-54: Reset recovery under unsupported schema success
fn test_reset_recovery_under_unsupported_schema_success() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let unsupported = r#"{"schema_version": "9.9.9", "future": true}"#;
    std::fs::write(&canon_path, unsupported).unwrap();

    let state = create_test_state(canon_path.clone(), PersistenceStatus::LockedUnsupportedVersion { version: "9.9.9".to_string() });
    let res = execute_reset_transaction(&state, None).expect("reset success");

    assert_eq!(res.config, BornoConfig::default());
    assert_eq!(res.status, PersistenceStatus::Active);
    assert!(res.backup_created.is_some());
    let bp = res.backup_created.unwrap();
    assert!(Path::new(&bp).exists());
    assert_eq!(std::fs::read_to_string(&bp).unwrap(), unsupported);
    assert_eq!(*state.persistence_status.lock().unwrap(), PersistenceStatus::Active);
}

// TEST-MIG-55: Reset revalidates disk and detects stale in-memory status
fn test_reset_revalidates_disk_detects_stale_in_memory_status() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    save_config_safe(&canon_path, &BornoConfig::default()).unwrap();

    // In-memory status is stale LockedUnsupportedVersion
    let state = create_test_state(canon_path.clone(), PersistenceStatus::LockedUnsupportedVersion { version: "9.9.9".to_string() });
    let res = execute_reset_transaction(&state, None).expect("reset success");

    assert_eq!(res.status, PersistenceStatus::Active);
    assert!(res.backup_created.is_none(), "No backup created if disk is not actually unsupported");
}

// TEST-MIG-56: Reset recovery backup failure preserves unsupported file
fn test_reset_recovery_backup_failure_preserves_unsupported_file() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let unsupported = r#"{"schema_version": "9.9.9"}"#;
    std::fs::write(&canon_path, unsupported).unwrap();

    let state = create_test_state(canon_path.clone(), PersistenceStatus::LockedUnsupportedVersion { version: "9.9.9".to_string() });
    let faults = ResetFaultInjection {
        fail_backup: true,
        ..Default::default()
    };
    let res = execute_reset_transaction(&state, Some(faults));

    assert!(res.is_err());
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), unsupported);
    assert!(!state.session_degraded.load(std::sync::atomic::Ordering::SeqCst));
}

// TEST-MIG-57: Reset recovery default write failure preserves file and lock
fn test_reset_recovery_default_write_failure_preserves_file_and_lock() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let unsupported = r#"{"schema_version": "9.9.9"}"#;
    std::fs::write(&canon_path, unsupported).unwrap();

    let state = create_test_state(canon_path.clone(), PersistenceStatus::LockedUnsupportedVersion { version: "9.9.9".to_string() });
    let faults = ResetFaultInjection {
        fail_default_write: true,
        ..Default::default()
    };
    let res = execute_reset_transaction(&state, Some(faults));

    assert!(res.is_err());
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), unsupported);
    assert!(!state.session_degraded.load(std::sync::atomic::Ordering::SeqCst));
}

// TEST-MIG-58: Reset recovery rejected under ReadOnlyMissingWriteFailed
fn test_reset_recovery_rejected_under_readonly_missing_write_failed() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let state = create_test_state(canon_path.clone(), PersistenceStatus::ReadOnlyMissingWriteFailed("simulated error".to_string()));
    let res = execute_reset_transaction(&state, None);

    assert!(res.is_err());
    assert!(res.unwrap_err().contains("missing write failed"));
}

// TEST-MIG-59: Reset recovery rejected under ReadOnlyPermissionDenied
fn test_reset_recovery_rejected_under_readonly_permission_denied() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let state = create_test_state(canon_path.clone(), PersistenceStatus::ReadOnlyPermissionDenied("simulated error".to_string()));
    let res = execute_reset_transaction(&state, None);

    assert!(res.is_err());
    assert!(res.unwrap_err().contains("permission denied"));
}

// TEST-MIG-60: Reset transaction disk success engine failure triggers degraded session
fn test_reset_transaction_disk_success_engine_failure_degraded_session() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let unsupported = r#"{"schema_version": "9.9.9"}"#;
    std::fs::write(&canon_path, unsupported).unwrap();

    let state = create_test_state(canon_path.clone(), PersistenceStatus::LockedUnsupportedVersion { version: "9.9.9".to_string() });
    let faults = ResetFaultInjection {
        fail_engine_lock: true,
        ..Default::default()
    };
    let res = execute_reset_transaction(&state, Some(faults));

    assert!(res.is_err());
    assert!(res.unwrap_err().contains("degraded"));
    assert!(state.session_degraded.load(std::sync::atomic::Ordering::SeqCst));

    // Verify subsequent writes blocked
    let save_res = apply_and_persist_config(&state, BornoConfig::default());
    assert!(save_res.is_err());
    assert!(save_res.unwrap_err().contains("degraded"));
}

// TEST-MIG-61: Reset transaction disk success state failure triggers degraded session
fn test_reset_transaction_disk_success_state_failure_degraded_session() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    save_config_safe(&canon_path, &BornoConfig::default()).unwrap();

    let state = create_test_state(canon_path.clone(), PersistenceStatus::Active);
    let faults = ResetFaultInjection {
        fail_effective_config_lock: true,
        ..Default::default()
    };
    let res = execute_reset_transaction(&state, Some(faults));

    assert!(res.is_err());
    assert!(state.session_degraded.load(std::sync::atomic::Ordering::SeqCst));

    let save_res = apply_and_persist_config(&state, BornoConfig::default());
    assert!(save_res.is_err());
    assert!(save_res.unwrap_err().contains("degraded"));
}

// TEST-MIG-62: Reset transaction disk success status failure triggers degraded session
fn test_reset_transaction_disk_success_status_failure_degraded_session() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    save_config_safe(&canon_path, &BornoConfig::default()).unwrap();

    let state = create_test_state(canon_path.clone(), PersistenceStatus::Active);
    let faults = ResetFaultInjection {
        fail_persistence_status_lock: true,
        ..Default::default()
    };
    let res = execute_reset_transaction(&state, Some(faults));

    assert!(res.is_err());
    assert!(state.session_degraded.load(std::sync::atomic::Ordering::SeqCst));

    let save_res = apply_and_persist_config(&state, BornoConfig::default());
    assert!(save_res.is_err());
    assert!(save_res.unwrap_err().contains("degraded"));
}

// TEST-MIG-63: save_config command enforces persistence lock, read-only status, and degraded session
fn test_ipc_save_config_command_enforces_lock() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let content = r#"{"schema_version": "9.9.9"}"#;
    std::fs::write(&canon_path, content).unwrap();

    // 1. LockedUnsupportedVersion rejects save
    let state = create_test_state(canon_path.clone(), PersistenceStatus::LockedUnsupportedVersion { version: "9.9.9".to_string() });
    let res = apply_and_persist_config(&state, BornoConfig::default());
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("unsupported schema version '9.9.9'"));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), content);

    // 2. ReadOnlyMissingWriteFailed rejects save
    let state_missing = create_test_state(canon_path.clone(), PersistenceStatus::ReadOnlyMissingWriteFailed("Disk full".to_string()));
    let res_missing = apply_and_persist_config(&state_missing, BornoConfig::default());
    assert!(res_missing.is_err());
    assert!(res_missing.unwrap_err().contains("read-only (missing write failed"));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), content);

    // 3. ReadOnlyPermissionDenied rejects save
    let state_perm = create_test_state(canon_path.clone(), PersistenceStatus::ReadOnlyPermissionDenied("Access denied".to_string()));
    let res_perm = apply_and_persist_config(&state_perm, BornoConfig::default());
    assert!(res_perm.is_err());
    assert!(res_perm.unwrap_err().contains("read-only (permission denied"));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), content);

    // 4. Session degraded rejects save
    let state_deg = create_test_state(canon_path.clone(), PersistenceStatus::Active);
    state_deg.session_degraded.store(true, std::sync::atomic::Ordering::SeqCst);
    let res_deg = apply_and_persist_config(&state_deg, BornoConfig::default());
    assert!(res_deg.is_err());
    assert!(res_deg.unwrap_err().contains("degraded"));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), content);
}

// TEST-MIG-63B: import_config command enforces persistence lock, read-only status, and degraded session
fn test_ipc_import_config_command_enforces_lock_and_degraded() {
    let dir = TestTempDir::new();
    let canon_path = dir.path().join("config.json");
    let initial_content = r#"{"schema_version": "9.9.9"}"#;
    std::fs::write(&canon_path, initial_content).unwrap();

    let import_json = serde_json::to_string(&BornoConfig::default()).unwrap();

    // 1. LockedUnsupportedVersion rejects import
    let state = create_test_state(canon_path.clone(), PersistenceStatus::LockedUnsupportedVersion { version: "9.9.9".to_string() });
    let res = import_config_internal(&import_json, &state);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("unsupported schema version '9.9.9'"));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), initial_content);

    // 2. ReadOnlyMissingWriteFailed rejects import
    let state_missing = create_test_state(canon_path.clone(), PersistenceStatus::ReadOnlyMissingWriteFailed("Disk full".to_string()));
    let res_missing = import_config_internal(&import_json, &state_missing);
    assert!(res_missing.is_err());
    assert!(res_missing.unwrap_err().contains("read-only (missing write failed"));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), initial_content);

    // 3. ReadOnlyPermissionDenied rejects import
    let state_perm = create_test_state(canon_path.clone(), PersistenceStatus::ReadOnlyPermissionDenied("Access denied".to_string()));
    let res_perm = import_config_internal(&import_json, &state_perm);
    assert!(res_perm.is_err());
    assert!(res_perm.unwrap_err().contains("read-only (permission denied"));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), initial_content);

    // 4. Session degraded rejects import
    let state_degraded = create_test_state(canon_path.clone(), PersistenceStatus::Active);
    state_degraded.session_degraded.store(true, std::sync::atomic::Ordering::SeqCst);
    let res_degraded = import_config_internal(&import_json, &state_degraded);
    assert!(res_degraded.is_err());
    assert!(res_degraded.unwrap_err().contains("degraded"));
    assert_eq!(std::fs::read_to_string(&canon_path).unwrap(), initial_content);

    // 5. Active state successfully imports and persists
    let state_active = create_test_state(canon_path.clone(), PersistenceStatus::Active);
    let mut custom = BornoConfig::default();
    custom.candidate_display_limit = 9;
    let custom_json = serde_json::to_string(&custom).unwrap();
    let res_active = import_config_internal(&custom_json, &state_active);
    assert!(res_active.is_ok());
    assert_eq!(res_active.unwrap().candidate_display_limit, 9);
    let disk_str = std::fs::read_to_string(&canon_path).unwrap();
    let disk_parsed: BornoConfig = serde_json::from_str(&disk_str).unwrap();
    assert_eq!(disk_parsed.candidate_display_limit, 9);
}

// TEST-MIG-64: Startup path resolution failure disables persistence
fn test_startup_path_resolution_failure_disables_persistence() {
    let resolver = StaticConfigPathResolver {
        path: Err("Simulated Tauri path failure".to_string()),
    };
    let (config, status, outcome, resolved_path) = startup_pipeline(&resolver);

    assert_eq!(config, BornoConfig::default());
    assert_eq!(
        status,
        PersistenceStatus::ReadOnlyPermissionDenied("Path resolution failed: Simulated Tauri path failure".to_string())
    );
    assert_eq!(
        outcome,
        StartupOutcome::PathResolutionFailed("Simulated Tauri path failure".to_string())
    );
    assert!(resolved_path.as_os_str().is_empty());

    // Ensure working directory . has no newly created config.json
    assert!(!Path::new("com.bornomala.desktop").exists());
}

// ============================================================================
// Test Runner with Strict Accounting & Non-Zero Exit Code on Failure/Omission
// ============================================================================

struct TestRunner {
    tests: Vec<(&'static str, Box<dyn Fn() + std::panic::RefUnwindSafe>)>,
}

impl TestRunner {
    fn new() -> Self {
        Self { tests: Vec::new() }
    }

    fn register<F>(&mut self, name: &'static str, f: F)
    where
        F: Fn() + std::panic::RefUnwindSafe + 'static,
    {
        self.tests.push((name, Box::new(f)));
    }

    fn run(self) {
        let total_registered = self.tests.len();
        let mut passed = 0;
        let mut failed = 0;
        let skipped = 0;

        println!("[Persistence Suite] Running {} Layer 3 persistence and recovery tests...\n", total_registered);

        for (name, test_fn) in self.tests {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                test_fn();
            }));
            match result {
                Ok(()) => {
                    passed += 1;
                    println!("✓ {} passed", name);
                }
                Err(e) => {
                    failed += 1;
                    let msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic".to_string()
                    };
                    println!("✗ {} FAILED: {}", name, msg);
                }
            }
        }

        let executed = passed + failed;
        println!("\n[Persistence Suite] Execution Summary:");
        println!("Registered Tests: {} (8 Baseline + {} Proposed)", total_registered, total_registered - 8);
        println!("Executed Tests:   {}", executed);
        println!("Passed Tests:     {}", passed);
        println!("Failed Tests:     {}", failed);
        println!("Skipped Tests:    {}", skipped);

        let balanced = passed + failed == executed;
        if balanced {
            println!("Balance Invariant: Passed ({}) + Failed ({}) == Executed ({}) [OK]", passed, failed, executed);
        } else {
            println!("Balance Invariant: Passed ({}) + Failed ({}) != Executed ({}) [FAIL]", passed, failed, executed);
        }

        if failed > 0 || executed < total_registered || !balanced {
            eprintln!("\n[Persistence Suite] FAILED: suite did not complete with 100% success.");
            std::process::exit(1);
        } else {
            println!("\n[Persistence Suite] All {} tests PASSED successfully!", total_registered);
        }
    }
}

fn main() {
    let mut runner = TestRunner::new();

    // 8 Baseline Tests
    runner.register("test_persistence_initializes_default_on_missing_file", test_persistence_initializes_default_on_missing_file);
    runner.register("test_persistence_save_and_reload_roundtrip", test_persistence_save_and_reload_roundtrip);
    runner.register("test_persistence_atomic_replacement_of_existing_file", test_persistence_atomic_replacement_of_existing_file);
    runner.register("test_persistence_replacement_failure_preserves_existing_config", test_persistence_replacement_failure_preserves_existing_config);
    runner.register("test_persistence_corrupted_json_recovers_and_backs_up", test_persistence_corrupted_json_recovers_and_backs_up);
    runner.register("test_persistence_unsupported_schema_version_is_preserved_byte_for_byte", test_persistence_unsupported_schema_version_is_preserved_byte_for_byte);
    runner.register("test_persistence_unsupported_schema_reset_creates_backup_and_restores_defaults", test_persistence_unsupported_schema_reset_creates_backup_and_restores_defaults);
    runner.register("test_persistence_migration_upgrades_to_v1_2_0_and_adds_defaults", test_persistence_migration_upgrades_to_v1_2_0_and_adds_defaults);

    // 64 Migration, Recovery, and Safety Tests (TEST-MIG-01 to TEST-MIG-64)
    runner.register("test_schema_canonical_malformed_json_recovers_and_backs_up", test_schema_canonical_malformed_json_recovers_and_backs_up);
    runner.register("test_schema_canonical_missing_schema_version_recovers_and_backs_up", test_schema_canonical_missing_schema_version_recovers_and_backs_up);
    runner.register("test_schema_canonical_non_string_schema_version_recovers", test_schema_canonical_non_string_schema_version_recovers);
    runner.register("test_schema_canonical_malformed_semver_string_not_future", test_schema_canonical_malformed_semver_string_not_future);
    runner.register("test_schema_canonical_structural_type_mismatch_recovers", test_schema_canonical_structural_type_mismatch_recovers);
    runner.register("test_schema_canonical_semantic_validation_failure_recovers", test_schema_canonical_semantic_validation_failure_recovers);
    runner.register("test_schema_canonical_obsolete_semver_recovers", test_schema_canonical_obsolete_semver_recovers);
    runner.register("test_schema_canonical_intermediate_semver_recovers", test_schema_canonical_intermediate_semver_recovers);
    runner.register("test_schema_canonical_prerelease_semver_recovers", test_schema_canonical_prerelease_semver_recovers);
    runner.register("test_schema_canonical_build_metadata_normalization_preserves_unknown_properties_in_backup", test_schema_canonical_build_metadata_normalization_preserves_unknown_properties_in_backup);
    runner.register("test_schema_canonical_legacy_build_metadata_upgrade_preserves_unknown_properties_in_backup", test_schema_canonical_legacy_build_metadata_upgrade_preserves_unknown_properties_in_backup);
    runner.register("test_schema_canonical_supported_v1_0_0_creates_preupgrade_backup", test_schema_canonical_supported_v1_0_0_creates_preupgrade_backup);
    runner.register("test_schema_canonical_supported_v1_1_0_creates_preupgrade_backup", test_schema_canonical_supported_v1_1_0_creates_preupgrade_backup);
    runner.register("test_canonical_v1_0_0_upgrade_backup_failure_prevents_replacement", test_canonical_v1_0_0_upgrade_backup_failure_prevents_replacement);
    runner.register("test_canonical_v1_1_0_upgrade_backup_failure_prevents_replacement", test_canonical_v1_1_0_upgrade_backup_failure_prevents_replacement);
    runner.register("test_schema_canonical_future_semver_preserves_and_locks", test_schema_canonical_future_semver_preserves_and_locks);
    runner.register("test_schema_canonical_future_prerelease_preserves_and_locks", test_schema_canonical_future_prerelease_preserves_and_locks);
    runner.register("test_schema_canonical_future_prerelease_patch_preserves_and_locks", test_schema_canonical_future_prerelease_patch_preserves_and_locks);
    runner.register("test_schema_canonical_future_semver_with_build_metadata_locked", test_schema_canonical_future_semver_with_build_metadata_locked);
    runner.register("test_migration_canonical_precedence_over_legacy", test_migration_canonical_precedence_over_legacy);
    runner.register("test_migration_primary_precedence_over_secondary", test_migration_primary_precedence_over_secondary);
    runner.register("test_migration_secondary_migrated_when_primary_absent", test_migration_secondary_migrated_when_primary_absent);
    runner.register("test_migration_resolve_legacy_sibling_paths", test_migration_resolve_legacy_sibling_paths);
    runner.register("test_derive_sibling_paths_rejects_invalid_canonical_directory_name", test_derive_sibling_paths_rejects_invalid_canonical_directory_name);
    runner.register("test_filesystem_canonical_container_junction_rejected", test_filesystem_canonical_container_junction_rejected);
    runner.register("test_filesystem_primary_legacy_container_junction_rejected", test_filesystem_primary_legacy_container_junction_rejected);
    runner.register("test_filesystem_secondary_legacy_container_junction_rejected", test_filesystem_secondary_legacy_container_junction_rejected);
    runner.register("test_filesystem_canonical_config_path_is_directory_rejected", test_filesystem_canonical_config_path_is_directory_rejected);
    runner.register("test_filesystem_application_container_is_regular_file_rejected", test_filesystem_application_container_is_regular_file_rejected);
    runner.register("test_filesystem_symbolic_link_file_distinct_privilege_and_rejection", test_filesystem_symbolic_link_file_distinct_privilege_and_rejection);
    runner.register("test_filesystem_target_config_file_reparse_rejected_all_containers", test_filesystem_target_config_file_reparse_rejected_all_containers);
    runner.register("test_migration_corrupt_primary_legacy_falls_through_to_defaults", test_migration_corrupt_primary_legacy_falls_through_to_defaults);
    runner.register("test_migration_corrupt_secondary_legacy_falls_through_to_defaults", test_migration_corrupt_secondary_legacy_falls_through_to_defaults);
    runner.register("test_migration_invalid_primary_missing_schema_preserved", test_migration_invalid_primary_missing_schema_preserved);
    runner.register("test_migration_invalid_secondary_non_string_schema_preserved", test_migration_invalid_secondary_non_string_schema_preserved);
    runner.register("test_migration_invalid_secondary_malformed_semver_preserved", test_migration_invalid_secondary_malformed_semver_preserved);
    runner.register("test_migration_obsolete_primary_legacy_preserved", test_migration_obsolete_primary_legacy_preserved);
    runner.register("test_migration_future_primary_legacy_preserved_and_locked", test_migration_future_primary_legacy_preserved_and_locked);
    runner.register("test_migration_future_secondary_legacy_preserved_and_locked", test_migration_future_secondary_legacy_preserved_and_locked);
    runner.register("test_startup_primary_legacy_source_read_failure_aborts", test_startup_primary_legacy_source_read_failure_aborts);
    runner.register("test_startup_secondary_legacy_source_read_failure_aborts", test_startup_secondary_legacy_source_read_failure_aborts);
    runner.register("test_startup_canonical_inspection_error_aborts", test_startup_canonical_inspection_error_aborts);
    runner.register("test_canonical_read_failure_after_inspection_reports_readonly", test_canonical_read_failure_after_inspection_reports_readonly);
    runner.register("test_startup_primary_legacy_inspection_error_aborts", test_startup_primary_legacy_inspection_error_aborts);
    runner.register("test_startup_secondary_legacy_inspection_error_aborts", test_startup_secondary_legacy_inspection_error_aborts);
    runner.register("test_startup_clean_install_parent_dir_creation_failure", test_startup_clean_install_parent_dir_creation_failure);
    runner.register("test_startup_migration_write_failure_preserves_legacy", test_startup_migration_write_failure_preserves_legacy);
    runner.register("test_persistence_temp_file_cleanup_on_write_failure", test_persistence_temp_file_cleanup_on_write_failure);
    runner.register("test_persistence_temp_file_cleanup_on_rename_failure", test_persistence_temp_file_cleanup_on_rename_failure);
    runner.register("test_startup_migration_backup_creation_failure_decoupled", test_startup_migration_backup_creation_failure_decoupled);
    runner.register("test_startup_migration_backup_midstream_write_failure_cleaned_up", test_startup_migration_backup_midstream_write_failure_cleaned_up);
    runner.register("test_startup_migration_backup_cleanup_failure_reports_accurately", test_startup_migration_backup_cleanup_failure_reports_accurately);
    runner.register("test_backup_collision_loop_preserves_preexisting_backup_files", test_backup_collision_loop_preserves_preexisting_backup_files);
    runner.register("test_startup_migration_backup_collision_exhaustion", test_startup_migration_backup_collision_exhaustion);
    runner.register("test_canonical_corrupt_recovery_backup_failure_preserves_corrupt_file", test_canonical_corrupt_recovery_backup_failure_preserves_corrupt_file);
    runner.register("test_canonical_invalid_recovery_backup_failure_preserves_invalid_file", test_canonical_invalid_recovery_backup_failure_preserves_invalid_file);
    runner.register("test_canonical_corrupt_recovery_default_write_failure_reports_readonly", test_canonical_corrupt_recovery_default_write_failure_reports_readonly);
    runner.register("test_canonical_invalid_recovery_default_write_failure_reports_readonly", test_canonical_invalid_recovery_default_write_failure_reports_readonly);
    runner.register("test_reset_recovery_under_unsupported_schema_success", test_reset_recovery_under_unsupported_schema_success);
    runner.register("test_reset_revalidates_disk_detects_stale_in_memory_status", test_reset_revalidates_disk_detects_stale_in_memory_status);
    runner.register("test_reset_recovery_backup_failure_preserves_unsupported_file", test_reset_recovery_backup_failure_preserves_unsupported_file);
    runner.register("test_reset_recovery_default_write_failure_preserves_file_and_lock", test_reset_recovery_default_write_failure_preserves_file_and_lock);
    runner.register("test_reset_recovery_rejected_under_readonly_missing_write_failed", test_reset_recovery_rejected_under_readonly_missing_write_failed);
    runner.register("test_reset_recovery_rejected_under_readonly_permission_denied", test_reset_recovery_rejected_under_readonly_permission_denied);
    runner.register("test_reset_transaction_disk_success_engine_failure_degraded_session", test_reset_transaction_disk_success_engine_failure_degraded_session);
    runner.register("test_reset_transaction_disk_success_state_failure_degraded_session", test_reset_transaction_disk_success_state_failure_degraded_session);
    runner.register("test_reset_transaction_disk_success_status_failure_degraded_session", test_reset_transaction_disk_success_status_failure_degraded_session);
    runner.register("test_ipc_save_config_command_enforces_lock", test_ipc_save_config_command_enforces_lock);
    runner.register("test_ipc_import_config_command_enforces_lock_and_degraded", test_ipc_import_config_command_enforces_lock_and_degraded);
    runner.register("test_startup_path_resolution_failure_disables_persistence", test_startup_path_resolution_failure_disables_persistence);

    runner.run();
}
