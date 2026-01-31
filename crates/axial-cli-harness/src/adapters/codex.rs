use crate::{ToolAdapter, ToolStatus, ToolResult};
use async_trait::async_trait;
use anyhow::Result;

pub struct CodexAdapter;

#[async_trait]
impl ToolAdapter for CodexAdapter {
    fn name(&self) -> &str { "codex" }
    async fn probe(&self) -> ToolStatus {
        ToolStatus { name: self.name().to_string(), installed: false, version: None }
    }
    async fn run(&self, _task: &str, _dry_run: bool) -> Result<ToolResult> {
        Ok(ToolResult { stdout: "".to_string(), stderr: "".to_string(), diff: None })
    }
}
