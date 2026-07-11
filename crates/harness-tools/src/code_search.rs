use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;
use walkdir::WalkDir;

pub struct CodeSearch;

impl CodeSearch {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CodeSearch {
    fn name(&self) -> &str {
        "code_search"
    }

    fn description(&self) -> &str {
        "Search for a pattern in files under a directory."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Search pattern (substring)" },
                "path": { "type": "string", "description": "Directory to search in (default: current dir)" },
                "include": { "type": "string", "description": "File pattern to include (e.g. '*.rs')" }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let pattern = match args["pattern"].as_str() {
            Some(p) => p,
            None => return ToolResult { tool_call_id: String::new(), content: "Missing 'pattern'".into(), is_error: true },
        };
        let search_path = args["path"].as_str().unwrap_or(".");
        let include = args["include"].as_str();

        let mut results = Vec::new();
        for entry in WalkDir::new(search_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Some(inc) = include {
                let name = entry.file_name().to_string_lossy();
                if !name.ends_with(&inc[1..]) { continue; }
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for (i, line) in content.lines().enumerate() {
                    if line.contains(pattern) {
                        results.push(format!("{}:{}: {}", entry.path().display(), i + 1, line));
                    }
                }
            }
            if results.len() >= 50 { break; }
        }

        if results.is_empty() {
            ToolResult { tool_call_id: String::new(), content: format!("No matches for '{}' in {}", pattern, search_path), is_error: false }
        } else {
            ToolResult { tool_call_id: String::new(), content: results.join("\n"), is_error: false }
        }
    }
}