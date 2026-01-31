use crate::{ToolAdapter, ToolStatus, ToolResult};
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use std::process::Command;

pub struct CursorAdapter;

#[async_trait]
impl ToolAdapter for CursorAdapter {
    fn name(&self) -> &str { "cursor" }

    async fn probe(&self) -> ToolStatus {
        let output = Command::new("cursor").arg("--version").output();
        match output {
            Ok(out) if out.status.success() => {
                ToolStatus {
                    name: self.name().to_string(),
                    installed: true,
                    version: Some(String::from_utf8_lossy(&out.stdout).trim().to_string()),
                }
            }
            _ => ToolStatus {
                name: self.name().to_string(),
                installed: false,
                version: None,
            }
        }
    }

    async fn run(&self, task: &str, dry_run: bool) -> Result<ToolResult> {
        let mut cmd = Command::new("cursor");
        if dry_run {
            // Mocking dry run for now
            return Ok(ToolResult {
                stdout: "Cursor dry run successful".to_string(),
                stderr: "".to_string(),
                diff: Some("--- a/file\n+++ b/file\n+ change".to_string()),
            });
        }
        
        // Full implementation would wrap cursor's specific automation flags
        cmd.arg("--edit").arg(task);
        let output = cmd.output()?;
        
        Ok(ToolResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            diff: None,
        })
    }
}
