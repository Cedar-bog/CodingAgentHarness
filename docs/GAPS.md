# GAPS: 项目未完成项清单

> 对照作业要求（通用要求 + A·Coding Agent Harness 项目文件）与 SPEC.md 验收标准，逐项检查当前项目状态。

---

## 一、关键缺失（影响提交，必须修复）

### 1.1 `REFLECTION.md` 不存在

- **要求**：通用要求 §五.8 — 必须提交 1500–2500 字反思报告
- **当前状态**：完全不存在

### 1.2 HITL 确认流程未接入主循环

- **要求**：SPEC §3.5 — 治理护栏应支持 HITL 确认
- **当前状态**：
  - `StdioHitlConfirmer` 已实现（`hitl.rs`）
  - 但 `GuardrailAction` 只有 `Allow` 和 `Block`，缺少 `RequireApproval` 变体
  - `main.rs` 主循环中未使用 `StdioHitlConfirmer`
- **涉及文件**：`crates/harness-guard/src/lib.rs`、`crates/harness-bin/src/main.rs`

### 1.3 记忆系统未注入 Agent 上下文

- **要求**：SPEC §3.4 — 语义记忆（项目约定）始终注入，情景记忆按相关性注入
- **当前状态**：
  - `MemoryStore` 在主循环中创建但未被使用
  - Agent 构建 LLM 上下文时未调用 `search()` 或 `by_category()` 检索记忆
- **涉及文件**：`crates/harness-bin/src/main.rs`

### 1.4 多轮自我修正未实现

- **要求**：SPEC §3.6 — 最大修正轮次 3 轮，每轮注入上一步的反馈
- **当前状态**：
  - 3 个反馈校验器已实现
  - 但 Agent 主循环中未实现基于反馈的修正循环
  - `max_fix_rounds` 配置项已定义但未使用
- **涉及文件**：`crates/harness-bin/src/main.rs`

### 1.5 Mock 模式为 `todo!()`

- **要求**：SPEC §3.3 — Mock LLM 可替换真实 LLM 运行离线测试
- **当前状态**：`main.rs` 中 `"mock"` provider 分支写的是 `todo!("mock mode")`，无法在无 API key 时运行
- **涉及文件**：`crates/harness-bin/src/main.rs:24`

### 1.6 无 CLI 参数解析

- **要求**：SPEC §7 定义 `harness key show/set/clear` 等子命令
- **当前状态**：`main.rs` 仅用 `args().nth(1)` 读取任务描述，无 `--config` 参数，无子命令
- **涉及文件**：`crates/harness-bin/src/main.rs`

### 1.7 冷启动验证未记录

- **要求**：通用要求 §4.5 — 用第二个不同的 agent，仅凭 SPEC + PLAN 实现 1–2 个 task，记录过程
- **当前状态**：`SPEC_PROCESS.md` 中未提及冷启动验证
- **涉及文件**：`docs/SPEC_PROCESS.md`

---

## 二、修复优先级建议

### P0（阻塞提交，必须先做）

1. 创建 `REFLECTION.md`（1500–2500 字，需学生本人撰写）
2. 创建根目录下 `PLAN.md`、`AGENT_LOG.md`、`SPEC_PROCESS.md` 的文件或链接

### P1（功能不完整，强烈建议修复）

3. 实现 HITL 接入主循环（含 `GuardrailAction::RequireApproval` 变体）
4. 实现记忆注入 Agent 上下文
5. 实现多轮自我修正
6. 实现 Mock 模式（替换 `todo!`）
7. 使用 `clap` 完善 CLI 参数解析
8. 添加冷启动验证记录到 `SPEC_PROCESS.md`

### P2（功能完善，可选）

9. 集成 OS 钥匙串（keyring）+ key 管理 CLI 命令
10. 实现 WebSearch 工具
11. 实现 FTS5 全文搜索