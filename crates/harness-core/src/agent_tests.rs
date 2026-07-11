use crate::agent::*;
use crate::types::*;

#[test]
fn agent_stops_on_finish_stop() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        message: Message {
            role: Role::Assistant,
            content: "done".into(),
            tool_calls: None,
            tool_call_id: None,
        },
        finish_reason: FinishReason::Stop,
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        },
    };
    assert!(agent.should_stop(&response));
}

#[test]
fn agent_does_not_stop_on_tool_use() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        message: Message {
            role: Role::Assistant,
            content: String::new(),
            tool_calls: None,
            tool_call_id: None,
        },
        finish_reason: FinishReason::ToolUse,
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        },
    };
    assert!(!agent.should_stop(&response));
}

#[test]
fn agent_stops_at_max_turns() {
    let mut agent = Agent::new(AgentConfig {
        max_turns: 2,
        max_tools_per_turn: 5,
    });
    agent.turn_count = 2;
    let response = CompletionResponse {
        message: Message {
            role: Role::Assistant,
            content: String::new(),
            tool_calls: None,
            tool_call_id: None,
        },
        finish_reason: FinishReason::ToolUse,
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        },
    };
    assert!(agent.should_stop(&response));
}

#[test]
fn parse_actions_extracts_tool_calls() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        message: Message {
            role: Role::Assistant,
            content: "thinking".into(),
            tool_calls: Some(vec![ToolCall {
                id: "1".into(),
                name: "read_file".into(),
                arguments: "{\"path\":\"a.rs\"}".into(),
            }]),
            tool_call_id: None,
        },
        finish_reason: FinishReason::ToolUse,
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        },
    };
    let actions = agent.parse_actions(&response);
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], Action::ToolUse(tc) if tc[0].name == "read_file"));
}

#[test]
fn parse_actions_returns_continue_when_no_tool_calls() {
    let agent = Agent::new(AgentConfig::default());
    let response = CompletionResponse {
        message: Message {
            role: Role::Assistant,
            content: "just text".into(),
            tool_calls: None,
            tool_call_id: None,
        },
        finish_reason: FinishReason::Stop,
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        },
    };
    let actions = agent.parse_actions(&response);
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], Action::Continue));
}