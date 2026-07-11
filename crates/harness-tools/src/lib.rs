pub mod read_file;
pub mod write_file;

use async_trait::async_trait;
use harness_core::{ToolResult, ToolSchema, FunctionSchema};
use std::collections::HashMap;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, args: serde_json::Value) -> ToolResult;
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    pub fn unregister(&mut self, name: &str) {
        self.tools.remove(name);
    }

    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
            })
            .collect()
    }

    pub fn to_llm_tools(&self) -> Vec<ToolSchema> {
        self.tools
            .values()
            .map(|t| ToolSchema {
                name: t.name().to_string(),
                description: t.description().to_string(),
                function: FunctionSchema {
                    name: t.name().to_string(),
                    description: t.description().to_string(),
                    parameters: t.parameters_schema(),
                },
            })
            .collect()
    }

    pub async fn execute(&self, name: &str, args: &serde_json::Value) -> Result<ToolResult, String> {
        let tool = self.tools.get(name).ok_or_else(|| format!("Tool '{}' not found", name))?;
        Ok(tool.execute(args.clone()).await)
    }
}

#[cfg(test)]
mod registry_tests;