use harness_core::*;
use harness_llm::openai::OpenAiCompatibleProvider;
use harness_llm::{LlmProvider, CompletionRequest};
use harness_tools::ToolRegistry;
use harness_tools::read_file::ReadFile;
use harness_tools::write_file::WriteFile;
use harness_tools::shell_exec::ShellExec;
use harness_tools::git_op::GitOp;
use harness_tools::code_search::CodeSearch;
use harness_guard::Guardrail;
use harness_feedback::FeedbackValidator;
use harness_memory::MemoryStore;
use harness_config::HarnessConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let config_str = std::fs::read_to_string("harness.toml").unwrap_or_default();
    let config: HarnessConfig = toml::from_str(&config_str).unwrap_or_default();

    let api_key = std::env::var(&config.llm.api_key_env).unwrap_or_default();

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
    tools.register(Box::new(ReadFile::new()));
    tools.register(Box::new(WriteFile::new()));
    tools.register(Box::new(ShellExec::new()));
    tools.register(Box::new(GitOp::new()));
    tools.register(Box::new(CodeSearch::new()));

    let guardrail = Guardrail::new_default();
    let feedback = FeedbackValidator::new_default();
    let _memory = MemoryStore::new(&config.memory.db_path)
        .unwrap_or_else(|_| MemoryStore::new_in_memory().unwrap());

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
        let req = CompletionRequest {
            messages: agent.conversation.clone(),
            tools: Some(tools.to_llm_tools()),
            temperature: config.llm.temperature,
            max_tokens: config.llm.max_tokens,
        };

        let response = llm.complete(req).await?;

        if agent.should_stop(&response) {
            println!("{}", response.message.content);
            break;
        }

        agent.add_assistant_message(response.message.clone());

        let actions = agent.parse_actions(&response);
        for action in actions {
            match action {
                Action::ToolUse(tool_calls) => {
                    for tc in tool_calls {
                        let tool_action = ToolCall { id: tc.id.clone(), name: tc.name.clone(), arguments: tc.arguments.clone() };
                        match guardrail.check(&tool_action) {
                            harness_guard::GuardrailAction::Block { reason } => {
                                eprintln!("[GUARD] Blocked: {}", reason);
                                agent.add_tool_result(ToolResult {
                                    tool_call_id: tc.id.clone(),
                                    content: format!("BLOCKED by guardrail: {}", reason),
                                    is_error: true,
                                });
                            }
                            harness_guard::GuardrailAction::Allow => {
                                let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(serde_json::Value::Null);
                                match tools.execute(&tc.name, &args).await {
                                    Ok(mut result) => {
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
                                            content: format!("Tool error: {}", e),
                                            is_error: true,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                Action::Continue => {
                    // LLM provided text response, already added to conversation via add_assistant_message
                }
                Action::Finish(_) => {
                    break;
                }
            }
        }

        agent.turn_count += 1;
    }

    Ok(())
}