use crate::{Tool, ToolRegistry};
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;

struct DummyTool;

#[async_trait]
impl Tool for DummyTool {
    fn name(&self) -> &str {
        "dummy"
    }

    fn description(&self) -> &str {
        "A dummy tool"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "msg": { "type": "string" }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let msg = args["msg"].as_str().unwrap_or("default");
        ToolResult {
            tool_call_id: "call-1".into(),
            content: format!("echo: {}", msg),
            is_error: false,
        }
    }
}

#[tokio::test]
async fn register_and_execute_tool() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(DummyTool));

    let result = registry
        .execute("dummy", &json!({"msg": "hello"}))
        .await
        .unwrap();
    assert_eq!(result.content, "echo: hello");
    assert!(!result.is_error);
}

#[tokio::test]
async fn list_tools_returns_info() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(DummyTool));

    let tools = registry.list_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "dummy");
    assert_eq!(tools[0].description, "A dummy tool");
}

#[tokio::test]
async fn to_llm_tools_generates_schemas() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(DummyTool));

    let schemas = registry.to_llm_tools();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].name, "dummy");
}

#[tokio::test]
async fn unregister_removes_tool() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(DummyTool));
    assert!(registry.execute("dummy", &json!({})).await.is_ok());

    registry.unregister("dummy");
    assert!(registry.execute("dummy", &json!({})).await.is_err());
}

#[tokio::test]
async fn execute_unknown_tool_returns_error() {
    let registry = ToolRegistry::new();
    assert!(registry.execute("nonexistent", &json!({})).await.is_err());
}

// --- ReadFile tests ---
#[tokio::test]
async fn read_file_tool_reads_content() {
    let tool = crate::read_file::ReadFile::new();
    let result = tool.execute(serde_json::json!({"path": "../harness-core/Cargo.toml"})).await;
    assert!(!result.is_error, "Failed: {:?}", result.content);
    assert!(result.content.contains("[package]"));
}

#[tokio::test]
async fn read_file_tool_returns_error_for_missing() {
    let tool = crate::read_file::ReadFile::new();
    let result = tool.execute(serde_json::json!({"path": "nonexistent.txt"})).await;
    assert!(result.is_error);
}

// --- WriteFile tests ---
#[tokio::test]
async fn write_file_tool_creates_file() {
    let tool = crate::write_file::WriteFile::new();
    let test_path = "test_output_write.txt";
    let result = tool.execute(serde_json::json!({"path": test_path, "content": "hello world"})).await;
    assert!(!result.is_error);
    assert_eq!(std::fs::read_to_string(test_path).unwrap(), "hello world");
    std::fs::remove_file(test_path).ok();
}

#[tokio::test]
async fn write_file_tool_creates_parent_dirs() {
    let tool = crate::write_file::WriteFile::new();
    let test_path = "test_dir_nested/output.txt";
    let result = tool.execute(serde_json::json!({"path": test_path, "content": "nested"})).await;
    assert!(!result.is_error);
    assert_eq!(std::fs::read_to_string(test_path).unwrap(), "nested");
    std::fs::remove_dir_all("test_dir_nested").ok();
}

// --- ShellExec tests ---
#[tokio::test]
async fn shell_exec_runs_command() {
    let tool = crate::shell_exec::ShellExec::new();
    let result = tool.execute(serde_json::json!({"command": "echo hello"})).await;
    assert!(!result.is_error);
    assert!(result.content.contains("hello"));
}

#[tokio::test]
async fn shell_exec_returns_error_for_bad_command() {
    let tool = crate::shell_exec::ShellExec::new();
    let result = tool.execute(serde_json::json!({"command": "exit 1"})).await;
    assert!(result.is_error);
}

// --- GitOp tests ---
#[tokio::test]
async fn git_op_status_works() {
    let tool = crate::git_op::GitOp::new();
    let result = tool.execute(serde_json::json!({"operation": "status"})).await;
    assert!(!result.is_error);
}

// --- Plugin tests ---
use crate::plugin::{Plugin, PluginLoader, PluginContext};
use std::sync::Arc;

struct TestPlugin;

#[async_trait::async_trait]
impl Tool for TestPlugin {
    fn name(&self) -> &str { "test_plugin" }
    fn description(&self) -> &str { "test" }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }
    async fn execute(&self, _args: serde_json::Value) -> harness_core::ToolResult {
        harness_core::ToolResult { tool_call_id: String::new(), content: "plugin works".into(), is_error: false }
    }
}

#[async_trait::async_trait]
impl Plugin for TestPlugin {
    fn version(&self) -> &str { "0.1.0" }
    fn dependencies(&self) -> Vec<&str> { vec![] }
    fn init(&mut self, _ctx: &PluginContext) -> harness_core::Result<()> { Ok(()) }
}

#[tokio::test]
async fn plugin_loader_registers_plugins() {
    let mut loader = PluginLoader::new();
    loader.register(Arc::new(TestPlugin));
    assert_eq!(loader.list().len(), 1);
}

#[tokio::test]
async fn plugin_loader_loads_into_registry() {
    let mut loader = PluginLoader::new();
    loader.register(Arc::new(TestPlugin));
    let mut registry = crate::ToolRegistry::new();
    loader.load_all(&mut registry).unwrap();
    let result = registry.execute("test_plugin", &serde_json::json!({})).await.unwrap();
    assert_eq!(result.content, "plugin works");
}
#[tokio::test]
async fn code_search_finds_pattern() {
    let tool = crate::code_search::CodeSearch::new();
    let result = tool.execute(serde_json::json!({"pattern": "harness-core", "path": "."})).await;
    assert!(!result.is_error);
    assert!(result.content.contains("Cargo.toml"));
}