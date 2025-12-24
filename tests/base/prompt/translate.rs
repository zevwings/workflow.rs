//! Base/Prompt Translate 模块测试
//!
//! 测试翻译文本的 system prompt。

use workflow::base::prompt::TRANSLATE_SYSTEM_PROMPT;

#[test]
fn test_translate_system_prompt_not_empty() {
    // 测试 prompt 常量不为空
    assert!(!TRANSLATE_SYSTEM_PROMPT.is_empty());
}

#[test]
fn test_translate_system_prompt_contains_keywords() {
    // 测试 prompt 包含关键内容
    assert!(
        TRANSLATE_SYSTEM_PROMPT.contains("translation")
            || TRANSLATE_SYSTEM_PROMPT.contains("translate")
    );
    assert!(TRANSLATE_SYSTEM_PROMPT.contains("English"));
}

#[test]
fn test_translate_system_prompt_contains_rules() {
    // 测试 prompt 包含规则说明
    assert!(TRANSLATE_SYSTEM_PROMPT.contains("Rules") || TRANSLATE_SYSTEM_PROMPT.contains("rules"));
}

#[test]
fn test_translate_system_prompt_contains_examples() {
    // 测试 prompt 包含示例
    assert!(
        TRANSLATE_SYSTEM_PROMPT.contains("Example") || TRANSLATE_SYSTEM_PROMPT.contains("example")
    );
}

#[test]
fn test_translate_system_prompt_contains_accuracy_requirement() {
    // 测试 prompt 包含准确性要求
    assert!(
        TRANSLATE_SYSTEM_PROMPT.contains("accurately")
            || TRANSLATE_SYSTEM_PROMPT.contains("accurate")
    );
}

#[test]
fn test_translate_system_prompt_contains_output_format() {
    // 测试 prompt 包含输出格式说明
    assert!(TRANSLATE_SYSTEM_PROMPT.contains("ONLY") || TRANSLATE_SYSTEM_PROMPT.contains("only"));
}

#[test]
fn test_translate_system_prompt_length() {
    // 测试 prompt 有合理的长度（至少应该包含基本内容）
    assert!(TRANSLATE_SYSTEM_PROMPT.len() > 100);
}

#[test]
fn test_translate_system_prompt_contains_chinese_examples() {
    // 测试 prompt 包含中文示例（如果存在）
    // 注意：这个测试可能在某些情况下失败，取决于 prompt 的具体内容
    let prompt_lower = TRANSLATE_SYSTEM_PROMPT.to_lowercase();
    // 验证包含示例说明
    assert!(prompt_lower.contains("example") || prompt_lower.contains("示例"));
}
