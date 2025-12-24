//! Base/Prompt Generate Branch 模块测试
//!
//! 测试生成分支名的 system prompt。

use workflow::base::prompt::GENERATE_BRANCH_SYSTEM_PROMPT;

#[test]
fn test_generate_branch_system_prompt_not_empty() {
    // 测试 prompt 常量不为空
    assert!(!GENERATE_BRANCH_SYSTEM_PROMPT.is_empty());
}

#[test]
fn test_generate_branch_system_prompt_contains_keywords() {
    // 测试 prompt 包含关键内容
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("branch name"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("PR title"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("description"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("scope"));
}

#[test]
fn test_generate_branch_system_prompt_contains_rules() {
    // 测试 prompt 包含规则说明
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("Branch Name Rules"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("PR Title Rules"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("Description Rules"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("Scope Rules"));
}

#[test]
fn test_generate_branch_system_prompt_contains_examples() {
    // 测试 prompt 包含示例
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("Examples"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("Response Format"));
}

#[test]
fn test_generate_branch_system_prompt_contains_json_format() {
    // 测试 prompt 包含 JSON 格式说明
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("JSON"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("branch_name"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("pr_title"));
}

#[test]
fn test_generate_branch_system_prompt_contains_language_requirement() {
    // 测试 prompt 包含语言要求
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("English"));
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.contains("All outputs MUST be in English"));
}

#[test]
fn test_generate_branch_system_prompt_length() {
    // 测试 prompt 有合理的长度（至少应该包含基本内容）
    assert!(GENERATE_BRANCH_SYSTEM_PROMPT.len() > 500);
}
