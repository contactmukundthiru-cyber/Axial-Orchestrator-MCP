pub mod adapters;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub stdout: String,
    pub stderr: String,
    pub diff: Option<String>,
}

#[async_trait]
pub trait ToolAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn probe(&self) -> ToolStatus;
    async fn run(&self, task: &str, dry_run: bool) -> Result<ToolResult>;
}

pub struct Harness {
    adapters: Vec<Box<dyn ToolAdapter>>,
}

impl Harness {
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    pub fn add_adapter(&mut self, adapter: Box<dyn ToolAdapter>) {
        self.adapters.push(adapter);
    }

    pub async fn probe_all(&self) -> Vec<ToolStatus> {
        let mut results = Vec::new();
        for adapter in &self.adapters {
            results.push(adapter.probe().await);
        }
        results
    }
}
