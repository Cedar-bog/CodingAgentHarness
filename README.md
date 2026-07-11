# Coding Agent Harness

一个用 Rust 编写的编码智能体框架（Coding Agent Harness），提供可扩展的工具分发、治理护栏、反馈闭环和记忆系统。

## 快速开始

### 前置要求

- Rust 1.78+
- DeepSeek API key（或其他兼容 OpenAI 格式的 API）

### 配置密钥

创建 `.env` 文件（或复制 `.env.example`）：

```bash
echo "DEEPSEEK_API_KEY=sk-你的密钥" > .env
```

### 运行

```bash
# 运行智能体
cargo run -p coding-agent-harness -- "你的任务描述"

# 或编译后运行
cargo build --release
./target/release/harness "你的任务描述"
```

### 运行演示（无需 API key）

```bash
cargo run --bin demo
```

### Docker

```bash
docker build -t coding-agent-harness .
docker run -it -e DEEPSEEK_API_KEY=sk-你的密钥 -v $(pwd):/workspace coding-agent-harness "你的任务描述"
```

## 配置

编辑 `harness.toml` 可调整所有参数，包括 LLM 供应商、模型、温度、最大轮次等。

默认模型：`deepseek-v4-flash`（可在 `harness.toml` 中修改）。

## 密钥管理

密钥按以下优先级读取：
1. 环境变量 `DEEPSEEK_API_KEY`
2. `.env` 文件（自动加载，UTF-8 编码）
3. 配置文件中的 `api_key_env` 字段

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