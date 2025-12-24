//! Base/Prompt Summarize PR 模块测试
//!
//! 测试 PR 总结 system prompt 生成功能。

use workflow::base::prompt::generate_summarize_pr_system_prompt;

#[test]
fn test_generate_summarize_pr_system_prompt() {
    // 测试生成 PR 总结的 system prompt
    let prompt = generate_summarize_pr_system_prompt();

    // 验证返回的 prompt 不为空
    assert!(!prompt.is_empty());

    // 验证包含关键内容
    assert!(prompt.contains("CRITICAL LANGUAGE REQUIREMENT"));
    assert!(prompt.contains("Summary Document Rules"));
    assert!(prompt.contains("Filename Rules"));
    assert!(prompt.contains("Response Format"));
    assert!(prompt.contains("summary"));
    assert!(prompt.contains("filename"));

    // 验证包含 JSON 示例
    assert!(prompt.contains("add-user-authentication"));
}

#[test]
fn test_generate_summarize_pr_system_prompt_contains_language_requirement() {
    // 测试 prompt 包含语言要求
    let prompt = generate_summarize_pr_system_prompt();

    // 验证包含语言增强内容（通过 get_language_requirement 添加）
    assert!(prompt.contains("CRITICAL LANGUAGE REQUIREMENT"));
    assert!(prompt.contains("REMINDER: Language Requirement"));
}

#[test]
fn test_generate_summarize_pr_system_prompt_contains_document_structure() {
    // 测试 prompt 包含文档结构要求
    let prompt = generate_summarize_pr_system_prompt();

    // 验证包含文档结构说明
    assert!(prompt.contains("PR Title"));
    assert!(prompt.contains("Overview"));
    assert!(prompt.contains("Requirements Analysis"));
    assert!(prompt.contains("Key Changes"));
    assert!(prompt.contains("Files Changed"));
    assert!(prompt.contains("Technical Details"));
    assert!(prompt.contains("Testing"));
    assert!(prompt.contains("Usage Instructions"));
}

#[test]
fn test_generate_summarize_pr_system_prompt_consistent() {
    // 测试多次调用返回一致的结果
    let prompt1 = generate_summarize_pr_system_prompt();
    let prompt2 = generate_summarize_pr_system_prompt();

    assert_eq!(prompt1, prompt2);
}
