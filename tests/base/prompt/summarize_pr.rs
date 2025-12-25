//! Base/Prompt Summarize PR 模块测试
//!
//! 测试 PR 总结 system prompt 生成功能。

use workflow::base::prompt::generate_summarize_pr_system_prompt;

// ==================== Summarize PR System Prompt Generation Tests ====================

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

#[test]
fn test_generate_summarize_pr_system_prompt_contains_language_requirement_with_prompt_contains_language() {
    // Arrange: 准备生成prompt
    let prompt = generate_summarize_pr_system_prompt();

    // Act & Assert: 验证包含语言增强内容（通过get_language_requirement添加）
    assert!(prompt.contains("CRITICAL LANGUAGE REQUIREMENT"));
    assert!(prompt.contains("REMINDER: Language Requirement"));
}

#[test]
fn test_generate_summarize_pr_system_prompt_contains_document_structure_with_prompt_contains_structure() {
    // Arrange: 准备生成prompt和文档结构关键词列表
    let prompt = generate_summarize_pr_system_prompt();
    let structure_keywords = [
        "PR Title", "Overview", "Requirements Analysis", "Key Changes",
        "Files Changed", "Technical Details", "Testing", "Usage Instructions",
    ];

    // Act & Assert: 验证包含文档结构说明
    for keyword in structure_keywords.iter() {
        assert!(prompt.contains(keyword));
    }
}

#[test]
fn test_generate_summarize_pr_system_prompt_consistent_with_multiple_calls_returns_same_result() {
    // Arrange: 准备多次调用

    // Act: 多次生成prompt
    let prompt1 = generate_summarize_pr_system_prompt();
    let prompt2 = generate_summarize_pr_system_prompt();

    // Assert: 验证多次调用返回一致的结果
    assert_eq!(prompt1, prompt2);
}
