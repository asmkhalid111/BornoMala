pub mod config_store;

use std::path::PathBuf;
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

#[tauri::command]
fn save_config(config: BornoConfig, state: State<'_, AppState>) -> Result<BornoConfig, String> {
    let _guard = state.config_mutex.lock().map_err(|e| e.to_string())?;

    // Check if persistence is locked due to unsupported version
    {
        let status = state.persistence_status.lock().map_err(|e| e.to_string())?;
        if let config_store::PersistenceStatus::LockedUnsupportedVersion { ref version } = *status {
            return Err(format!(
                "Configuration saving is locked: unsupported schema version '{version}' is present on disk. Reset to restore defaults."
            ));
        }
    }

    // Step 1: Validate config in memory
    config.validate().map_err(|e| e.to_string())?;

    // Step 2: Execute safe replacement to disk
    config_store::save_config_safe(&state.config_path, &config)?;

    // Step 3: Reconfigure running engine under engine lock
    let mut engine = state.engine.lock().map_err(|_| {
        "Settings saved to disk, but active engine is unresponsive. Restart required.".to_string()
    })?;
    engine.set_candidate_display_limit(config.candidate_display_limit);
    engine.set_auto_commit_on_mode_switch(config.auto_commit_on_mode_switch);
    engine.set_preferred_fixed_layout(config.preferred_fixed_layout);
    engine.set_enable_dictionary_assistance(config.enable_dictionary_assistance);
    engine.set_enable_word_completion(config.enable_word_completion);
    engine.set_max_dictionary_suggestions(config.max_dictionary_suggestions);

    // Step 4: Update effective_config and persistence_status
    let mut eff = state.effective_config.lock().map_err(|e| e.to_string())?;
    *eff = config.clone();

    let mut status = state.persistence_status.lock().map_err(|e| e.to_string())?;
    *status = config_store::PersistenceStatus::Active;

    Ok(config)
}

#[tauri::command]
fn reset_config(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    let _guard = state.config_mutex.lock().map_err(|e| e.to_string())?;

    let is_unsupported = {
        let status = state.persistence_status.lock().map_err(|e| e.to_string())?;
        matches!(*status, config_store::PersistenceStatus::LockedUnsupportedVersion { .. })
    };

    let (default_config, backup_path) = config_store::reset_config_safe(&state.config_path, is_unsupported)?;

    // Reconfigure running engine under lock
    let mut engine = state.engine.lock().map_err(|_| {
        "Settings reset on disk, but active engine is unresponsive. Restart required.".to_string()
    })?;
    engine.set_candidate_display_limit(default_config.candidate_display_limit);
    engine.set_auto_commit_on_mode_switch(default_config.auto_commit_on_mode_switch);
    engine.set_preferred_fixed_layout(default_config.preferred_fixed_layout);
    engine.set_enable_dictionary_assistance(default_config.enable_dictionary_assistance);
    engine.set_enable_word_completion(default_config.enable_word_completion);
    engine.set_max_dictionary_suggestions(default_config.max_dictionary_suggestions);

    // Update effective_config and persistence_status
    let mut eff = state.effective_config.lock().map_err(|e| e.to_string())?;
    *eff = default_config.clone();

    let mut status = state.persistence_status.lock().map_err(|e| e.to_string())?;
    *status = config_store::PersistenceStatus::Active;

    Ok(ConfigResponse {
        config: default_config,
        status: config_store::PersistenceStatus::Active,
        config_path: state.config_path.to_string_lossy().to_string(),
        backup_created: backup_path,
    })
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

            // Resolve OS application configuration path
            let config_dir = app.path().app_config_dir().unwrap_or_else(|e| {
                eprintln!("[Borno] Warning: Failed to resolve app_config_dir: {e}. Falling back to local directory.");
                PathBuf::from(".")
            });
            let config_path = config_dir.join("config.json");

            let (config, status, outcome) = config_store::load_config(&config_path);
            println!(
                "[Borno] Configuration loaded from {}: outcome={:?}, status={:?}",
                config_path.display(),
                outcome,
                status
            );

            let engine = BornoEngine::with_config(&config);

            app.manage(AppState {
                engine: Mutex::new(engine),
                config_mutex: Mutex::new(()),
                config_path,
                persistence_status: Mutex::new(status),
                effective_config: Mutex::new(config),
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
            reset_config
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        let err_msg = format!("Tauri runtime error: {}\n", err);
        let _ = std::fs::write("borno_crash.log", &err_msg);
        eprintln!("{}", err_msg);
    }
}
