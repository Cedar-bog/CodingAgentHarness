use crate::{FailureCategory, ValidationFeedback, ValidationRule};
use harness_core::ToolResult;
use regex::Regex;

pub struct TestResultValidator;

impl ValidationRule for TestResultValidator {
    fn name(&self) -> &str { "test_result" }
    fn validate(&self, result: &ToolResult) -> Option<ValidationFeedback> {
        let output = &result.content;
        let re_failed = Regex::new(r"(\d+) failed").ok()?;
        let re_passed = Regex::new(r"(\d+) passed").ok()?;

        let failed = re_failed.captures(output)?.get(1)?.as_str().parse::<usize>().ok()?;
        let passed = re_passed.captures(output).and_then(|c| c.get(1)).and_then(|m| m.as_str().parse::<usize>().ok()).unwrap_or(0);

        if failed > 0 {
            Some(ValidationFeedback {
                is_success: false,
                category: FailureCategory::TestFailure,
                summary: format!("{} tests failed, {} passed", failed, passed),
                details: output.lines().filter(|l| l.contains("FAILED") || l.contains("failures")).collect::<Vec<_>>().join("\n"),
            })
        } else {
            Some(ValidationFeedback {
                is_success: true,
                category: FailureCategory::TestFailure,
                summary: format!("All {} tests passed", passed),
                details: String::new(),
            })
        }
    }
}

pub struct CompileErrorValidator;

impl ValidationRule for CompileErrorValidator {
    fn name(&self) -> &str { "compile_error" }
    fn validate(&self, result: &ToolResult) -> Option<ValidationFeedback> {
        let output = &result.content;
        if output.contains("error[") || output.contains("error:") {
            let re = Regex::new(r"error\[E\d+\]:\s*(.+)").ok();
            let summary = re
                .and_then(|r| r.captures(output))
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "Compilation error".into());
            Some(ValidationFeedback {
                is_success: false,
                category: FailureCategory::CompileError,
                summary,
                details: output.lines().filter(|l| l.starts_with("error") || l.starts_with(" -->")).take(10).collect::<Vec<_>>().join("\n"),
            })
        } else {
            None
        }
    }
}

pub struct LintResultValidator;

impl ValidationRule for LintResultValidator {
    fn name(&self) -> &str { "lint_result" }
    fn validate(&self, result: &ToolResult) -> Option<ValidationFeedback> {
        let output = &result.content;
        if output.contains("warning:") || output.contains("warning[") {
            let count = output.lines().filter(|l| l.contains("warning:")).count();
            if count > 0 {
                Some(ValidationFeedback {
                    is_success: false,
                    category: FailureCategory::LintWarning,
                    summary: format!("{} lint warnings found", count),
                    details: output.lines().filter(|l| l.contains("warning:")).take(5).collect::<Vec<_>>().join("\n"),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}