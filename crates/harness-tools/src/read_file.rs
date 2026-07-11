use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;

pub struct ReadFile;

impl ReadFile {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read file contents. Returns the content of a file at the given path."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "File path to read" },
                "offset": { "type": "integer", "description": "Line number to start from (1-indexed)", "default": 1 },
                "limit": { "type": "integer", "description": "Max lines to read", "default": 2000 }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let path = match args["path"].as_str() {
            Some(p) => p,
            None => return ToolResult { tool_call_id: String::new(), content: "Missing 'path' parameter".into(), is_error: true },
        };
        let offset = args["offset"].as_u64().unwrap_or(1).max(1) as usize;
        let limit = args["limit"].as_u64().unwrap_or(2000) as usize;

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let start = (offset - 1).min(lines.len());
                let end = (start + limit).min(lines.len());
                let output: String = lines[start..end]
                    .iter()
                    .enumerate()
                    .map(|(i, l)| format!("{}: {}", start + i + 1, l))
                    .collect::<Vec<_>>()
                    .join("\n");
                ToolResult { tool_call_id: String::new(), content: output, is_error: false }
            }
            Err(e) => ToolResult { tool_call_id: String::new(), content: format!("Error reading {}: {}", path, e), is_error: true },
        }
    }
}