use harness_core::{ToolCall, ToolResult};
use harness_guard::Guardrail;
use harness_feedback::FeedbackValidator;
use harness_tools::ToolRegistry;
use harness_tools::plugin::{Plugin, PluginLoader, PluginContext};
use async_trait::async_trait;
use std::sync::Arc;

pub fn demo_guardrail_blocks() {
    let guardrail = Guardrail::new_default();
    let action = ToolCall {
        id: "demo-1".into(),
        name: "shell_exec".into(),
        arguments: r#"{"command": "rm -rf /"}"#.into(),
    };

    match guardrail.check(&action) {
        harness_guard::GuardrailAction::Block { reason } => {
            println!("[DEMO 1] Guardrail BLOCKED: {}", reason);
            println!("[DEMO 1] Result: Action was intercepted as expected");
        }
        harness_guard::GuardrailAction::Allow => {
            println!("[DEMO 1] FAIL: Action was allowed (should have been blocked)");
        }
    }
}

pub fn demo_feedback_loop() {
    let validator = FeedbackValidator::new_default();
    let result = ToolResult {
        tool_call_id: "demo-2".into(),
        content: "test result: 3 failed, 2 passed".into(),
        is_error: true,
    };

    let feedback = validator.validate(&result);
    println!(
        "[DEMO 2] Feedback: is_success={}, category={:?}, summary={}",
        feedback.is_success, feedback.category, feedback.summary
    );
    println!("[DEMO 2] Result: Agent received feedback and can retry");
}

pub fn demo_plugin_extension() {
    struct DemoPlugin;

    #[async_trait]
    impl harness_tools::Tool for DemoPlugin {
        fn name(&self) -> &str {
            "demo_plugin"
        }
        fn description(&self) -> &str {
            "Demo plugin tool"
        }
        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type":"object","properties":{}})
        }
        async fn execute(&self, _args: serde_json::Value) -> ToolResult {
            ToolResult {
                tool_call_id: String::new(),
                content: "Plugin executed successfully!".into(),
                is_error: false,
            }
        }
    }

    #[async_trait]
    impl Plugin for DemoPlugin {
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn dependencies(&self) -> Vec<&str> {
            vec![]
        }
        fn init(&mut self, _ctx: &PluginContext) -> harness_core::Result<()> {
            Ok(())
        }
    }

    let mut loader = PluginLoader::new();
    loader.register(Arc::new(DemoPlugin));
    let mut registry = ToolRegistry::new();
    loader.load_all(&mut registry).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(registry.execute("demo_plugin", &serde_json::json!({}))).unwrap();
    println!("[DEMO 3] Plugin: {}", result.content);
    println!("[DEMO 3] Result: Custom tool registered and executed at runtime");
}