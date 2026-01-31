use crate::{ToolAdapter, ToolStatus, ToolResult};
use async_trait::async_trait;
use anyhow::Result;
use std::process::Command;

pub struct AiderAdapter;

#[async_trait]
impl ToolAdapter for AiderAdapter {
    fn name(&self) -> &str { "aider" }
    async fn probe(&self) -> ToolStatus {
        let output = Command::new("aider").arg("--version").output();
        ToolStatus {
            name: self.name().to_string(),
            installed: output.is_ok(),
            version: output.ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()),
        }
    }
    async fn run(&self, task: &str, _dry_run: bool) -> Result<ToolResult> {
        let output = Command::new("aider")
            .arg("--message")
            .arg(task)
            .arg("--no-auto-commits")
            .output()?;

        Ok(ToolResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            diff: None, // v1-max: in future, parse git diff
        })
    }
}
