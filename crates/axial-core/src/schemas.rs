use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct PlanPacket {
    pub id: Uuid,
    pub title: String,
    pub version: String,
    pub graph: TaskGraph,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct TaskGraph {
    pub nodes: Vec<TaskNode>,
    pub edges: Vec<TaskEdge>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct TaskNode {
    pub id: String,
    pub task_type: String,
    pub params: serde_json::Value,
    pub invariants: Vec<Invariant>,
    pub approval_gate: Option<ApprovalGate>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct TaskEdge {
    pub from: String,
    pub to: String,
    pub condition: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Invariant {
    pub id: String,
    pub check_type: String, // e.g., "no-vulnerabilities", "test-pass"
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ApprovalGate {
    pub required_approvers: Vec<String>,
    pub notification_channel: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Artifact {
    pub id: Uuid,
    pub task_id: String,
    pub artifact_type: String,
    pub data: serde_json::Value,
    pub hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct LedgerEntry {
    pub index: u64,
    pub previous_hash: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Profile {
    pub name: String,
    pub constraints: Vec<Constraint>,
    pub preferred_tools: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Constraint {
    pub key: String,
    pub value: serde_json::Value,
}
