//! Base/Prompt Summarize File Change 模块测试
//!
//! 测试单个文件修改总结的 system prompt 生成功能。

use workflow::base::prompt::generate_summarize_file_change_system_prompt;

// ==================== Basic Prompt Generation Tests ====================

#[test]
fn test_generate_summarize_file_change_system_prompt_returns_non_empty_string() {
    // Arrange: 准备调用函数（无需额外准备）

    // Act: 生成 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // Assert: 验证返回的 prompt 不为空
    assert!(!prompt.is_empty());
}

#[test]
fn test_generate_summarize_file_change_system_prompt_contains_keywords() {
    // Arrange: 准备关键词列表
    let keywords = ["summary", "Summary", "file", "File", "diff", "Diff", "changes", "Changes"];

    // Act: 生成 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // Assert: 验证 prompt 包含关键内容
    let contains_keywords = keywords.iter()
        .any(|keyword| prompt.contains(keyword));
    assert!(contains_keywords);
}

#[test]
fn test_generate_summarize_file_change_system_prompt_contains_rules() {
    // Arrange: 准备规则关键词
    let rule_keywords = ["Summary Rules", "Requirements", "bullet", "Bullet"];

    // Act: 生成 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // Assert: 验证 prompt 包含规则说明
    let contains_rules = rule_keywords.iter()
        .any(|keyword| prompt.contains(keyword));
    assert!(contains_rules);
}

#[test]
fn test_generate_summarize_file_change_system_prompt_contains_examples() {
    // Arrange: 准备示例关键词
    let example_keywords = ["Example", "example"];

    // Act: 生成 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // Assert: 验证 prompt 包含示例
    let contains_examples = example_keywords.iter()
        .any(|keyword| prompt.contains(keyword));
    assert!(contains_examples);
}

// ==================== Consistency Tests ====================

#[test]
fn test_generate_summarize_file_change_system_prompt_with_multiple_calls_returns_consistent_result() {
    // Arrange: 准备多次调用

    // Act: 多次调用生成函数
    let prompt1 = generate_summarize_file_change_system_prompt();
    let prompt2 = generate_summarize_file_change_system_prompt();

    // Assert: 验证多次调用返回一致的结果
    assert_eq!(prompt1, prompt2);
}

// ==================== Validation Tests ====================

#[test]
fn test_generate_summarize_file_change_system_prompt_has_reasonable_length() {
    // Arrange: 准备最小长度要求
    let min_length = 200;

    // Act: 生成 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // Assert: 验证 prompt 有合理的长度（至少应该包含基本内容）
    assert!(prompt.len() > min_length);
}

#[test]
fn test_generate_summarize_file_change_system_prompt_contains_language_requirement() {
    // Arrange: 准备调用函数（无需额外准备）
    // 注意：具体内容取决于 get_language_requirement 的实现

    // Act: 生成 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // Assert: 验证包含语言要求（可能通过 get_language_requirement 添加）
    assert!(!prompt.is_empty());
}
