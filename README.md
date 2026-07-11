# Coding Agent Harness

一个用 Rust 编写的编码智能体框架（Coding Agent Harness），提供可扩展的工具分发、治理护栏、反馈闭环和记忆系统。

## 快速开始

### Docker

```bash
docker build -t coding-agent-harness .
docker run -it -e DEEPSEEK_API_KEY=xxx -v $(pwd):/workspace coding-agent-harness "你的任务描述"
```

### 本地运行

```bash
cargo build --release
export DEEPSEEK_API_KEY=你的密钥
./target/release/harness "你的任务描述"
```

## 配置

复制 `harness.toml` 并根据需要编辑。所有配置项均有默认值。

## 密钥管理

```bash
export DEEPSEEK_API_KEY=你的密钥
```

密钥优先级：环境变量 > OS 钥匙串 > 配置文件。

## 架构

| Crate | 功能 |
|-------|------|
| `harness-core` | Agent 主循环 + 共享类型 |
| `harness-llm` | LLM 抽象层（DeepSeek/OpenAI/Mock） |
| `harness-tools` | Tool trait + 5 个内置工具 + 插件系统 |
| `harness-memory` | SQLite 记忆存储 |
| `harness-guard` | 治理护栏 + 人工确认（HITL） |
| `harness-feedback` | 测试/编译/lint 校验器 |
| `harness-config` | TOML 配置管理 |

## 测试

```bash
cargo test --workspace
```

所有核心机制均使用 Mock LLM 编写了确定性单元测试，不依赖网络和真实 API。