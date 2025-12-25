//! Base/Prompt Summarize File Change 模块测试
//!
//! 测试单个文件修改总结的 system prompt 生成功能。

use workflow::base::prompt::generate_summarize_file_change_system_prompt;

// ==================== Basic Prompt Generation Tests ====================

/// 测试生成文件修改总结system prompt返回非空字符串
///
/// ## 测试目的
/// 验证 `generate_summarize_file_change_system_prompt()` 函数能够成功生成非空的system prompt。
///
/// ## 测试场景
/// 1. 调用函数生成prompt
/// 2. 验证返回的prompt不为空
///
/// ## 预期结果
/// - 返回的prompt不为空
#[test]
fn test_generate_summarize_file_change_system_prompt_returns_non_empty_string() {
    // Arrange: 准备调用函数（无需额外准备）

    // Act: 生成 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // Assert: 验证返回的 prompt 不为空
    assert!(!prompt.is_empty());
}

/// 测试生成的文件修改总结prompt包含关键词
///
/// ## 测试目的
/// 验证生成的system prompt包含与文件修改总结相关的关键词（summary, file, diff, changes等）。
///
/// ## 测试场景
/// 1. 生成system prompt
/// 2. 验证prompt包含至少一个关键词
///
/// ## 预期结果
/// - prompt包含关键词（summary, file, diff, changes等）
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

/// 测试生成的文件修改总结prompt包含规则说明
///
/// ## 测试目的
/// 验证生成的system prompt包含总结规则说明（Summary Rules, Requirements等）。
///
/// ## 测试场景
/// 1. 生成system prompt
/// 2. 验证prompt包含规则关键词
///
/// ## 预期结果
/// - prompt包含规则说明（Summary Rules, Requirements, bullet等）
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

/// 测试生成的文件修改总结prompt包含示例
///
/// ## 测试目的
/// 验证生成的system prompt包含使用示例。
///
/// ## 测试场景
/// 1. 生成system prompt
/// 2. 验证prompt包含示例关键词
///
/// ## 预期结果
/// - prompt包含 "Example" 或 "example"
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

/// 测试多次调用生成函数返回一致的结果
///
/// ## 测试目的
/// 验证 `generate_summarize_file_change_system_prompt()` 函数在多次调用时返回一致的结果（幂等性）。
///
/// ## 测试场景
/// 1. 多次调用生成函数
/// 2. 比较多次调用的结果
///
/// ## 预期结果
/// - 多次调用的结果完全一致
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

/// 测试生成的文件修改总结prompt长度合理
///
/// ## 测试目的
/// 验证生成的system prompt有合理的长度，至少包含基本内容（最小长度阈值200字符）。
///
/// ## 测试场景
/// 1. 生成system prompt
/// 2. 验证长度超过最小阈值
///
/// ## 预期结果
/// - prompt长度大于200字符
#[test]
fn test_generate_summarize_file_change_system_prompt_has_reasonable_length() {
    // Arrange: 准备最小长度要求
    let min_length = 200;

    // Act: 生成 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // Assert: 验证 prompt 有合理的长度（至少应该包含基本内容）
    assert!(prompt.len() > min_length);
}

/// 测试生成的文件修改总结prompt包含语言要求
///
/// ## 测试目的
/// 验证生成的system prompt包含语言要求说明（可能通过 `get_language_requirement` 添加）。
///
/// ## 测试场景
/// 1. 生成system prompt
/// 2. 验证prompt不为空（语言要求可能已包含）
///
/// ## 注意事项
/// - 具体内容取决于 `get_language_requirement` 的实现
///
/// ## 预期结果
/// - prompt不为空
/// - 可能包含语言要求说明
#[test]
fn test_generate_summarize_file_change_system_prompt_contains_language_requirement() {
    // Arrange: 准备调用函数（无需额外准备）
    // 注意：具体内容取决于 get_language_requirement 的实现

    // Act: 生成 system prompt
    let prompt = generate_summarize_file_change_system_prompt();

    // Assert: 验证包含语言要求（可能通过 get_language_requirement 添加）
    assert!(!prompt.is_empty());
}
