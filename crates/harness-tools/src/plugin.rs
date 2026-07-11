use crate::{Tool, ToolRegistry};
use async_trait::async_trait;
use std::sync::Arc;

pub struct PluginContext {
    pub workspace_dir: String,
}

#[async_trait]
pub trait Plugin: Tool + Send + Sync {
    fn version(&self) -> &str;
    fn dependencies(&self) -> Vec<&str>;
    fn init(&mut self, ctx: &PluginContext) -> harness_core::Result<()>;
}

struct PluginToolWrapper {
    plugin: Arc<dyn Plugin>,
}

#[async_trait]
impl Tool for PluginToolWrapper {
    fn name(&self) -> &str { self.plugin.name() }
    fn description(&self) -> &str { self.plugin.description() }
    fn parameters_schema(&self) -> serde_json::Value { self.plugin.parameters_schema() }
    async fn execute(&self, args: serde_json::Value) -> harness_core::ToolResult {
        self.plugin.execute(args).await
    }
}

pub struct PluginLoader {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn list(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.name().to_string()).collect()
    }

    pub fn load_all(&self, registry: &mut ToolRegistry) -> harness_core::Result<()> {
        for plugin in &self.plugins {
            let wrapper = PluginToolWrapper { plugin: plugin.clone() };
            registry.register(Box::new(wrapper));
        }
        Ok(())
    }
}