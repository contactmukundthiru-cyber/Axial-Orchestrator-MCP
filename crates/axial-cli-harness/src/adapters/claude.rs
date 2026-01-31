use crate::{ToolAdapter, ToolStatus, ToolResult};
use async_trait::async_trait;
use anyhow::Result;
use std::process::Command;

pub struct ClaudeCodeAdapter;

#[async_trait]
impl ToolAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &str { "claude-code" }
    async fn probe(&self) -> ToolStatus {
        let output = Command::new("claude").arg("--version").output();
        ToolStatus {
            name: self.name().to_string(),
            installed: output.is_ok(),
            version: output.ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()),
        }
    }
    async fn run(&self, task: &str, _dry_run: bool) -> Result<ToolResult> {
        let output = Command::new("claude")
            .arg(task)
            .output()?;

        Ok(ToolResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            diff: None,
        })
    }
}
