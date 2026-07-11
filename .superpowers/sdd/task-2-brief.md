# Task 2: LLM Abstraction Layer + Mock Provider

**Files:**
- Create: `crates/harness-llm/Cargo.toml`
- Create: `crates/harness-llm/src/lib.rs`
- Create: `crates/harness-llm/src/mock.rs`
- Create: `crates/harness-llm/src/mock_tests.rs`

**Interfaces:**
- Consumes: `Message`, `ToolCall`, `FinishReason`, `Usage`, `ToolSchema` from `harness-core`
- Produces: `LlmProvider` trait, `MockLlmProvider`, `CompletionRequest`, `CompletionResponse`

## Steps

1. Create `crates/harness-llm/Cargo.toml`:

```toml
[package]
name = "harness-llm"
version = "0.1.0"
edition = "2024"

[dependencies]
harness-core = { path = "../harness-core" }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
reqwest = { version = "0.12", features = ["json"] }
tokio = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
```

2. Write failing tests for MockLlmProvider (mock_tests.rs):

```rust
use crate::mock::MockLlmProvider;
use crate::CompletionRequest;
use crate::LlmProvider;
use harness_core::{CompletionResponse, FinishReason, Message, Role, ToolCall, Usage};

#[tokio::test]
async fn mock_returns_preset_responses_in_order() {
    let responses = vec![
        CompletionResponse {
            content: Some("Hello".to_string()),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
            usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
        },
        CompletionResponse {
            content: Some("World".to_string()),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
            usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
        },
    ];
    let mock = MockLlmProvider::new(responses);

    let req = CompletionRequest {
        messages: vec![Message { role: Role::User, content: "hi".into(), tool_calls: vec![], tool_call_id: None }],
        tools: None,
        temperature: 0.0,
        max_tokens: 100,
    };

    let resp1 = mock.complete(req.clone()).await.unwrap();
    assert_eq!(resp1.content.as_deref(), Some("Hello"));

    let resp2 = mock.complete(req).await.unwrap();
    assert_eq!(resp2.content.as_deref(), Some("World"));
}

#[tokio::test]
async fn mock_records_all_requests() {
    let responses = vec![
        CompletionResponse {
            content: Some("ok".into()),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
            usage: Usage { prompt_tokens: 5, completion_tokens: 2, total_tokens: 7 },
        },
    ];
    let mock = MockLlmProvider::new(responses);

    let req = CompletionRequest {
        messages: vec![Message { role: Role::User, content: "test".into(), tool_calls: vec![], tool_call_id: None }],
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
        messages: vec![Message { role: Role::User, content: "hi".into(), tool_calls: vec![], tool_call_id: None }],
        tools: None,
        temperature: 0.0,
        max_tokens: 100,
    };
    let result = mock.complete(req).await;
    assert!(result.is_err());
}
```

3. Run `cargo test -p harness-llm` — expect FAIL (RED)

4. Implement `LlmProvider` trait + `MockLlmProvider`:

**lib.rs:**
```rust
pub mod mock;

#[cfg(test)]
mod mock_tests;

use async_trait::async_trait;
use harness_core::{Message, CompletionResponse, ToolSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub tools: Option<Vec<ToolSchema>>,
    pub temperature: f32,
    pub max_tokens: usize,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> harness_core::Result<CompletionResponse>;
    fn supports_tools(&self) -> bool { true }
    fn max_context_tokens(&self) -> usize { 128000 }
    fn name(&self) -> &str;
}
```

**mock.rs:**
```rust
use crate::CompletionRequest;
use crate::LlmProvider;
use harness_core::CompletionResponse;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct MockLlmProvider {
    responses: Arc<Mutex<VecDeque<CompletionResponse>>>,
    call_log: Arc<Mutex<Vec<CompletionRequest>>>,
}

impl MockLlmProvider {
    pub fn new(responses: Vec<CompletionResponse>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses.into())),
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn call_log(&self) -> Vec<CompletionRequest> {
        self.call_log.lock().unwrap().clone()
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn complete(&self, request: CompletionRequest) -> harness_core::Result<CompletionResponse> {
        self.call_log.lock().unwrap().push(request);
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| harness_core::HarnessError::Llm("No preset responses left".into()))
    }

    fn name(&self) -> &str { "mock" }
}
```

5. Run `cargo test -p harness-llm` — expect 3 tests PASS (GREEN)

6. Commit
