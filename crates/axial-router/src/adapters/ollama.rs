use crate::{Provider, ProviderInfo, Capability};
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::json;

pub struct OllamaProvider {
    pub model: String,
    pub base_url: String,
}

#[async_trait]
impl Provider for OllamaProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: format!("ollama-{}", self.model),
            name: format!("Ollama ({})", self.model),
            capabilities: vec![
                Capability { name: "text-generation".to_string(), score: 70, cost_per_1k_tokens: 0.0 },
                Capability { name: "code-editing".to_string(), score: 60, cost_per_1k_tokens: 0.0 },
                Capability { name: "local-privacy".to_string(), score: 100, cost_per_1k_tokens: 0.0 },
            ],
            latency_ms: 100,
            privacy_level: crate::PrivacyLevel::Local,
        }
    }

    async fn execute(&self, task: &str, _params: serde_json::Value) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/api/generate", self.base_url))
            .json(&json!({
                "model": self.model,
                "prompt": task,
                "stream": false
            }))
            .send().await?;

        if res.status().is_success() {
            let body: serde_json::Value = res.json().await?;
            Ok(body)
        } else {
            Err(anyhow!("Ollama failed with status: {}", res.status()))
        }
    }
}
