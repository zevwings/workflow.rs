//! Base/Prompt Translate 模块测试
//!
//! 测试翻译文本的 system prompt。

use workflow::base::prompt::TRANSLATE_SYSTEM_PROMPT;

// ==================== Translate System Prompt Tests ====================

#[test]
fn test_translate_system_prompt_with_valid_constant_returns_non_empty() {
    // Arrange: 准备检查 prompt 常量

    // Act: 验证 prompt 常量不为空
    // (验证在 Assert 中完成)

    // Assert: 验证 prompt 常量不为空
    assert!(!TRANSLATE_SYSTEM_PROMPT.is_empty());
}

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
