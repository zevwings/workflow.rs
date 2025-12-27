//! Base/Prompt Summarize PR 模块测试
//!
//! 测试 PR 总结 system prompt 生成功能。

use workflow::base::prompt::generate_summarize_pr_system_prompt;

// ==================== Summarize PR System Prompt Generation Tests ====================

/// 测试生成PR总结system prompt（无参数）
///
/// ## 测试目的
/// 验证 `generate_summarize_pr_system_prompt()` 函数能够生成非空的system prompt，并包含所有关键内容。
///
/// ## 测试场景
/// 1. 调用函数生成prompt
/// 2. 验证prompt不为空
/// 3. 验证prompt包含关键内容
///
/// ## 预期结果
/// - prompt不为空
/// - 包含 "CRITICAL LANGUAGE REQUIREMENT"
/// - 包含 "Summary Document Rules"、"Filename Rules"、"Response Format"
/// - 包含示例关键词（summary, filename, add-user-authentication）
#[test]
fn test_generate_summarize_pr_system_prompt_with_no_parameters_returns_prompt() {
    // Arrange: 准备生成prompt

    // Act: 生成PR总结的system prompt
    let prompt = generate_summarize_pr_system_prompt();

    // Assert: 验证返回的prompt不为空且包含关键内容
    assert!(!prompt.is_empty());
    assert!(prompt.contains("CRITICAL LANGUAGE REQUIREMENT"));
    assert!(prompt.contains("Summary Document Rules"));
    assert!(prompt.contains("Filename Rules"));
    assert!(prompt.contains("Response Format"));
    assert!(prompt.contains("summary"));
    assert!(prompt.contains("filename"));
    assert!(prompt.contains("add-user-authentication"));
}

/// 测试生成的PR总结prompt包含语言要求
///
/// ## 测试目的
/// 验证生成的system prompt包含语言要求说明（通过 `get_language_requirement` 添加）。
///
/// ## 测试场景
/// 1. 生成system prompt
/// 2. 验证包含语言要求关键词
///
/// ## 预期结果
/// - prompt包含 "CRITICAL LANGUAGE REQUIREMENT"
/// - prompt包含 "REMINDER: Language Requirement"
#[test]
fn test_generate_summarize_pr_system_prompt_contains_language_requirement_with_prompt_contains_language(
) {
    // Arrange: 准备生成prompt
    let prompt = generate_summarize_pr_system_prompt();

    // Act & Assert: 验证包含语言增强内容（通过get_language_requirement添加）
    assert!(prompt.contains("CRITICAL LANGUAGE REQUIREMENT"));
    assert!(prompt.contains("REMINDER: Language Requirement"));
}

/// 测试生成的PR总结prompt包含文档结构说明
///
/// ## 测试目的
/// 验证生成的system prompt包含文档结构说明，包括所有必需的章节。
///
/// ## 测试场景
/// 1. 生成system prompt
/// 2. 准备文档结构关键词列表
/// 3. 验证prompt包含所有关键词
///
/// ## 预期结果
/// - prompt包含所有文档结构关键词：
///   - PR Title, Overview, Requirements Analysis, Key Changes
///   - Files Changed, Technical Details, Testing, Usage Instructions
#[test]
fn test_generate_summarize_pr_system_prompt_contains_document_structure_with_prompt_contains_structure(
) {
    // Arrange: 准备生成prompt和文档结构关键词列表
    let prompt = generate_summarize_pr_system_prompt();
    let structure_keywords = [
        "PR Title",
        "Overview",
        "Requirements Analysis",
        "Key Changes",
        "Files Changed",
        "Technical Details",
        "Testing",
        "Usage Instructions",
    ];

    // Act & Assert: 验证包含文档结构说明
    for keyword in structure_keywords.iter() {
        assert!(prompt.contains(keyword));
    }
}

/// 测试多次调用生成函数返回一致的结果
///
/// ## 测试目的
/// 验证 `generate_summarize_pr_system_prompt()` 函数在多次调用时返回一致的结果（幂等性）。
///
/// ## 测试场景
/// 1. 多次调用生成函数
/// 2. 比较多次调用的结果
///
/// ## 预期结果
/// - 多次调用的结果完全一致
/// - 函数具有幂等性
#[test]
fn test_generate_summarize_pr_system_prompt_consistent_with_multiple_calls_returns_same_result() {
    // Arrange: 准备多次调用

    // Act: 多次生成prompt
    let prompt1 = generate_summarize_pr_system_prompt();
    let prompt2 = generate_summarize_pr_system_prompt();

    // Assert: 验证多次调用返回一致的结果
    assert_eq!(prompt1, prompt2);
}
