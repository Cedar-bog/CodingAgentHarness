use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;

pub struct ShellExec;

impl ShellExec {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ShellExec {
    fn name(&self) -> &str {
        "shell_exec"
    }

    fn description(&self) -> &str {
        "Execute a shell command. Returns stdout and stderr."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command to execute" },
                "cwd": { "type": "string", "description": "Working directory (optional)" },
                "timeout": { "type": "integer", "description": "Timeout in seconds (default 30)" }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let command = match args["command"].as_str() {
            Some(c) => c,
            None => return ToolResult { tool_call_id: String::new(), content: "Missing 'command' parameter".into(), is_error: true },
        };

        let (shell, flag) = if cfg!(target_os = "windows") {
            ("cmd", "/c")
        } else {
            ("sh", "-c")
        };
        let mut cmd = std::process::Command::new(shell);
        cmd.arg(flag).arg(command);
        if let Some(cwd) = args["cwd"].as_str() {
            cmd.current_dir(cwd);
        }
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let mut result = String::new();
                if !stdout.is_empty() {
                    result.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !result.is_empty() { result.push('\n'); }
                    result.push_str(&stderr);
                }
                ToolResult {
                    tool_call_id: String::new(),
                    content: result,
                    is_error: !output.status.success(),
                }
            }
            Err(e) => ToolResult { tool_call_id: String::new(), content: format!("Failed to execute: {}", e), is_error: true },
        }
    }
}