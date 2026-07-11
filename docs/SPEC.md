# SPEC: Coding Agent Harness

> AI4SE 期末项目 · A · Coding Agent Harness

---

## 1. 问题陈述

### 要解决的问题

现有coding agent框架（LangChain、AutoGen、CrewAI等）将核心循环封装为黑盒，开发者难以自定义治理、反馈、工具分发机制。而直接在prompt里写规则不可靠、不可测试——LLM的行为每次都不同，无法用确定性测试验证。

本项目构建一个**用Rust编码实现的coding agent harness**，将agent的每个核心机制（主循环、工具分发、治理护栏、反馈闭环、记忆、配置）落实为可独立测试的代码，而非依赖prompt。

### 目标用户

需要可控、可测试、可扩展coding agent的开发者。典型场景：让agent自动读写代码、执行测试、根据反馈自我修正，同时确保危险操作被拦截。

### 为什么值得做

1. **Rust实现**：内存安全、零成本抽象，适合构建可靠的agent系统
2. **扩展性设计**：工具、治理规则、反馈机制都可插拔，新增功能只需实现trait
3. **可测试性**：每个机制都能用mock LLM做确定性单测，满足作业A.4(C)要求
4. **实际价值**：当LLM能完成大部分编码时，工程师的价值在harness层——治理、反馈、上下文、安全、分发

---

## 2. 用户故事

1. **作为开发者**，我希望能用一条命令启动agent，让它自动完成编码任务，这样我可以专注于更高层的设计决策。

2. **作为开发者**，我希望agent能在执行危险操作（如删除文件、推送代码）前询问我确认，这样我不会意外丢失数据。

3. **作为开发者**，我希望agent能在测试失败后自动分析错误并尝试修复，这样我不需要手动介入每次失败。

4. **作为开发者**，我希望能通过配置文件自定义agent的行为（如允许的工具、护栏规则、最大轮次），这样我可以适配不同项目的需求。

5. **作为开发者**，我希望能为agent添加自定义工具（如内部API调用），这样我可以扩展它的能力而不修改核心代码。

6. **作为开发者**，我希望agent能记住项目的约定和历史决策，这样我不需要每次会话都重复说明。

7. **作为开发者**，我希望通过Docker一条命令就能运行agent，这样我不需要配置复杂的依赖环境。

---

## 3. 功能规约

### 3.1 Agent主循环

**输入**：用户任务描述 + 工作目录路径

**行为**：
1. 构建上下文（系统prompt + 记忆 + 对话历史）
2. 调用LLM获取响应
3. 解析LLM返回的动作（tool_calls或文本）
4. 对每个动作：治理拦截检查 → 执行工具 → 回灌结果
5. 重复步骤2-4，直到LLM返回finish_stop或达到最大轮次

**输出**：任务执行结果 + 对话历史

**边界条件**：
- 最大轮次限制（默认50轮，可配置）
- 每轮最多执行5个工具调用（可配置）
- 用户Ctrl+C优雅退出
- LLM返回错误时重试3次后暂停

**错误处理**：
- LLM API超时：重试3次，间隔指数退避
- 工具执行失败：错误信息回灌给LLM，由LLM决定下一步
- 网络不可用：缓存最近的响应，离线模式（有限功能）

### 3.2 工具分发与插件系统（主要贡献）

**Tool trait**：

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, args: serde_json::Value) -> ToolResult;
}
```

**ToolRegistry**：
- 维护`HashMap<String, Box<dyn Tool>>`，支持运行时注册/注销
- 按类别分组，方便LLM选择
- 将工具列表转换为LLM的tools参数格式（OpenAI兼容）

**内置工具**：

| 工具 | 功能 | 输入 | 输出 |
|------|------|------|------|
| ReadFile | 读取文件内容 | path, offset?, limit? | 文件内容文本 |
| WriteFile | 创建/覆盖文件 | path, content | 成功/失败状态 |
| ShellExec | 执行shell命令 | command, cwd?, timeout? | stdout + stderr |
| GitOp | Git操作 | operation, args | 操作结果 |
| WebSearch | 网页搜索 | query, num_results? | 搜索结果列表 |
| CodeSearch | 代码库搜索 | pattern, path?, include? | 匹配文件和行号 |

**插件系统**：

```rust
pub trait Plugin: Tool {
    fn version(&self) -> &str;
    fn dependencies(&self) -> Vec<&str>;
    fn init(&mut self, ctx: &PluginContext) -> Result<()>;
}
```

- 插件可声明依赖关系，加载器自动排序
- 运行时可动态添加/移除工具（通过配置文件）
- 每个工具有JSON Schema描述，LLM自动知道如何调用

### 3.3 LLM抽象层

**LLM Provider trait**：

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    fn supports_tools(&self) -> bool;
    fn max_context_tokens(&self) -> usize;
    fn name(&self) -> &str;
}
```

**多供应商支持**：
- DeepSeek（主要）：使用OpenAI兼容API格式
- OpenAI：GPT-4/GPT-3.5
- Mock：测试用，预设响应队列

**Mock LLM**：
- 预设响应队列，按顺序返回
- 记录所有调用请求，用于断言
- 无需网络和真实API key

### 3.4 记忆系统

**三层架构**：
- **工作记忆**：当前会话上下文（消息列表）
- **情景记忆**：历史会话摘要、关键决策
- **语义记忆**：项目约定、代码知识、用户偏好

**存储后端**：SQLite + FTS5全文搜索

**记忆注入策略**：
- 语义记忆（项目约定）始终注入
- 情景记忆按相关性注入
- 总token预算可配置（默认2000 tokens）

### 3.5 治理与护栏

**危险动作分类与拦截策略**：

| 类别 | 示例 | 策略 |
|------|------|------|
| 文件删除 | `rm -rf`, `del /s` | 拦截，需HITL确认 |
| 系统级命令 | `sudo`, `chmod 777` | 拦截，需HITL确认 |
| 对外推送 | `git push origin main` | 拦截，需HITL确认 |
| 网络请求 | `curl`, `wget` | 拦截，需HITL确认 |
| 凭据泄露 | 输出包含API key | 拦截并脱敏 |

**HITL状态机**：
- 通过stdin/stdout与用户交互
- 支持"本次会话全部允许"快捷选项
- 超时自动拒绝（默认60秒）

**沙箱**：
- 文件操作限制在工作目录内
- Shell命令禁止访问系统目录

### 3.6 反馈闭环

**反馈信号来源**：
- 测试结果：`cargo test`, `npm test` → 解析pass/fail
- 编译结果：`cargo build` → 检查exit code + 错误信息
- Lint结果：`cargo clippy`, `eslint` → 解析warning/error
- 类型检查：`cargo check` → 检查类型错误

**确定性校验器**：
- TestResultValidator：解析测试输出
- CompileErrorValidator：解析编译错误
- LintResultValidator：解析lint警告

**多轮自我修正**：
- 最大修正轮次：3轮（可配置）
- 每轮注入上一步的反馈
- 连续失败3次同一类别 → 暂停，请求用户干预
- 成功后自动记录到情景记忆

### 3.7 配置管理

支持TOML配置文件，覆盖：agent行为、LLM供应商、治理规则、记忆设置、工具列表。配置示例见第9节。

---

## 4. 非功能性需求

### 性能
- 主循环延迟：LLM调用除外，本地操作<100ms
- 记忆检索：<50ms（FTS5索引）
- 工具注册表查询：<1ms

### 安全
- API key不硬编码、不提交Git、不写入日志
- 首次运行引导安全录入key（OS钥匙串存储）
- 凭据查看/更新/清除不回显明文
- 危险操作必须HITL确认

### 可用性
- CLI交互模式，支持Ctrl+C优雅退出
- 配置文件驱动，无需修改代码
- Docker一键启动

### 可观测性
- 每轮日志：LLM请求、工具调用、治理决策
- 可选verbose模式输出详细信息
- 对话历史持久化

---

## 5. 系统架构

### 架构图

```
┌─────────────────────────────────────────────┐
│                Agent 主循环                  │
│  组织上下文 → 调用LLM → 解析动作 → 分发执行  │
│       ↑                              ↓      │
│       └──────── 回灌结果 ←──────────┘      │
└─────────────────────────────────────────────┘
         │              │              │
    ┌────▼────┐   ┌─────▼─────┐  ┌────▼────┐
    │ LLM层   │   │ 工具注册表 │  │ 记忆系统 │
    │(多供应商)│   │(Trait动态) │  │(SQLite) │
    └─────────┘   └───────────┘  └─────────┘
         │              │
    ┌────▼────┐   ┌─────▼─────┐
    │ 治理护栏 │   │ 反馈校验器 │
    └─────────┘   └───────────┘
```

### 模块划分（Rust crate结构）

| Crate | 职责 |
|-------|------|
| `harness-core` | 主循环、Agent状态机 |
| `harness-llm` | LLM抽象层，支持DeepSeek/OpenAI兼容API |
| `harness-tools` | 工具trait定义 + 内置工具实现 |
| `harness-memory` | 记忆存储与检索（SQLite + FTS5） |
| `harness-guard` | 治理护栏（危险动作拦截、HITL） |
| `harness-feedback` | 反馈校验器（测试结果解析、失败分类） |
| `harness-config` | 配置管理（TOML） |

### 数据流

1. 用户输入任务 → `harness-core`接收
2. `harness-core`从`harness-memory`加载相关记忆
3. `harness-core`构建上下文，调用`harness-llm`
4. `harness-llm`返回响应（文本/tool_calls）
5. `harness-core`解析动作，交给`harness-tools`
6. `harness-tools`执行前，`harness-guard`检查治理规则
7. 工具执行结果回灌给`harness-core`
8. `harness-feedback`分析结果（如有测试/lint输出）
9. 循环直到完成

### 外部依赖

| 依赖 | 用途 |
|------|------|
| `reqwest` | HTTP请求（调用LLM API） |
| `tokio` | 异步运行时 |
| `rusqlite` | SQLite存储 |
| `serde`/`serde_json` | 序列化 |
| `regex` | 模式匹配 |
| `walkdir` | 文件遍历 |
| `keyring` | OS钥匙串访问 |
| `dotenv` | .env文件加载 |

---

## 6. 数据模型

### 核心实体

```rust
// Agent状态
pub struct Agent {
    config: AgentConfig,
    llm: Box<dyn LlmProvider>,
    tools: ToolRegistry,
    memory: MemoryStore,
    guardrail: Guardrail,
    feedback: FeedbackValidator,
    conversation: Vec<Message>,
    turn_count: usize,
}

// LLM消息
pub struct Message {
    pub role: Role,        // System, User, Assistant, Tool
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_call_id: Option<String>,
}

// 工具调用
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

// 工具结果
pub struct ToolResult {
    pub output: String,
    pub is_error: bool,
    pub metadata: Option<serde_json::Value>,
}

// 记忆条目
pub struct MemoryEntry {
    pub id: Option<i64>,
    pub category: String,    // convention, decision, codebase, preference
    pub key: String,
    pub value: String,
    pub confidence: f64,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

// 治理决策
pub enum GuardrailAction {
    Allow,
    Block { reason: String },
    RequireApproval { reason: String, action: Action },
}

// 反馈
pub struct ValidationFeedback {
    pub is_success: bool,
    pub category: FailureCategory,
    pub summary: String,
    pub details: String,
    pub suggested_fix: Option<String>,
}

pub enum FailureCategory {
    CompileError,
    TestFailure,
    LintWarning,
    RuntimeError,
}
```

### 数据库Schema

```sql
-- 情景记忆
CREATE TABLE episodic_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    summary TEXT NOT NULL,
    key_decisions TEXT,  -- JSON数组
    tags TEXT            -- JSON数组
);

-- 语义记忆
CREATE TABLE semantic_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    confidence REAL DEFAULT 1.0,
    last_accessed DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(category, key)
);

-- FTS5全文搜索索引
CREATE VIRTUAL TABLE memory_fts USING fts5(
    content,
    content='semantic_memory',
    content_rowid='id'
);
```

---

## 7. 凭据与分发设计

### 凭据存储方案

**存储层**：OS钥匙串（`keyring` crate）
- macOS：Keychain
- Windows：Credential Manager
- Linux：Secret Service

**录入流程**：
1. 首次运行检测无key → 提示"请设置DeepSeek API key"
2. 隐藏输入（无回显） → 存入OS钥匙串
3. 后续运行自动从钥匙串读取

**查看/更新/清除**：
- `harness key show` → 显示key后4位（不回显全文）
- `harness key set` → 重新输入
- `harness key clear` → 从钥匙串删除

**环境变量作为备选**：
- 支持`DEEPSEEK_API_KEY`环境变量
- 优先级：命令行参数 > 环境变量 > 钥匙串
- `.env`文件通过`dotenv`加载

### 威胁模型

| 威胁 | 对策 |
|------|------|
| key泄露到Git | .env加入.gitignore，代码中不硬编码 |
| key泄露到日志 | 日志中脱敏，不输出key值 |
| key泄露到终端history | 不通过命令行参数传递key |
| key泄露到进程环境 | 仅在需要时加载，不全局export |

### Docker分发

**获取方式**：
```bash
docker build -t coding-agent-harness .
```

**运行命令**：
```bash
docker run -it \
  -e DEEPSEEK_API_KEY=<your-key> \
  -v $(pwd):/workspace \
  coding-agent-harness \
  --config /workspace/harness.toml
```

**Key安全配置**：
- 通过环境变量传入（`-e`参数）
- 不写入镜像层
- 建议使用`--env-file`或Docker secrets

**已知限制**：
- 容器内Shell执行受限于容器权限
- 文件操作限制在挂载的工作目录内
- 需要挂载工作目录才能操作项目文件

---

## 8. 技术选型与理由

### 语言：Rust

**理由**：
1. 内存安全：所有权系统防止内存泄漏，适合长期运行的agent
2. 零成本抽象：trait动态分发开销小，适合工具注册表
3. 异步生态：tokio提供成熟的异步运行时
4. 编译期检查：减少运行时错误
5. 作业要求不限语言，Rust能体现工程深度

### LLM供应商：DeepSeek为主，支持多供应商

**理由**：
1. DeepSeek API完全兼容OpenAI格式，切换成本低
2. DeepSeek价格低，适合学生项目
3. 多供应商支持体现扩展性设计
4. Mock LLM用于测试，无需真实API

### 存储：SQLite

**理由**：
1. 零配置，嵌入式数据库
2. FTS5支持全文搜索，满足记忆检索需求
3. 单文件存储，便于备份和分发
4. rusqlite crate成熟稳定

### 分发：Docker

**理由**：
1. 跨平台：一条命令启动
2. 环境隔离：不污染宿主系统
3. 凭据通过环境变量传入，安全
4. 作业要求容器/二进制/包任选，Docker最简单

---

## 9. 验收标准

### 功能验收

- [ ] Agent主循环能完整运行：接收任务 → 调用LLM → 执行工具 → 回灌结果 → 循环
- [ ] 6个内置工具均能正确执行
- [ ] 治理护栏能拦截5类危险动作
- [ ] HITL确认流程正常工作
- [ ] 反馈闭环能检测测试/编译失败并回灌
- [ ] 记忆系统能存储和检索项目约定
- [ ] 配置文件能驱动所有行为参数

### 机制验收（A.4(C)）

- [ ] 移除真实LLM后，每个核心机制能用确定性单测验证
- [ ] Mock LLM单元测试覆盖：工具分发、治理拦截、反馈回灌、记忆读写、停机判断、插件加载

### 机制演示（A.6）

- [ ] 演示1：Mock LLM尝试危险操作 → 被Guardrail拦截
- [ ] 演示2：Mock LLM返回失败代码 → 反馈闭环触发修正
- [ ] 演示3：运行时注册自定义工具 → 被正确调用

### 工程验收

- [ ] `cargo test`一键运行所有测试
- [ ] Docker镜像可正常构建和运行
- [ ] 凭据不硬编码、不提交Git
- [ ] 至少3个职责清晰的功能模块
- [ ] CI配置包含unit-test job

---

## 10. 风险与未决问题

### 风险

1. **LLM响应格式不稳定**：DeepSeek的tool_calls格式可能与OpenAI有细微差异
   - 缓解：实现格式适配层，做兼容性测试

2. **Rust异步复杂度**：tokio + async trait增加学习曲线
   - 缓解：先实现同步版本，再迁移异步

3. **SQLite FTS5性能**：大量记忆时检索可能变慢
   - 缓解：限制记忆总量，定期清理低置信度条目

4. **Docker内Shell执行**：容器权限受限
   - 缓解：文档说明需要的Docker权限参数

### 未决问题

1. 是否需要支持流式LLM响应？
   - 当前设计不支持流式，如果需要可后续添加

2. 多agent编排的具体模式？
   - 当前设计以单agent为主，Coordinator模式作为扩展点

3. 插件的热加载是否需要？
   - 当前设计需要重启加载插件，热加载可作为后续优化

---

## 11. 领域与机制设计

### Coding领域的反馈信号

| 信号 | 来源 | 客观性 | 可编码性 |
|------|------|--------|----------|
| 测试通过/失败 | cargo test / npm test | 高（确定性） | 高（解析输出） |
| 编译成功/失败 | cargo build | 高（确定性） | 高（exit code + 错误信息） |
| Lint警告数量 | cargo clippy | 中（可能有误报） | 高（解析输出） |
| 类型错误 | cargo check | 高（确定性） | 高（解析输出） |

### Coding领域的危险动作

| 动作 | 风险等级 | 拦截必要性 |
|------|----------|------------|
| rm -rf / 删除系统文件 | 极高 | 必须拦截 |
| sudo / 系统级操作 | 极高 | 必须拦截 |
| git push origin main | 高 | 建议拦截 |
| curl / wget 外部请求 | 中 | 建议拦截 |
| 输出含API key | 高 | 必须拦截 |

### Coding领域所需工具

| 工具 | 必要性 | 复杂度 |
|------|--------|--------|
| 读文件 | 必需 | 低 |
| 写文件 | 必需 | 低 |
| Shell执行 | 必需 | 中 |
| Git操作 | 必需 | 中 |
| 网页搜索 | 可选 | 中 |
| 代码搜索 | 必需 | 低 |

### Coding领域记忆需求

| 记忆类型 | 示例 | 持久性 |
|----------|------|--------|
| 项目约定 | 使用snake_case、Rust edition 2024 | 长期 |
| 历史决策 | 选择SQLite而非PostgreSQL | 长期 |
| 代码知识 | src/lib.rs是核心模块 | 中期 |
| 用户偏好 | 详细输出模式 | 长期 |
| 会话历史 | 上次修复了编译错误 | 短期 |

### 重点维度：扩展性（工具分发 + 插件系统）

**为什么选择扩展性作为主要贡献**：
1. **天然由代码构成**：工具分发、插件加载都是纯工程问题，不依赖prompt
2. **可深度实现**：trait系统、依赖解析、动态加载都有足够深度
3. **满足A.4要求**：移除LLM后，工具注册表和插件加载器仍可独立测试
4. **实用价值**：扩展性好的harness能适应不同场景，不只是玩具

**如何编码实现**：
- Tool trait抽象：所有工具实现统一接口
- ToolRegistry：HashMap + 动态分发，运行时注册/注销
- Plugin trait：扩展Tool，添加版本和依赖管理
- PluginLoader：扫描目录、解析依赖、排序加载
- 每个组件都有mock测试，不依赖真实LLM

---

## 附录：Agent状态机详细定义

```
States: Idle, Thinking, ToolCall, Executing, Observing, Guarded, WaitingForApproval, Done

Transitions:
  Idle → Thinking: 接收用户任务
  Thinking → ToolCall: LLM返回tool_calls
  Thinking → Done: LLM返回finish_stop或达到最大轮次
  ToolCall → Guarded: 动作需要治理检查
  ToolCall → Executing: 动作通过治理检查
  Guarded → WaitingForApproval: 需要HITL确认
  Guarded → Executing: 动作被允许
  Guarded → Thinking: 动作被拦截，反馈给LLM
  WaitingForApproval → Executing: 用户批准
  WaitingForApproval → Thinking: 用户拒绝
  Executing → Observing: 工具执行完成
  Observing → Thinking: 循环继续
```
