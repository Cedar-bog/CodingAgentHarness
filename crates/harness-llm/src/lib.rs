pub mod mock;

#[cfg(test)]
mod mock_tests;

use async_trait::async_trait;
use harness_core::{CompletionResponse, Message, ToolSchema};
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
    fn supports_tools(&self) -> bool {
        true
    }
    fn max_context_tokens(&self) -> usize {
        128000
    }
    fn name(&self) -> &str;
}
