# Task 2: LLM Abstraction Layer + Mock Provider — Report

## What Was Implemented

- `harness-llm` crate with:
  - `LlmProvider` async trait (`complete`, `supports_tools`, `max_context_tokens`, `name`)
  - `CompletionRequest` struct (messages, tools, temperature, max_tokens)
  - `MockLlmProvider` (preset responses via VecDeque, records all calls, returns error when exhausted)

## TDD Evidence

### RED (commit `a1b8fa4`)
Compilation failed as expected:
```
error[E0583]: file not found for module `mock`
 --> crates\harness-llm\src\lib.rs:1:1
```

### GREEN (commit `64462dc`)
All 3 tests pass:
```
test mock_tests::mock_records_all_requests ... ok
test mock_tests::mock_returns_error_when_no_responses_left ... ok
test mock_tests::mock_returns_preset_responses_in_order ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Files Created

| File | Purpose |
|------|---------|
| `crates/harness-llm/Cargo.toml` | Crate manifest with harness-core, async-trait, reqwest, tokio deps |
| `crates/harness-llm/src/lib.rs` | `CompletionRequest`, `LlmProvider` trait, module declarations |
| `crates/harness-llm/src/mock.rs` | `MockLlmProvider` implementation |
| `crates/harness-llm/src/mock_tests.rs` | 3 tests: ordered responses, call recording, empty error |

## Commits

1. `a1b8fa4` — `test(harness-llm): add mock provider tests (RED)`
2. `64462dc` — `feat(harness-llm): add MockLlmProvider implementation (GREEN)`

## Adaptation Note

The task brief assumed `CompletionResponse` had `content: Option<String>` and `tool_calls: Vec<ToolCall>` fields directly. The actual `harness-core` types use `message: Message` where `Message` contains `content: String` and `tool_calls: Option<Vec<ToolCall>>`. Tests were adapted accordingly — all assertions verify the same behavior through the real type structure.
