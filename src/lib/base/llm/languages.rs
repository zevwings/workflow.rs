//! LLM 支持的语言定义
//!
//! 定义了支持的语言列表及其对应的 instruction，用于增强 LLM prompt 中的语言要求。

use crate::base::settings::Settings;

/// 支持的语言信息
#[derive(Debug, Clone)]
pub struct SupportedLanguage {
    /// 语言代码（ISO 639-1 或 ISO 639-1 + ISO 3166-1，如 "en", "zh-CN"）
    pub code: &'static str,
    /// 语言名称（英文）
    pub name: &'static str,
    /// 语言名称（本地化，用于显示）
    pub native_name: &'static str,
    /// 语言 instruction 模板
    /// 使用 {language_name} 作为占位符
    pub instruction_template: &'static str,
}

/// 支持的语言列表
///
/// 包含主流语言：英语、中文（简体/繁体）、日语、韩语、德语、法语、西班牙语等
pub const SUPPORTED_LANGUAGES: &[SupportedLanguage] = &[
    SupportedLanguage {
        code: "en",
        name: "English",
        native_name: "English",
        instruction_template: "**All outputs MUST be in English only.** If the PR title or content contains non-English text (like Chinese), translate it to English in the summary.",
    },
    SupportedLanguage {
        code: "zh-CN",
        name: "Simplified Chinese",
        native_name: "简体中文",
        instruction_template: "**所有输出必须使用简体中文。** 如果 PR 标题或内容包含非中文文本（如英文），请在总结中翻译为中文。",
    },
    SupportedLanguage {
        code: "zh-TW",
        name: "Traditional Chinese",
        native_name: "繁體中文",
        instruction_template: "**所有輸出必須使用繁體中文。** 如果 PR 標題或內容包含非中文文本（如英文），請在總結中翻譯為繁體中文。",
    },
    SupportedLanguage {
        code: "ja",
        name: "Japanese",
        native_name: "日本語",
        instruction_template: "**すべての出力は日本語のみで行う必要があります。** PR タイトルまたはコンテンツに非日本語テキスト（英語など）が含まれている場合は、要約で日本語に翻訳してください。",
    },
    SupportedLanguage {
        code: "ko",
        name: "Korean",
        native_name: "한국어",
        instruction_template: "**모든 출력은 한국어로만 작성해야 합니다.** PR 제목이나 내용에 비한국어 텍스트(예: 영어)가 포함된 경우 요약에서 한국어로 번역하세요.",
    },
    SupportedLanguage {
        code: "de",
        name: "German",
        native_name: "Deutsch",
        instruction_template: "**Alle Ausgaben MÜSSEN ausschließlich auf Deutsch sein.** Wenn der PR-Titel oder Inhalt nicht-deutschen Text (z.B. Englisch) enthält, übersetzen Sie ihn in der Zusammenfassung ins Deutsche.",
    },
    SupportedLanguage {
        code: "fr",
        name: "French",
        native_name: "Français",
        instruction_template: "**Toutes les sorties DOIVENT être uniquement en français.** Si le titre ou le contenu de la PR contient du texte non français (comme l'anglais), traduisez-le en français dans le résumé.",
    },
    SupportedLanguage {
        code: "es",
        name: "Spanish",
        native_name: "Español",
        instruction_template: "**Todas las salidas DEBEN estar únicamente en español.** Si el título o el contenido de la PR contiene texto no español (como inglés), tradúzcalo al español en el resumen.",
    },
    SupportedLanguage {
        code: "pt",
        name: "Portuguese",
        native_name: "Português",
        instruction_template: "**Todas as saídas DEVEM estar exclusivamente em português.** Se o título ou o conteúdo da PR contiver texto não português (como inglês), traduza-o para português no resumo.",
    },
    SupportedLanguage {
        code: "ru",
        name: "Russian",
        native_name: "Русский",
        instruction_template: "**Все выходные данные ДОЛЖНЫ быть только на русском языке.** Если заголовок или содержимое PR содержит текст не на русском языке (например, английский), переведите его на русский в резюме.",
    },
];

/// 根据语言代码查找支持的语言
///
/// # 参数
///
/// * `code` - 语言代码（如 "en", "zh-CN", "zh" 等）
///
/// # 返回
///
/// 如果找到匹配的语言，返回 `Some(SupportedLanguage)`，否则返回 `None`
///
/// # 说明
///
/// 支持的语言代码变体：
/// - "zh" 和 "zh-CN" 都匹配简体中文
/// - "zh-TW" 匹配繁体中文
/// - 其他语言代码精确匹配
pub fn find_language(code: &str) -> Option<&SupportedLanguage> {
    let code_lower = code.to_lowercase();

    // 特殊处理：zh 和 zh-cn 都匹配简体中文
    if code_lower == "zh" || code_lower == "zh-cn" {
        return SUPPORTED_LANGUAGES.iter().find(|lang| lang.code == "zh-CN");
    }

    // 精确匹配
    SUPPORTED_LANGUAGES.iter().find(|lang| lang.code.to_lowercase() == code_lower)
}

/// 获取语言的 instruction
///
/// # 参数
///
/// * `code` - 语言代码
///
/// # 返回
///
/// 如果找到匹配的语言，返回对应的 instruction，否则返回英文的默认 instruction
pub fn get_language_instruction(code: &str) -> String {
    find_language(code)
        .map(|lang| lang.instruction_template.to_string())
        .unwrap_or_else(|| {
            // 如果找不到匹配的语言，使用英文的默认 instruction
            SUPPORTED_LANGUAGES[0].instruction_template.to_string()
        })
}

/// 增强 system prompt 中的语言要求
///
/// 在给定的 system prompt 开头添加强化的语言要求，确保 LLM 严格按照指定语言生成内容。
///
/// # 参数
///
/// * `system_prompt` - 原始 system prompt
/// # 返回
///
/// 返回增强后的 system prompt，包含强化的语言要求
///
/// # 说明
///
/// 语言选择优先级：配置文件 > 默认值（"en"）
/// 如果配置文件中的语言代码不在支持列表中，将使用英文作为默认语言。
///
/// # 示例
///
/// ```rust
/// use workflow::get_language_requirement;
///
/// let original = "You are a helpful assistant.";
/// let enhanced = get_language_requirement(original);
/// // 返回包含强化语言要求的 prompt（语言从配置文件读取）
/// ```
pub fn get_language_requirement(system_prompt: &str) -> String {
    // 从配置文件读取语言设置
    let settings = Settings::get();
    let language_code = if settings.llm.language.is_empty() {
        "en"
    } else {
        settings.llm.language.as_str()
    };

    let language_instruction = get_language_instruction(language_code);
    let language_info =
        find_language(language_code).map(|lang| lang.native_name).unwrap_or("English");

    format!(
        r#"## CRITICAL LANGUAGE REQUIREMENT

{}

**IMPORTANT REMINDER**: The entire output, including all sections, headings, content, and text MUST be written in {} only. This is a strict requirement. Do NOT use English or any other language. Every single word in the output must be in {}.

---

{}

---

## REMINDER: Language Requirement

Remember: ALL output must be in {} only. No exceptions."#,
        language_instruction, language_info, language_info, system_prompt, language_info
    )
}

/// 获取所有支持的语言代码列表
///
/// # 返回
///
/// 返回所有支持的语言代码的向量
pub fn get_supported_language_codes() -> Vec<&'static str> {
    SUPPORTED_LANGUAGES.iter().map(|lang| lang.code).collect()
}

/// 获取所有支持的语言显示名称列表
///
/// 格式："{native_name} ({name}) - {code}"
///
/// # 返回
///
/// 返回格式化的语言名称列表
pub fn get_supported_language_display_names() -> Vec<String> {
    SUPPORTED_LANGUAGES
        .iter()
        .map(|lang| format!("{} ({}) - {}", lang.native_name, lang.name, lang.code))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use pretty_assertions::assert_eq;

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
    fn test_find_language_exact_match_with_valid_code_returns_language() -> Result<()> {
        // Arrange: 准备有效的语言代码
        let code = "en";

        // Act: 查找语言
        let lang = find_language(code);

        // Assert: 验证找到正确的语言
        let lang = lang.ok_or_else(|| color_eyre::eyre::eyre!("Language 'en' should be found"))?;
        assert_eq!(lang.code, "en");
        assert_eq!(lang.name, "English");
        Ok(())
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
    fn test_find_language_case_insensitive_with_different_cases_returns_same_language() -> Result<()>
    {
        // Arrange: 准备不同大小写的语言代码
        let codes = ["EN", "en", "En"];

        // Act: 查找不同大小写的语言
        let lang1 = find_language(codes[0]);
        let lang2 = find_language(codes[1]);
        let lang3 = find_language(codes[2]);

        // Assert: 验证所有变体都找到相同的语言
        let lang1 =
            lang1.ok_or_else(|| color_eyre::eyre::eyre!("Language 'EN' should be found"))?;
        let lang2 =
            lang2.ok_or_else(|| color_eyre::eyre::eyre!("Language 'en' should be found"))?;
        let lang3 =
            lang3.ok_or_else(|| color_eyre::eyre::eyre!("Language 'En' should be found"))?;
        assert_eq!(lang1.code, lang2.code);
        assert_eq!(lang2.code, lang3.code);
        Ok(())
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
    fn test_find_language_zh_variants_with_zh_codes_returns_zh_cn() -> Result<()> {
        // Arrange: 准备中文变体代码
        let codes = ["zh", "zh-CN"];

        // Act: 查找中文变体
        let lang_zh = find_language(codes[0]);
        let lang_zh_cn = find_language(codes[1]);

        // Assert: 验证都返回 zh-CN
        let lang_zh =
            lang_zh.ok_or_else(|| color_eyre::eyre::eyre!("Language 'zh' should be found"))?;
        let lang_zh_cn = lang_zh_cn
            .ok_or_else(|| color_eyre::eyre::eyre!("Language 'zh-CN' should be found"))?;
        assert_eq!(lang_zh.code, "zh-CN");
        assert_eq!(lang_zh_cn.code, "zh-CN");
        Ok(())
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
    fn test_find_language_zh_tw_with_valid_code_returns_traditional_chinese() -> Result<()> {
        // Arrange: 准备繁体中文代码
        let code = "zh-TW";

        // Act: 查找语言
        let lang = find_language(code);

        // Assert: 验证返回繁体中文
        let lang =
            lang.ok_or_else(|| color_eyre::eyre::eyre!("Language 'zh-TW' should be found"))?;
        assert_eq!(lang.code, "zh-TW");
        assert_eq!(lang.name, "Traditional Chinese");
        Ok(())
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
    fn test_get_supported_language_display_names() -> Result<()> {
        // Arrange: 准备测试获取所有支持的语言显示名称列表
        let display_names = get_supported_language_display_names();

        assert!(!display_names.is_empty());
        assert_eq!(display_names.len(), SUPPORTED_LANGUAGES.len());

        // Assert: 验证格式："{native_name} ({name}) - {code}"
        let en_display = display_names
            .iter()
            .find(|n| n.contains("English"))
            .ok_or_else(|| color_eyre::eyre::eyre!("English display name should be found"))?;
        assert!(en_display.contains("en"));
        Ok(())
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
        // Note: SUPPORTED_LANGUAGES is a compile-time constant, so is_empty() check is not needed

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
    fn test_find_language_all_supported() -> Result<()> {
        // Arrange: 准备测试查找所有支持的语言
        for lang in SUPPORTED_LANGUAGES {
            let found = find_language(lang.code)
                .ok_or_else(|| color_eyre::eyre::eyre!("Language {} should be found", lang.code))?;
            assert_eq!(found.code, lang.code);
        }
        Ok(())
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
}
