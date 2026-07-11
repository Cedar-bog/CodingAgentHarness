pub mod hitl;
pub mod rules;

use harness_core::ToolCall;

#[derive(Debug, Clone, PartialEq)]
pub enum GuardrailAction {
    Allow,
    RequireApproval { reason: String },
    Block { reason: String },
}

pub trait GuardrailRule: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, action: &ToolCall) -> Option<String>;
}

pub struct Guardrail {
    rules: Vec<Box<dyn GuardrailRule>>,
}

impl Guardrail {
    pub fn new_default() -> Self {
        let rules: Vec<Box<dyn GuardrailRule>> = vec![
            Box::new(rules::DangerousCommandRule),
            Box::new(rules::SudoCommandRule),
            Box::new(rules::GitPushMainRule),
            Box::new(rules::NetworkRequestRule),
            Box::new(rules::CredentialLeakRule),
        ];
        Self { rules }
    }

    pub fn with_rules(rules: Vec<Box<dyn GuardrailRule>>) -> Self {
        Self { rules }
    }

    pub fn check(&self, action: &ToolCall) -> GuardrailAction {
        for rule in &self.rules {
            if let Some(reason) = rule.check(action) {
                return GuardrailAction::Block { reason };
            }
        }
        GuardrailAction::Allow
    }
}

#[cfg(test)]
mod guard_tests;