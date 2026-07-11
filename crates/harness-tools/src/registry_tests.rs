use crate::{Tool, ToolRegistry, ToolInfo};
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
    let result = tool.execute(serde_json::json!({"path": "crates/harness-core/Cargo.toml"})).await;
    assert!(!result.is_error);
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