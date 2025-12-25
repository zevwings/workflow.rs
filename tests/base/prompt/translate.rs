//! Base/Prompt Translate 模块测试
//!
//! 测试翻译文本的 system prompt。

use workflow::base::prompt::TRANSLATE_SYSTEM_PROMPT;

// ==================== Translate System Prompt Tests ====================

/// 测试翻译system prompt常量不为空
///
/// ## 测试目的
/// 验证 `TRANSLATE_SYSTEM_PROMPT` 常量已正确定义且不为空。
///
/// ## 测试场景
/// 1. 检查prompt常量
/// 2. 验证常量不为空
///
/// ## 预期结果
/// - prompt常量不为空
#[test]
fn test_translate_system_prompt_with_valid_constant_returns_non_empty() {
    // Arrange: 准备检查 prompt 常量

    // Act: 验证 prompt 常量不为空
    // (验证在 Assert 中完成)

    // Assert: 验证 prompt 常量不为空
    assert!(!TRANSLATE_SYSTEM_PROMPT.is_empty());
}

/// 测试翻译system prompt包含必需的关键词
///
/// ## 测试目的
/// 验证 `TRANSLATE_SYSTEM_PROMPT` 包含翻译和语言相关的关键词。
///
/// ## 测试场景
/// 1. 准备关键词列表（translation/translate, English）
/// 2. 验证prompt包含这些关键词
///
/// ## 预期结果
/// - prompt包含翻译关键词（translation或translate）
/// - prompt包含语言关键词（English）
#[test]
fn test_translate_system_prompt_contains_required_keywords() {
    // Arrange: 准备关键词列表
    let translation_keywords = ["translation", "translate"];
    let language_keyword = "English";

    // Act & Assert: 验证 prompt 包含翻译和语言关键词
    assert!(
        TRANSLATE_SYSTEM_PROMPT.contains(translation_keywords[0])
            || TRANSLATE_SYSTEM_PROMPT.contains(translation_keywords[1]),
        "Prompt should contain translation keyword"
    );
    assert!(
        TRANSLATE_SYSTEM_PROMPT.contains(language_keyword),
        "Prompt should contain language keyword"
    );
}

/// 测试翻译system prompt包含规则说明
///
/// ## 测试目的
/// 验证 `TRANSLATE_SYSTEM_PROMPT` 包含翻译规则说明。
///
/// ## 测试场景
/// 1. 准备规则关键词
/// 2. 验证prompt包含规则说明
///
/// ## 预期结果
/// - prompt包含 "Rules" 或 "rules"
#[test]
fn test_translate_system_prompt_contains_required_rules() {
    // Arrange: 准备规则关键词
    let rule_keywords = ["Rules", "rules"];

    // Act & Assert: 验证 prompt 包含规则说明
    assert!(
        TRANSLATE_SYSTEM_PROMPT.contains(rule_keywords[0])
            || TRANSLATE_SYSTEM_PROMPT.contains(rule_keywords[1]),
        "Prompt should contain rules"
    );
}

/// 测试翻译system prompt包含示例
///
/// ## 测试目的
/// 验证 `TRANSLATE_SYSTEM_PROMPT` 包含使用示例。
///
/// ## 测试场景
/// 1. 准备示例关键词
/// 2. 验证prompt包含示例
///
/// ## 预期结果
/// - prompt包含 "Example" 或 "example"
#[test]
fn test_translate_system_prompt_contains_examples() {
    // Arrange: 准备示例关键词
    let example_keywords = ["Example", "example"];

    // Act & Assert: 验证 prompt 包含示例
    assert!(
        TRANSLATE_SYSTEM_PROMPT.contains(example_keywords[0])
            || TRANSLATE_SYSTEM_PROMPT.contains(example_keywords[1]),
        "Prompt should contain examples"
    );
}

/// 测试翻译system prompt包含准确性要求
///
/// ## 测试目的
/// 验证 `TRANSLATE_SYSTEM_PROMPT` 包含翻译准确性要求说明。
///
/// ## 测试场景
/// 1. 准备准确性要求关键词
/// 2. 验证prompt包含准确性要求
///
/// ## 预期结果
/// - prompt包含 "accurately" 或 "accurate"
#[test]
fn test_translate_system_prompt_contains_accuracy_requirement() {
    // Arrange: 准备准确性要求关键词
    let accuracy_keywords = ["accurately", "accurate"];

    // Act & Assert: 验证 prompt 包含准确性要求
    assert!(
        TRANSLATE_SYSTEM_PROMPT.contains(accuracy_keywords[0])
            || TRANSLATE_SYSTEM_PROMPT.contains(accuracy_keywords[1]),
        "Prompt should contain accuracy requirement"
    );
}

/// 测试翻译system prompt包含输出格式说明
///
/// ## 测试目的
/// 验证 `TRANSLATE_SYSTEM_PROMPT` 包含输出格式的说明（只输出翻译结果）。
///
/// ## 测试场景
/// 1. 准备输出格式关键词
/// 2. 验证prompt包含输出格式说明
///
/// ## 预期结果
/// - prompt包含 "ONLY" 或 "only"
#[test]
fn test_translate_system_prompt_contains_output_format() {
    // Arrange: 准备输出格式关键词
    let format_keywords = ["ONLY", "only"];

    // Act & Assert: 验证 prompt 包含输出格式说明
    assert!(
        TRANSLATE_SYSTEM_PROMPT.contains(format_keywords[0])
            || TRANSLATE_SYSTEM_PROMPT.contains(format_keywords[1]),
        "Prompt should contain output format"
    );
}

/// 测试翻译system prompt长度合理
///
/// ## 测试目的
/// 验证 `TRANSLATE_SYSTEM_PROMPT` 有合理的长度，至少包含基本内容（最小长度阈值100字符）。
///
/// ## 测试场景
/// 1. 获取prompt长度
/// 2. 验证长度超过最小阈值
///
/// ## 预期结果
/// - prompt长度大于100字符
#[test]
fn test_translate_system_prompt_has_reasonable_length() {
    // Arrange: 准备最小长度要求
    let min_length = 100;

    // Act: 获取 prompt 长度
    let prompt_length = TRANSLATE_SYSTEM_PROMPT.len();

    // Assert: 验证 prompt 有合理的长度
    assert!(
        prompt_length > min_length,
        "Prompt should have reasonable length (at least {}), got {}",
        min_length,
        prompt_length
    );
}

/// 测试翻译system prompt包含示例说明
///
/// ## 测试目的
/// 验证 `TRANSLATE_SYSTEM_PROMPT` 包含示例说明（支持中英文关键词）。
///
/// ## 测试场景
/// 1. 将prompt转换为小写
/// 2. 验证包含示例关键词（example或示例）
///
/// ## 预期结果
/// - prompt包含 "example" 或 "示例"
#[test]
fn test_translate_system_prompt_contains_example_specification() {
    // Arrange: 准备示例说明关键词（转换为小写）
    let prompt_lower = TRANSLATE_SYSTEM_PROMPT.to_lowercase();
    let example_keywords = ["example", "示例"];

    // Act & Assert: 验证 prompt 包含示例说明
    assert!(
        prompt_lower.contains(example_keywords[0]) || prompt_lower.contains(example_keywords[1]),
        "Prompt should contain example specification"
    );
}
