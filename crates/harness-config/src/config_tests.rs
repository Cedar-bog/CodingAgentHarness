use crate::HarnessConfig;

#[test]
fn config_deserializes_from_toml() {
    let toml = r#"
[agent]
max_turns = 30
max_tools_per_turn = 3

[llm]
provider = "deepseek"
model = "deepseek-chat"
temperature = 0.1
max_tokens = 2048

[guardrails]
mode = "strict"
timeout_seconds = 30

[memory]
max_context_tokens = 1000

[tools]
allowed = ["read_file", "write_file"]
"#;
    let config: HarnessConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.agent.max_turns, 30);
    assert_eq!(config.agent.max_tools_per_turn, 3);
    assert_eq!(config.llm.provider, "deepseek");
    assert_eq!(config.llm.model, "deepseek-chat");
    assert_eq!(config.guardrails.mode, "strict");
    assert_eq!(config.memory.max_context_tokens, 1000);
    assert_eq!(config.tools.allowed, vec!["read_file", "write_file"]);
}

#[test]
fn config_uses_defaults_for_missing_fields() {
    let toml = r#"
[llm]
provider = "deepseek"
"#;
    let config: HarnessConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.agent.max_turns, 50);
    assert_eq!(config.tools.allowed.len(), 5);
}