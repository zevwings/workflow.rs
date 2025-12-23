//! Base/LLM Languages 模块测试
//!
//! 测试语言查找和 instruction 生成功能。

use pretty_assertions::assert_eq;
use workflow::base::llm::languages::{
    find_language, get_language_instruction, get_language_requirement,
    get_supported_language_codes, get_supported_language_display_names,
    SUPPORTED_LANGUAGES,
};

#[test]
fn test_find_language_exact_match() {
    // 测试精确匹配
    let lang = find_language("en");
    assert!(lang.is_some());
    assert_eq!(lang.unwrap().code, "en");
    assert_eq!(lang.unwrap().name, "English");
}

#[test]
fn test_find_language_case_insensitive() {
    // 测试大小写不敏感匹配
    let lang1 = find_language("EN");
    let lang2 = find_language("en");
    let lang3 = find_language("En");

    assert!(lang1.is_some());
    assert!(lang2.is_some());
    assert!(lang3.is_some());
    assert_eq!(lang1.unwrap().code, lang2.unwrap().code);
    assert_eq!(lang2.unwrap().code, lang3.unwrap().code);
}

#[test]
fn test_find_language_zh_variants() {
    // 测试中文变体匹配
    let lang_zh = find_language("zh");
    let lang_zh_cn = find_language("zh-CN");

    assert!(lang_zh.is_some());
    assert!(lang_zh_cn.is_some());
    assert_eq!(lang_zh.unwrap().code, "zh-CN");
    assert_eq!(lang_zh_cn.unwrap().code, "zh-CN");
}

#[test]
fn test_find_language_zh_tw() {
    // 测试繁体中文
    let lang = find_language("zh-TW");
    assert!(lang.is_some());
    assert_eq!(lang.unwrap().code, "zh-TW");
    assert_eq!(lang.unwrap().name, "Traditional Chinese");
}

#[test]
fn test_find_language_not_found() {
    // 测试未找到的语言
    let lang = find_language("xx");
    assert!(lang.is_none());
}

#[test]
fn test_get_language_instruction_found() {
    // 测试获取找到的语言 instruction
    let instruction = get_language_instruction("en");
    assert!(!instruction.is_empty());
    assert!(instruction.contains("English"));
}

#[test]
fn test_get_language_instruction_not_found() {
    // 测试获取未找到的语言 instruction（应该返回英文默认值）
    let instruction = get_language_instruction("xx");
    assert!(!instruction.is_empty());
    // 应该返回英文的默认 instruction
    assert!(instruction.contains("English"));
}

#[test]
fn test_get_language_instruction_zh_variants() {
    // 测试中文变体的 instruction
    let instruction_zh = get_language_instruction("zh");
    let instruction_zh_cn = get_language_instruction("zh-CN");

    assert_eq!(instruction_zh, instruction_zh_cn);
    assert!(instruction_zh.contains("简体中文"));
}

#[test]
fn test_get_language_requirement_default() {
    // 测试获取语言要求（增强 system prompt）- 默认语言
    let original = "You are a helpful assistant.";
    let enhanced = get_language_requirement(original);

    assert!(enhanced.contains(original));
    assert!(enhanced.contains("CRITICAL LANGUAGE REQUIREMENT"));
    assert!(enhanced.contains("REMINDER: Language Requirement"));
    // 默认应该是英文
    assert!(enhanced.contains("English"));
}

#[test]
fn test_get_language_requirement_with_language() {
    // 测试获取语言要求（增强 system prompt）- 指定语言
    // 注意：这个测试依赖于 Settings，可能需要设置环境变量或配置文件
    let original = "You are a helpful assistant.";
    let enhanced = get_language_requirement(original);

    // 验证基本结构
    assert!(enhanced.contains(original));
    assert!(enhanced.contains("CRITICAL LANGUAGE REQUIREMENT"));
    assert!(enhanced.contains("REMINDER: Language Requirement"));
}

#[test]
fn test_get_language_requirement_format() {
    // 测试 get_language_requirement 的格式
    let original = "Test prompt";
    let enhanced = get_language_requirement(original);

    // 验证格式包含所有必要的部分
    assert!(enhanced.starts_with("## CRITICAL LANGUAGE REQUIREMENT"));
    assert!(enhanced.contains("**IMPORTANT REMINDER**"));
    assert!(enhanced.contains(original));
    assert!(enhanced.contains("## REMINDER: Language Requirement"));
    assert!(enhanced.ends_with("No exceptions."));
}

#[test]
fn test_get_supported_language_codes() {
    // 测试获取所有支持的语言代码列表
    let codes = get_supported_language_codes();

    assert!(!codes.is_empty());
    assert!(codes.contains(&"en"));
    assert!(codes.contains(&"zh-CN"));
    assert!(codes.contains(&"zh-TW"));
    assert!(codes.contains(&"ja"));
    assert!(codes.contains(&"ko"));
}

#[test]
fn test_get_supported_language_display_names() {
    // 测试获取所有支持的语言显示名称列表
    let display_names = get_supported_language_display_names();

    assert!(!display_names.is_empty());
    assert_eq!(display_names.len(), SUPPORTED_LANGUAGES.len());

    // 验证格式："{native_name} ({name}) - {code}"
    let en_display = display_names.iter().find(|n| n.contains("English"));
    assert!(en_display.is_some());
    assert!(en_display.unwrap().contains("en"));
}

#[test]
fn test_supported_languages_structure() {
    // 测试 SUPPORTED_LANGUAGES 的结构
    assert!(!SUPPORTED_LANGUAGES.is_empty());

    for lang in SUPPORTED_LANGUAGES {
        assert!(!lang.code.is_empty());
        assert!(!lang.name.is_empty());
        assert!(!lang.native_name.is_empty());
        assert!(!lang.instruction_template.is_empty());
    }
}

#[test]
fn test_find_language_all_supported() {
    // 测试查找所有支持的语言
    for lang in SUPPORTED_LANGUAGES {
        let found = find_language(lang.code);
        assert!(found.is_some(), "Language {} should be found", lang.code);
        assert_eq!(found.unwrap().code, lang.code);
    }
}

#[test]
fn test_get_language_instruction_all_supported() {
    // 测试获取所有支持语言的 instruction
    for lang in SUPPORTED_LANGUAGES {
        let instruction = get_language_instruction(lang.code);
        assert!(!instruction.is_empty(), "Instruction for {} should not be empty", lang.code);
        assert_eq!(instruction, lang.instruction_template);
    }
}

