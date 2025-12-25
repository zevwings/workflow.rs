//! Base/Prompt Reword PR 模块测试
//!
//! 测试 PR Reword 的 system prompt。

use workflow::base::prompt::REWORD_PR_SYSTEM_PROMPT;

// ==================== Reword PR System Prompt Tests ====================

/// 测试Reword PR system prompt常量不为空
///
/// ## 测试目的
/// 验证 `REWORD_PR_SYSTEM_PROMPT` 常量已正确定义且不为空。
///
/// ## 测试场景
/// 1. 检查prompt常量
/// 2. 验证常量不为空
///
/// ## 预期结果
/// - prompt常量不为空
#[test]
fn test_reword_pr_system_prompt_not_empty_with_constant_returns_non_empty() {
    // Arrange: 准备检查prompt常量

    // Act & Assert: 验证prompt常量不为空
    assert!(!REWORD_PR_SYSTEM_PROMPT.is_empty());
}

/// 测试Reword PR system prompt包含关键词
///
/// ## 测试目的
/// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含所有必需的关键词（PR title, description, PR diff）。
///
/// ## 测试场景
/// 1. 准备关键词列表
/// 2. 验证prompt包含所有关键词
///
/// ## 预期结果
/// - prompt包含所有必需的关键词
#[test]
fn test_reword_pr_system_prompt_contains_keywords_with_prompt_contains_keywords() {
    // Arrange: 准备关键词列表
    let keywords = ["PR title", "description", "PR diff"];

    // Act & Assert: 验证prompt包含所有关键词
    for keyword in keywords.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(keyword));
    }
}

/// 测试Reword PR system prompt包含规则说明
///
/// ## 测试目的
/// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含PR标题和描述的规则说明。
///
/// ## 测试场景
/// 1. 准备规则关键词列表
/// 2. 验证prompt包含所有规则说明
///
/// ## 预期结果
/// - prompt包含 "PR Title Rules" 和 "Description Rules"
#[test]
fn test_reword_pr_system_prompt_contains_rules_with_prompt_contains_rules() {
    // Arrange: 准备规则关键词列表
    let rules = ["PR Title Rules", "Description Rules"];

    // Act & Assert: 验证prompt包含所有规则说明
    for rule in rules.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(rule));
    }
}

/// 测试Reword PR system prompt包含示例
///
/// ## 测试目的
/// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含使用示例和响应格式说明。
///
/// ## 测试场景
/// 1. 准备示例关键词列表
/// 2. 验证prompt包含示例
///
/// ## 预期结果
/// - prompt包含 "Example" 和 "Response Format"
#[test]
fn test_reword_pr_system_prompt_contains_examples_with_prompt_contains_examples() {
    // Arrange: 准备示例关键词列表
    let examples = ["Example", "Response Format"];

    // Act & Assert: 验证prompt包含示例
    for example in examples.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(example));
    }
}

/// 测试Reword PR system prompt包含JSON格式说明
///
/// ## 测试目的
/// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含JSON响应格式的说明。
///
/// ## 测试场景
/// 1. 准备JSON格式关键词列表
/// 2. 验证prompt包含JSON格式说明
///
/// ## 预期结果
/// - prompt包含 "JSON" 和 "pr_title" 等关键词
#[test]
fn test_reword_pr_system_prompt_contains_json_format_with_prompt_contains_json() {
    // Arrange: 准备JSON格式关键词列表
    let json_keywords = ["JSON", "pr_title"];

    // Act & Assert: 验证prompt包含JSON格式说明
    for keyword in json_keywords.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(keyword));
    }
}

/// 测试Reword PR system prompt包含语言要求
///
/// ## 测试目的
/// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含语言要求说明（所有输出必须是英文）。
///
/// ## 测试场景
/// 1. 准备语言要求关键词列表
/// 2. 验证prompt包含语言要求
///
/// ## 预期结果
/// - prompt包含 "English" 和 "All outputs MUST be in English"
#[test]
fn test_reword_pr_system_prompt_contains_language_requirement_with_prompt_contains_language() {
    // Arrange: 准备语言要求关键词列表
    let language_keywords = ["English", "All outputs MUST be in English"];

    // Act & Assert: 验证prompt包含语言要求
    for keyword in language_keywords.iter() {
        assert!(REWORD_PR_SYSTEM_PROMPT.contains(keyword));
    }
}

/// 测试Reword PR system prompt包含Markdown格式支持说明
///
/// ## 测试目的
/// 验证 `REWORD_PR_SYSTEM_PROMPT` 包含Markdown格式支持的说明。
///
/// ## 测试场景
/// 1. 准备markdown关键词列表
/// 2. 验证prompt包含markdown格式支持说明
///
/// ## 预期结果
/// - prompt包含 "markdown" 或 "markdown heading" 等关键词
#[test]
fn test_reword_pr_system_prompt_contains_markdown_support_with_prompt_contains_markdown() {
    // Arrange: 准备markdown关键词列表
    let markdown_keywords = ["markdown", "markdown heading"];

    // Act & Assert: 验证prompt包含markdown格式支持说明
    let contains_markdown = markdown_keywords.iter()
        .any(|keyword| REWORD_PR_SYSTEM_PROMPT.contains(keyword));
    assert!(contains_markdown);
}

/// 测试Reword PR system prompt长度合理
///
/// ## 测试目的
/// 验证 `REWORD_PR_SYSTEM_PROMPT` 有合理的长度，至少包含基本内容（最小长度阈值500字符）。
///
/// ## 测试场景
/// 1. 检查prompt长度
/// 2. 验证长度超过最小阈值
///
/// ## 预期结果
/// - prompt长度大于500字符
#[test]
fn test_reword_pr_system_prompt_length_with_prompt_has_reasonable_length() {
    // Arrange: 准备最小长度阈值
    let min_length = 500;

    // Act & Assert: 验证prompt有合理的长度（至少应该包含基本内容）
    assert!(REWORD_PR_SYSTEM_PROMPT.len() > min_length);
}
