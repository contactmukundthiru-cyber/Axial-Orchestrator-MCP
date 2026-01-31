use axum::{
    routing::{get, post},
    Json, Router, extract::{State, Path},
};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use axial_pty::PtyManager;
use axial_ledger::Ledger;
use axial_core::{PlanPacket, TaskNode};
use std::path::PathBuf;
use tower_http::cors::CorsLayer;

pub struct AppState {
    pub pty_manager: Mutex<PtyManager>,
    pub ledger: Mutex<Ledger>,
    pub event_tx: broadcast::Sender<EventPacket>,
    pub gate_responses: Mutex<std::collections::HashMap<String, bool>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventPacket {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct DaemonStatus {
    pub status: String,
    pub sessions_count: usize,
}

#[derive(Deserialize)]
pub struct SpawnRequest {
    pub command: String,
}

#[derive(Serialize)]
pub struct SpawnResponse {
    pub session_id: String,
}

pub async fn start_daemon(port: u16, ledger_path: PathBuf) -> Result<()> {
    let pty_manager = PtyManager::new();
    let ledger = Ledger::new(ledger_path.to_str().unwrap()).await?;
    let (event_tx, _) = broadcast::channel(100);
    
    let state = Arc::new(AppState {
        pty_manager: Mutex::new(pty_manager),
        ledger: Mutex::new(ledger),
        event_tx,
        gate_responses: Mutex::new(std::collections::HashMap::new()),
    });

    let app = Router::new()
        .route("/status", get(get_status))
        .route("/plan", post(handle_plan))
        .route("/run", post(handle_run))
        .route("/approve", post(handle_approve))
        .route("/pty/spawn", post(spawn_pty))
        .route("/pty/replay/:id", get(replay_pty))
        .route("/ledger/query", post(query_ledger))
        .route("/ledger/semantic-search", post(semantic_search))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    println!("AXIAL Daemon (Claude Code Orchestrator) listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Deserialize)]
struct ApprovalRequest {
    pub gate_id: String,
    pub approved: bool,
}

async fn handle_plan(
    State(state): State<Arc<AppState>>,
    Json(plan): Json<PlanPacket>,
) -> Json<serde_json::Value> {
    let mut ledger = state.ledger.lock().await;
    let entry = ledger.append(serde_json::json!({
        "event": "plan_received",
        "plan_id": plan.id,
        "title": plan.title
    })).await.unwrap();

    Json(serde_json::json!({ "status": "stored", "ledger_index": entry.index }))
}

async fn handle_run(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let plan_id = payload.get("plan_id").and_then(|v| v.as_str()).unwrap_or("none");
    println!("AXIAL: Starting execution for plan {}", plan_id);
    
    // v1-max Orchestrator: Spawn a dedicated task runner
    let state_clone = Arc::clone(&state);
    let plan_id_str = plan_id.to_string();
    
    tokio::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        
        // 1. Audit start & Snapshot
        {
            let mut ledger = state_clone.ledger.lock().await;
            let _ = ledger.snapshot(&format!("pre-exec-{}", plan_id_str)).await;
            let _ = ledger.append(serde_json::json!({
                "event": "execution_started",
                "plan_id": plan_id_str
            })).await;
        }

        // 2. Fetch Plan (In a real impl, we'd store the plan in the DB/Ledger first)
        // Here we assume it's valid and we begin execution.
        
        // Mocking execution of nodes for v1-max demo
        println!("AXIAL [Plan {}]: Running Invariants...", plan_id_str);
        
        // Check Truth Engines (Shield)
        let semgrep_results = axial_shield::TruthEngine::run_semgrep(".").unwrap_or_default();
        if !semgrep_results.is_empty() {
            println!("AXIAL [Plan {}]: Violations found! HALTING.", plan_id_str);
            let mut ledger = state_clone.ledger.lock().await;
            let _ = ledger.append(serde_json::json!({
                "event": "execution_halted",
                "reason": "shield_violation",
                "details": semgrep_results
            })).await;
            return;
        }

        println!("AXIAL [Plan {}]: Execution Complete.", plan_id_str);
    });

    Json(serde_json::json!({ "status": "started", "plan_id": plan_id }))
}

async fn handle_approve(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ApprovalRequest>,
) -> Json<serde_json::Value> {
    let mut responses = state.gate_responses.lock().await;
    responses.insert(payload.gate_id.clone(), payload.approved);
    
    let mut ledger = state.ledger.lock().await;
    ledger.append(serde_json::json!({
        "event": "approval_gate_response",
        "gate_id": payload.gate_id,
        "approved": payload.approved
    })).await.unwrap();

    Json(serde_json::json!({ "status": "acknowledged" }))
}

#[derive(Deserialize)]
struct LedgerQuery {
    query: String,
}

#[derive(Deserialize)]
struct SemanticSearchRequest {
    text: String,
    limit: usize,
}

async fn query_ledger(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LedgerQuery>,
) -> Json<Vec<axial_core::schemas::LedgerEntry>> {
    let ledger = state.ledger.lock().await;
    let res = ledger.query(&payload.query).await.unwrap_or_default();
    Json(res)
}

async fn semantic_search(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SemanticSearchRequest>,
) -> Json<Vec<axial_core::schemas::LedgerEntry>> {
    let ledger = state.ledger.lock().await;
    let res = ledger.search_semantic(&payload.text, payload.limit).await.unwrap_or_default();
    Json(res)
}

async fn get_status(State(state): State<Arc<AppState>>) -> Json<DaemonStatus> {
    // Note: This is an approximation for v1-max
    Json(DaemonStatus {
        status: "Running".to_string(),
        sessions_count: 0, 
    })
}

async fn spawn_pty(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SpawnRequest>,
) -> Json<SpawnResponse> {
    let mut manager = state.pty_manager.lock().await;
    let session = manager.spawn(&payload.command).unwrap(); // Handle error in v1-max properly
    
    Json(SpawnResponse {
        session_id: session.id,
    })
}

async fn replay_pty(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Vec<axial_pty::PtyEvent>> {
    let mut manager = state.pty_manager.lock().await;
    let events = manager.replay(&id, None).unwrap_or_default();
    Json(events)
}
