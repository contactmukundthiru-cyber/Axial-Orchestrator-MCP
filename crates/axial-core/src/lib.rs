pub mod schemas;

pub use schemas::*;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_plan_packet_serialization() {
        let plan = PlanPacket {
            id: Uuid::new_v4(),
            title: "Test Plan".to_string(),
            version: "1.0".to_string(),
            graph: TaskGraph {
                nodes: vec![
                    TaskNode {
                        id: "node1".to_string(),
                        task_type: "nlp".to_string(),
                        params: serde_json::json!({"input": "hello"}),
                        invariants: vec![],
                        approval_gate: None,
                    }
                ],
                edges: vec![],
            },
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&plan).unwrap();
        let deserialized: PlanPacket = serde_json::from_str(&serialized).unwrap();
        assert_eq!(plan.id, deserialized.id);
        assert_eq!(plan.title, deserialized.title);
    }
}
