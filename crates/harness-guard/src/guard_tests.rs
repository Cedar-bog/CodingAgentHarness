use harness_core::ToolCall;
use serde_json::json;

#[test]
fn blocks_rm_rf_command() {
    let action = ToolCall {
        id: "1".into(),
        name: "shell_exec".into(),
        arguments: serde_json::to_string(&json!({"command": "rm -rf /tmp/data"})).unwrap(),
    };
    let rule = GuardrailRule::DangerousCommand;
    let result = rule.check(&action);
    assert!(result.is_some());
}

#[test]
fn allows_safe_command() {
    let action = ToolCall {
        id: "1".into(),
        name: "shell_exec".into(),
        arguments: serde_json::to_string(&json!({"command": "cargo test"})).unwrap(),
    };
    let guardrail = Guardrail::new(vec![]);
    let results = guardrail.check(&action);
    assert!(results.is_empty());
}