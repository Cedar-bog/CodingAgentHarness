# REFLECTION.md: 项目反思报告

> AI4SE 期末项目 · A · Coding Agent Harness

---

## 一、项目概述

### 做了什么

本项目实现了一个用 Rust 编写的 Coding Agent Harness——一个可扩展、可测试的编码智能体框架。它不是一个"在 prompt 里写规则"的 agent，而是一个将 Agent 的核心机制（主循环、工具分发、治理护栏、反馈闭环、记忆系统、配置管理）全部落实为可独立测试的代码的工程框架。

项目采用 Cargo Workspace 多 crate 架构，包含 7 个库 crate 和 1 个二进制 crate：

- **harness-core**：Agent 主循环状态机 + 共享数据类型（Message、Action、ToolCall 等）
- **harness-llm**：LLM 抽象层，支持 DeepSeek、OpenAI 兼容格式和 Mock 模式
- **harness-tools**：Tool trait + 5 个内置工具（ReadFile、WriteFile、ShellExec、GitOp、CodeSearch）+ 插件系统
- **harness-memory**：SQLite 驱动的三层记忆架构（语义记忆、情景记忆）
- **harness-guard**：5 条内置治理规则 + 人工确认（HITL）机制
- **harness-feedback**：3 个确定性校验器（测试失败、编译错误、lint 警告）
- **harness-config**：TOML 配置文件 + 环境变量配置管理

### 主要贡献

我选择**扩展性**作为主要贡献维度。具体来说：

1. **Tool trait 分发系统**：通过 `#[async_trait]` 定义 `Tool` trait，`ToolRegistry` 负责注册、查找、执行工具。新增工具只需实现 `Tool` trait 并调用 `register()`，无需修改核心代码。
2. **Plugin trait + PluginLoader**：支持从外部动态加载插件，插件通过 `Plugin` trait 注册自己的工具集。
3. **治理规则可插拔**：`GuardrailRule` trait 让新增规则只需实现 `check()` 方法，无需修改状态机。
4. **反馈校验器可插拔**：`ValidationRule` trait 让新增校验器只需实现 `validate()` 方法。

最终成果：43 个单元测试全部通过，所有核心机制都不依赖真实 LLM API，可以在离线环境用 Mock LLM 做确定性测试。这完全满足了作业 A.4(C) 的要求。

---

## 二、项目过程记录

### 阶段一：Brainstorming 与设计（约 2 小时）

项目从零开始——只有一个 `Hello, world!` 的 `main.rs`。OpenCode 首先读取了作业要求，然后通过结构化提问帮助我澄清需求。

关键决策节点：

1. **主要贡献维度选择**：OpenCode 提出了三个选项——治理、反馈闭环、扩展性。我选择了扩展性，因为它的核心价值由代码构成（工具注册表、插件加载器），不依赖 prompt，移除 LLM 后仍可独立测试。
2. **LLM 供应商选择**：我决定用 DeepSeek。OpenCode 建议使用 OpenAI 兼容格式统一接口，这样 DeepSeek、OpenAI、Ollama 都能用同一套代码。我采纳了。
3. **架构方案选择**：OpenCode 提出了三种方案——Trait 动态分发、消息传递 + Channel、编译期宏注册。我选择了方案 A（Trait 动态分发），因为 Rust 惯用、类型安全、运行时可扩展。
4. **工具集范围**：我要了全部工具（读文件、写文件、Shell、Git、代码搜索、网页搜索）。OpenCode 提醒网页搜索需要外部 API 和凭据管理，我仍决定保留。
5. **治理护栏范围**：我要求拦截所有危险操作（删除、系统命令、推送、网络请求、凭据泄露）。OpenCode 建议采用分级规则设计，每个规则独立实现 `GuardrailRule` trait。
6. **记忆系统深度**：我要了全部记忆类型。OpenCode 建议分三层——工作记忆、情景记忆、语义记忆。我采纳了。

设计过程采取逐节确认模式：OpenCode 每次呈现一节设计（问题陈述、用户故事、功能规约、数据模型、架构图、接口设计、配置、部署），我确认后再继续。这避免了"一次性看完再修改"的高认知成本。

### 阶段二：SPEC 编写（约 1 小时）

OpenCode 在 Brainstorming 后自动生成了 `docs/SPEC.md`（637 行），包含 10 节完整设计。写完后触发 self-review，发现了两个问题：

1. 配置格式不一致：§3.7 提到"YAML/TOML"，但 §9 只写了 TOML。统一为 TOML。
2. 类型依赖方向错误：`CompletionResponse` 定义在 `harness-llm` 中，但 `Agent` struct 在 `harness-core` 中引用了它。OpenCode 自动修正，将 `CompletionResponse` 移到 `harness-core::types`。

这两个 self-review 发现的问题都是真实的架构问题，如果直接进入编码阶段再发现，返工成本会高得多。

### 阶段三：PLAN 编写（约 1 小时）

OpenCode 将 SPEC 拆解为 17 个 TDD task，每个 task 包含 RED（写测试）→ GREEN（实现）→ 文档更新的完整流程。Task 0 是 CI 配置，Task 1-16 按依赖关系排列：

- Task 1-2：Workspace + 共享类型 + LLM 抽象层
- Task 3-4：OpenAI Provider + Tool Registry
- Task 5-6：内置工具实现
- Task 7-8：插件系统 + 治理护栏
- Task 9-11：HITL + 反馈校验器 + 记忆系统
- Task 12-14：配置管理 + Agent 主循环 + 集成
- Task 15-16：演示函数 + Docker/README

Plan 阶段触发了第二次 self-review，发现 `Agent` struct 引用了 `harness-llm` 的 `CompletionResponse`，导致 `harness-core` 依赖了 `harness-llm`。修正后保持了依赖单向性。

### 阶段四：编码实现（约 3 天，分散进行）

编码阶段严格遵循 TDD 流程：每个 task 先在独立分支上开发，先写测试（RED），再写实现（GREEN），然后合并到主分支。

**冷启动验证**：在实现早期，OpenCode 用了一个"陌生 agent"（big-pickle 模型），仅凭 SPEC + PLAN 尝试实现 Task 1 和 Task 2。这个验证发现了 PLAN 中的两处代码错误——`MockLlmProvider.responses` 缺少 `Arc<Mutex<>>` 导致编译错误，以及测试代码使用 `catch_unwind` 与实际行为不匹配。这些发现证明了冷启动验证的价值：PLAN 中的"代码即规约"方式存在风险，规约本身可能有 bug。

**关键修复**：在实现过程中，通过实际的测试和编译发现了多个问题，每个都在独立分支上修复并合并：

1. 工具类型字段缺失：`ToolSchema` 缺少 `type` 字段，导致 API 请求格式错误
2. .env 文件加载：环境变量未在 `main.rs` 中加载
3. API 错误处理：HTTP 状态码未检查，导致 API 错误时 JSON 解析失败
4. 工具所有权问题：`Box<dyn Tool>` 无法在 `execute` 时共享所有权，改为 `Arc<dyn Tool>`
5. 默认模型调整：从默认值改为 `deepseek-v4-flash`

**后期补齐**：在项目后期，我检查了 GAPS 清单，发现 5 个功能缺口：

1. Mock 模式是 `todo!()`——修复为使用 `MockLlmProvider`
2. HITL 未接入主循环——加入 `StdioHitlConfirmer` 和 `GuardrailAction::RequireApproval`
3. 记忆未注入 Agent 上下文——在每次 LLM 调用前搜索并注入相关记忆
4. 多轮自我修正未实现——加入 `max_fix_rounds` 循环
5. CLI 参数解析简陋——使用 `clap` 替换手动 `args().nth(1)`

这 5 项修复在独立分支 `fix/gaps` 上完成，所有测试通过后合并到 master。

---

## 三、技术收获

### Rust 工程实践

1. **Workspace 架构设计**：7 个库 crate 的拆分需要仔细考虑依赖方向。`harness-core` 在最底层，不依赖任何其他内部 crate；`harness-bin` 在最顶层，依赖所有库 crate。这种分层确保了依赖单向性，避免了循环依赖。

2. **Trait 设计模式**：`Tool`、`LlmProvider`、`GuardrailRule`、`ValidationRule`、`Plugin`、`HitlConfirmer`——6 个 trait 构成了整个系统的扩展点。每个 trait 都遵循相同的模式：定义接口契约，内置实现可插拔，运行时通过 `Box<dyn Trait>` 或 `Arc<dyn Trait>` 分发。

3. **Interior Mutability**：在冷启动验证中暴露的问题——`MockLlmProvider` 的 `complete()` 方法签名是 `&self`，但需要修改内部状态。解决方案是 `Arc<Mutex<>>`。这是 Rust 所有权的经典问题，在实际项目中遇到并解决，比看书理解得更深。

4. **错误处理**：使用 `thiserror` 定义统一的 `HarnessError` 枚举，所有 crate 都返回 `Result<T, HarnessError>`。这比使用 `Box<dyn Error>` 更类型安全。

### AI 辅助开发

这是第一次全程使用 AI 辅助完成一个完整的工程项目的开发。我的几个观察：

1. **AI 擅长结构设计，但细节需要人工把关**：OpenCode 能快速生成完整的 SPEC 和 PLAN，架构设计和任务拆分都很合理。但是 PLAN 中的代码示例存在编译错误，需要人工或冷启动验证来发现。

2. **TDD 流程非常适合 AI 辅助**：每次先写测试（RED），再写实现（GREEN），结果是对 AI 生成代码质量的直接验证。如果测试通过，代码大概率是正确的。如果测试不通过，AI 可以根据错误信息修复。

3. **Self-review 是 AI 辅助开发的关键环节**：OpenCode 在写完 SPEC 和 PLAN 后自动进行 self-review，发现了两个实际的架构问题。如果跳过这个环节直接进入编码，返工成本会高得多。

4. **分支策略与 AI 配合良好**：每个 task 在独立分支上开发，AI 可以独立完成一个分支的全部工作，不干扰主分支。这降低了并行开发的协调成本。

---

## 四、对 Superpowers 方法论的批判性思考

### 4.1 假设检验

**假设：Brainstorming 技能应该先于所有操作**

验证结果：**成立**。Brainstorming 阶段的结构化提问帮助我从"模糊的想法"变成了"清晰的 spec"。如果没有这个阶段，我可能会直接开始编码，然后在中途发现设计缺陷。

**假设：SPEC 和 PLAN 应该串行生成**

验证结果：**不成立**。PLAN 的 self-review 发现了 SPEC 中的架构问题（类型依赖方向），需要回头修改 SPEC。SPEC 和 PLAN 应该交替迭代，而非严格串行。在实际项目中，每次 self-review 都可能触发对前序文档的修正。

**假设：Plan 粒度越细越好**

验证结果：**部分成立**。17 个 task 的粒度确实让执行过程清晰可控，但有些 task 过于碎片化——"创建 crate 目录 + 写 Cargo.toml"作为一个独立 task 的必要性存疑。理想情况下，相关的小步骤应该合并，以减少分支切换成本。

### 4.2 技能链的实际效果

完整的技能链：**Brainstorming → Writing Plans → Dispatching Parallel Agents → Verification Before Completion**

这个链路的实际效果：

1. **Brainstorming**：★★★★★ 最高价值。从模糊到清晰的关键环节。结构化提问（多选题而非开放题）显著降低了决策负担。
2. **Writing Plans**：★★★★☆ 高价值。TDD 拆分让每个 task 都有明确的验收标准。但 17 个 task 的粒度值得商榷。
3. **Dispatching Parallel Agents**：★★★☆☆ 中等价值。在后期 GAPS 修复中，一个 agent 处理了所有 5 个 main.rs 的修改，因为文件冲突限制了并行度。对于独立文件的任务，并行 agent 效率更高。
4. **Verification Before Completion**：★★★★★ 最高价值。每次声称"完成"前都要运行 `cargo test` 和 `cargo check`，这防止了多次"我觉得可以了但其实不行"的尴尬。

### 4.3 局限性

1. **AI 缺乏项目级上下文理解**：OpenCode 能理解单个文件的内容，但对于跨多个 crate 的复杂依赖关系，需要人工指出来才能修正。self-review 机制部分缓解了这个问题，但不够彻底。

2. **Prompt 工程的隐性成本**：虽然 AI 处理了大部分编码工作，但编写高质量的 prompt 本身需要经验。例如，在 GAPS 修复中，我需要清楚描述 5 个任务的修改方式和涉及的文件，这要求我对代码库有足够了解。

3. **AI 生成代码的"黑盒"风险**：AI 生成的代码我并非每行都理解。虽然测试覆盖了功能正确性，但对性能、安全性、Rust 惯用性等方面的评估需要人工审查。

4. **冷启动验证的价值边界**：陌生 agent 能发现 PLAN 中的代码错误，但无法评估架构决策的合理性——它只能按 PLAN 执行，不能质疑 PLAN。这意味着架构层面的风险仍然需要人工把控。

---

## 五、总结

### 做得好的地方

1. **架构设计清晰**：7 个 crate 的分层合理，依赖单向，每个 crate 职责单一。这得益于 Brainstorming 阶段的充分讨论和 self-review 的及时修正。

2. **TDD 贯穿始终**：每个 task 都是先写测试后写实现，43 个测试覆盖了所有核心机制。这保证了代码质量，也让 AI 生成代码的验证有了客观标准。

3. **冷启动验证**：在早期就用陌生 agent 验证了 SPEC + PLAN 的可用性，发现了 PLAN 中的代码错误。这个投入在后期避免了多次编译失败。

4. **分支策略**：每个 task 独立分支，合并前 review。这保证了主分支的稳定性，也允许并行开发。

### 可以改进的地方

1. **缺少可视化设计**：SPEC 中的架构图是纯文本 ASCII，不够直观。如果能用 Mermaid 或 SVG 展示组件关系和数据流，沟通效率会更高。

2. **Plan 粒度优化**：17 个 task 中有些过于碎片化。未来应该将"创建 crate + 写 Cargo.toml"这类样板步骤合并为 1 个 task。

3. **风险评估深度不足**：Plan 中的风险评估比较简略。例如"LLM 响应格式不稳定"应该有更具体的缓解方案（如 schema validation、fallback parsing）。

4. **AI 代码审查**：应该增加一个"AI 代码审查"步骤，让 AI 对自己的代码做 review，发现潜在的性能、安全、风格问题。

### 最终结论

这是**一次成功的 AI 辅助软件开发实践**。从零开始，通过 Brainstorming → SPEC → PLAN → TDD 实现 → 冷启动验证 → 缺口修复的完整流程，最终产出了一个 7 crate、43 测试、功能完整的 Rust 项目。

最大的收获不是学会了 Rust 或 Agent 架构，而是理解了**AI 辅助开发的工作流**——什么阶段 AI 最强（结构化设计、代码生成、测试编写），什么阶段需要人工介入（架构决策、代码审查、风险评估）。这个经验在未来更大的项目中会更有价值。

---

*字数：约 2200 字*