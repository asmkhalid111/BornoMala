//! Layer 3 Application Persistence & Recovery Test Suite
//!
//! Validates atomic safe replacement, recovery from malformed files,
//! backup creation, and unsupported schema version protection.

#[path = "../config_store.rs"]
mod config_store;

use config_store::*;
use bornomala_core::BornoConfig;
use std::path::Path;

fn main() {
    println!("[Persistence Suite] Running Layer 3 persistence and recovery tests...\n");

    test_persistence_initializes_default_on_missing_file();
    println!("✓ test_persistence_initializes_default_on_missing_file passed");

    test_persistence_save_and_reload_roundtrip();
    println!("✓ test_persistence_save_and_reload_roundtrip passed");

    test_persistence_atomic_replacement_of_existing_file();
    println!("✓ test_persistence_atomic_replacement_of_existing_file passed");

    test_persistence_replacement_failure_preserves_existing_config();
    println!("✓ test_persistence_replacement_failure_preserves_existing_config passed");

    test_persistence_corrupted_json_recovers_and_backs_up();
    println!("✓ test_persistence_corrupted_json_recovers_and_backs_up passed");

    test_persistence_unsupported_schema_version_is_preserved_byte_for_byte();
    println!("✓ test_persistence_unsupported_schema_version_is_preserved_byte_for_byte passed");

    test_persistence_unsupported_schema_reset_requires_force_and_backs_up();
    println!("✓ test_persistence_unsupported_schema_reset_requires_force_and_backs_up passed");

    test_persistence_migration_upgrades_to_v1_2_0_and_adds_defaults();
    println!("✓ test_persistence_migration_upgrades_to_v1_2_0_and_adds_defaults passed");

    println!("\n[Persistence Suite] All 8 Layer 3 persistence tests PASSED successfully!");
}

struct TestTempDir {
    path: std::path::PathBuf,
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

fn test_persistence_unsupported_schema_reset_requires_force_and_backs_up() {
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

    // Load legacy config: should upgrade to 1.2.0 in-memory and write back to disk
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

    // Verify disk content is upgraded to 1.2.0 JSON
    let disk_content = std::fs::read_to_string(&path).expect("read upgraded json");
    let disk_config: BornoConfig = serde_json::from_str(&disk_content).expect("parse upgraded disk config");
    assert_eq!(disk_config.schema_version, "1.2.0");
    assert_eq!(disk_config.candidate_display_limit, 4);
    assert!(!disk_config.auto_commit_on_mode_switch);
    assert!(disk_config.enable_dictionary_assistance);
    assert!(disk_config.enable_word_completion);
    assert_eq!(disk_config.max_dictionary_suggestions, 3);
    assert!(!disk_config.auto_convert_period_to_dari);

    // Reloading again should load cleanly as LoadedOk
    let (reloaded, re_status, re_outcome) = load_config(&path);
    assert_eq!(reloaded, loaded);
    assert_eq!(re_status, PersistenceStatus::Active);
    assert_eq!(re_outcome, LoadOutcome::LoadedOk);
}

