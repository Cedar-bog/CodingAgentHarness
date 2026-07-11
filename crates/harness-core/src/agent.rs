use crate::types::*;
use crate::error::Result;

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
            tool_calls: None,
            tool_call_id: None,
        });
    }

    pub fn add_assistant_message(&mut self, msg: Message) {
        self.conversation.push(msg);
    }

    pub fn add_tool_result(&mut self, result: ToolResult) {
        self.conversation.push(Message {
            role: Role::Tool,
            content: result.content,
            tool_calls: None,
            tool_call_id: Some(result.tool_call_id),
        });
    }

    pub fn should_stop(&self, response: &CompletionResponse) -> bool {
        response.finish_reason == FinishReason::Stop || self.turn_count >= self.config.max_turns
    }

    pub fn parse_actions(&self, response: &CompletionResponse) -> Vec<Action> {
        if let Some(ref tool_calls) = response.message.tool_calls {
            if !tool_calls.is_empty() {
                return vec![Action::ToolUse(tool_calls.clone())];
            }
        }
        vec![Action::Continue]
    }
}

#[cfg(test)]
#[path = "agent_tests.rs"]
mod agent_tests;