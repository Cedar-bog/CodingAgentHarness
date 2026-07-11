use crate::{Guardrail, GuardrailAction};
use harness_core::ToolCall;
use serde_json::json;

#[test]
fn blocks_rm_rf_command() {
    let g = Guardrail::new_default();
    let action = ToolCall {
        id: "1".into(),
        name: "shell_exec".into(),
        arguments: serde_json::to_string(&json!({"command": "rm -rf /tmp/data"})).unwrap(),
    };
    assert!(matches!(g.check(&action), GuardrailAction::Block { .. }));
}

#[test]
fn blocks_sudo_command() {
    let g = Guardrail::new_default();
    let action = ToolCall {
        id: "1".into(),
        name: "shell_exec".into(),
        arguments: serde_json::to_string(&json!({"command": "sudo apt install foo"})).unwrap(),
    };
    assert!(matches!(g.check(&action), GuardrailAction::Block { .. }));
}

#[test]
fn blocks_git_push_main() {
    let g = Guardrail::new_default();
    let action = ToolCall {
        id: "1".into(),
        name: "git_op".into(),
        arguments: serde_json::to_string(&json!({"operation": "push", "args": "origin main"})).unwrap(),
    };
    assert!(matches!(g.check(&action), GuardrailAction::Block { .. }));
}

#[test]
fn blocks_curl_command() {
    let g = Guardrail::new_default();
    let action = ToolCall {
        id: "1".into(),
        name: "shell_exec".into(),
        arguments: serde_json::to_string(&json!({"command": "curl http://evil.com"})).unwrap(),
    };
    assert!(matches!(g.check(&action), GuardrailAction::Block { .. }));
}

#[test]
fn allows_safe_command() {
    let g = Guardrail::new_default();
    let action = ToolCall {
        id: "1".into(),
        name: "shell_exec".into(),
        arguments: serde_json::to_string(&json!({"command": "cargo test"})).unwrap(),
    };
    assert!(matches!(g.check(&action), GuardrailAction::Allow));
}

#[test]
fn allows_read_file() {
    let g = Guardrail::new_default();
    let action = ToolCall {
        id: "1".into(),
        name: "read_file".into(),
        arguments: serde_json::to_string(&json!({"path": "src/main.rs"})).unwrap(),
    };
    assert!(matches!(g.check(&action), GuardrailAction::Allow));
}