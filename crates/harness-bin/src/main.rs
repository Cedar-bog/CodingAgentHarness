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
use harness_guard::hitl::{StdioHitlConfirmer, HitlConfirmer};
use harness_feedback::FeedbackValidator;
use harness_memory::MemoryStore;
use harness_config::HarnessConfig;
use clap::Parser;

#[derive(clap::Parser)]
struct Cli {
    /// Task description
    task: String,
    /// Path to config file
    #[arg(short, long, default_value = "harness.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();
    let config_str = std::fs::read_to_string(&cli.config).unwrap_or_default();
    let config: HarnessConfig = toml::from_str(&config_str).unwrap_or_default();

    let api_key = std::env::var(&config.llm.api_key_env).unwrap_or_default();

    let llm: Box<dyn LlmProvider> = if config.llm.provider == "mock" {
        Box::new(harness_llm::mock::MockLlmProvider::new(vec![]))
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
    let hitl = StdioHitlConfirmer;
    let feedback = FeedbackValidator::new_default();
    let memory = MemoryStore::new(&config.memory.db_path)
        .unwrap_or_else(|_| MemoryStore::new_in_memory().unwrap());

    let mut agent = Agent::new(AgentConfig {
        max_turns: config.agent.max_turns,
        max_tools_per_turn: config.agent.max_tools_per_turn,
    });

    let task = cli.task;

    agent.add_user_message(&task);

    // Inject semantic memory by category at the start
    for category in &["project_convention", "user_preference"] {
        if let Ok(entries) = memory.by_category(category) {
            for entry in &entries {
                agent.conversation.push(Message {
                    role: Role::System,
                    content: format!("[Memory: {} = {}]", entry.key, entry.value),
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
        }
    }

    let mut fix_rounds_used = 0usize;

    loop {
        // Inject relevant memory entries based on the latest user message
        if let Ok(entries) = memory.search(&task, 5) {
            for entry in &entries {
                let mem_msg = format!("[Memory: {} = {}]", entry.key, entry.value);
                if !agent.conversation.iter().any(|m| m.content == mem_msg) {
                    agent.conversation.push(Message {
                        role: Role::System,
                        content: mem_msg,
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }
            }
        }

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

        let mut had_failure = false;
        let actions = agent.parse_actions(&response);
        for action in actions {
            match action {
                Action::ToolUse(tool_calls) => {
                    for tc in tool_calls {
                        let tool_action = ToolCall { id: tc.id.clone(), name: tc.name.clone(), arguments: tc.arguments.clone() };
                        match guardrail.check(&tool_action) {
                            harness_guard::GuardrailAction::Block { reason } => {
                                eprintln!("[GUARD] Blocked: {}", reason);
                                let action = harness_guard::GuardrailAction::Block { reason: reason.clone() };
                                if hitl.confirm(&action).await.unwrap_or(false) {
                                    // HITL approved — execute anyway
                                    let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(serde_json::Value::Null);
                                    match tools.execute(&tc.name, &args).await {
                                        Ok(mut result) => {
                                            result.tool_call_id = tc.id.clone();
                                            let fb = feedback.validate(&result);
                                            if !fb.is_success {
                                                eprintln!("[FEEDBACK] {}", fb.summary);
                                                had_failure = true;
                                            }
                                            agent.add_tool_result(result);
                                        }
                                        Err(e) => {
                                            had_failure = true;
                                            agent.add_tool_result(ToolResult {
                                                tool_call_id: tc.id.clone(),
                                                content: format!("Tool error: {}", e),
                                                is_error: true,
                                            });
                                        }
                                    }
                                } else {
                                    had_failure = true;
                                    agent.add_tool_result(ToolResult {
                                        tool_call_id: tc.id.clone(),
                                        content: format!("BLOCKED by guardrail: {}", reason),
                                        is_error: true,
                                    });
                                }
                            }
                            harness_guard::GuardrailAction::RequireApproval { reason } => {
                                let action = harness_guard::GuardrailAction::RequireApproval { reason: reason.clone() };
                                if hitl.confirm(&action).await.unwrap_or(false) {
                                    let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(serde_json::Value::Null);
                                    match tools.execute(&tc.name, &args).await {
                                        Ok(mut result) => {
                                            result.tool_call_id = tc.id.clone();
                                            let fb = feedback.validate(&result);
                                            if !fb.is_success {
                                                eprintln!("[FEEDBACK] {}", fb.summary);
                                                had_failure = true;
                                            }
                                            agent.add_tool_result(result);
                                        }
                                        Err(e) => {
                                            had_failure = true;
                                            agent.add_tool_result(ToolResult {
                                                tool_call_id: tc.id.clone(),
                                                content: format!("Tool error: {}", e),
                                                is_error: true,
                                            });
                                        }
                                    }
                                } else {
                                    had_failure = true;
                                    agent.add_tool_result(ToolResult {
                                        tool_call_id: tc.id.clone(),
                                        content: format!("BLOCKED by guardrail: {}", reason),
                                        is_error: true,
                                    });
                                }
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

        if had_failure && fix_rounds_used < config.agent.max_fix_rounds {
            fix_rounds_used += 1;
            agent.conversation.push(Message {
                role: Role::System,
                content: format!("The previous tool calls had failures. Please fix the issues and try again. (Fix round {}/{})", fix_rounds_used, config.agent.max_fix_rounds),
                tool_calls: None,
                tool_call_id: None,
            });
            continue;
        }
    }

    Ok(())
}