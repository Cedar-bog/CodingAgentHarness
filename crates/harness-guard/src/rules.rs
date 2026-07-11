use crate::GuardrailRule;
use harness_core::ToolCall;

fn get_command(action: &ToolCall) -> Option<String> {
    if action.name != "shell_exec" { return None; }
    serde_json::from_str::<serde_json::Value>(&action.arguments).ok()
        .and_then(|v| v["command"].as_str().map(|s| s.to_string()))
}

fn get_git_op(action: &ToolCall) -> Option<(String, String)> {
    if action.name != "git_op" { return None; }
    serde_json::from_str::<serde_json::Value>(&action.arguments).ok().map(|v| {
        let op = v["operation"].as_str().unwrap_or("").to_string();
        let args = v["args"].as_str().unwrap_or("").to_string();
        (op, args)
    })
}

pub struct DangerousCommandRule;

impl GuardrailRule for DangerousCommandRule {
    fn name(&self) -> &str { "dangerous_command" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        let cmd = get_command(action)?;
        let patterns = ["rm -rf", "rm -r /", "del /s", "format c:", "mkfs", "> /dev/sda"];
        for p in patterns {
            if cmd.contains(p) {
                return Some(format!("Dangerous command detected: '{}'", p));
            }
        }
        None
    }
}

pub struct SudoCommandRule;

impl GuardrailRule for SudoCommandRule {
    fn name(&self) -> &str { "sudo_command" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        let cmd = get_command(action)?;
        if cmd.starts_with("sudo ") || cmd.contains(" chmod 777") || cmd.contains(" chmod -R 777") {
            return Some(format!("System-level command detected"));
        }
        None
    }
}

pub struct GitPushMainRule;

impl GuardrailRule for GitPushMainRule {
    fn name(&self) -> &str { "git_push_main" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        let (op, args) = get_git_op(action)?;
        if op == "push" && (args.contains("main") || args.contains("master")) {
            return Some("Push to main/master branch requires approval".into());
        }
        None
    }
}

pub struct NetworkRequestRule;

impl GuardrailRule for NetworkRequestRule {
    fn name(&self) -> &str { "network_request" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        let cmd = get_command(action)?;
        let patterns = ["curl ", "wget ", "fetch ", "http://", "https://"];
        for p in patterns {
            if cmd.contains(p) {
                return Some(format!("Network request detected"));
            }
        }
        None
    }
}

pub struct CredentialLeakRule;

impl GuardrailRule for CredentialLeakRule {
    fn name(&self) -> &str { "credential_leak" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        let content = match action.name.as_str() {
            "write_file" => {
                serde_json::from_str::<serde_json::Value>(&action.arguments).ok()
                    .and_then(|v| v["content"].as_str().map(|s| s.to_string()))
            }
            "shell_exec" => {
                serde_json::from_str::<serde_json::Value>(&action.arguments).ok()
                    .and_then(|v| v["command"].as_str().map(|s| s.to_string()))
            }
            _ => return None,
        }?;
        let key_patterns = ["sk-", "api_key=", "API_KEY=", "DEEPSEEK_API_KEY="];
        for p in key_patterns {
            if content.contains(p) {
                return Some("Potential credential leak detected".into());
            }
        }
        None
    }
}