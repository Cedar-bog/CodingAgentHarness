use crate::mock::MockLlmProvider;
use crate::CompletionRequest;
use crate::LlmProvider;
use harness_core::{CompletionResponse, FinishReason, Message, Role, Usage};

#[tokio::test]
async fn mock_returns_preset_responses_in_order() {
    let responses = vec![
        CompletionResponse {
            message: Message {
                role: Role::Assistant,
                content: "Hello".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: FinishReason::Stop,
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        },
        CompletionResponse {
            message: Message {
                role: Role::Assistant,
                content: "World".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: FinishReason::Stop,
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        },
    ];
    let mock = MockLlmProvider::new(responses);

    let req = CompletionRequest {
        messages: vec![Message {
            role: Role::User,
            content: "hi".into(),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: 0.0,
        max_tokens: 100,
    };

    let resp1 = mock.complete(req.clone()).await.unwrap();
    assert_eq!(resp1.message.content.as_str(), "Hello");

    let resp2 = mock.complete(req).await.unwrap();
    assert_eq!(resp2.message.content.as_str(), "World");
}

#[tokio::test]
async fn mock_records_all_requests() {
    let responses = vec![CompletionResponse {
        message: Message {
            role: Role::Assistant,
            content: "ok".into(),
            tool_calls: None,
            tool_call_id: None,
        },
        finish_reason: FinishReason::Stop,
        usage: Usage {
            prompt_tokens: 5,
            completion_tokens: 2,
            total_tokens: 7,
        },
    }];
    let mock = MockLlmProvider::new(responses);

    let req = CompletionRequest {
        messages: vec![Message {
            role: Role::User,
            content: "test".into(),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: 0.0,
        max_tokens: 50,
    };

    mock.complete(req).await.unwrap();
    assert_eq!(mock.call_log().len(), 1);
    assert_eq!(mock.call_log()[0].messages[0].content, "test");
}

#[tokio::test]
async fn mock_returns_error_when_no_responses_left() {
    let mock = MockLlmProvider::new(vec![]);
    let req = CompletionRequest {
        messages: vec![Message {
            role: Role::User,
            content: "hi".into(),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: 0.0,
        max_tokens: 100,
    };
    let result = mock.complete(req).await;
    assert!(result.is_err());
}
