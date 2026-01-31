use crate::{ToolAdapter, ToolStatus, ToolResult};
use async_trait::async_trait;
use anyhow::Result;
use std::process::Command;

pub struct ClineAdapter;

#[async_trait]
impl ToolAdapter for ClineAdapter {
    fn name(&self) -> &str { "cline" }
    async fn probe(&self) -> ToolStatus {
        // Cline is usually a VS Code extension, so we might probe its CLI if it has one
        ToolStatus {
            name: self.name().to_string(),
            installed: false,
            version: None,
        }
    }
    async fn run(&self, task: &str, _dry_run: bool) -> Result<ToolResult> {
        Ok(ToolResult { stdout: "".to_string(), stderr: "".to_string(), diff: None })
    }
}
