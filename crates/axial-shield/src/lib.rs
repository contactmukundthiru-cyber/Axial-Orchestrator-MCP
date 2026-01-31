pub mod proxy;
pub mod truth;

pub use proxy::ShieldProxy;
pub use truth::TruthEngine;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShieldConfig {
    pub allowed_domains: HashSet<String>,
    pub pii_patterns: Vec<String>,
    pub redacted_placeholder: String,
}

pub struct Shield {
    config: ShieldConfig,
    patterns: Vec<Regex>,
    kill_switch: std::sync::atomic::AtomicBool,
}

impl Shield {
    pub fn new(config: ShieldConfig) -> Result<Self> {
        let patterns = config.pii_patterns.iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(Self { 
            config, 
            patterns,
            kill_switch: std::sync::atomic::AtomicBool::new(false),
        })
    }

    pub fn redact(&self, input: &str) -> String {
        if self.kill_switch.load(std::sync::atomic::Ordering::Relaxed) {
            return "[SHIELD KILL SWITCH ACTIVE]".to_string();
        }

        let mut output = input.to_string();
        for pattern in &self.patterns {
            output = pattern.replace_all(&output, &self.config.redacted_placeholder).to_string();
        }
        
        // v1-max: Local Model Pass Hook
        // output = self.local_model_redact(&output);
        
        output
    }

    pub fn trigger_kill_switch(&self) {
        self.kill_switch.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn validate_request(&self, domain: &str) -> Result<()> {
        if self.kill_switch.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(anyhow!("Shield Kill Switch Active: Request Blocked"));
        }

        if self.config.allowed_domains.contains(domain) {
            Ok(())
        } else {
            Err(anyhow!("Security Violation: Domain {} is not in the allowlist", domain))
        }
    }

    pub fn validate_file_export(&self, path: &std::path::Path) -> Result<()> {
        // v1-max: Ensure file export is within allowed workspace boundaries
        let workspace_root = std::env::current_dir()?;
        let canonical_path = if path.exists() {
            path.canonicalize()?
        } else {
            path.to_path_buf()
        };

        if canonical_path.starts_with(&workspace_root) {
            Ok(())
        } else {
            Err(anyhow!("Security Violation: File export to {:?} is outside allowed boundaries", path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn mock_config() -> ShieldConfig {
        let mut allowed_domains = HashSet::new();
        allowed_domains.insert("localhost".to_string());
        allowed_domains.insert("api.openai.com".to_string());

        ShieldConfig {
            allowed_domains,
            pii_patterns: vec![
                r"\b\d{4}-\d{4}-\d{4}-\d{4}\b".to_string(), // Fake CC
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(), // Email
            ],
            redacted_placeholder: "[REDACTED]".to_string(),
        }
    }

    #[test]
    fn test_shield_redact() {
        let shield = Shield::new(mock_config()).unwrap();
        let input = "Contact me at test@example.com or use card 1234-5678-1234-5678";
        let output = shield.redact(input);
        assert!(!output.contains("test@example.com"));
        assert!(!output.contains("1234-5678-1234-5678"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_shield_domain_validation() {
        let shield = Shield::new(mock_config()).unwrap();
        assert!(shield.validate_request("localhost").is_ok());
        assert!(shield.validate_request("malicious.com").is_err());
    }

    #[test]
    fn test_shield_kill_switch() {
        let shield = Shield::new(mock_config()).unwrap();
        shield.trigger_kill_switch();
        assert!(shield.validate_request("localhost").is_err());
        assert_eq!(shield.redact("some text"), "[SHIELD KILL SWITCH ACTIVE]");
    }
}

pub struct ShieldInterceptor {
    shield: std::sync::Arc<Shield>,
}

impl ShieldInterceptor {
    pub fn new(shield: std::sync::Arc<Shield>) -> Self {
        Self { shield }
    }

    pub async fn intercept_and_scrub(&self, content: &str, target_domain: &str) -> Result<String> {
        self.shield.validate_request(target_domain)?;
        Ok(self.shield.redact(content))
    }
}
