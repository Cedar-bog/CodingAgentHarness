use crate::validators::*;
use harness_core::ToolResult;

#[test]
fn test_result_validator_detects_failure() {
    let v = crate::validators::TestResultValidator;
    let result = ToolResult { tool_call_id: "1".into(), content: "test result: 2 failed, 0 passed".into(), is_error: true };
    let feedback = v.validate(&result);
    assert!(!feedback.is_success);
}

#[test]
fn test_result_validator_detects_success() {
    let v = crate::validators::TestResultValidator;
    let result = ToolResult { tool_call_id: "1".into(), content: "test result: 0 failed, 5 passed".into(), is_error: false };
    let feedback = v.validate(&result);
    assert!(feedback.is_success);
}

#[test]
fn compile_error_validator_detects_error() {
    let v = crate::validators::CompileErrorValidator;
    let result = ToolResult { tool_call_id: "1".into(), content: "error[E0308]: mismatched types\n --> src/main.rs:5:12".into(), is_error: true };
    let feedback = v.validate(&result);
    assert!(!feedback.is_success);
}

#[test]
fn lint_result_validator_detects_warnings() {
    let v = crate::validators::LintResultValidator;
    let result = ToolResult { tool_call_id: "1".into(), content: "warning: unused variable `x`\n --> src/main.rs:3:9".into(), is_error: false };
    let feedback = v.validate(&result);
    assert!(!feedback.is_success);
}