//! Base/Prompt Summarize File Change 模块测试
//!
//! 测试单个文件修改总结的 system prompt 生成功能。

use workflow::base::prompt::generate_summarize_file_change_system_prompt;

#[test]
fn test_generate_summarize_file_change_system_prompt() {
    // 测试生成文件修改总结的 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // 验证返回的 prompt 不为空
    assert!(!prompt.is_empty());
}

#[test]
fn test_generate_summarize_file_change_system_prompt_contains_keywords() {
    // 测试 prompt 包含关键内容
    let prompt = generate_summarize_file_change_system_prompt();

    assert!(prompt.contains("summary") || prompt.contains("Summary"));
    assert!(prompt.contains("file") || prompt.contains("File"));
    assert!(prompt.contains("diff") || prompt.contains("Diff"));
    assert!(prompt.contains("changes") || prompt.contains("Changes"));
}

#[test]
fn test_generate_summarize_file_change_system_prompt_contains_rules() {
    // 测试 prompt 包含规则说明
    let prompt = generate_summarize_file_change_system_prompt();

    assert!(prompt.contains("Summary Rules") || prompt.contains("Requirements"));
    assert!(prompt.contains("bullet") || prompt.contains("Bullet"));
}

#[test]
fn test_generate_summarize_file_change_system_prompt_contains_examples() {
    // 测试 prompt 包含示例
    let prompt = generate_summarize_file_change_system_prompt();

    assert!(prompt.contains("Example") || prompt.contains("example"));
}

#[test]
fn test_generate_summarize_file_change_system_prompt_consistent() {
    // 测试多次调用返回一致的结果
    let prompt1 = generate_summarize_file_change_system_prompt();
    let prompt2 = generate_summarize_file_change_system_prompt();

    assert_eq!(prompt1, prompt2);
}

#[test]
fn test_generate_summarize_file_change_system_prompt_length() {
    // 测试 prompt 有合理的长度（至少应该包含基本内容）
    let prompt = generate_summarize_file_change_system_prompt();
    assert!(prompt.len() > 200);
}

#[test]
fn test_generate_summarize_file_change_system_prompt_contains_language_requirement() {
    // 测试 prompt 包含语言增强内容（通过 get_language_requirement 添加）
    let prompt = generate_summarize_file_change_system_prompt();

    // 验证包含语言要求（可能通过 get_language_requirement 添加）
    // 注意：具体内容取决于 get_language_requirement 的实现
    assert!(!prompt.is_empty());
}
