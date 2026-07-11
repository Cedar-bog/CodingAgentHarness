pub mod validators;

use harness_core::ToolResult;

#[derive(Debug, Clone, PartialEq)]
pub enum FailureCategory {
    CompileError,
    TestFailure,
    LintWarning,
    RuntimeError,
}

#[derive(Debug, Clone)]
pub struct ValidationFeedback {
    pub is_success: bool,
    pub category: FailureCategory,
    pub summary: String,
    pub details: String,
}

pub trait ValidationRule: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, result: &ToolResult) -> Option<ValidationFeedback>;
}

pub struct FeedbackValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl FeedbackValidator {
    pub fn new_default() -> Self {
        let rules: Vec<Box<dyn ValidationRule>> = vec![
            Box::new(validators::TestResultValidator),
            Box::new(validators::CompileErrorValidator),
            Box::new(validators::LintResultValidator),
        ];
        Self { rules }
    }

    pub fn validate(&self, result: &ToolResult) -> ValidationFeedback {
        for rule in &self.rules {
            if let Some(feedback) = rule.validate(result) {
                return feedback;
            }
        }
        ValidationFeedback {
            is_success: !result.is_error,
            category: FailureCategory::RuntimeError,
            summary: "No specific validation matched".into(),
            details: result.content.clone(),
        }
    }
}

#[cfg(test)]
mod feedback_tests;