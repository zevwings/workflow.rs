//! Base/LLM Languages 模块测试
//!
//! 测试语言查找和 instruction 生成功能。
//!
//! ## 测试策略
//!
//! - 测试语言查找、指令生成和语言要求功能
//! - 使用 `expect()` 替代 `unwrap()` 提供清晰的错误消息
//! - 测试所有支持的语言和边界情况

use pretty_assertions::assert_eq;
use workflow::base::llm::languages::{
    find_language, get_language_instruction, get_language_requirement,
    get_supported_language_codes, get_supported_language_display_names, SUPPORTED_LANGUAGES,
};

// ==================== Language Finding Tests ====================

/// 测试精确匹配查找语言
///
/// ## 测试目的
/// 验证 find_language() 能够通过有效的语言代码查找语言。
///
/// ## 测试场景
/// 1. 使用有效的语言代码查找语言
/// 2. 验证找到正确的语言
///
/// ## 预期结果
/// - 返回对应的语言信息
#[test]
fn test_find_language_exact_match_with_valid_code_returns_language() {
    // Arrange: 准备有效的语言代码
    let code = "en";

    // Act: 查找语言
    let lang = find_language(code);

    // Assert: 验证找到正确的语言
    assert!(lang.is_some());
    let lang = lang.expect("Language 'en' should be found");
    assert_eq!(lang.code, "en");
    assert_eq!(lang.name, "English");
}

/// 测试大小写不敏感查找语言
///
/// ## 测试目的
/// 验证 find_language() 支持大小写不敏感的查找。
///
/// ## 测试场景
/// 1. 使用不同大小写的语言代码查找语言
/// 2. 验证所有变体都找到相同的语言
///
/// ## 预期结果
/// - 所有大小写变体都返回相同的语言
#[test]
fn test_find_language_case_insensitive_with_different_cases_returns_same_language() {
    // Arrange: 准备不同大小写的语言代码
    let codes = ["EN", "en", "En"];

    // Act: 查找不同大小写的语言
    let lang1 = find_language(codes[0]);
    let lang2 = find_language(codes[1]);
    let lang3 = find_language(codes[2]);

    // Assert: 验证所有变体都找到相同的语言
    assert!(lang1.is_some());
    assert!(lang2.is_some());
    assert!(lang3.is_some());
    let lang1 = lang1.expect("Language 'EN' should be found");
    let lang2 = lang2.expect("Language 'en' should be found");
    let lang3 = lang3.expect("Language 'En' should be found");
    assert_eq!(lang1.code, lang2.code);
    assert_eq!(lang2.code, lang3.code);
}

/// 测试中文变体代码查找
///
/// ## 测试目的
/// 验证 find_language() 能够正确处理中文变体代码（zh、zh-CN）。
///
/// ## 测试场景
/// 1. 使用中文变体代码查找语言
/// 2. 验证都返回 zh-CN
///
/// ## 预期结果
/// - zh 和 zh-CN 都返回 zh-CN
#[test]
fn test_find_language_zh_variants_with_zh_codes_returns_zh_cn() {
    // Arrange: 准备中文变体代码
    let codes = ["zh", "zh-CN"];

    // Act: 查找中文变体
    let lang_zh = find_language(codes[0]);
    let lang_zh_cn = find_language(codes[1]);

    // Assert: 验证都返回 zh-CN
    assert!(lang_zh.is_some());
    assert!(lang_zh_cn.is_some());
    let lang_zh = lang_zh.expect("Language 'zh' should be found");
    let lang_zh_cn = lang_zh_cn.expect("Language 'zh-CN' should be found");
    assert_eq!(lang_zh.code, "zh-CN");
    assert_eq!(lang_zh_cn.code, "zh-CN");
}

/// 测试繁体中文代码查找
///
/// ## 测试目的
/// 验证 find_language() 能够通过 zh-TW 代码查找繁体中文。
///
/// ## 测试场景
/// 1. 使用 zh-TW 代码查找语言
/// 2. 验证返回繁体中文
///
/// ## 预期结果
/// - 返回繁体中文语言信息
#[test]
fn test_find_language_zh_tw_with_valid_code_returns_traditional_chinese() {
    // Arrange: 准备繁体中文代码
    let code = "zh-TW";

    // Act: 查找语言
    let lang = find_language(code);

    // Assert: 验证返回繁体中文
    assert!(lang.is_some());
    let lang = lang.expect("Language 'zh-TW' should be found");
    assert_eq!(lang.code, "zh-TW");
    assert_eq!(lang.name, "Traditional Chinese");
}

/// 测试无效语言代码查找
///
/// ## 测试目的
/// 验证 find_language() 对无效的语言代码返回 None。
///
/// ## 测试场景
/// 1. 使用无效的语言代码查找语言
/// 2. 验证返回 None
///
/// ## 预期结果
/// - 无效代码返回 None
#[test]
fn test_find_language_not_found_with_invalid_code_returns_none() {
    // Arrange: 准备无效的语言代码
    let code = "xx";

    // Act: 查找语言
    let lang = find_language(code);

    // Assert: 验证返回 None
    assert!(lang.is_none());
}

// ==================== Language Instruction Tests ====================

/// 测试获取语言指令（有效代码）
///
/// ## 测试目的
/// 验证 get_language_instruction() 能够为有效的语言代码返回指令。
///
/// ## 测试场景
/// 1. 使用有效的语言代码获取指令
/// 2. 验证返回非空指令且包含语言名称
///
/// ## 预期结果
/// - 返回包含语言名称的指令
#[test]
fn test_get_language_instruction_found_with_valid_code_returns_instruction() {
    // Arrange: 准备有效的语言代码
    let code = "en";

    // Act: 获取语言指令
    let instruction = get_language_instruction(code);

    // Assert: 验证返回非空指令且包含语言名称
    assert!(!instruction.is_empty());
    assert!(instruction.contains("English"));
}

/// 测试获取语言指令（无效代码）
///
/// ## 测试目的
/// 验证 get_language_instruction() 对无效的语言代码返回默认英文指令。
///
/// ## 测试场景
/// 1. 使用无效的语言代码获取指令
/// 2. 验证返回默认英文指令
///
/// ## 预期结果
/// - 返回默认英文指令
#[test]
fn test_get_language_instruction_not_found_with_invalid_code_returns_default() {
    // Arrange: 准备无效的语言代码
    let code = "xx";

    // Act: 获取语言指令
    let instruction = get_language_instruction(code);

    // Assert: 验证返回默认英文指令
    assert!(!instruction.is_empty());
    assert!(instruction.contains("English"));
}

/// 测试获取中文变体的语言指令
///
/// ## 测试目的
/// 验证 get_language_instruction() 对中文变体返回相同的指令。
///
/// ## 测试场景
/// 1. 获取 zh 和 zh-CN 的指令
/// 2. 验证指令相同且包含简体中文
///
/// ## 预期结果
/// - zh 和 zh-CN 返回相同的指令，包含"简体中文"
#[test]
fn test_get_language_instruction_zh_variants() {
    // Arrange: 准备测试中文变体的 instruction
    let instruction_zh = get_language_instruction("zh");
    let instruction_zh_cn = get_language_instruction("zh-CN");

    assert_eq!(instruction_zh, instruction_zh_cn);
    assert!(instruction_zh.contains("简体中文"));
}

/// 测试获取语言要求（默认语言）
///
/// ## 测试目的
/// 验证 get_language_requirement() 能够增强 system prompt 并添加语言要求。
///
/// ## 测试场景
/// 1. 提供原始 prompt
/// 2. 获取增强后的 prompt
/// 3. 验证包含语言要求部分
///
/// ## 预期结果
/// - 增强后的 prompt 包含原始内容和语言要求，默认使用英文
#[test]
fn test_get_language_requirement_default() {
    // Arrange: 准备测试获取语言要求（增强 system prompt）- 默认语言
    let original = "You are a helpful assistant.";
    let enhanced = get_language_requirement(original);

    assert!(enhanced.contains(original));
    assert!(enhanced.contains("CRITICAL LANGUAGE REQUIREMENT"));
    assert!(enhanced.contains("REMINDER: Language Requirement"));
    // 默认应该是英文
    assert!(enhanced.contains("English"));
}

/// 测试获取语言要求（指定语言）
///
/// ## 测试目的
/// 验证 get_language_requirement() 能够为指定语言增强 system prompt。
///
/// ## 测试场景
/// 1. 提供原始 prompt
/// 2. 获取增强后的 prompt（可能使用配置的语言）
/// 3. 验证包含语言要求部分
///
/// ## 预期结果
/// - 增强后的 prompt 包含语言要求部分
#[test]
fn test_get_language_requirement_with_language() {
    // Arrange: 准备测试获取语言要求（增强 system prompt）- 指定语言
    // 注意：这个测试依赖于 Settings，可能需要设置环境变量或配置文件
    let original = "You are a helpful assistant.";
    let enhanced = get_language_requirement(original);

    // Assert: 验证基本结构
    assert!(enhanced.contains(original));
    assert!(enhanced.contains("CRITICAL LANGUAGE REQUIREMENT"));
    assert!(enhanced.contains("REMINDER: Language Requirement"));
}

/// 测试语言要求的格式
///
/// ## 测试目的
/// 验证 get_language_requirement() 返回的格式包含所有必要的部分。
///
/// ## 测试场景
/// 1. 提供原始 prompt
/// 2. 获取增强后的 prompt
/// 3. 验证格式包含所有必要的部分
///
/// ## 预期结果
/// - 格式包含 CRITICAL LANGUAGE REQUIREMENT、REMINDER 等部分
#[test]
fn test_get_language_requirement_format() {
    // Arrange: 准备测试 get_language_requirement 的格式
    let original = "Test prompt";
    let enhanced = get_language_requirement(original);

    // Assert: 验证格式包含所有必要的部分
    assert!(enhanced.starts_with("## CRITICAL LANGUAGE REQUIREMENT"));
    assert!(enhanced.contains("**IMPORTANT REMINDER**"));
    assert!(enhanced.contains(original));
    assert!(enhanced.contains("## REMINDER: Language Requirement"));
    assert!(enhanced.ends_with("No exceptions."));
}

/// 测试获取所有支持的语言代码列表
///
/// ## 测试目的
/// 验证 get_supported_language_codes() 返回所有支持的语言代码。
///
/// ## 测试场景
/// 1. 获取支持的语言代码列表
/// 2. 验证列表不为空且包含常见语言代码
///
/// ## 预期结果
/// - 列表包含 en、zh-CN、zh-TW、ja、ko 等语言代码
#[test]
fn test_get_supported_language_codes() {
    // Arrange: 准备测试获取所有支持的语言代码列表
    let codes = get_supported_language_codes();

    assert!(!codes.is_empty());
    assert!(codes.contains(&"en"));
    assert!(codes.contains(&"zh-CN"));
    assert!(codes.contains(&"zh-TW"));
    assert!(codes.contains(&"ja"));
    assert!(codes.contains(&"ko"));
}

/// 测试获取所有支持的语言显示名称列表
///
/// ## 测试目的
/// 验证 get_supported_language_display_names() 返回所有支持的语言显示名称。
///
/// ## 测试场景
/// 1. 获取支持的语言显示名称列表
/// 2. 验证列表不为空且格式正确
///
/// ## 预期结果
/// - 列表包含格式为 "{native_name} ({name}) - {code}" 的显示名称
#[test]
fn test_get_supported_language_display_names() {
    // Arrange: 准备测试获取所有支持的语言显示名称列表
    let display_names = get_supported_language_display_names();

    assert!(!display_names.is_empty());
    assert_eq!(display_names.len(), SUPPORTED_LANGUAGES.len());

    // Assert: 验证格式："{native_name} ({name}) - {code}"
    let en_display = display_names.iter().find(|n| n.contains("English"));
    assert!(en_display.is_some());
    let en_display = en_display.expect("English display name should be found");
    assert!(en_display.contains("en"));
}

/// 测试 SUPPORTED_LANGUAGES 的结构
///
/// ## 测试目的
/// 验证 SUPPORTED_LANGUAGES 常量中所有语言的结构正确。
///
/// ## 测试场景
/// 1. 遍历所有支持的语言
/// 2. 验证每个语言的字段都不为空
///
/// ## 预期结果
/// - 所有语言的 code、name、native_name、instruction_template 都不为空
#[test]
fn test_supported_languages_structure() {
    // Arrange: 准备测试 SUPPORTED_LANGUAGES 的结构
    assert!(!SUPPORTED_LANGUAGES.is_empty());

    for lang in SUPPORTED_LANGUAGES {
        assert!(!lang.code.is_empty());
        assert!(!lang.name.is_empty());
        assert!(!lang.native_name.is_empty());
        assert!(!lang.instruction_template.is_empty());
    }
}

/// 测试查找所有支持的语言
///
/// ## 测试目的
/// 验证 find_language() 能够查找所有支持的语言。
///
/// ## 测试场景
/// 1. 遍历所有支持的语言
/// 2. 使用每个语言的代码查找
/// 3. 验证都能找到对应的语言
///
/// ## 预期结果
/// - 所有支持的语言都能被找到
#[test]
fn test_find_language_all_supported() {
    // Arrange: 准备测试查找所有支持的语言
    for lang in SUPPORTED_LANGUAGES {
        let found = find_language(lang.code);
        assert!(found.is_some(), "Language {} should be found", lang.code);
        let found = found.expect(&format!("Language {} should be found", lang.code));
        assert_eq!(found.code, lang.code);
    }
}

/// 测试获取所有支持语言的指令
///
/// ## 测试目的
/// 验证 get_language_instruction() 能够为所有支持的语言返回指令。
///
/// ## 测试场景
/// 1. 遍历所有支持的语言
/// 2. 获取每个语言的指令
/// 3. 验证指令不为空且与模板一致
///
/// ## 预期结果
/// - 所有语言的指令都不为空且与模板一致
#[test]
fn test_get_language_instruction_all_supported() {
    // Arrange: 准备测试获取所有支持语言的 instruction
    for lang in SUPPORTED_LANGUAGES {
        let instruction = get_language_instruction(lang.code);
        assert!(
            !instruction.is_empty(),
            "Instruction for {} should not be empty",
            lang.code
        );
        assert_eq!(instruction, lang.instruction_template);
    }
}
