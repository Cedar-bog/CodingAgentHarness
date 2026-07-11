# SPEC_PROCESS.md: Brainstorming 与规约生成过程记录

> AI4SE 期末项目 · Coding Agent Harness

---

## 一、Brainstorming 关键节点

### 1.1 初始探索

OpenCode 首先读取了作业要求文件（`AI4SE_Final_Project_A_Coding_Agent_Harness.md` 和 `AI4SE 期末项目 · 通用要求.md`），以及当前代码库状态（只有一个 `Hello, world!` 的 `main.rs` 和空的 `Cargo.toml`）。

**关键发现**：项目是从零开始，需要完成一个完整的 Coding Agent Harness。

### 1.2 关键决策：主要贡献维度

OpenCode 提出了三个可选的主要贡献维度：
- **治理**（护栏/沙箱/HITL状态机/范围围栏）
- **反馈闭环**（确定性校验器+失败分类+多轮自我修正）
- **扩展性**（工具分发/多agent编排/插件系统）

**我的决策**：选择了**扩展性**。

**为什么**：扩展性维度天然由代码构成（工具注册表、插件加载器），不依赖 prompt，移除 LLM 后仍可独立测试，符合 A.4(C) 的硬标准。同时，trait 系统和依赖解析有足够的工程深度。

### 1.3 关键决策：LLM 供应商

**我的决策**：DeepSeek 为主，支持多供应商切换。

**AI 的建议**：OpenAI 兼容格式是最佳选择，因为 DeepSeek、OpenAI、Ollama 都使用相同的 API 格式。我采纳了这个建议——统一使用 OpenAI 兼容格式，通过 `base_url` 和 `model` 参数切换供应商。

### 1.4 关键决策：架构方案

OpenCode 提出了三种方案：
- **方案A：Trait动态分发**（推荐）
- **方案B：消息传递+Channel**
- **方案C：编译期宏注册**

**我的决策**：选择方案A。

**原因**：Rust 惯用、类型安全、运行时可扩展。方案B 复杂度太高，方案C 运行时灵活性不够。

---

## 二、至少 3 轮关键迭代

### 迭代 1：工具集选择

**OpenCode 提问**：你的 coding agent 需要支持哪些工具操作？

**我的回答**：全部都要（读文件、写文件、Shell执行、Git操作、网页搜索、代码搜索）。

**AI 追问**：网页搜索需要外部 API（如 Google Search API），这会增加凭据管理复杂度。是否确定需要？

**我的决策**：保留。网页搜索在 coding agent 场景中有实际价值（搜索文档、Stack Overflow），且工具注册机制天然支持按需启用/禁用。

### 迭代 2：治理护栏范围

**OpenCode 提问**：你的护栏需要拦截哪些危险动作？

**我的回答**：全部都要（删除文件/目录、系统级命令、对外推送、网络请求、凭据泄露）。

**AI 追问**：全部拦截意味着每个 shell 命令都需要经过规则检查，可能影响性能。是否需要分级处理？

**我的决策**：采用分级规则设计——每个规则独立实现 `GuardrailRule` trait，Guardrail 按顺序执行所有规则。性能影响可忽略（正则匹配 < 1ms）。

### 迭代 3：记忆系统深度

**OpenCode 提问**：跨会话记忆需要存储哪些信息？

**我的回答**：全部（项目约定、历史决策、代码库知识、用户偏好、会话历史）。

**AI 追问**：全部存储意味着更大的 schema 和更复杂的检索逻辑。建议分三层：工作记忆（当前会话）、情景记忆（历史摘要）、语义记忆（结构化知识）。是否同意？

**我的决策**：采纳三层架构。这个分层清晰，每层职责单一，SQLite + FTS5 足够支撑。

---

## 三、AI 建议采纳与修正

### 采纳的建议

1. **OpenAI 兼容格式统一**：所有 LLM 供应商使用相同的 API 格式，通过 `base_url` 切换。这大大简化了 LLM 抽象层。

2. **三层记忆架构**：工作记忆/情景记忆/语义记忆的分层比扁平存储更清晰，检索效率更高。

3. **Arc\<dyn Tool\> 而非 Box\<dyn Tool\>**：在 Task 4 的实现中，OpenCode 发现 `Box<dyn Tool>` 无法在 execute 时共享所有权，建议改为 `Arc<dyn Tool>`。这是一个重要的 Rust 所有权问题，直接采纳。

4. **CompletionResponse 放在 harness-core**：OpenCode 在 self-review 中发现 Agent struct 引用了其他 crate 的类型，建议将 `CompletionResponse` 移到 `harness-core::types` 以保持依赖单向。这避免了循环依赖。

### 推翻/修正的建议

1. **配置文件格式**：OpenCode 最初在 §3.7 提到"YAML/TOML"，在 §9 只写了 TOML。我在 self-review 中统一为 TOML——Rust 生态对 TOML 支持更好（`toml` crate），且配置文件不需要 YAML 的复杂特性。

2. **Demos 位置**：OpenCode 最初将 demos 放在 `harness-core/src/demos.rs`，但 demos 使用了多个 crate 的类型（Guardrail、FeedbackValidator 等）。我修正为放在 `harness-bin/src/demos.rs`，因为二进制 crate 依赖所有库 crate。

---

## 四、Brainstorming 技能反思

### 做得好的地方

1. **结构化提问**：OpenCode 用多选题代替开放题，降低了决策负担。例如主要贡献维度、工具集、护栏范围都用了选项列表。

2. **逐节确认**：设计分 10 节呈现，每节确认后再继续。这避免了"一次性看完全部设计再修改"的高成本。

3. **Self-review 机制**：在写完 spec 后自动检查一致性问题（如 YAML/TOML 不一致），并在写完 plan 后检查类型一致性（如 CompletionResponse 位置问题）。这些 self-review 发现了实际的架构问题。

4. **方案对比**：在架构选择时提出 3 种方案并列出 trade-off，而不是直接给出"最佳方案"。这让决策过程更透明。

### 不满意的地方

1. **缺少可视化**：对于架构图和数据流，纯文本 ASCII 图不够直观。如果能用 Mermaid 或 SVG 展示组件关系会更好。

2. **Plan 粒度可能过细**：17 个 task 中，有些 task 只是"创建 crate 目录 + 写 Cargo.toml"，可能过于碎片化。理想情况下，相关的小步骤应该合并。

3. **缺少风险评估深度**：Plan 中的风险评估（§10）比较简略。例如"LLM 响应格式不稳定"应该有更具体的缓解方案（如 schema validation、fallback parsing）。

---

## 五、关键对话节选

### 节选 1：主要贡献维度选择

> **OpenCode**: 分配作业要求你选一个"主要贡献"维度深入实现。你想在哪个维度投入最深？
>
> **我**: 扩展性
>
> **OpenCode**: 好，扩展性（工具分发 + 多agent编排 + 插件系统）作为主要贡献。

### 节选 2：架构方案选择

> **OpenCode**: 提出三种架构方案：
> - 方案A：Trait动态分发（推荐）— Rust惯用，运行时可扩展
> - 方案B：消息传递 — 并发好但复杂度高
> - 方案C：编译期宏 — 最快但不够灵活
>
> **我**: 方案A
>
> **OpenCode**: 好，方案A。现在我逐节呈现设计，每节确认后再继续。

### 节选 3：Self-review 发现问题

> **OpenCode** (self-review): 发现一个不一致：第3.7节提到"YAML/TOML"，但第9节只说TOML。统一为TOML。
>
> **我**: 没问题，继续。

### 节选 4：Plan 中的类型一致性修复

> **OpenCode** (plan self-review): Task 13 的 Agent struct 引用了 `CompletionResponse`，但该类型在 harness-llm 中定义。harness-core 不应依赖 harness-llm。建议将 `CompletionResponse` 移到 `harness-core::types`。
>
> **我**: （OpenCode 自动修正了这个问题，无需人工干预）

---

## 六、对 Superpowers 方法论的批判性思考

### 假设与现实

1. **假设："用户有清晰的初始想法"**
   - 现实：我最初只有一个模糊的概念（"做一个 coding agent"），是 OpenCode 的结构化提问帮助我澄清了具体需求。
   - 反思：brainstorming 技能在"从模糊到清晰"的过程中发挥了最大价值。

2. **假设："设计可以一次成型"**
   - 现实：设计经历了多轮迭代（工具集、护栏范围、记忆深度），每次迭代都修正了之前的过度简化。
   - 反思：逐节确认的模式比"看完再改"更高效。

3. **假设："spec 和 plan 可以线性生成"**
   - 现实：plan 的 self-review 发现了 spec 中的架构问题（类型依赖方向），需要回头修改 spec。
   - 反思：spec 和 plan 应该交替迭代，而非严格串行。
