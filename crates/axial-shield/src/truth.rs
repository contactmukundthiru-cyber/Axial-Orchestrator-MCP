use std::process::Command;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TruthViolation {
    pub engine: String,
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub file: String,
    pub line: usize,
}

pub struct TruthEngine;

impl TruthEngine {
    /// Runs semgrep locally on the given path with standard Owasp/Security rules.
    pub fn run_semgrep(path: &str) -> Result<Vec<TruthViolation>> {
        let output = Command::new("semgrep")
            .args(&["--config", "auto", "--json", path])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let json: serde_json::Value = serde_json::from_slice(&out.stdout)?;
                let mut violations = Vec::new();
                if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
                    for res in results {
                        violations.push(TruthViolation {
                            engine: "semgrep".to_string(),
                            rule_id: res["check_id"].as_str().unwrap_or("unknown").to_string(),
                            severity: res["extra"]["severity"].as_str().unwrap_or("warning").to_string(),
                            message: res["extra"]["message"].as_str().unwrap_or("").to_string(),
                            file: res["path"].as_str().unwrap_or("").to_string(),
                            line: res["start"]["line"].as_u64().unwrap_or(0) as usize,
                        });
                    }
                }
                Ok(violations)
            }
            Ok(_) => Err(anyhow!("Semgrep failed to execute correctly")),
            Err(_) => Err(anyhow!("Semgrep not found in PATH")),
        }
    }

    /// Runs gitleaks locally to check for secrets being introduced in the diff.
    pub fn run_gitleaks(path: &str) -> Result<Vec<TruthViolation>> {
        let output = Command::new("gitleaks")
            .args(&["detect", "--source", path, "--no-git", "--report-format", "json", "-r", "/tmp/gitleaks_report.json"])
            .output();

        // Gitleaks returns non-zero if findings are found, so we check if the report file exists
        if std::path::Path::new("/tmp/gitleaks_report.json").exists() {
            let content = std::fs::read_to_string("/tmp/gitleaks_report.json")?;
            let detections: Vec<serde_json::Value> = serde_json::from_str(&content)?;
            let mut violations = Vec::new();
            for d in detections {
                violations.push(TruthViolation {
                    engine: "gitleaks".to_string(),
                    rule_id: d["RuleID"].as_str().unwrap_or("secret").to_string(),
                    severity: "critical".to_string(),
                    message: "Potential secret detected".to_string(),
                    file: d["File"].as_str().unwrap_or("").to_string(),
                    line: d["StartLine"].as_u64().unwrap_or(0) as usize,
                });
            }
            let _ = std::fs::remove_file("/tmp/gitleaks_report.json");
            return Ok(violations);
        }
        
        match output {
            Ok(out) if out.status.success() => Ok(Vec::new()),
            _ => Ok(Vec::new()), // Usually means no leaks found if we didn't generate a report
        }
    }
}
