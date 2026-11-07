pub mod config_store;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::{Manager, State};
use bornomala_core::{BornoConfig, BornoEngine, EngineEvent, EngineStatus, ProcessResult};
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub engine: Mutex<BornoEngine>,
    pub config_mutex: Mutex<()>,
    pub config_path: PathBuf,
    pub persistence_status: Mutex<config_store::PersistenceStatus>,
    pub effective_config: Mutex<BornoConfig>,
    pub session_degraded: AtomicBool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub config: BornoConfig,
    pub status: config_store::PersistenceStatus,
    pub config_path: String,
    pub backup_created: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeterministicTestResult {
    pub success: bool,
    pub input_sequence: String,
    pub expected_preedit: String,
    pub actual_preedit: String,
    pub candidates: Vec<String>,
    pub message: String,
}

/// Tauri-specific path resolver implementing `config_store::ConfigPathResolver`.
pub struct TauriConfigPathResolver {
    canonical_path: Result<PathBuf, String>,
}

impl TauriConfigPathResolver {
    pub fn from_app<R: tauri::Runtime>(app: &tauri::App<R>) -> Self {
        let canonical_path = app
            .path()
            .app_config_dir()
            .map(|dir| dir.join("config.json"))
            .map_err(|e| e.to_string());
        Self { canonical_path }
    }
}

impl config_store::ConfigPathResolver for TauriConfigPathResolver {
    fn resolve_canonical_path(&self) -> Result<PathBuf, String> {
        self.canonical_path.clone()
    }
}

#[tauri::command]
fn get_engine_status(state: State<'_, AppState>) -> Result<EngineStatus, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.status())
}

#[tauri::command]
fn send_engine_event(event: EngineEvent, state: State<'_, AppState>) -> Result<ProcessResult, String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    let result = engine.process_event(event);
    Ok(result)
}

#[tauri::command]
fn reset_engine(state: State<'_, AppState>) -> Result<ProcessResult, String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    let result = engine.reset();
    Ok(result)
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    let config = state.effective_config.lock().map_err(|e| e.to_string())?.clone();
    let status = state.persistence_status.lock().map_err(|e| e.to_string())?.clone();
    Ok(ConfigResponse {
        config,
        status,
        config_path: state.config_path.to_string_lossy().to_string(),
        backup_created: None,
    })
}

#[tauri::command]
fn get_config_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.config_path.to_string_lossy().to_string())
}

/// Shared internal helper for validated configuration modification and atomic persistence.
pub fn apply_and_persist_config(
    state: &AppState,
    config: BornoConfig,
) -> Result<BornoConfig, String> {
    let _guard = state.config_mutex.lock().map_err(|e| e.to_string())?;

    // Reject if session is already degraded from prior post-commit failure
    if state.session_degraded.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(
            "Cannot modify configuration: session is in a degraded state. Restart the application."
                .to_string(),
        );
    }

    // Reject empty path upfront
    if state.config_path.as_os_str().is_empty() {
        return Err(
            "Cannot modify configuration: configuration path is empty (unresolved).".to_string(),
        );
    }

    // Inspect persistence status: strictly reject non-Active states
    {
        let status = state.persistence_status.lock().map_err(|e| e.to_string())?;
        match *status {
            config_store::PersistenceStatus::LockedUnsupportedVersion { ref version } => {
                return Err(format!(
                    "Configuration saving is locked: unsupported schema version '{version}' is present on disk. Reset to restore defaults."
                ));
            }
            config_store::PersistenceStatus::ReadOnlyMissingWriteFailed(ref err) => {
                return Err(format!(
                    "Cannot modify configuration: persistence is read-only (missing write failed: {err})"
                ));
            }
            config_store::PersistenceStatus::ReadOnlyPermissionDenied(ref err) => {
                return Err(format!(
                    "Cannot modify configuration: persistence is read-only (permission denied: {err})"
                ));
            }
            config_store::PersistenceStatus::Active => {}
        }
    }

    // Step 1: Validate config in memory
    config.validate().map_err(|e| e.to_string())?;

    // Step 2: Execute safe replacement to disk
    config_store::save_config_safe(&state.config_path, &config)?;

    // Step 3: Reconfigure running engine under engine lock
    let mut engine = state.engine.lock().map_err(|_| {
        state.session_degraded.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = state.persistence_status.lock().map(|mut s| {
            *s = config_store::PersistenceStatus::ReadOnlyPermissionDenied(
                "Degraded session: configuration saved to disk, but runtime engine sync failed. Restart required.".to_string(),
            );
        });
        "Settings saved to disk, but active typing engine is unresponsive (mutex poisoned). Restart required.".to_string()
    })?;
    engine.configure(&config);
    drop(engine);

    // Step 4: Update effective_config
    let mut eff = state.effective_config.lock().map_err(|e| {
        state.session_degraded.store(true, std::sync::atomic::Ordering::SeqCst);
        e.to_string()
    })?;
    *eff = config.clone();
    drop(eff);

    // Step 5: Update persistence status
    let mut status = state.persistence_status.lock().map_err(|e| {
        state.session_degraded.store(true, std::sync::atomic::Ordering::SeqCst);
        e.to_string()
    })?;
    *status = config_store::PersistenceStatus::Active;

    Ok(config)
}

#[tauri::command]
fn save_config(config: BornoConfig, state: State<'_, AppState>) -> Result<BornoConfig, String> {
    apply_and_persist_config(&state, config)
}

/// Executes the full reset transaction including disk revalidation and degraded-session recovery.
pub fn execute_reset_transaction(
    state: &AppState,
    faults: Option<config_store::ResetFaultInjection>,
) -> Result<ConfigResponse, String> {
    // 1. Acquire coarse-grained configuration mutex
    let _guard = state.config_mutex.lock().map_err(|e| e.to_string())?;

    // 2. Reject if session is already degraded from prior post-commit failure
    if state.session_degraded.load(std::sync::atomic::Ordering::SeqCst) {
        return Err("Cannot reset configuration: session is in a degraded state following a previous partial-commit failure. Restart the application.".to_string());
    }

    // 3. Reject empty path upfront
    if state.config_path.as_os_str().is_empty() {
        return Err("Cannot reset configuration: configuration path is empty (unresolved).".to_string());
    }

    // 4. Confirm in-memory persistence status permits recovery
    let status_guard = state.persistence_status.lock().map_err(|e| e.to_string())?;
    match *status_guard {
        config_store::PersistenceStatus::ReadOnlyMissingWriteFailed(ref err) => {
            return Err(format!("Cannot reset configuration: persistence is read-only (missing write failed: {err})"));
        }
        config_store::PersistenceStatus::ReadOnlyPermissionDenied(ref err) => {
            return Err(format!("Cannot reset configuration: persistence is read-only (permission denied: {err})"));
        }
        _ => {}
    }
    drop(status_guard);

    // 5. Revalidate disk file state before attempting reset
    let disk_bytes = match std::fs::read(&state.config_path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Vec::new()
        }
        Err(e) => {
            return Err(format!("Cannot reset configuration: failed to inspect current configuration on disk: {e}"));
        }
    };

    let is_unsupported_on_disk = if !disk_bytes.is_empty() {
        if let Ok(content_str) = std::str::from_utf8(&disk_bytes) {
            match config_store::classify_schema_version(content_str) {
                config_store::SchemaClassification::UnsupportedFutureStable(_)
                | config_store::SchemaClassification::UnsupportedFuturePrerelease(_) => true,
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    };

    // 6. Execute disk reset:
    // If unsupported, creates .bak_unsupported.<timestamp> using exact in-memory disk_bytes
    // via exclusive creation with collision retry, followed by atomic default replacement.
    let (default_config, backup_path) = config_store::reset_config_safe_with_faults(
        &state.config_path,
        if is_unsupported_on_disk { Some(&disk_bytes) } else { None },
        faults.map_or(false, |f| f.fail_backup),
        faults.map_or(false, |f| f.fail_default_write),
    )?;

    // --- POINT OF DISK COMMIT ---
    // At this point, canonical file on disk has been replaced with defaults!

    // 7. Reconfigure running engine under engine lock
    let engine_fail = faults.map_or(false, |f| f.fail_engine_lock);
    let engine_res = (|| -> Result<(), String> {
        if engine_fail {
            return Err("Simulated engine lock/poison failure".to_string());
        }
        let mut engine = state.engine.lock().map_err(|_| {
            "Settings reset on disk, but active typing engine is unresponsive (mutex poisoned). Restart required.".to_string()
        })?;
        engine.configure(&default_config);
        Ok(())
    })();

    if let Err(e) = engine_res {
        // Degraded Session State: Disk is committed, runtime engine sync failed!
        state.session_degraded.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = state.persistence_status.lock().map(|mut s| {
            *s = config_store::PersistenceStatus::ReadOnlyPermissionDenied(
                "Degraded session: disk reset to defaults, but runtime engine sync failed. Restart required.".to_string(),
            );
        });
        return Err(format!("Configuration reset to defaults on disk, but active typing engine failed to reconfigure ({e}). The session is now degraded. Restart required."));
    }

    // 8. Update effective in-memory configuration
    let eff_fail = faults.map_or(false, |f| f.fail_effective_config_lock);
    let eff_res = (|| -> Result<(), String> {
        if eff_fail {
            return Err("Simulated effective_config lock/poison failure".to_string());
        }
        let mut eff = state.effective_config.lock().map_err(|_| {
            "Settings reset on disk, but in-memory configuration mutex is poisoned. Restart required.".to_string()
        })?;
        *eff = default_config.clone();
        Ok(())
    })();

    if let Err(e) = eff_res {
        state.session_degraded.store(true, std::sync::atomic::Ordering::SeqCst);
        return Err(format!("Configuration reset to defaults on disk, but in-memory state update failed ({e}). Restart required."));
    }

    // 9. Update persistence status to Active
    let status_fail = faults.map_or(false, |f| f.fail_persistence_status_lock);
    let status_res = (|| -> Result<(), String> {
        if status_fail {
            return Err("Simulated persistence_status lock/poison failure".to_string());
        }
        let mut status = state.persistence_status.lock().map_err(|_| {
            "Settings reset on disk, but persistence status mutex is poisoned. Restart required.".to_string()
        })?;
        *status = config_store::PersistenceStatus::Active;
        Ok(())
    })();

    if let Err(e) = status_res {
        state.session_degraded.store(true, std::sync::atomic::Ordering::SeqCst);
        return Err(format!("Configuration reset to defaults on disk, but persistence status update failed ({e}). Restart required."));
    }

    // 10. Return truthful success response
    Ok(ConfigResponse {
        config: default_config,
        status: config_store::PersistenceStatus::Active,
        config_path: state.config_path.to_string_lossy().to_string(),
        backup_created: backup_path,
    })
}

#[tauri::command]
fn reset_config(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    execute_reset_transaction(&state, None)
}

#[tauri::command]
fn reset_config_to_defaults(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    reset_config(state)
}

#[tauri::command]
fn export_config(path: Option<String>, state: State<'_, AppState>) -> Result<String, String> {
    let config = state.effective_config.lock().map_err(|e| e.to_string())?.clone();
    if let Some(ref p) = path {
        if !p.is_empty() {
            let export_path = std::path::Path::new(p);
            config_store::save_config_safe(export_path, &config)?;
        }
    }
    serde_json::to_string_pretty(&config).map_err(|e| e.to_string())
}

pub fn import_config_internal(
    json_or_path: &str,
    state: &AppState,
) -> Result<BornoConfig, String> {
    let content = if json_or_path.trim().starts_with('{') {
        json_or_path.to_string()
    } else {
        let import_path = std::path::Path::new(json_or_path);
        if !import_path.exists() {
            return Err(format!("Import file not found: {json_or_path}"));
        }
        std::fs::read_to_string(import_path).map_err(|e| e.to_string())?
    };
    let config = BornoConfig::from_json_str(&content).map_err(|e| e.to_string())?;
    apply_and_persist_config(state, config)
}

#[tauri::command]
fn import_config(json_or_path: String, state: State<'_, AppState>) -> Result<BornoConfig, String> {
    import_config_internal(&json_or_path, &state)
}

#[tauri::command]
fn run_deterministic_test(state: State<'_, AppState>) -> Result<DeterministicTestResult, String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.reset();

    // Test sequence: 'k', 'a' -> expected "কা"
    engine.process_event(EngineEvent::InputChar('k'));
    let res = engine.process_event(EngineEvent::InputChar('a'));

    let expected = "কা".to_string();
    let actual = res.state.preedit.clone();
    let success = actual == expected;

    Ok(DeterministicTestResult {
        success,
        input_sequence: "k + a".to_string(),
        expected_preedit: expected,
        actual_preedit: actual,
        candidates: res.state.candidates.clone(),
        message: if success {
            "Deterministic transliteration passed: 'ka' correctly generated Bangla Unicode 'কা' via Borno Core."
                .to_string()
        } else {
            "Deterministic test failed: mismatch in preedit output.".to_string()
        },
    })
}

#[tauri::command]
fn get_layout_definition(layout_id: String) -> Result<bornomala_core::layout::LayoutDefinition, String> {
    let id = match layout_id.as_str() {
        "national" => bornomala_core::layout::FixedLayoutId::National,
        "probhat" => bornomala_core::layout::FixedLayoutId::Probhat,
        "bijoy" => bornomala_core::layout::FixedLayoutId::Bijoy,
        other => return Err(format!("Unknown layout id: {}", other)),
    };
    Ok(bornomala_core::layout::get_layout_by_id(id).clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("Borno Crash/Panic: {}\n", info);
        let _ = std::fs::write("borno_crash.log", msg);
    }));

    println!("[Borno] Initializing Tauri application builder...");
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            println!("[Borno] Setup hook called! Active windows: {}", app.webview_windows().len());

            // Resolve OS application configuration path via startup_pipeline
            let resolver = TauriConfigPathResolver::from_app(app);
            let (config, status, outcome, resolved_path) = config_store::startup_pipeline(&resolver);
            println!(
                "[Borno] Configuration startup pipeline completed: path={}, outcome={:?}, status={:?}",
                resolved_path.display(),
                outcome,
                status
            );

            let engine = BornoEngine::with_config(&config);

            app.manage(AppState {
                engine: Mutex::new(engine),
                config_mutex: Mutex::new(()),
                config_path: resolved_path,
                persistence_status: Mutex::new(status),
                effective_config: Mutex::new(config),
                session_degraded: AtomicBool::new(false),
            });

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                println!("[Borno] Main window shown and focused successfully.");
            } else {
                eprintln!("[Borno] Warning: Main window with label 'main' was not found!");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_engine_status,
            send_engine_event,
            reset_engine,
            run_deterministic_test,
            get_layout_definition,
            get_config,
            get_config_path,
            save_config,
            reset_config,
            reset_config_to_defaults,
            export_config,
            import_config
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        let err_msg = format!("Tauri runtime error: {}\n", err);
        let _ = std::fs::write("borno_crash.log", &err_msg);
        eprintln!("{}", err_msg);
    }
}
