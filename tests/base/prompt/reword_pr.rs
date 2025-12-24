//! Base/Prompt Reword PR 模块测试
//!
//! 测试 PR Reword 的 system prompt。

use workflow::base::prompt::REWORD_PR_SYSTEM_PROMPT;

#[test]
fn test_reword_pr_system_prompt_not_empty() {
    // 测试 prompt 常量不为空
    assert!(!REWORD_PR_SYSTEM_PROMPT.is_empty());
}

#[test]
fn test_reword_pr_system_prompt_contains_keywords() {
    // 测试 prompt 包含关键内容
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("PR title"));
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("description"));
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("PR diff"));
}

#[test]
fn test_reword_pr_system_prompt_contains_rules() {
    // 测试 prompt 包含规则说明
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("PR Title Rules"));
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("Description Rules"));
}

#[test]
fn test_reword_pr_system_prompt_contains_examples() {
    // 测试 prompt 包含示例
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("Example"));
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("Response Format"));
}

#[test]
fn test_reword_pr_system_prompt_contains_json_format() {
    // 测试 prompt 包含 JSON 格式说明
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("JSON"));
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("pr_title"));
}

#[test]
fn test_reword_pr_system_prompt_contains_language_requirement() {
    // 测试 prompt 包含语言要求
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("English"));
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("All outputs MUST be in English"));
}

#[test]
fn test_reword_pr_system_prompt_contains_markdown_support() {
    // 测试 prompt 包含 markdown 格式支持说明
    assert!(REWORD_PR_SYSTEM_PROMPT.contains("markdown") || REWORD_PR_SYSTEM_PROMPT.contains("markdown heading"));
}

#[test]
fn test_reword_pr_system_prompt_length() {
    // 测试 prompt 有合理的长度（至少应该包含基本内容）
    assert!(REWORD_PR_SYSTEM_PROMPT.len() > 500);
}

