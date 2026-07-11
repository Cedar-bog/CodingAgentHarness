use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;

pub struct GitOp;

impl GitOp {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitOp {
    fn name(&self) -> &str {
        "git_op"
    }

    fn description(&self) -> &str {
        "Execute git operations: status, diff, log, branch."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": { "type": "string", "enum": ["status", "diff", "log", "branch"], "description": "Git operation" },
                "args": { "type": "string", "description": "Additional arguments (optional)" }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let op = match args["operation"].as_str() {
            Some(o) => o,
            None => return ToolResult { tool_call_id: String::new(), content: "Missing 'operation'".into(), is_error: true },
        };
        let extra = args["args"].as_str().unwrap_or("");

        let mut cmd_args = vec![op.to_string()];
        if !extra.is_empty() {
            cmd_args.extend(extra.split_whitespace().map(|s| s.to_string()));
        }

        match std::process::Command::new("git").args(&cmd_args).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let mut content = stdout;
                if !stderr.is_empty() {
                    if !content.is_empty() { content.push('\n'); }
                    content.push_str(&stderr);
                }
                ToolResult { tool_call_id: String::new(), content, is_error: !output.status.success() }
            }
            Err(e) => ToolResult { tool_call_id: String::new(), content: format!("Git error: {}", e), is_error: true },
        }
    }
}