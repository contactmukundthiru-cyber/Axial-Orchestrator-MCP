use tree_sitter::{Parser, Language, Query, QueryCursor};
use anyhow::{Result, anyhow};
use std::path::Path;
use std::fs;

pub struct PerceptionEngine {
    parser: Parser,
}

impl PerceptionEngine {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    pub fn analyze_rust(&mut self, source_code: &str) -> Result<serde_json::Value> {
        let language = tree_sitter_rust::language();
        self.parser.set_language(language)
            .map_err(|_| anyhow!("Failed to load Rust language"))?;

        let tree = self.parser.parse(source_code, None)
            .ok_or_else(|| anyhow!("Failed to parse source code"))?;

        // Query for function definitions and structs
        let query_str = "(function_item name: (identifier) @fn_name)
                        (struct_item name: (type_identifier) @struct_name)";
        let query = Query::new(language, query_str)?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

        let mut symbols = Vec::new();
        for m in matches {
            for capture in m.captures {
                let name = capture.node.utf8_text(source_code.as_bytes())?;
                symbols.push(serde_json::json!({
                    "symbol": name,
                    "kind": if capture.index == 0 { "function" } else { "struct" },
                    "range": {
                        "start": capture.node.start_position().row,
                        "end": capture.node.end_position().row
                    }
                }));
            }
        }

        Ok(serde_json::json!({ "symbols": symbols }))
    }

    pub fn build_dependency_graph(&self, root: &Path) -> Result<serde_json::Value> {
        // v1-max: Scan files for 'mod', 'use', 'import'
        let mut graph = Vec::new();
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(entry.path())?;
                if content.contains("mod ") || content.contains("use ") {
                    graph.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
        Ok(serde_json::json!({ "files": graph }))
    }
}
