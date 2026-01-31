pub mod adapters;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use thiserror::Error;
use std::collections::HashMap;
use tracing::{info, warn, debug, instrument};

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("No suitable provider found for requirements: {0:?}")]
    NoProviderFound(Vec<String>),
    #[error("Rate limit exceeded for provider: {0}")]
    RateLimitExceeded(String),
    #[error("Provider execution failed: {0}")]
    ExecutionError(#[from] anyhow::Error),
    #[error("Failed to decompose goal into steps: {0}")]
    DecompositionError(String),
}

use governor::{Quota, RateLimiter, state::DirectStateStore, state::NotKeyed, clock::DefaultClock};
use std::num::NonZeroU32;

use axial_core::{PlanPacket, TaskGraph, TaskNode, TaskEdge};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub score: u8, // 1-100
    pub cost_per_1k_tokens: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<Capability>,
    pub latency_ms: u32,
    pub privacy_level: PrivacyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrivacyLevel {
    Local,
    Shielded,
    Cloud,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn info(&self) -> ProviderInfo;
    async fn execute(&self, task: &str, params: serde_json::Value) -> Result<serde_json::Value>;
}

pub struct RouteDecision {
    pub provider_id: String,
    pub explanation: String,
    pub estimated_cost: f64,
    pub strategy_used: String,
}

pub struct CapabilityGraph {
    pub weights: HashMap<String, f64>,
}

impl Default for CapabilityGraph {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert("code-editing".to_string(), 1.5);
        weights.insert("reasoning".to_string(), 2.0);
        weights.insert("speed".to_string(), 1.0);
        Self { weights }
    }
}

pub struct Router {
    providers: HashMap<String, Box<dyn Provider>>,
    limiters: HashMap<String, RateLimiter<NotKeyed, DirectStateStore, DefaultClock>>,
    graph: CapabilityGraph,
}

impl Router {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            limiters: HashMap::new(),
            graph: CapabilityGraph::default(),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn Provider>) {
        let info = provider.info();
        info!("Adding provider: {} (ID: {})", info.name, info.id);
        // Default rate limit: 10 calls per second
        let quota = Quota::per_second(NonZeroU32::new(10).unwrap());
        let limiter = RateLimiter::direct(quota);
        
        self.limiters.insert(info.id.clone(), limiter);
        self.providers.insert(info.id.clone(), provider);
    }

    #[instrument(skip(self), fields(requirements = ?requirements, strategy = strategy))]
    pub fn route(&self, requirements: Vec<String>, strategy: &str) -> Option<RouteDecision> {
        debug!("Routing request with strategy: {}", strategy);
        let mut candidates: Vec<(&String, &Box<dyn Provider>)> = self.providers.iter().collect();
        
        // Filter by availability/rate limits (simplified)
        candidates.retain(|(id, _)| {
            let ok = self.limiters.get(*id).map_or(false, |l| l.check().is_ok());
            if !ok {
                debug!("Provider {} filtered out due to rate limiting", id);
            }
            ok
        });

        if candidates.is_empty() { 
            warn!("No available candidates for routing");
            return None; 
        }

        // Ranking based on strategy
        candidates.sort_by(|(_, a), (_, b)| {
            let a_score = self.score_provider(a.as_ref(), &requirements, strategy);
            let b_score = self.score_provider(b.as_ref(), &requirements, strategy);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        let (id, provider) = candidates[0];
        let info = provider.info();
        
        info!("Routed to {} using strategy {}", id, strategy);
        Some(RouteDecision {
            provider_id: id.clone(),
            explanation: format!("Selected {} (privacy: {:?}) for requirements {:?} using strategy '{}'", info.name, info.privacy_level, requirements, strategy),
            estimated_cost: 0.0,
            strategy_used: strategy.to_string(),
        })
    }

    #[instrument(skip(self))]
    pub async fn decompose(&self, goal: &str) -> Result<PlanPacket> {
        info!("Decomposing goal: {}", goal);
        // v1-max: This would normally call an LLM to generate the graph.
        // For now, we use a rule-based decomposition or a hardcoded template for demo.
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        if goal.contains("refactor") {
            nodes.push(TaskNode {
                id: "analyze".to_string(),
                task_type: "research".to_string(),
                params: serde_json::json!({ "goal": "Analyze codebase for refactoring targets" }),
                invariants: vec![],
                approval_gate: None,
            });
            nodes.push(TaskNode {
                id: "edit".to_string(),
                task_type: "coding".to_string(),
                params: serde_json::json!({ "goal": "Apply refactoring changes" }),
                invariants: vec![],
                approval_gate: None,
            });
            nodes.push(TaskNode {
                id: "test".to_string(),
                task_type: "verification".to_string(),
                params: serde_json::json!({ "goal": "Verify changes with tests" }),
                invariants: vec![],
                approval_gate: None,
            });
            edges.push(TaskEdge { from: "analyze".to_string(), to: "edit".to_string(), condition: None });
            edges.push(TaskEdge { from: "edit".to_string(), to: "test".to_string(), condition: None });
        } else {
            nodes.push(TaskNode {
                id: "generic-task".to_string(),
                task_type: "nlp".to_string(),
                params: serde_json::json!({ "goal": goal }),
                invariants: vec![],
                approval_gate: None,
            });
        }

        Ok(PlanPacket {
            id: Uuid::new_v4(),
            title: format!("Plan for: {}", goal),
            version: "1.0".to_string(),
            graph: TaskGraph { nodes, edges },
            metadata: HashMap::new(),
        })
    }

    fn score_provider(&self, provider: &dyn Provider, requirements: &[String], strategy: &str) -> f64 {
        let info = provider.info();
        let mut score = 0.0;

        // Weighted capability match
        for req in requirements {
            if let Some(cap) = info.capabilities.iter().find(|c| &c.name == req) {
                let weight = self.graph.weights.get(req).unwrap_or(&1.0);
                score += (cap.score as f64) * weight;
            }
        }

        // Strategy adjustment (normalized to 100-pt range for strategy bias)
        match strategy {
            "privacy_first" => {
                match info.privacy_level {
                    PrivacyLevel::Local => score += 500.0,
                    PrivacyLevel::Shielded => score += 100.0,
                    PrivacyLevel::Cloud => score -= 500.0,
                }
            }
            "performance" => {
                score += (1000.0 - info.latency_ms as f64).max(0.0) / 2.0;
            }
            "cost_efficient" => {
                let avg_cost: f64 = info.capabilities.iter().map(|c| c.cost_per_1k_tokens).sum::<f64>() / info.capabilities.len().max(1) as f64;
                score += (1.0 - avg_cost).max(0.0) * 200.0;
            }
            _ => {}
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct MockProvider {
        id: String,
        privacy: PrivacyLevel,
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn info(&self) -> ProviderInfo {
            ProviderInfo {
                id: self.id.clone(),
                name: self.id.clone(),
                capabilities: vec![
                    Capability { name: "text".to_string(), score: 80, cost_per_1k_tokens: 0.1 }
                ],
                latency_ms: 100,
                privacy_level: self.privacy.clone(),
            }
        }
        async fn execute(&self, _task: &str, _params: serde_json::Value) -> Result<serde_json::Value> {
            Ok(json!({"ok": true}))
        }
    }

    #[test]
    fn test_router_privacy_strategy() {
        let mut router = Router::new();
        router.add_provider(Box::new(MockProvider { id: "local".to_string(), privacy: PrivacyLevel::Local }));
        router.add_provider(Box::new(MockProvider { id: "cloud".to_string(), privacy: PrivacyLevel::Cloud }));

        let decision = router.route(vec!["text".to_string()], "privacy_first").unwrap();
        assert_eq!(decision.provider_id, "local");

        let decision_perf = router.route(vec!["text".to_string()], "performance").unwrap();
        // Since both have same latency in mock, it will pick one (the first one usually)
        assert!(!decision_perf.provider_id.is_empty());
    }
}
