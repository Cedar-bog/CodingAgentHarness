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
export DEEPSEEK_API_KEY=your-key
```

## Architecture

- `harness-core`: Agent loop + shared types
- `harness-llm`: LLM abstraction (DeepSeek/OpenAI/Mock)
- `harness-tools`: Tool trait + 6 built-in tools + plugin system
- `harness-memory`: SQLite memory store
- `harness-guard`: Guardrails + HITL
- `harness-feedback`: Test/compile/lint validators
- `harness-config`: TOML configuration

## Testing

```bash
cargo test --workspace
```

All core mechanisms have mock-LLM deterministic unit tests.