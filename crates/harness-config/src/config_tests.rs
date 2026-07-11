use crate::HarnessConfig;

#[test]
fn config_deserializes_from_toml() {
    let toml = r#"
[agent]
max_turns = 30
"#;
    let _config: HarnessConfig = toml::from_str(toml).unwrap();
}