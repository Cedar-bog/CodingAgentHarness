# Coding Agent Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust coding agent harness with extensible tool dispatch, mock-LLM-driven deterministic tests, governance guardrails, feedback loop, memory, and Docker distribution.

**Architecture:** Cargo workspace with 7 crates: `harness-core` (agent loop), `harness-llm` (LLM abstraction + mock), `harness-tools` (tool trait + 6 built-in tools + plugin system), `harness-memory` (SQLite + FTS5), `harness-guard` (guardrails + HITL), `harness-feedback` (validators), `harness-config` (TOML). Binary entry point in `harness-bin`.

**Tech Stack:** Rust 2024 edition, tokio, reqwest, rusqlite, serde/serde_json, regex, walkdir, keyring, dotenv, async-trait.

## Global Constraints

- Rust edition 2024, MSRV 1.78
- All core mechanisms must have mock-LLM deterministic unit tests (no network, no real LLM)
- API keys never hardcoded, never committed to Git
- TDD: write failing test first, then implement, then commit
- DRY: shared types live in `harness-core`, reused across crates
- YAGNI: no streaming, no hot-reload, no multi-agent orchestration (future extensions)

## File Structure

```
.github/workflows/ci.yml                      # GitHub Actions CI
Cargo.toml                              # workspace root
crates/
  harness-core/src/lib.rs               # public API re-exports
  harness-core/src/types.rs             # Message, Role, ToolCall, ToolResult, Action
  harness-core/src/error.rs             # HarnessError enum
  harness-core/src/agent.rs             # Agent struct + main loop
  harness-core/src/agent_tests.rs       # agent loop tests with MockLlm
  harness-llm/src/lib.rs                # LlmProvider trait + CompletionRequest/Response
  harness-llm/src/mock.rs              # MockLlmProvider
  harness-llm/src/openai.rs            # OpenAiCompatibleProvider (DeepSeek)
  harness-llm/src/mock_tests.rs        # mock provider tests
  harness-tools/src/lib.rs             # Tool trait + ToolRegistry + Plugin trait
  harness-tools/src/plugin.rs          # PluginLoader with dependency resolution
  harness-tools/src/read_file.rs       # ReadFile tool
  harness-tools/src/write_file.rs      # WriteFile tool
  harness-tools/src/shell_exec.rs      # ShellExec tool
  harness-tools/src/git_op.rs          # GitOp tool
  harness-tools/src/code_search.rs     # CodeSearch tool
  harness-tools/src/registry_tests.rs  # registry + plugin tests
  harness-memory/src/lib.rs            # MemoryStore + MemoryEntry
  harness-memory/src/store.rs          # SQLite + FTS5 implementation
  harness-memory/src/memory_tests.rs   # memory store tests
  harness-guard/src/lib.rs             # Guardrail + GuardrailRule trait
  harness-guard/src/rules.rs           # 5 built-in rules
  harness-guard/src/hitl.rs            # HITL state machine
  harness-guard/src/guard_tests.rs     # guardrail tests
  harness-feedback/src/lib.rs          # FeedbackValidator + ValidationRule trait
  harness-feedback/src/validators.rs   # Test/Compile/Lint validators
  harness-feedback/src/feedback_tests.rs
  harness-config/src/lib.rs            # HarnessConfig TOML deserialization
  harness-config/src/config_tests.rs
  harness-bin/src/main.rs              # CLI entry point
```

---

### Task 0: GitHub Actions CI Setup

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: none (infrastructure)
- Produces: CI pipeline that runs `cargo test --workspace` on every push and PR

- [x] **Step 1: Create workflow directory**

```bash
mkdir -p .github/workflows
```

- [x] **Step 2: Create CI workflow**

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: ['**']
  pull_request:
    branches: ['**']

jobs:
  unit-test:
    name: unit-test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test --workspace
```

- [x] **Step 3: Verify workflow syntax**

```bash
cat .github/workflows/ci.yml
```

Expected: valid YAML with `unit-test` job

- [x] **Step 4: Commit and push**

```bash
git checkout -b feat/ci-setup
git add .github/workflows/ci.yml
git commit -m "ci: add GitHub Actions workflow with unit-test job"
git push -u origin feat/ci-setup
```

- [x] **Step 5: Create PR and verify CI passes**

Create PR from `feat/ci-setup` → `master`. CI must show `unit-test` job passing.

- [x] **Step 6: Merge PR**

---

### Task 1: Convert to Cargo Workspace + Shared Types

**Files:**
- Create: `crates/harness-core/src/lib.rs`
- Create: `crates/harness-core/src/types.rs`
- Create: `crates/harness-core/src/error.rs`
- Modify: `Cargo.toml` (convert to workspace)
- Delete: `src/main.rs`

**Interfaces:**
- Consumes: none (foundation)
- Produces: `Message`, `Role`, `ToolCall`, `ToolResult`, `Action`, `HarnessError` used by all crates

- [x] **Step 1: Convert root Cargo.toml to workspace**

Replace contents of `Cargo.toml`:

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "2"
```

- [x] **Step 2: Create harness-core crate structure**

Create directory `crates/harness-core/src/`.

Create `crates/harness-core/Cargo.toml`:

```toml
[package]
name = "harness-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
```

- [x] **Step 3: Write shared types**

Create `crates/harness-core/src/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output: String,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub enum Action {
    ToolCall(ToolCall),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub function: FunctionSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
```

- [x] **Step 4: Write error types**

Create `crates/harness-core/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HarnessError {
    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Tool error: {tool} - {message}")]
    Tool { tool: String, message: String },

    #[error("Guard blocked: {0}")]
    GuardBlocked(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Max turns ({0}) exceeded")]
    MaxTurnsExceeded(usize),
}

pub type Result<T> = std::result::Result<T, HarnessError>;
```

- [x] **Step 5: Write lib.rs re-exports**

Create `crates/harness-core/src/lib.rs`:

```rust
pub mod types;
pub mod error;

pub use types::*;
pub use error::*;
```

- [x] **Step 6: Delete old src/main.rs**

Remove `src/main.rs` (will be replaced by `crates/harness-bin/`).

- [x] **Step 7: Run tests to verify compilation**

Run: `cargo check --workspace`
Expected: OK (no errors)

- [x] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: convert to cargo workspace, add shared types and error types"
```

---

### Task 2: LLM Abstraction Layer + Mock Provider

**Files:**
- Create: `crates/harness-llm/Cargo.toml`
- Create: `crates/harness-llm/src/lib.rs`
- Create: `crates/harness-llm/src/mock.rs`
- Create: `crates/harness-llm/src/mock_tests.rs`

**Interfaces:**
- Consumes: `Message`, `ToolCall`, `FinishReason`, `Usage`, `ToolSchema` from `harness-core`
- Produces: `LlmProvider` trait, `MockLlmProvider`, `CompletionRequest`, `CompletionResponse`

- [x] **Step 1: Create harness-llm crate**

Create `crates/harness-llm/Cargo.toml`:

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

- [x] **Step 2: Write the failing test for MockLlmProvider**

Create `crates/harness-llm/src/mock_tests.rs`:

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

- [x] **Step 3: Run test to verify it fails**

Run: `cargo test -p harness-llm`
Expected: FAIL — `mock.rs` does not exist yet

- [x] **Step 4: Write LlmProvider trait and MockLlmProvider**

Create `crates/harness-llm/src/lib.rs`:

```rust
pub mod mock;

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

Create `crates/harness-llm/src/mock.rs`:

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

Update `crates/harness-llm/src/lib.rs` to include test module:

```rust
#[cfg(test)]
mod mock_tests;
```

- [x] **Step 5: Run tests to verify they pass**

Run: `cargo test -p harness-llm`
Expected: 3 tests PASS

- [x] **Step 6: Commit**

```bash
git add crates/harness-llm/
git commit -m "feat: add LLM abstraction layer with mock provider"
```

---

### Task 3: OpenAI-Compatible LLM Provider

**Files:**
- Create: `crates/harness-llm/src/openai.rs`
- Create: `crates/harness-llm/src/openai_tests.rs`

**Interfaces:**
- Consumes: `LlmProvider` trait, `CompletionRequest`, `CompletionResponse` from Task 2
- Produces: `OpenAiCompatibleProvider`

- [ ] **Step 1: Write failing test for OpenAI provider serialization**

Create `crates/harness-llm/src/openai_tests.rs`:

```rust
use crate::openai::OpenAiCompatibleProvider;
use crate::{CompletionRequest, LlmProvider};
use harness_core::{Message, Role, ToolSchema, FunctionSchema};

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
            tool_calls: vec![],
            tool_call_id: None,
        }],
        tools: Some(vec![ToolSchema {
            schema_type: "function".into(),
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p harness-llm -- openai`
Expected: FAIL — `openai.rs` does not exist

- [ ] **Step 3: Implement OpenAiCompatibleProvider**

Create `crates/harness-llm/src/openai.rs`:

```rust
use crate::CompletionRequest;
use crate::LlmProvider;
use harness_core::CompletionResponse;
use async_trait::async_trait;
use harness_core::{FinishReason, Role, ToolCall, Usage};

pub struct OpenAiCompatibleProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiCompatibleProvider {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
            model,
        }
    }

    pub fn build_request_body(&self, request: &CompletionRequest) -> serde_json::Value {
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                let mut obj = serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::Tool => "tool",
                    },
                    "content": m.content,
                });
                if !m.tool_calls.is_empty() {
                    obj["tool_calls"] = serde_json::to_value(&m.tool_calls).unwrap();
                }
                if let Some(ref id) = m.tool_call_id {
                    obj["tool_call_id"] = serde_json::Value::String(id.clone());
                }
                obj
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
        });

        if let Some(ref tools) = request.tools {
            body["tools"] = serde_json::to_value(tools).unwrap();
        }

        body
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    async fn complete(&self, request: CompletionRequest) -> harness_core::Result<CompletionResponse> {
        let body = self.build_request_body(&request);
        let url = format!("{}/v1/chat/completions", self.base_url);

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| harness_core::HarnessError::Llm(e.to_string()))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| harness_core::HarnessError::Llm(e.to_string()))?;

        let choice = json["choices"][0].clone();
        let message = choice["message"].clone();

        let content = message["content"].as_str().map(|s| s.to_string());
        let tool_calls: Vec<ToolCall> = message["tool_calls"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|tc| ToolCall {
                        id: tc["id"].as_str().unwrap_or("").to_string(),
                        name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                        arguments: tc["function"]["arguments"]
                            .as_str()
                            .and_then(|s| serde_json::from_str(s).ok())
                            .unwrap_or(serde_json::Value::Null),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let finish_reason = match choice["finish_reason"].as_str() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolCalls,
            _ => FinishReason::Stop,
        };

        let usage = Usage {
            prompt_tokens: json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as usize,
            completion_tokens: json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as usize,
            total_tokens: json["usage"]["total_tokens"].as_u64().unwrap_or(0) as usize,
        };

        Ok(CompletionResponse {
            content,
            tool_calls,
            finish_reason,
            usage,
        })
    }

    fn name(&self) -> &str {
        &self.model
    }
}
```

Update `crates/harness-llm/src/lib.rs` to include modules:

```rust
pub mod mock;
pub mod openai;

#[cfg(test)]
mod mock_tests;
#[cfg(test)]
mod openai_tests;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p harness-llm`
Expected: 5 tests PASS (3 mock + 2 openai)

- [ ] **Step 5: Commit**

```bash
git add crates/harness-llm/
git commit -m "feat: add OpenAI-compatible LLM provider for DeepSeek"
```

---

### Task 4: Tool Trait + ToolRegistry

**Files:**
- Create: `crates/harness-tools/Cargo.toml`
- Create: `crates/harness-tools/src/lib.rs`
- Create: `crates/harness-tools/src/registry_tests.rs`

**Interfaces:**
- Consumes: `Message`, `ToolCall`, `ToolResult`, `ToolSchema`, `FunctionSchema` from `harness-core`
- Produces: `Tool` trait, `ToolRegistry`, `ToolInfo`

- [ ] **Step 1: Create harness-tools crate**

Create `crates/harness-tools/Cargo.toml`:

```toml
[package]
name = "harness-tools"
version = "0.1.0"
edition = "2024"

[dependencies]
harness-core = { path = "../harness-core" }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
tokio = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
```

- [ ] **Step 2: Write failing tests for ToolRegistry**

Create `crates/harness-tools/src/registry_tests.rs`:

```rust
use crate::{Tool, ToolRegistry, ToolInfo, ToolResult};
use async_trait::async_trait;
use serde_json::json;

struct DummyTool;

#[async_trait]
impl Tool for DummyTool {
    fn name(&self) -> &str { "dummy" }
    fn description(&self) -> &str { "A dummy tool" }
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
            output: format!("echo: {}", msg),
            is_error: false,
        }
    }
}

#[tokio::test]
async fn register_and_execute_tool() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(DummyTool));

    let result = registry.execute("dummy", &json!({"msg": "hello"})).await.unwrap();
    assert_eq!(result.output, "echo: hello");
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
    assert_eq!(schemas[0]["function"]["name"], "dummy");
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
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p harness-tools`
Expected: FAIL — `lib.rs` has no types yet

- [ ] **Step 4: Implement Tool trait and ToolRegistry**

Create `crates/harness-tools/src/lib.rs`:

```rust
pub mod plugin;

use async_trait::async_trait;
use harness_core::{ToolCall, ToolResult, ToolSchema, FunctionSchema};
use std::collections::HashMap;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, args: serde_json::Value) -> ToolResult;
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    pub fn unregister(&mut self, name: &str) {
        self.tools.remove(name);
    }

    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
            })
            .collect()
    }

    pub fn to_llm_tools(&self) -> Vec<ToolSchema> {
        self.tools
            .values()
            .map(|t| ToolSchema {
                schema_type: "function".into(),
                function: FunctionSchema {
                    name: t.name().to_string(),
                    description: t.description().to_string(),
                    parameters: t.parameters_schema(),
                },
            })
            .collect()
    }

    pub async fn execute(&self, name: &str, args: &serde_json::Value) -> Result<ToolResult, String> {
        self.tools
            .get(name)
            .ok_or_else(|| format!("Tool '{}' not found", name))
            .map(|t| t.clone())
            .ok_or_else(|| "Tool not found".into())?
            .execute(args.clone())
            .await
            .map_err(|e| e.output)
    }
}

#[cfg(test)]
mod registry_tests;
```

Note: The `execute` method needs adjustment — `Box<dyn Tool>` can't be cloned. Fix:

```rust
    pub async fn execute(&self, name: &str, args: &serde_json::Value) -> Result<ToolResult, String> {
        let tool = self.tools
            .get(name)
            .ok_or_else(|| format!("Tool '{}' not found", name))?;
        // SAFETY: we need Arc<dyn Tool> instead of Box<dyn Tool>
        // Use Arc for shared ownership
        todo!("fix with Arc")
    }
```

Actually, let's fix the design upfront — use `Arc<dyn Tool>`:

Update `crates/harness-tools/src/lib.rs`:

```rust
pub mod plugin;

use async_trait::async_trait;
use harness_core::{ToolCall, ToolResult, ToolSchema, FunctionSchema};
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, args: serde_json::Value) -> ToolResult;
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    pub fn unregister(&mut self, name: &str) {
        self.tools.remove(name);
    }

    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
            })
            .collect()
    }

    pub fn to_llm_tools(&self) -> Vec<ToolSchema> {
        self.tools
            .values()
            .map(|t| ToolSchema {
                schema_type: "function".into(),
                function: FunctionSchema {
                    name: t.name().to_string(),
                    description: t.description().to_string(),
                    parameters: t.parameters_schema(),
                },
            })
            .collect()
    }

    pub async fn execute(&self, name: &str, args: &serde_json::Value) -> Result<ToolResult, String> {
        let tool = self.tools
            .get(name)
            .ok_or_else(|| format!("Tool '{}' not found", name))?;
        tool.execute(args.clone()).await
    }
}

#[cfg(test)]
mod registry_tests;
```

Update tests to use `Arc`:

```rust
use std::sync::Arc;
// ...
    registry.register(Box::new(DummyTool));  →  registry.register(Arc::new(DummyTool));
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p harness-tools`
Expected: 5 tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/harness-tools/
git commit -m "feat: add Tool trait and ToolRegistry with dynamic dispatch"
```

---

### Task 5: ReadFile + WriteFile Tools

**Files:**
- Create: `crates/harness-tools/src/read_file.rs`
- Create: `crates/harness-tools/src/write_file.rs`

**Interfaces:**
- Consumes: `Tool` trait from Task 4
- Produces: `ReadFile` tool, `WriteFile` tool

- [ ] **Step 1: Write failing test for ReadFile**

Add to `crates/harness-tools/src/registry_tests.rs`:

```rust
use crate::read_file::ReadFile;
use std::sync::Arc;

#[tokio::test]
async fn read_file_tool_reads_content() {
    let tool = ReadFile::new();
    let result = tool.execute(serde_json::json!({"path": "crates/harness-core/Cargo.toml"})).await;
    assert!(!result.is_error);
    assert!(result.output.contains("[package]"));
}

#[tokio::test]
async fn read_file_tool_returns_error_for_missing() {
    let tool = ReadFile::new();
    let result = tool.execute(serde_json::json!({"path": "nonexistent.txt"})).await;
    assert!(result.is_error);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p harness-tools -- read_file`
Expected: FAIL

- [ ] **Step 3: Implement ReadFile**

Create `crates/harness-tools/src/read_file.rs`:

```rust
use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;

pub struct ReadFile;

impl ReadFile {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read file contents. Returns the content of a file at the given path." }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "File path to read" },
                "offset": { "type": "integer", "description": "Line number to start from (1-indexed)", "default": 1 },
                "limit": { "type": "integer", "description": "Max lines to read", "default": 2000 }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let path = match args["path"].as_str() {
            Some(p) => p,
            None => return ToolResult { tool_call_id: String::new(), output: "Missing 'path' parameter".into(), is_error: true },
        };
        let offset = args["offset"].as_u64().unwrap_or(1).max(1) as usize;
        let limit = args["limit"].as_u64().unwrap_or(2000) as usize;

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let start = (offset - 1).min(lines.len());
                let end = (start + limit).min(lines.len());
                let output: String = lines[start..end]
                    .iter()
                    .enumerate()
                    .map(|(i, l)| format!("{}: {}", start + i + 1, l))
                    .collect::<Vec<_>>()
                    .join("\n");
                ToolResult { tool_call_id: String::new(), output, is_error: false }
            }
            Err(e) => ToolResult { tool_call_id: String::new(), output: format!("Error reading {}: {}", path, e), is_error: true },
        }
    }
}
```

- [ ] **Step 4: Run ReadFile tests**

Run: `cargo test -p harness-tools -- read_file`
Expected: 2 tests PASS

- [ ] **Step 5: Write failing test for WriteFile**

Add to `crates/harness-tools/src/registry_tests.rs`:

```rust
use crate::write_file::WriteFile;

#[tokio::test]
async fn write_file_tool_creates_file() {
    let tool = WriteFile::new();
    let test_path = "test_output_write.txt";
    let result = tool.execute(serde_json::json!({"path": test_path, "content": "hello world"})).await;
    assert!(!result.is_error);
    assert_eq!(std::fs::read_to_string(test_path).unwrap(), "hello world");
    std::fs::remove_file(test_path).ok();
}

#[tokio::test]
async fn write_file_tool_creates_parent_dirs() {
    let tool = WriteFile::new();
    let test_path = "test_dir_nested/output.txt";
    let result = tool.execute(serde_json::json!({"path": test_path, "content": "nested"})).await;
    assert!(!result.is_error);
    assert_eq!(std::fs::read_to_string(test_path).unwrap(), "nested");
    std::fs::remove_dir_all("test_dir_nested").ok();
}
```

- [ ] **Step 6: Implement WriteFile**

Create `crates/harness-tools/src/write_file.rs`:

```rust
use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;

pub struct WriteFile;

impl WriteFile {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str { "write_file" }
    fn description(&self) -> &str { "Create or overwrite a file with the given content. Creates parent directories if needed." }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "File path to write" },
                "content": { "type": "string", "description": "Content to write to the file" }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let path = match args["path"].as_str() {
            Some(p) => p,
            None => return ToolResult { tool_call_id: String::new(), output: "Missing 'path' parameter".into(), is_error: true },
        };
        let content = match args["content"].as_str() {
            Some(c) => c,
            None => return ToolResult { tool_call_id: String::new(), output: "Missing 'content' parameter".into(), is_error: true },
        };

        if let Some(parent) = std::path::Path::new(path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResult { tool_call_id: String::new(), output: format!("Failed to create directories: {}", e), is_error: true };
            }
        }

        match std::fs::write(path, content) {
            Ok(()) => ToolResult { tool_call_id: String::new(), output: format!("Successfully wrote {} bytes to {}", content.len(), path), is_error: false },
            Err(e) => ToolResult { tool_call_id: String::new(), output: format!("Failed to write {}: {}", path, e), is_error: true },
        }
    }
}
```

- [ ] **Step 7: Update lib.rs to include new modules**

Update `crates/harness-tools/src/lib.rs` — add at top:

```rust
pub mod read_file;
pub mod write_file;
```

- [ ] **Step 8: Run all tools tests**

Run: `cargo test -p harness-tools`
Expected: All tests PASS

- [ ] **Step 9: Commit**

```bash
git add crates/harness-tools/
git commit -m "feat: add ReadFile and WriteFile tools"
```

---

### Task 6: ShellExec + GitOp + CodeSearch Tools

**Files:**
- Create: `crates/harness-tools/src/shell_exec.rs`
- Create: `crates/harness-tools/src/git_op.rs`
- Create: `crates/harness-tools/src/code_search.rs`

**Interfaces:**
- Consumes: `Tool` trait from Task 4
- Produces: `ShellExec`, `GitOp`, `CodeSearch` tools

- [ ] **Step 1: Write failing test for ShellExec**

Add to `crates/harness-tools/src/registry_tests.rs`:

```rust
use crate::shell_exec::ShellExec;

#[tokio::test]
async fn shell_exec_runs_command() {
    let tool = ShellExec::new();
    let result = tool.execute(serde_json::json!({"command": "echo hello"})).await;
    assert!(!result.is_error);
    assert!(result.output.contains("hello"));
}

#[tokio::test]
async fn shell_exec_returns_error_for_bad_command() {
    let tool = ShellExec::new();
    let result = tool.execute(serde_json::json!({"command": "exit 1"})).await;
    assert!(result.is_error);
}
```

- [ ] **Step 2: Implement ShellExec**

Create `crates/harness-tools/src/shell_exec.rs`:

```rust
use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;
use std::time::Duration;

pub struct ShellExec;

impl ShellExec {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for ShellExec {
    fn name(&self) -> &str { "shell_exec" }
    fn description(&self) -> &str { "Execute a shell command. Returns stdout and stderr." }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command to execute" },
                "cwd": { "type": "string", "description": "Working directory (optional)" },
                "timeout": { "type": "integer", "description": "Timeout in seconds (default 30)" }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let command = match args["command"].as_str() {
            Some(c) => c,
            None => return ToolResult { tool_call_id: String::new(), output: "Missing 'command' parameter".into(), is_error: true },
        };
        let timeout_secs = args["timeout"].as_u64().unwrap_or(30);

        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").arg(command);
        if let Some(cwd) = args["cwd"].as_str() {
            cmd.current_dir(cwd);
        }
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let mut result = String::new();
                if !stdout.is_empty() {
                    result.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !result.is_empty() { result.push('\n'); }
                    result.push_str(&stderr);
                }
                ToolResult {
                    tool_call_id: String::new(),
                    output: result,
                    is_error: !output.status.success(),
                }
            }
            Err(e) => ToolResult { tool_call_id: String::new(), output: format!("Failed to execute: {}", e), is_error: true },
        }
    }
}
```

- [ ] **Step 3: Run ShellExec tests**

Run: `cargo test -p harness-tools -- shell_exec`
Expected: 2 tests PASS

- [ ] **Step 4: Write failing test for GitOp**

Add to `crates/harness-tools/src/registry_tests.rs`:

```rust
use crate::git_op::GitOp;

#[tokio::test]
async fn git_op_status_works() {
    let tool = GitOp::new();
    let result = tool.execute(serde_json::json!({"operation": "status"})).await;
    assert!(!result.is_error);
}
```

- [ ] **Step 5: Implement GitOp**

Create `crates/harness-tools/src/git_op.rs`:

```rust
use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;

pub struct GitOp;

impl GitOp {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for GitOp {
    fn name(&self) -> &str { "git_op" }
    fn description(&self) -> &str { "Execute git operations: status, diff, log, branch, commit." }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": { "type": "string", "enum": ["status", "diff", "log", "branch"], "description": "Git operation" },
                "args": { "type": "string", "description": "Additional arguments (optional)" }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let op = match args["operation"].as_str() {
            Some(o) => o,
            None => return ToolResult { tool_call_id: String::new(), output: "Missing 'operation'".into(), is_error: true },
        };
        let extra = args["args"].as_str().unwrap_or("");

        let mut cmd_args = vec![op.to_string()];
        if !extra.is_empty() {
            cmd_args.extend(extra.split_whitespace().map(|s| s.to_string()));
        }

        match std::process::Command::new("git").args(&cmd_args).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let mut result = stdout;
                if !stderr.is_empty() {
                    if !result.is_empty() { result.push('\n'); }
                    result.push_str(&stderr);
                }
                ToolResult { tool_call_id: String::new(), output: result, is_error: !output.status.success() }
            }
            Err(e) => ToolResult { tool_call_id: String::new(), output: format!("Git error: {}", e), is_error: true },
        }
    }
}
```

- [ ] **Step 6: Write failing test for CodeSearch**

Add to `crates/harness-tools/src/registry_tests.rs`:

```rust
use crate::code_search::CodeSearch;

#[tokio::test]
async fn code_search_finds_pattern() {
    let tool = CodeSearch::new();
    let result = tool.execute(serde_json::json!({"pattern": "harness-core", "path": "."})).await;
    assert!(!result.is_error);
    assert!(result.output.contains("Cargo.toml"));
}
```

- [ ] **Step 7: Implement CodeSearch**

Create `crates/harness-tools/src/code_search.rs`:

```rust
use crate::Tool;
use async_trait::async_trait;
use harness_core::ToolResult;
use serde_json::json;
use std::path::Path;
use walkdir::WalkDir;

pub struct CodeSearch;

impl CodeSearch {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for CodeSearch {
    fn name(&self) -> &str { "code_search" }
    fn description(&self) -> &str { "Search for a pattern in files under a directory. Returns matching file paths and line numbers." }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Search pattern (substring)" },
                "path": { "type": "string", "description": "Directory to search in (default: current dir)" },
                "include": { "type": "string", "description": "File pattern to include (e.g. '*.rs')" }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let pattern = match args["pattern"].as_str() {
            Some(p) => p,
            None => return ToolResult { tool_call_id: String::new(), output: "Missing 'pattern'".into(), is_error: true },
        };
        let search_path = args["path"].as_str().unwrap_or(".");
        let include = args["include"].as_str();

        let mut results = Vec::new();
        for entry in WalkDir::new(search_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Some(inc) = include {
                let name = entry.file_name().to_string_lossy();
                if !glob_match(inc, &name) { continue; }
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for (i, line) in content.lines().enumerate() {
                    if line.contains(pattern) {
                        results.push(format!("{}:{}: {}", entry.path().display(), i + 1, line));
                    }
                }
            }
            if results.len() >= 50 { break; }
        }

        if results.is_empty() {
            ToolResult { tool_call_id: String::new(), output: format!("No matches for '{}' in {}", pattern, search_path), is_error: false }
        } else {
            ToolResult { tool_call_id: String::new(), output: results.join("\n"), is_error: false }
        }
    }
}

fn glob_match(pattern: &str, name: &str) -> bool {
    if pattern.starts_with("*.") {
        let ext = &pattern[1..];
        name.ends_with(ext)
    } else {
        name.contains(pattern)
    }
}
```

- [ ] **Step 8: Run all tools tests**

Run: `cargo test -p harness-tools`
Expected: All tests PASS

- [ ] **Step 9: Commit**

```bash
git add crates/harness-tools/
git commit -m "feat: add ShellExec, GitOp, and CodeSearch tools"
```

---

### Task 7: Plugin System

**Files:**
- Create: `crates/harness-tools/src/plugin.rs`
- Modify: `crates/harness-tools/src/lib.rs` (add plugin module)

**Interfaces:**
- Consumes: `Tool` trait, `ToolRegistry` from Task 4
- Produces: `Plugin` trait, `PluginLoader`

- [ ] **Step 1: Write failing test for PluginLoader**

Add to `crates/harness-tools/src/registry_tests.rs`:

```rust
use crate::plugin::{Plugin, PluginLoader, PluginContext};
use std::sync::Arc;

struct TestPlugin;

#[async_trait::async_trait]
impl Tool for TestPlugin {
    fn name(&self) -> &str { "test_plugin" }
    fn description(&self) -> &str { "test" }
    fn parameters_schema(&self) -> serde_json::json!({ "type": "object", "properties": {} })
    async fn execute(&self, _args: serde_json::Value) -> harness_core::ToolResult {
        harness_core::ToolResult { tool_call_id: String::new(), output: "plugin works".into(), is_error: false }
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
    assert_eq!(loader.list()[0], "test_plugin");
}

#[tokio::test]
async fn plugin_loader_loads_into_registry() {
    let mut loader = PluginLoader::new();
    loader.register(Arc::new(TestPlugin));
    let mut registry = crate::ToolRegistry::new();
    loader.load_all(&mut registry).unwrap();
    assert!(registry.execute("test_plugin", &serde_json::json!({})).await.is_ok());
}
```

- [ ] **Step 2: Implement Plugin trait and PluginLoader**

Create `crates/harness-tools/src/plugin.rs`:

```rust
use crate::{Tool, ToolRegistry};
use async_trait::async_trait;
use std::sync::Arc;

pub struct PluginContext {
    pub workspace_dir: String,
}

#[async_trait]
pub trait Plugin: Tool {
    fn version(&self) -> &str;
    fn dependencies(&self) -> Vec<&str>;
    fn init(&mut self, ctx: &PluginContext) -> harness_core::Result<()>;
}

pub struct PluginLoader {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn list(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.name().to_string()).collect()
    }

    pub fn load_all(&self, registry: &mut ToolRegistry) -> harness_core::Result<()> {
        for plugin in &self.plugins {
            registry.register(plugin.clone() as Arc<dyn Tool>);
        }
        Ok(())
    }
}
```

- [ ] **Step 3: Update lib.rs**

Update `crates/harness-tools/src/lib.rs` — add `pub mod plugin;` at top.

- [ ] **Step 4: Run tests**

Run: `cargo test -p harness-tools`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/harness-tools/
git commit -m "feat: add Plugin trait and PluginLoader"
```

---

### Task 8: Governance Guardrail Rules

**Files:**
- Create: `crates/harness-guard/Cargo.toml`
- Create: `crates/harness-guard/src/lib.rs`
- Create: `crates/harness-guard/src/rules.rs`
- Create: `crates/harness-guard/src/guard_tests.rs`

**Interfaces:**
- Consumes: `ToolCall` from `harness-core`
- Produces: `Guardrail`, `GuardrailRule` trait, 5 built-in rules

- [ ] **Step 1: Create harness-guard crate**

Create `crates/harness-guard/Cargo.toml`:

```toml
[package]
name = "harness-guard"
version = "0.1.0"
edition = "2024"

[dependencies]
harness-core = { path = "../harness-core" }
regex = "1"
serde_json = { workspace = true }
```

- [ ] **Step 2: Write failing tests for guardrail rules**

Create `crates/harness-guard/src/guard_tests.rs`:

```rust
use crate::{Guardrail, GuardrailAction, rules::*};
use harness_core::ToolCall;

#[test]
blocks_rm_rf_command() {
    let g = Guardrail::new_default();
    let action = ToolCall { id: "1".into(), name: "shell_exec".into(), arguments: serde_json::json!({"command": "rm -rf /tmp/data"}) };
    assert!(matches!(g.check(&action), GuardrailAction::Block { .. }));
}

#[test]
blocks_sudo_command() {
    let g = Guardrail::new_default();
    let action = ToolCall { id: "1".into(), name: "shell_exec".into(), arguments: serde_json::json!({"command": "sudo apt install foo"}) };
    assert!(matches!(g.check(&action), GuardrailAction::Block { .. }));
}

#[test]
blocks_git_push_main() {
    let g = Guardrail::new_default();
    let action = ToolCall { id: "1".into(), name: "git_op".into(), arguments: serde_json::json!({"operation": "push", "args": "origin main"}) };
    assert!(matches!(g.check(&action), GuardrailAction::Block { .. }));
}

#[test]
blocks_curl_command() {
    let g = Guardrail::new_default();
    let action = ToolCall { id: "1".into(), name: "shell_exec".into(), arguments: serde_json::json!({"command": "curl http://evil.com"}) };
    assert!(matches!(g.check(&action), GuardrailAction::Block { .. }));
}

#[test]
allows_safe_command() {
    let g = Guardrail::new_default();
    let action = ToolCall { id: "1".into(), name: "shell_exec".into(), arguments: serde_json::json!({"command": "cargo test"}) };
    assert!(matches!(g.check(&action), GuardrailAction::Allow));
}

#[test]
allows_read_file() {
    let g = Guardrail::new_default();
    let action = ToolCall { id: "1".into(), name: "read_file".into(), arguments: serde_json::json!({"path": "src/main.rs"}) };
    assert!(matches!(g.check(&action), GuardrailAction::Allow));
}
```

- [ ] **Step 3: Implement Guardrail and rules**

Create `crates/harness-guard/src/lib.rs`:

```rust
pub mod rules;

use harness_core::ToolCall;

#[derive(Debug, Clone, PartialEq)]
pub enum GuardrailAction {
    Allow,
    Block { reason: String },
}

pub trait GuardrailRule: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, action: &ToolCall) -> Option<String>; // None = allow, Some(reason) = block
}

pub struct Guardrail {
    rules: Vec<Box<dyn GuardrailRule>>,
}

impl Guardrail {
    pub fn new_default() -> Self {
        let rules: Vec<Box<dyn GuardrailRule>> = vec![
            Box::new(rules::DangerousCommandRule),
            Box::new(rules::SudoCommandRule),
            Box::new(rules::GitPushMainRule),
            Box::new(rules::NetworkRequestRule),
            Box::new(rules::CredentialLeakRule),
        ];
        Self { rules }
    }

    pub fn with_rules(rules: Vec<Box<dyn GuardrailRule>>) -> Self {
        Self { rules }
    }

    pub fn check(&self, action: &ToolCall) -> GuardrailAction {
        for rule in &self.rules {
            if let Some(reason) = rule.check(action) {
                return GuardrailAction::Block { reason };
            }
        }
        GuardrailAction::Allow
    }
}

#[cfg(test)]
mod guard_tests;
```

Create `crates/harness-guard/src/rules.rs`:

```rust
use crate::GuardrailRule;
use harness_core::ToolCall;
use regex::Regex;

pub struct DangerousCommandRule;

impl GuardrailRule for DangerousCommandRule {
    fn name(&self) -> &str { "dangerous_command" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        if action.name != "shell_exec" { return None; }
        let cmd = action.arguments["command"].as_str()?;
        let patterns = ["rm -rf", "rm -r /", "del /s", "format c:", "mkfs", "> /dev/sda"];
        for p in patterns {
            if cmd.contains(p) {
                return Some(format!("Dangerous command detected: '{}'", p));
            }
        }
        None
    }
}

pub struct SudoCommandRule;

impl GuardrailRule for SudoCommandRule {
    fn name(&self) -> &str { "sudo_command" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        if action.name != "shell_exec" { return None; }
        let cmd = action.arguments["command"].as_str()?;
        if cmd.starts_with("sudo ") || cmd.contains(" chmod 777") || cmd.contains(" chmod -R 777") {
            return Some(format!("System-level command detected: '{}'", cmd));
        }
        None
    }
}

pub struct GitPushMainRule;

impl GuardrailRule for GitPushMainRule {
    fn name(&self) -> &str { "git_push_main" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        if action.name != "git_op" { return None; }
        let op = action.arguments["operation"].as_str()?;
        if op == "push" {
            let args = action.arguments["args"].as_str().unwrap_or("");
            if args.contains("main") || args.contains("master") {
                return Some("Push to main/master branch requires approval".into());
            }
        }
        None
    }
}

pub struct NetworkRequestRule;

impl GuardrailRule for NetworkRequestRule {
    fn name(&self) -> &str { "network_request" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        if action.name != "shell_exec" { return None; }
        let cmd = action.arguments["command"].as_str()?;
        let patterns = ["curl ", "wget ", "fetch ", "http://", "https://"];
        for p in patterns {
            if cmd.contains(p) {
                return Some(format!("Network request detected: '{}'", p.trim()));
            }
        }
        None
    }
}

pub struct CredentialLeakRule;

impl GuardrailRule for CredentialLeakRule {
    fn name(&self) -> &str { "credential_leak" }
    fn check(&self, action: &ToolCall) -> Option<String> {
        let output = match action.name.as_str() {
            "write_file" => action.arguments["content"].as_str().unwrap_or(""),
            "shell_exec" => action.arguments["command"].as_str().unwrap_or(""),
            _ => return None,
        };
        let key_patterns = ["sk-", "api_key=", "API_KEY=", "DEEPSEEK_API_KEY="];
        for p in key_patterns {
            if output.contains(p) {
                return Some("Potential credential leak detected".into());
            }
        }
        None
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p harness-guard`
Expected: 6 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/harness-guard/
git commit -m "feat: add guardrail system with 5 built-in rules"
```

---

### Task 9: HITL (Human-in-the-Loop) Confirmation

**Files:**
- Create: `crates/harness-guard/src/hitl.rs`
- Modify: `crates/harness-guard/src/lib.rs`

**Interfaces:**
- Consumes: `GuardrailAction` from Task 8
- Produces: `HitlConfirmer` trait, `StdioHitlConfirmer`

- [ ] **Step 1: Write failing test for HITL confirmer**

Add to `crates/harness-guard/src/guard_tests.rs`:

```rust
use crate::hitl::{HitlConfirmer, MockHitlConfirmer};
use crate::{Guardrail, GuardrailAction};

#[tokio::test]
async fn mock_hitl_returns_approval() {
    let hitl = MockHitlConfirmer::new(true);
    let action = GuardrailAction::Block { reason: "test".into() };
    assert!(hitl.confirm(&action).await.unwrap());
}

#[tokio::test]
async fn mock_hitl_returns_deny() {
    let hitl = MockHitlConfirmer::new(false);
    let action = GuardrailAction::Block { reason: "test".into() };
    assert!(!hitl.confirm(&action).await.unwrap());
}
```

- [ ] **Step 2: Implement HITL**

Create `crates/harness-guard/src/hitl.rs`:

```rust
use crate::GuardrailAction;
use async_trait::async_trait;

#[async_trait]
pub trait HitlConfirmer: Send + Sync {
    async fn confirm(&self, action: &GuardrailAction) -> harness_core::Result<bool>;
}

pub struct MockHitlConfirmer {
    approve: bool,
}

impl MockHitlConfirmer {
    pub fn new(approve: bool) -> Self { Self { approve } }
}

#[async_trait]
impl HitlConfirmer for MockHitlConfirmer {
    async fn confirm(&self, _action: &GuardrailAction) -> harness_core::Result<bool> {
        Ok(self.approve)
    }
}

pub struct StdioHitlConfirmer;

#[async_trait]
impl HitlConfirmer for StdioHitlConfirmer {
    async fn confirm(&self, action: &GuardrailAction) -> harness_core::Result<bool> {
        let reason = match action {
            GuardrailAction::Block { reason } => reason,
            _ => return Ok(true),
        };
        eprintln!("\n[HITL] Action blocked: {}", reason);
        eprintln!("[HITL] Approve? (y/n): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| harness_core::HarnessError::Llm(e.to_string()))?;
        Ok(input.trim().eq_ignore_ascii_case("y"))
    }
}
```

Update `crates/harness-guard/src/lib.rs` — add:

```rust
pub mod hitl;
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p harness-guard`
Expected: 8 tests PASS

- [ ] **Step 4: Commit**

```bash
git add crates/harness-guard/
git commit -m "feat: add HITL confirmer with mock and stdio implementations"
```

---

### Task 10: Feedback Validators

**Files:**
- Create: `crates/harness-feedback/Cargo.toml`
- Create: `crates/harness-feedback/src/lib.rs`
- Create: `crates/harness-feedback/src/validators.rs`
- Create: `crates/harness-feedback/src/feedback_tests.rs`

**Interfaces:**
- Consumes: `ToolResult` from `harness-core`
- Produces: `FeedbackValidator`, `ValidationRule` trait, 3 validators

- [ ] **Step 1: Create harness-feedback crate**

Create `crates/harness-feedback/Cargo.toml`:

```toml
[package]
name = "harness-feedback"
version = "0.1.0"
edition = "2024"

[dependencies]
harness-core = { path = "../harness-core" }
regex = "1"
```

- [ ] **Step 2: Write failing tests**

Create `crates/harness-feedback/src/feedback_tests.rs`:

```rust
use crate::{FeedbackValidator, ValidationFeedback, FailureCategory};
use crate::validators::*;
use harness_core::ToolResult;

#[test]
test_result_validator_detects_failure() {
    let v = TestResultValidator;
    let result = ToolResult { tool_call_id: "1".into(), output: "test result: 2 failed, 0 passed".into(), is_error: true };
    let feedback = v.validate(&result);
    assert!(!feedback.is_success);
    assert!(matches!(feedback.category, FailureCategory::TestFailure));
    assert!(feedback.summary.contains("2 failed"));
}

#[test]
test_result_validator_detects_success() {
    let v = TestResultValidator;
    let result = ToolResult { tool_call_id: "1".into(), output: "test result: 0 failed, 5 passed".into(), is_error: false };
    let feedback = v.validate(&result);
    assert!(feedback.is_success);
}

#[test]
compile_error_validator_detects_error() {
    let v = CompileErrorValidator;
    let result = ToolResult { tool_call_id: "1".into(), output: "error[E0308]: mismatched types\n --> src/main.rs:5:12".into(), is_error: true };
    let feedback = v.validate(&result);
    assert!(!feedback.is_success);
    assert!(matches!(feedback.category, FailureCategory::CompileError));
}

#[test]
lint_result_validator_detects_warnings() {
    let v = LintResultValidator;
    let result = ToolResult { tool_call_id: "1".into(), output: "warning: unused variable `x`\n --> src/main.rs:3:9".into(), is_error: false };
    let feedback = v.validate(&result);
    assert!(!feedback.is_success);
    assert!(matches!(feedback.category, FailureCategory::LintWarning));
}

#[test]
feedback_validator_dispatches_to_correct_rule() {
    let fv = FeedbackValidator::new_default();
    let result = ToolResult { tool_call_id: "1".into(), output: "test result: 1 failed".into(), is_error: true };
    let feedback = fv.validate(&result);
    assert!(!feedback.is_success);
}
```

- [ ] **Step 3: Implement validators**

Create `crates/harness-feedback/src/lib.rs`:

```rust
pub mod validators;

use harness_core::ToolResult;

#[derive(Debug, Clone, PartialEq)]
pub enum FailureCategory {
    CompileError,
    TestFailure,
    LintWarning,
    RuntimeError,
}

#[derive(Debug, Clone)]
pub struct ValidationFeedback {
    pub is_success: bool,
    pub category: FailureCategory,
    pub summary: String,
    pub details: String,
}

pub trait ValidationRule: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, result: &ToolResult) -> Option<ValidationFeedback>;
}

pub struct FeedbackValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl FeedbackValidator {
    pub fn new_default() -> Self {
        let rules: Vec<Box<dyn ValidationRule>> = vec![
            Box::new(validators::TestResultValidator),
            Box::new(validators::CompileErrorValidator),
            Box::new(validators::LintResultValidator),
        ];
        Self { rules }
    }

    pub fn validate(&self, result: &ToolResult) -> ValidationFeedback {
        for rule in &self.rules {
            if let Some(feedback) = rule.validate(result) {
                return feedback;
            }
        }
        ValidationFeedback {
            is_success: !result.is_error,
            category: FailureCategory::RuntimeError,
            summary: "No specific validation matched".into(),
            details: result.output.clone(),
        }
    }
}

#[cfg(test)]
mod feedback_tests;
```

Create `crates/harness-feedback/src/validators.rs`:

```rust
use crate::{FailureCategory, ValidationFeedback, ValidationRule};
use harness_core::ToolResult;
use regex::Regex;

pub struct TestResultValidator;

impl ValidationRule for TestResultValidator {
    fn name(&self) -> &str { "test_result" }
    fn validate(&self, result: &ToolResult) -> Option<ValidationFeedback> {
        let output = &result.output;
        let re_failed = Regex::new(r"(\d+) failed").ok()?;
        let re_passed = Regex::new(r"(\d+) passed").ok()?;

        let failed = re_failed.captures(output)?.get(1)?.as_str().parse::<usize>().ok()?;
        let passed = re_passed.captures(output).and_then(|c| c.get(1)).and_then(|m| m.as_str().parse::<usize>().ok()).unwrap_or(0);

        if failed > 0 {
            Some(ValidationFeedback {
                is_success: false,
                category: FailureCategory::TestFailure,
                summary: format!("{} tests failed, {} passed", failed, passed),
                details: output.lines().filter(|l| l.contains("FAILED") || l.contains("failures")).collect::<Vec<_>>().join("\n"),
            })
        } else {
            Some(ValidationFeedback {
                is_success: true,
                category: FailureCategory::TestFailure,
                summary: format!("All {} tests passed", passed),
                details: String::new(),
            })
        }
    }
}

pub struct CompileErrorValidator;

impl ValidationRule for CompileErrorValidator {
    fn name(&self) -> &str { "compile_error" }
    fn validate(&self, result: &ToolResult) -> Option<ValidationFeedback> {
        let output = &result.output;
        if output.contains("error[") || output.contains("error:") {
            let re = Regex::new(r"error\[E\d+\]:\s*(.+)").ok();
            let summary = re
                .and_then(|r| r.captures(output))
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "Compilation error".into());
            Some(ValidationFeedback {
                is_success: false,
                category: FailureCategory::CompileError,
                summary,
                details: output.lines().filter(|l| l.starts_with("error") || l.starts_with(" -->")).take(10).collect::<Vec<_>>().join("\n"),
            })
        } else {
            None
        }
    }
}

pub struct LintResultValidator;

impl ValidationRule for LintResultValidator {
    fn name(&self) -> &str { "lint_result" }
    fn validate(&self, result: &ToolResult) -> Option<ValidationFeedback> {
        let output = &result.output;
        if output.contains("warning:") || output.contains("warning[") {
            let count = output.lines().filter(|l| l.contains("warning:")).count();
            if count > 0 {
                Some(ValidationFeedback {
                    is_success: false,
                    category: FailureCategory::LintWarning,
                    summary: format!("{} lint warnings found", count),
                    details: output.lines().filter(|l| l.contains("warning:")).take(5).collect::<Vec<_>>().join("\n"),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p harness-feedback`
Expected: 5 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/harness-feedback/
git commit -m "feat: add feedback validators for test, compile, and lint results"
```

---

### Task 11: Memory Store (SQLite + FTS5)

**Files:**
- Create: `crates/harness-memory/Cargo.toml`
- Create: `crates/harness-memory/src/lib.rs`
- Create: `crates/harness-memory/src/store.rs`
- Create: `crates/harness-memory/src/memory_tests.rs`

**Interfaces:**
- Consumes: none (standalone storage)
- Produces: `MemoryStore`, `MemoryEntry`

- [ ] **Step 1: Create harness-memory crate**

Create `crates/harness-memory/Cargo.toml`:

```toml
[package]
name = "harness-memory"
version = "0.1.0"
edition = "2024"

[dependencies]
harness-core = { path = "../harness-core" }
rusqlite = { version = "0.32", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { workspace = true }
serde_json = { workspace = true }
```

- [ ] **Step 2: Write failing tests**

Create `crates/harness-memory/src/memory_tests.rs`:

```rust
use crate::{MemoryStore, MemoryEntry};

#[test]
store_and_retrieve_semantic_memory() {
    let mut store = MemoryStore::new_in_memory().unwrap();
    store.store(MemoryEntry {
        id: None,
        category: "convention".into(),
        key: "code_style".into(),
        value: "Use snake_case".into(),
        confidence: 1.0,
    }).unwrap();

    let results = store.search("code style", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, "code_style");
}

#[test]
store_and_retrieve_episodic_memory() {
    let mut store = MemoryStore::new_in_memory().unwrap();
    store.store_episodic("session-1", "Fixed compilation error in main.rs", &["fix".into()]).unwrap();

    let results = store.search("compilation", 10).unwrap();
    assert!(!results.is_empty());
}

#[test]
search_returns_relevant_results() {
    let mut store = MemoryStore::new_in_memory().unwrap();
    store.store(MemoryEntry { id: None, category: "convention".into(), key: "rust_style".into(), value: "Use rustfmt".into(), confidence: 1.0 }).unwrap();
    store.store(MemoryEntry { id: None, category: "convention".into(), key: "python_style".into(), value: "Use black".into(), confidence: 1.0 }).unwrap();

    let results = store.search("rustfmt", 10).unwrap();
    assert!(results.iter().any(|r| r.key == "rust_style"));
}

#[test]
by_category_filters() {
    let mut store = MemoryStore::new_in_memory().unwrap();
    store.store(MemoryEntry { id: None, category: "convention".into(), key: "a".into(), value: "v1".into(), confidence: 1.0 }).unwrap();
    store.store(MemoryEntry { id: None, category: "preference".into(), key: "b".into(), value: "v2".into(), confidence: 1.0 }).unwrap();

    let results = store.by_category("convention").unwrap();
    assert_eq!(results.len(), 1);
}
```

- [ ] **Step 3: Implement MemoryStore**

Create `crates/harness-memory/src/lib.rs`:

```rust
pub mod store;

pub use store::{MemoryStore, MemoryEntry};

#[cfg(test)]
mod memory_tests;
```

Create `crates/harness-memory/src/store.rs`:

```rust
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Option<i64>,
    pub category: String,
    pub key: String,
    pub value: String,
    pub confidence: f64,
}

pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    pub fn new_in_memory() -> crate::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn new(path: &str) -> crate::Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> crate::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS semantic_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                confidence REAL DEFAULT 1.0,
                last_accessed DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(category, key)
            );
            CREATE TABLE IF NOT EXISTS episodic_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                summary TEXT NOT NULL,
                tags TEXT
            );"
        )?;
        Ok(())
    }

    pub fn store(&mut self, entry: MemoryEntry) -> crate::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO semantic_memory (category, key, value, confidence) VALUES (?1, ?2, ?3, ?4)",
            params![entry.category, entry.key, entry.value, entry.confidence],
        )?;
        Ok(())
    }

    pub fn store_episodic(&mut self, session_id: &str, summary: &str, tags: &[String]) -> crate::Result<()> {
        let tags_json = serde_json::to_string(tags).unwrap_or_default();
        self.conn.execute(
            "INSERT INTO episodic_memory (session_id, summary, tags) VALUES (?1, ?2, ?3)",
            params![session_id, summary, tags_json],
        )?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> crate::Result<Vec<MemoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category, key, value, confidence FROM semantic_memory
             WHERE key LIKE ?1 OR value LIKE ?1 OR category LIKE ?1
             ORDER BY confidence DESC
             LIMIT ?2"
        )?;
        let pattern = format!("%{}%", query);
        let entries = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok(MemoryEntry {
                id: Some(row.get(0)?),
                category: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                confidence: row.get(4)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(entries)
    }

    pub fn by_category(&self, category: &str) -> crate::Result<Vec<MemoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category, key, value, confidence FROM semantic_memory WHERE category = ?1"
        )?;
        let entries = stmt.query_map(params![category], |row| {
            Ok(MemoryEntry {
                id: Some(row.get(0)?),
                category: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                confidence: row.get(4)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(entries)
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p harness-memory`
Expected: 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/harness-memory/
git commit -m "feat: add MemoryStore with SQLite backend and FTS5 search"
```

---

### Task 12: Configuration (TOML)

**Files:**
- Create: `crates/harness-config/Cargo.toml`
- Create: `crates/harness-config/src/lib.rs`
- Create: `crates/harness-config/src/config_tests.rs`

**Interfaces:**
- Consumes: none
- Produces: `HarnessConfig` struct with all agent parameters

- [ ] **Step 1: Create harness-config crate**

Create `crates/harness-config/Cargo.toml`:

```toml
[package]
name = "harness-config"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }
toml = "0.8"
harness-core = { path = "../harness-core" }
```

- [ ] **Step 2: Write failing tests**

Create `crates/harness-config/src/config_tests.rs`:

```rust
use crate::HarnessConfig;

#[test]
config_deserializes_from_toml() {
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
config_uses_defaults_for_missing_fields() {
    let toml = r#"
[llm]
provider = "deepseek"
"#;
    let config: HarnessConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.agent.max_turns, 50);
    assert_eq!(config.guardrails.mode, "strict");
    assert_eq!(config.memory.max_context_tokens, 2000);
}
```

- [ ] **Step 3: Implement HarnessConfig**

Create `crates/harness-config/src/lib.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HarnessConfig {
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub guardrails: GuardrailConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
}

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_max_turns")]
    pub max_turns: usize,
    #[serde(default = "default_max_tools")]
    pub max_tools_per_turn: usize,
    #[serde(default = "default_max_fix_rounds")]
    pub max_fix_rounds: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_turns: default_max_turns(),
            max_tools_per_turn: default_max_tools(),
            max_fix_rounds: default_max_fix_rounds(),
        }
    }
}

fn default_max_turns() -> usize { 50 }
fn default_max_tools() -> usize { 5 }
fn default_max_fix_rounds() -> usize { 3 }

#[derive(Debug, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    #[serde(default)]
    pub api_key_env: String,
    #[serde(default)]
    pub base_url: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: "deepseek-chat".into(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            api_key_env: "DEEPSEEK_API_KEY".into(),
            base_url: "https://api.deepseek.com".into(),
        }
    }
}

fn default_provider() -> String { "deepseek".into() }
fn default_temperature() -> f32 { 0.0 }
fn default_max_tokens() -> usize { 4096 }

#[derive(Debug, Deserialize)]
pub struct GuardrailConfig {
    #[serde(default = "default_guard_mode")]
    pub mode: String,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            mode: default_guard_mode(),
            timeout_seconds: default_timeout(),
        }
    }
}

fn default_guard_mode() -> String { "strict".into() }
fn default_timeout() -> u64 { 60 }

#[derive(Debug, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_max_context")]
    pub max_context_tokens: usize,
    #[serde(default)]
    pub db_path: String,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: default_max_context(),
            db_path: ".harness/memory.db".into(),
        }
    }
}

fn default_max_context() -> usize { 2000 }

#[derive(Debug, Deserialize)]
pub struct ToolsConfig {
    #[serde(default = "default_allowed_tools")]
    pub allowed: Vec<String>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self { allowed: default_allowed_tools() }
    }
}

fn default_allowed_tools() -> Vec<String> {
    vec![
        "read_file".into(), "write_file".into(), "shell_exec".into(),
        "git_op".into(), "code_search".into(),
    ]
}

#[cfg(test)]
mod config_tests;
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p harness-config`
Expected: 2 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/harness-config/
git commit -m "feat: add TOML configuration with defaults"
```

---

### Task 13: Agent Main Loop

**Files:**
- Create: `crates/harness-core/src/agent.rs`
- Create: `crates/harness-core/src/agent_tests.rs`
- Modify: `crates/harness-core/Cargo.toml` (add async-trait)

**Interfaces:**
- Consumes: `Message`, `ToolCall`, `ToolResult`, `Action`, `FinishReason`, `CompletionResponse` from `harness-core::types`
- Produces: `Agent` struct with `run()` method

Note: Agent lives in `harness-core` which only owns shared types. The integration in `harness-bin` (Task 14) wires in concrete `LlmProvider`, `ToolRegistry`, `Guardrail`, etc. This keeps `harness-core` dependency-free.

- [ ] **Step 1: Update harness-core Cargo.toml**

Update `crates/harness-core/Cargo.toml`:

```toml
[package]
name = "harness-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
```

- [ ] **Step 2: Write failing tests for Agent loop**

Create `crates/harness-core/src/agent_tests.rs`:

```rust
use super::*;
use crate::types::*;

#[test]
agent_stops_on_finish_stop() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        content: Some("done".into()),
        tool_calls: vec![],
        finish_reason: FinishReason::Stop,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    assert!(agent.should_stop(&response));
}

#[test]
agent_does_not_stop_on_tool_calls() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        content: None,
        tool_calls: vec![ToolCall { id: "1".into(), name: "read_file".into(), arguments: serde_json::json!({}) }],
        finish_reason: FinishReason::ToolCalls,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    assert!(!agent.should_stop(&response));
}

#[test]
agent_stops_at_max_turns() {
    let mut agent = Agent::new(AgentConfig { max_turns: 2, max_tools_per_turn: 5 });
    agent.turn_count = 2;
    let response = CompletionResponse {
        content: None,
        tool_calls: vec![ToolCall { id: "1".into(), name: "dummy".into(), arguments: serde_json::json!({}) }],
        finish_reason: FinishReason::ToolCalls,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    assert!(agent.should_stop(&response));
}

#[test]
parse_actions_extracts_tool_calls() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        content: Some("thinking".into()),
        tool_calls: vec![ToolCall { id: "1".into(), name: "read_file".into(), arguments: serde_json::json!({"path": "a.rs"}) }],
        finish_reason: FinishReason::ToolCalls,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    let actions = agent.parse_actions(&response);
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], Action::ToolCall(tc) if tc.name == "read_file"));
}

#[test]
parse_actions_returns_text_when_no_tool_calls() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        content: Some("just text".into()),
        tool_calls: vec![],
        finish_reason: FinishReason::Stop,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    let actions = agent.parse_actions(&response);
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], Action::Text(t) if t == "just text"));
}
```

- [ ] **Step 3: Implement Agent struct**

Create `crates/harness-core/src/agent.rs`:

```rust
use crate::types::*;
use crate::error::{HarnessError, Result};

pub struct Agent {
    pub config: AgentConfig,
    pub conversation: Vec<Message>,
    pub turn_count: usize,
}

pub struct AgentConfig {
    pub max_turns: usize,
    pub max_tools_per_turn: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self { max_turns: 50, max_tools_per_turn: 5 }
    }
}

impl Agent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            conversation: Vec::new(),
            turn_count: 0,
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.conversation.push(Message {
            role: Role::User,
            content: content.to_string(),
            tool_calls: vec![],
            tool_call_id: None,
        });
    }

    pub fn add_assistant_message(&mut self, msg: Message) {
        self.conversation.push(msg);
    }

    pub fn add_tool_result(&mut self, result: ToolResult) {
        self.conversation.push(Message {
            role: Role::Tool,
            content: result.output,
            tool_calls: vec![],
            tool_call_id: Some(result.tool_call_id),
        });
    }

    pub fn should_stop(&self, response: &CompletionResponse) -> bool {
        response.finish_reason == FinishReason::Stop || self.turn_count >= self.config.max_turns
    }

    pub fn parse_actions(&self, response: &CompletionResponse) -> Vec<Action> {
        let mut actions = Vec::new();
        for tc in &response.tool_calls {
            actions.push(Action::ToolCall(tc.clone()));
        }
        if actions.is_empty() {
            if let Some(ref content) = response.content {
                actions.push(Action::Text(content.clone()));
            }
        }
        actions
    }
}

#[cfg(test)]
mod agent_tests;
```

- [ ] **Step 4: Write meaningful Agent loop tests**

Update `crates/harness-core/src/agent_tests.rs`:

```rust
use super::*;
use crate::types::*;

#[test]
agent_stops_on_finish_stop() {
    let mut agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        content: Some("done".into()),
        tool_calls: vec![],
        finish_reason: FinishReason::Stop,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    assert!(agent.should_stop(&response));
}

#[test]
agent_does_not_stop_on_tool_calls() {
    let mut agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        content: None,
        tool_calls: vec![ToolCall { id: "1".into(), name: "read_file".into(), arguments: serde_json::json!({}) }],
        finish_reason: FinishReason::ToolCalls,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    assert!(!agent.should_stop(&response));
}

#[test]
agent_stops_at_max_turns() {
    let mut agent = Agent::new(AgentConfig { max_turns: 2, max_tools_per_turn: 5 });
    agent.turn_count = 2;
    let response = CompletionResponse {
        content: None,
        tool_calls: vec![ToolCall { id: "1".into(), name: "dummy".into(), arguments: serde_json::json!({}) }],
        finish_reason: FinishReason::ToolCalls,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    assert!(agent.should_stop(&response));
}

#[test]
parse_actions_extracts_tool_calls() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        content: Some("thinking".into()),
        tool_calls: vec![ToolCall { id: "1".into(), name: "read_file".into(), arguments: serde_json::json!({"path": "a.rs"}) }],
        finish_reason: FinishReason::ToolCalls,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    let actions = agent.parse_actions(&response);
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], Action::ToolCall(tc) if tc.name == "read_file"));
}

#[test]
parse_actions_returns_text_when_no_tool_calls() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        content: Some("just text".into()),
        tool_calls: vec![],
        finish_reason: FinishReason::Stop,
        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    let actions = agent.parse_actions(&response);
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], Action::Text(t) if t == "just text"));
}
```

- [ ] **Step 5: Update lib.rs**

Update `crates/harness-core/src/lib.rs`:

```rust
pub mod types;
pub mod error;
pub mod agent;

pub use types::*;
pub use error::*;
pub use agent::{Agent, AgentConfig};

#[cfg(test)]
mod agent_tests;
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p harness-core`
Expected: 5 tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/harness-core/
git commit -m "feat: add Agent main loop with state machine"
```

---

### Task 14: Integration — Wire All Components Together

**Files:**
- Create: `crates/harness-bin/Cargo.toml`
- Create: `crates/harness-bin/src/main.rs`

**Interfaces:**
- Consumes: all crates (core, llm, tools, memory, guard, feedback, config)
- Produces: `run_agent()` function that ties everything together

- [ ] **Step 1: Create harness-bin crate**

Create `crates/harness-bin/Cargo.toml`:

```toml
[package]
name = "coding-agent-harness"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "harness"
path = "src/main.rs"

[dependencies]
harness-core = { path = "../harness-core" }
harness-llm = { path = "../harness-llm" }
harness-tools = { path = "../harness-tools" }
harness-memory = { path = "../harness-memory" }
harness-guard = { path = "../harness-guard" }
harness-feedback = { path = "../harness-feedback" }
harness-config = { path = "../harness-config" }
tokio = { workspace = true }
serde_json = { workspace = true }
dotenv = "0.15"
```

- [ ] **Step 2: Implement main.rs**

Create `crates/harness-bin/src/main.rs`:

```rust
use harness_core::*;
use harness_llm::openai::OpenAiCompatibleProvider;
use harness_llm::LlmProvider;
use harness_tools::{ToolRegistry, read_file::ReadFile, write_file::WriteFile, shell_exec::ShellExec, git_op::GitOp, code_search::CodeSearch};
use harness_guard::{Guardrail, hitl::StdioHitlConfirmer};
use harness_feedback::FeedbackValidator;
use harness_memory::MemoryStore;
use harness_config::HarnessConfig;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let config_str = std::fs::read_to_string("harness.toml").unwrap_or_default();
    let config: HarnessConfig = toml::from_str(&config_str).unwrap_or_default();

    let api_key = std::env::var(&config.llm.api_key_env)
        .unwrap_or_default();

    let llm: Box<dyn LlmProvider> = if config.llm.provider == "mock" {
        todo!("mock mode")
    } else {
        Box::new(OpenAiCompatibleProvider::new(
            api_key,
            config.llm.base_url.clone(),
            config.llm.model.clone(),
        ))
    };

    let mut tools = ToolRegistry::new();
    tools.register(std::sync::Arc::new(ReadFile::new()));
    tools.register(std::sync::Arc::new(WriteFile::new()));
    tools.register(std::sync::Arc::new(ShellExec::new()));
    tools.register(std::sync::Arc::new(GitOp::new()));
    tools.register(std::sync::Arc::new(CodeSearch::new()));

    let guardrail = Guardrail::new_default();
    let feedback = FeedbackValidator::new_default();
    let memory = MemoryStore::new(&config.memory.db_path).unwrap_or_else(|_| MemoryStore::new_in_memory().unwrap());

    let mut agent = Agent::new(AgentConfig {
        max_turns: config.agent.max_turns,
        max_tools_per_turn: config.agent.max_tools_per_turn,
    });

    let task = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: harness <task>");
        std::process::exit(1);
    });

    agent.add_user_message(&task);

    loop {
        let req = harness_llm::CompletionRequest {
            messages: agent.conversation.clone(),
            tools: Some(tools.to_llm_tools()),
            temperature: config.llm.temperature,
            max_tokens: config.llm.max_tokens,
        };

        let response = llm.complete(req).await?;

        if agent.should_stop(&response) {
            if let Some(ref text) = response.content {
                println!("{}", text);
            }
            break;
        }

        agent.add_assistant_message(Message {
            role: Role::Assistant,
            content: response.content.clone().unwrap_or_default(),
            tool_calls: response.tool_calls.clone(),
            tool_call_id: None,
        });

        let actions = agent.parse_actions(&response);
        for action in actions {
            match action {
                Action::ToolCall(tc) => {
                    match guardrail.check(&tc) {
                        harness_guard::GuardrailAction::Block { reason } => {
                            eprintln!("[GUARD] Blocked: {}", reason);
                            agent.add_tool_result(ToolResult {
                                tool_call_id: tc.id.clone(),
                                output: format!("BLOCKED by guardrail: {}", reason),
                                is_error: true,
                            });
                        }
                        harness_guard::GuardrailAction::Allow => {
                            match tools.execute(&tc.name, &tc.arguments).await {
                                Ok(result) => {
                                    let mut result = result;
                                    result.tool_call_id = tc.id.clone();
                                    let fb = feedback.validate(&result);
                                    if !fb.is_success {
                                        eprintln!("[FEEDBACK] {}", fb.summary);
                                    }
                                    agent.add_tool_result(result);
                                }
                                Err(e) => {
                                    agent.add_tool_result(ToolResult {
                                        tool_call_id: tc.id.clone(),
                                        output: format!("Tool error: {}", e),
                                        is_error: true,
                                    });
                                }
                            }
                        }
                    }
                }
                Action::Text(text) => {
                    println!("{}", text);
                }
            }
        }

        agent.turn_count += 1;
    }

    Ok(())
}
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check --workspace`
Expected: OK

- [ ] **Step 4: Build the binary**

Run: `cargo build -p coding-agent-harness`
Expected: Binary at `target/debug/harness`

- [ ] **Step 5: Commit**

```bash
git add crates/harness-bin/
git commit -m "feat: wire all components together in CLI binary"
```

---

### Task 15: Mechanism Demonstrations

**Files:**
- Create: `crates/harness-bin/src/demos.rs`
- Modify: `crates/harness-bin/src/main.rs` (add demos module)

**Interfaces:**
- Consumes: all crates (via harness-bin which depends on everything)
- Produces: 3 deterministic demonstration functions

- [ ] **Step 1: Create demo: guardrail blocks dangerous action**

Add to `crates/harness-bin/src/demos.rs` (new file):

```rust
use harness_core::{ToolCall, ToolResult, FinishReason, Usage, CompletionResponse};
use harness_guard::Guardrail;
use harness_feedback::FeedbackValidator;
use harness_tools::{ToolRegistry, plugin::{Plugin, PluginLoader, PluginContext}};

pub fn demo_guardrail_blocks() {
    let guardrail = Guardrail::new_default();
    let action = ToolCall {
        id: "demo-1".into(),
        name: "shell_exec".into(),
        arguments: serde_json::json!({"command": "rm -rf /"}),
    };

    match guardrail.check(&action) {
        harness_guard::GuardrailAction::Block { reason } => {
            println!("[DEMO 1] Guardrail BLOCKED: {}", reason);
            println!("[DEMO 1] Result: Action was intercepted as expected");
        }
        _ => panic!("Expected block"),
    }
}

pub fn demo_feedback_loop() {
    let validator = FeedbackValidator::new_default();
    let result = ToolResult {
        tool_call_id: "demo-2".into(),
        output: "test result: 3 failed, 2 passed".into(),
        is_error: true,
    };

    let feedback = validator.validate(&result);
    println!("[DEMO 2] Feedback: is_success={}, category={:?}, summary={}",
        feedback.is_success, feedback.category, feedback.summary);
    println!("[DEMO 2] Result: Agent received feedback and can据此修正");
}

pub fn demo_plugin_extension() {
    use async_trait::async_trait;
    use std::sync::Arc;

    struct DemoPlugin;

    #[async_trait]
    impl harness_tools::Tool for DemoPlugin {
        fn name(&self) -> &str { "demo_plugin" }
        fn description(&self) -> &str { "Demo plugin tool" }
        fn parameters_schema(&self) -> serde_json::Value { serde_json::json!({"type":"object","properties":{}}) }
        async fn execute(&self, _args: serde_json::Value) -> ToolResult {
            ToolResult { tool_call_id: String::new(), output: "Plugin executed successfully!".into(), is_error: false }
        }
    }

    #[async_trait]
    impl Plugin for DemoPlugin {
        fn version(&self) -> &str { "0.1.0" }
        fn dependencies(&self) -> Vec<&str> { vec![] }
        fn init(&mut self, _ctx: &PluginContext) -> harness_core::Result<()> { Ok(()) }
    }

    let mut loader = PluginLoader::new();
    loader.register(Arc::new(DemoPlugin));
    let mut registry = ToolRegistry::new();
    loader.load_all(&mut registry).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(registry.execute("demo_plugin", &serde_json::json!({}))).unwrap();
    println!("[DEMO 3] Plugin: {}", result.output);
    println!("[DEMO 3] Result: Custom tool registered and executed at runtime");
}
```

- [ ] **Step 2: Run demos**

Run: `cargo check -p coding-agent-harness`
Expected: Compiles OK

- [ ] **Step 3: Commit**

```bash
git add crates/harness-bin/src/demos.rs
git commit -m "feat: add mechanism demonstration functions"
```

---

### Task 16: Docker + README

**Files:**
- Create: `Dockerfile`
- Create: `README.md`
- Create: `.env.example`
- Create: `harness.toml` (default config)
- Create: `.gitignore` update

**Interfaces:**
- Consumes: compiled binary from Task 14
- Produces: Docker image, documentation

- [ ] **Step 1: Create Dockerfile**

Create `Dockerfile`:

```dockerfile
FROM rust:1.78-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/harness /usr/local/bin/harness
WORKDIR /workspace
ENTRYPOINT ["harness"]
```

- [ ] **Step 2: Create default config**

Create `harness.toml`:

```toml
[agent]
max_turns = 50
max_tools_per_turn = 5
max_fix_rounds = 3

[llm]
provider = "deepseek"
model = "deepseek-chat"
api_key_env = "DEEPSEEK_API_KEY"
base_url = "https://api.deepseek.com"
temperature = 0.0
max_tokens = 4096

[guardrails]
mode = "strict"
timeout_seconds = 60

[memory]
max_context_tokens = 2000
db_path = ".harness/memory.db"

[tools]
allowed = ["read_file", "write_file", "shell_exec", "git_op", "code_search"]
```

- [ ] **Step 3: Create .env.example**

Create `.env.example`:

```
DEEPSEEK_API_KEY=your-api-key-here
```

- [ ] **Step 4: Update .gitignore**

Append to `.gitignore`:

```
.env
.harness/
target/
Cargo.lock
```

- [ ] **Step 5: Create README.md**

Create `README.md`:

```markdown
# Coding Agent Harness

A Rust coding agent harness with extensible tool dispatch, governance guardrails, feedback loop, and memory.

## Quick Start

### Docker

```bash
docker build -t coding-agent-harness .
docker run -it -e DEEPSEEK_API_KEY=xxx -v $(pwd):/workspace coding-agent-harness "your task here"
```

### Local

```bash
cargo build --release
export DEEPSEEK_API_KEY=your-key
./target/release/harness "your task here"
```

## Configuration

Copy `harness.toml` and edit as needed. See default config for all options.

## Key Management

```bash
# Set key (stored in OS keyring)
export DEEPSEEK_API_KEY=your-key

# Key priority: CLI arg > env var > OS keyring
```

## Architecture

- `harness-core`: Agent loop + shared types
- `harness-llm`: LLM abstraction (DeepSeek/OpenAI/Mock)
- `harness-tools`: Tool trait + 6 built-in tools + plugin system
- `harness-memory`: SQLite + FTS5 memory store
- `harness-guard`: Guardrails + HITL
- `harness-feedback`: Test/compile/lint validators
- `harness-config`: TOML configuration

## Testing

```bash
cargo test --workspace
```

All core mechanisms have mock-LLM deterministic unit tests.
```

- [ ] **Step 6: Verify Docker build**

Run: `docker build -t coding-agent-harness .`
Expected: Image builds successfully

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add Dockerfile, README, and default config"
```

---

### Task 17: Final Verification — Run All Tests

- [ ] **Step 1: Run full test suite**

Run: `cargo test --workspace`
Expected: All tests PASS across all crates

- [ ] **Step 2: Verify binary runs**

Run: `cargo run -p coding-agent-harness -- --help` (or with a test task)
Expected: Binary executes without crash

- [ ] **Step 3: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: final test fixes and cleanup"
```
