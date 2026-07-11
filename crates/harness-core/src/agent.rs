use crate::types::*;

#[derive(Debug, Clone)]
pub struct Agent {
    pub config: AgentConfig,
    pub conversation: Vec<Message>,
    pub turn_count: usize,
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_turns: usize,
    pub max_tools_per_turn: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_turns: 50,
            max_tools_per_turn: 5,
        }
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

    pub fn should_stop(&self, _response: &CompletionResponse) -> bool {
        false
    }

    pub fn parse_actions(&self, _response: &CompletionResponse) -> Vec<Action> {
        vec![]
    }
}

