// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axial_ledger::Ledger;
use axial_router::{Router, Strategy};
use axial_core::TaskNode;
use std::path::PathBuf;

#[tauri::command]
async fn run_axial_task(task: String) -> Result<String, String> {
    let ledger_path = if cfg!(windows) {
        PathBuf::from(std::env::var("USERPROFILE").unwrap()).join(".axial").join("ledger.db")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap()).join(".axial").join("ledger.db")
    };

    let mut ledger = Ledger::new(ledger_path.to_str().unwrap()).await
        .map_err(|e| e.to_string())?;

    ledger.append(serde_json::json!({
        "event": "ui_task_request",
        "task": task
    })).await.map_err(|e| e.to_string())?;

    let router = Router::new(Strategy::Privacy);
    let node = TaskNode {
        id: "ui-task".to_string(),
        task_type: "nlp".to_string(),
        params: serde_json::json!({ "instruction": task }),
        invariants: vec![],
        approval_gate: None,
    };

    let result = router.route(&node).map_err(|e| e.to_string())?;

    Ok(format!("Routed to {} with score {}", result.adapter_id, result.score))
}

#[tauri::command]
async fn get_ledger_entries() -> Result<Vec<serde_json::Value>, String> {
    let ledger_path = if cfg!(windows) {
        PathBuf::from(std::env::var("USERPROFILE").unwrap()).join(".axial").join("ledger.db")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap()).join(".axial").join("ledger.db")
    };

    let ledger = Ledger::new(ledger_path.to_str().unwrap()).await
        .map_err(|e| e.to_string())?;

    let entries = ledger.query("SELECT payload FROM entries ORDER BY index DESC LIMIT 50")
        .await.map_err(|e| e.to_string())?;

    Ok(entries)
}

#[tauri::command]
async fn get_pty_events(session_id: String) -> Result<Vec<axial_pty::PtyEvent>, String> {
    // v1-max: In a real app, we'd fetch from the daemon or the manager
    // For now, we mock returning events for the current session
    Ok(vec![])
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![run_axial_task, get_ledger_entries, get_pty_events])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
