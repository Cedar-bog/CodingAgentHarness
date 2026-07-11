use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct HarnessConfig {
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub guardrails: GuardrailConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
}

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_max_turns")]
    pub max_turns: usize,
    #[serde(default = "default_max_tools")]
    pub max_tools_per_turn: usize,
    #[serde(default = "default_max_fix_rounds")]
    pub max_fix_rounds: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_turns: default_max_turns(),
            max_tools_per_turn: default_max_tools(),
            max_fix_rounds: default_max_fix_rounds(),
        }
    }
}

fn default_max_turns() -> usize { 50 }
fn default_max_tools() -> usize { 5 }
fn default_max_fix_rounds() -> usize { 3 }

#[derive(Debug, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    #[serde(default)]
    pub api_key_env: String,
    #[serde(default)]
    pub base_url: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: "deepseek-chat".into(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            api_key_env: "DEEPSEEK_API_KEY".into(),
            base_url: "https://api.deepseek.com".into(),
        }
    }
}

fn default_provider() -> String { "deepseek".into() }
fn default_temperature() -> f32 { 0.0 }
fn default_max_tokens() -> usize { 4096 }

#[derive(Debug, Deserialize)]
pub struct GuardrailConfig {
    #[serde(default = "default_guard_mode")]
    pub mode: String,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            mode: default_guard_mode(),
            timeout_seconds: default_timeout(),
        }
    }
}

fn default_guard_mode() -> String { "strict".into() }
fn default_timeout() -> u64 { 60 }

#[derive(Debug, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_max_context")]
    pub max_context_tokens: usize,
    #[serde(default)]
    pub db_path: String,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: default_max_context(),
            db_path: ".harness/memory.db".into(),
        }
    }
}

fn default_max_context() -> usize { 2000 }

#[derive(Debug, Deserialize)]
pub struct ToolsConfig {
    #[serde(default = "default_allowed_tools")]
    pub allowed: Vec<String>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self { allowed: default_allowed_tools() }
    }
}

fn default_allowed_tools() -> Vec<String> {
    vec![
        "read_file".into(), "write_file".into(), "shell_exec".into(),
        "git_op".into(), "code_search".into(),
    ]
}

#[cfg(test)]
mod config_tests;