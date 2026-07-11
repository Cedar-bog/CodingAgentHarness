use crate::CompletionRequest;
use crate::LlmProvider;
use async_trait::async_trait;
use harness_core::CompletionResponse;
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
            .ok_or_else(|| harness_core::HarnessError::Provider("No preset responses left".into()))
    }

    fn name(&self) -> &str {
        "mock"
    }
}
