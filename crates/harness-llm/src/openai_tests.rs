use crate::openai::OpenAiCompatibleProvider;
use crate::{CompletionRequest, LlmProvider};
use harness_core::{FunctionSchema, Message, Role, ToolSchema};

#[test]
fn provider_builds_correct_request_body() {
    let provider = OpenAiCompatibleProvider::new(
        "test-key".into(),
        "https://api.deepseek.com".into(),
        "deepseek-chat".into(),
    );

    let req = CompletionRequest {
        messages: vec![Message {
            role: Role::User,
            content: "hello".into(),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: Some(vec![ToolSchema {
            name: "read_file".into(),
            description: "Read a file".into(),
            function: FunctionSchema {
                name: "read_file".into(),
                description: "Read a file".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    }
                }),
            },
        }]),
        temperature: 0.0,
        max_tokens: 4096,
    };

    let body = provider.build_request_body(&req);
    assert_eq!(body["model"], "deepseek-chat");
    assert_eq!(body["messages"][0]["role"], "user");
    assert_eq!(body["messages"][0]["content"], "hello");
    assert_eq!(body["tools"][0]["function"]["name"], "read_file");
    assert_eq!(body["temperature"], 0.0);
    assert_eq!(body["max_tokens"], 4096);
}

#[test]
fn provider_name_and_metadata() {
    let provider = OpenAiCompatibleProvider::new(
        "key".into(),
        "https://api.deepseek.com".into(),
        "deepseek-chat".into(),
    );
    assert_eq!(provider.name(), "deepseek-chat");
    assert!(provider.supports_tools());
    assert_eq!(provider.max_context_tokens(), 128000);
}
