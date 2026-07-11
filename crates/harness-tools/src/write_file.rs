use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;

pub struct WriteFile;

impl WriteFile {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Create or overwrite a file with the given content. Creates parent directories if needed."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "File path to write" },
                "content": { "type": "string", "description": "Content to write to the file" }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let path = match args["path"].as_str() {
            Some(p) => p,
            None => return ToolResult { tool_call_id: String::new(), content: "Missing 'path' parameter".into(), is_error: true },
        };
        let content = match args["content"].as_str() {
            Some(c) => c,
            None => return ToolResult { tool_call_id: String::new(), content: "Missing 'content' parameter".into(), is_error: true },
        };

        if let Some(parent) = std::path::Path::new(path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResult { tool_call_id: String::new(), content: format!("Failed to create directories: {}", e), is_error: true };
            }
        }

        match std::fs::write(path, content) {
            Ok(()) => ToolResult { tool_call_id: String::new(), content: format!("Successfully wrote {} bytes to {}", content.len(), path), is_error: false },
            Err(e) => ToolResult { tool_call_id: String::new(), content: format!("Failed to write {}: {}", path, e), is_error: true },
        }
    }
}