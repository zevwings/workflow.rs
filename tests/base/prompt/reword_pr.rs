//! Base/Prompt Reword PR 模块测试
//!
//! 测试 PR Reword 的 system prompt。

use workflow::base::prompt::REWORD_PR_SYSTEM_PROMPT;

// ==================== Reword PR System Prompt Tests ====================

#[test]
fn test_reword_pr_system_prompt_not_empty_with_constant_returns_non_empty() {
    // Arrange: 准备检查prompt常量

    // Act & Assert: 验证prompt常量不为空
    assert!(!REWORD_PR_SYSTEM_PROMPT.is_empty());
}

#[test]
fn test_reword_pr_system_prompt_contains_keywords_with_prompt_contains_keywords() {
    // Arrange: 准备关键词列表
    let keywords = ["PR title", "description", "PR diff"];

    // Act & Assert: 验证prompt包含所有关键词
    for keyword in keywords.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(keyword));
    }
}

#[test]
fn test_reword_pr_system_prompt_contains_rules_with_prompt_contains_rules() {
    // Arrange: 准备规则关键词列表
    let rules = ["PR Title Rules", "Description Rules"];

    // Act & Assert: 验证prompt包含所有规则说明
    for rule in rules.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(rule));
    }
}

#[test]
fn test_reword_pr_system_prompt_contains_examples_with_prompt_contains_examples() {
    // Arrange: 准备示例关键词列表
    let examples = ["Example", "Response Format"];

    // Act & Assert: 验证prompt包含示例
    for example in examples.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(example));
    }
}

#[test]
fn test_reword_pr_system_prompt_contains_json_format_with_prompt_contains_json() {
    // Arrange: 准备JSON格式关键词列表
    let json_keywords = ["JSON", "pr_title"];

    // Act & Assert: 验证prompt包含JSON格式说明
    for keyword in json_keywords.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(keyword));
    }
}

#[test]
fn test_reword_pr_system_prompt_contains_language_requirement_with_prompt_contains_language() {
    // Arrange: 准备语言要求关键词列表
    let language_keywords = ["English", "All outputs MUST be in English"];

    // Act & Assert: 验证prompt包含语言要求
    for keyword in language_keywords.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(keyword));
    }
}

#[test]
fn test_reword_pr_system_prompt_contains_markdown_support_with_prompt_contains_markdown() {
    // Arrange: 准备markdown关键词列表
    let markdown_keywords = ["markdown", "markdown heading"];

    // Act & Assert: 验证prompt包含markdown格式支持说明
    let contains_markdown = markdown_keywords.iter()
        .any(|keyword| REWORD_PR_SYSTEM_PROMPT.contains(keyword));
    assert!(contains_markdown);
}

#[test]
fn test_reword_pr_system_prompt_length_with_prompt_has_reasonable_length() {
    // Arrange: 准备最小长度阈值
    let min_length = 500;

    // Act & Assert: 验证prompt有合理的长度（至少应该包含基本内容）
    assert!(REWORD_PR_SYSTEM_PROMPT.len() > min_length);
}
