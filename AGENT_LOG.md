# AGENT_LOG.md

> AI4SE 期末项目 · Coding Agent Harness · 过程日志

---

## 2025-07-12 — 规约与计划生成阶段

### 001 | 项目探索
- **时间**: 会话开始
- **触发技能**: brainstorming
- **操作**: 读取作业要求文件（`AI4SE_Final_Project_A_Coding_Agent_Harness.md`、`AI4SE 期末项目 · 通用要求.md`）及当前代码库（`main.rs` Hello World、空 `Cargo.toml`）
- **发现**: 项目从零开始，需实现完整的 Coding Agent Harness

### 002 | 主要贡献维度决策
- **时间**: brainstorming 提问阶段
- **触发技能**: brainstorming（结构化多选提问）
- **决策**: 选择"扩展性"（工具分发 + 插件系统 + 多agent编排）作为主要贡献维度
- **理由**: 天然由代码构成，移除 LLM 后仍可独立测试，符合 A.4(C)
- **人工干预**: 无，自主决策

### 003 | LLM 供应商决策
- **时间**: brainstorming 提问阶段
- **决策**: DeepSeek 为主，支持多供应商切换
- **采纳建议**: AI 提出使用 OpenAI 兼容格式统一所有供应商，通过 `base_url` 切换
- **人工干预**: 无，采纳 AI 建议

### 004 | 架构方案决策
- **时间**: brainstorming 提问阶段
- **决策**: 方案A（Trait 动态分发）
- **理由**: Rust 惯用、类型安全、运行时可扩展
- **人工干预**: 无，自主决策

### 005 | 设计逐节确认（10节）
- **时间**: brainstorming 设计呈现阶段
- **内容**: 问题陈述 → 系统架构 → Agent主循环 → 工具分发（主要贡献） → LLM抽象层 → 记忆系统 → 治理护栏 → 反馈闭环 → 配置/凭据/分发 → 测试策略
- **人工干预**: 每节确认"没问题，继续"
- **学到的教训**: 逐节确认比"一次性看完"更高效

### 006 | SPEC 写入与自审
- **时间**: 设计确认后
- **触发技能**: brainstorming（spec self-review）
- **操作**: 写入 `docs/SPEC.md`（630行，11节+附录）
- **自审发现**: §3.7 提到"YAML/TOML"但 §9 只说 TOML → 统一为 TOML
- **commit**: `2e3d07e` — `docs: add SPEC.md`
- **人工干预**: 无

### 007 | PLAN 写入与自审
- **时间**: SPEC 提交后
- **触发技能**: writing-plans
- **操作**: 写入 `docs/PLAN.md`（3426行，17个TDD任务）
- **自审发现**:
  - Task 13 Agent struct 引用其他 crate 类型 → 将 `CompletionResponse` 移到 `harness-core::types`
  - Task 15 demos 放在 harness-core 但依赖多 crate → 移到 harness-bin
- **commit**: `406405c` — `docs: add PLAN.md`
- **人工干预**: 无

### 008 | SPEC_PROCESS 写入
- **时间**: PLAN 提交后
- **触发技能**: brainstorming（过程文档）
- **操作**: 写入 `docs/SPEC_PROCESS.md`（170行）
- **commit**: `5055dda` — `docs: add SPEC_PROCESS.md`
- **人工干预**: 无

### 009 | 冷启动验证
- **时间**: SPEC_PROCESS 写入后
- **操作**: 由 OpenCode (big-pickle) 作为"陌生智能体"执行 Task 1 和 Task 2
- **结果**: 两个 task 均成功，但发现 3 个问题
- **发现的问题**:
  1. `MockLlmProvider.responses` 缺少 `Arc<Mutex<>>`（interior mutability 编译错误）
  2. `catch_unwind` 测试与实现行为不匹配（返回 Err 而非 panic）
  3. SPEC 缺少 `CompletionRequest` 结构体定义
- **人工干预**: 无

### 010 | 冷启动问题修复
- **时间**: 冷启动验证后
- **操作**: 修复 PLAN 和 SPEC
- **修复内容**:
  - PLAN Task 2: `responses` 改为 `Arc<Mutex<VecDeque<...>>>`
  - PLAN Task 2: 测试改为 `assert!(result.is_err())`
  - SPEC §3.3: 补充 `CompletionRequest` struct 定义
- **commit**: `5eb32b7` — `fix: address cold start validation findings`
- **人工干预**: 无

---

## 2025-07-12 — 实现阶段

### 011 | Task 0: GitHub Actions CI Setup
- **时间**: 实现阶段开始
- **操作**: 创建 `.github/workflows/ci.yml`，配置 unit-test job
- **CI 内容**: push/PR 触发 → checkout → Rust toolchain → cargo test --workspace
- **分支**: `feat/ci-setup` → 合并到 master
- **commit**: `1aa8f79` — `ci: add GitHub Actions workflow with unit-test job`
- **CI 验证**: 通过（仓库已公开，可直接查看 Actions 页面）
- **人工干预**: 无

### 012 | Task 1: Workspace + Shared Types
- **时间**: Task 0 完成后
- **触发技能**: subagent-driven-development, using-git-worktrees
- **操作**: 创建分支 `feat/task-1-workspace-types`，派发 subagent 实现
- **subagent**: general-purpose agent
- **结果**: DONE
- **产出**:
  - 根 `Cargo.toml` 转为 workspace
  - `harness-core` crate：`types.rs`（10个共享类型）+ `error.rs`（HarnessError + Result）
  - `src/main.rs` 已删除
- **CI 验证**: 分支推送 CI 通过（33s），合并后 master CI 通过（31s）
- **commit**: `c6b7706` — `feat: convert to cargo workspace, add shared types and error types`
- **合并**: `feat/task-1-workspace-types` → `master`（`--no-ff`）
- **人工干预**: 无

### 013 | Task 2: LLM Abstraction Layer + Mock Provider
- **时间**: Task 1 完成后
- **触发技能**: subagent-driven-development
- **操作**: 创建分支 `feat/task-2-llm-abstraction`，分 RED/GREEN 两阶段派发 subagent
- **TDD 过程**:
  - RED: subagent 写 `mock_tests.rs` + `lib.rs`（无 mock.rs）→ commit `3fd25e7` → push → CI #14 **failure** ✅
  - GREEN: subagent 写 `mock.rs` → commit `5420ca6` → push → CI #15 **success** ✅
- **产出**: `harness-llm` crate（LlmProvider trait + MockLlmProvider + CompletionRequest）
- **CI 验证**: RED 正确失败，GREEN 正确通过
- **commit**: `3fd25e7` (RED) + `5420ca6` (GREEN)
- **合并**: `feat/task-2-llm-abstraction` → `master`（`--no-ff`）
- **人工干预**: 无
- **教训**: subagent 两个 push 太快会导致 CI 只跑最终状态，必须分两次独立 dispatch

### 014 | Task 3: OpenAI-Compatible LLM Provider
- **时间**: Task 2 完成后
- **操作**: 创建分支 `feat/task-3-openai-provider`，分 RED/GREEN 两阶段
- **RED**: commit `66787b8` → CI #18 failure ✅
- **GREEN**: commit `0d30ff1` → CI #19 success ✅
- **产出**: `OpenAiCompatibleProvider`（`openai.rs`），5 tests passing
- **合并**: `feat/task-3-openai-provider` → `master`
- **人工干预**: 无

### 015 | Task 4: Tool Trait + ToolRegistry
- **时间**: Task 3 完成后
- **操作**: 创建分支 `feat/task-4-tool-registry`，分 RED/GREEN
- **RED**: commit `c1346ce` → CI #22 failure ✅
- **GREEN**: commit `d6d5931` → CI #23 success ✅
- **产出**: `harness-tools` crate（Tool trait + ToolRegistry + ToolInfo），5 tests
- **注意**: 适配实际类型（`ToolResult.content` 而非 `output` 等）
- **合并**: `feat/task-4-tool-registry` → `master`
- **人工干预**: 无

### 016 | Task 5: ReadFile + WriteFile Tools
- **时间**: Task 4 完成后
- **操作**: 创建分支 `feat/task-5-read-write-tools`，分 RED/GREEN
- **RED**: commit `9de5e1e` → CI #26 failure ✅
- **GREEN**: commit `2d517b4` → CI #27 success ✅
- **产出**: ReadFile（支持 offset/limit）+ WriteFile（支持父目录创建），9 tests
- **注意**: subagent 纠正了测试路径（`crates/harness-core/` → `../harness-core/`）
- **合并**: `feat/task-5-read-write-tools` → `master`
- **人工干预**: 无

### 017 | Task 6: ShellExec + GitOp + CodeSearch Tools
- **时间**: Task 5 完成后
- **操作**: 创建分支 `feat/task-6-shell-git-search`，分 RED/GREEN
- **RED**: commit `9949ee2` → CI #30 failure ✅
- **GREEN**: commit `2643fdb` → CI #31 success ✅
- **产出**: 3 个工具（ShellExec/GitOp/CodeSearch），13 tests
- **合并**: `feat/task-6-shell-git-search` → `master`
- **人工干预**: 无

---

## 统计

| 指标 | 值 |
|------|-----|
| 总 commit 数 | 18 |
| 实现阶段 task 完成数 | 7/18（Task 0, 1, 2, 3, 4, 5, 6） |
| brainstorming 提问轮次 | 6（维度/供应商/架构/工具/护栏/记忆） |
| 设计迭代轮次 | 3（工具集/护栏范围/记忆深度） |
| AI 建议采纳 | 4（OpenAI格式/三层记忆/Arc/CompletionResponse位置） |
| AI 建议推翻 | 2（配置格式/Demos位置） |
| 冷启动发现问题 | 3 |
| 冷启动问题修复 | 3 |
| 人工干预次数 | 0（所有决策自主或采纳AI建议） |
