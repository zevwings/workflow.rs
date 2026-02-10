//! LLM 支持的语言定义
//!
//! 定义了支持的语言列表及其对应的 instruction，用于增强 LLM prompt 中的语言要求。

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
const SUPPORTED_LANGUAGES: &[SupportedLanguage] = &[
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

impl SupportedLanguage {
    /// 获取默认语言（英文）
    ///
    /// # 返回
    ///
    /// 返回英文语言的静态引用
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use domain::SupportedLanguage;
    ///
    /// let default_lang = SupportedLanguage::default_language();
    /// assert_eq!(default_lang.code, "en");
    /// ```
    pub fn default_language() -> &'static SupportedLanguage {
        &SUPPORTED_LANGUAGES[0]
    }

    /// 获取所有支持的语言列表
    ///
    /// # 返回
    ///
    /// 返回所有支持的语言的静态引用切片
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use domain::SupportedLanguage;
    ///
    /// let languages = SupportedLanguage::get();
    /// assert!(!languages.is_empty());
    /// ```
    pub fn get() -> &'static [SupportedLanguage] {
        SUPPORTED_LANGUAGES
    }

    /// 根据语言代码查找支持的语言
    ///
    /// # 参数
    ///
    /// * `code` - 语言代码（如 "en", "zh-CN", "zh" 等）
    ///
    /// # 返回
    ///
    /// 如果找到匹配的语言，返回 `Some(&SupportedLanguage)`，否则返回 `None`
    ///
    /// # 说明
    ///
    /// 支持的语言代码变体：
    /// - "zh" 和 "zh-CN" 都匹配简体中文
    /// - "zh-TW" 匹配繁体中文
    /// - 其他语言代码精确匹配
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use domain::SupportedLanguage;
    ///
    /// let lang = SupportedLanguage::find("zh-CN");
    /// assert!(lang.is_some());
    /// ```
    pub fn find(code: &str) -> Option<&'static SupportedLanguage> {
        let code_lower = code.to_lowercase();

        // 特殊处理：zh 和 zh-cn 都匹配简体中文
        if code_lower == "zh" || code_lower == "zh-cn" {
            return Self::get().iter().find(|lang| lang.code == "zh-CN");
        }

        // 精确匹配
        Self::get().iter().find(|lang| lang.code.to_lowercase() == code_lower)
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
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use domain::SupportedLanguage;
    ///
    /// let instruction = SupportedLanguage::get_instruction("zh-CN");
    /// assert!(!instruction.is_empty());
    /// ```
    pub fn get_instruction(code: &str) -> String {
        Self::find(code)
            .map(|lang| lang.instruction_template.to_string())
            .unwrap_or_else(|| {
                // 如果找不到匹配的语言，使用英文的默认 instruction
                Self::get()[0].instruction_template.to_string()
            })
    }

    /// 获取所有支持的语言代码列表
    ///
    /// # 返回
    ///
    /// 返回所有支持的语言代码的向量
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use domain::SupportedLanguage;
    ///
    /// let codes = SupportedLanguage::supported_codes();
    /// assert!(!codes.is_empty());
    /// ```
    pub fn supported_codes() -> Vec<&'static str> {
        Self::get().iter().map(|lang| lang.code).collect()
    }

    /// 获取所有支持的语言显示名称列表
    ///
    /// 格式："{native_name} ({name}) - {code}"
    ///
    /// # 返回
    ///
    /// 返回格式化的语言名称列表
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use domain::SupportedLanguage;
    ///
    /// let names = SupportedLanguage::supported_display_names();
    /// assert!(!names.is_empty());
    /// ```
    pub fn supported_display_names() -> Vec<String> {
        Self::get()
            .iter()
            .map(|lang| format!("{} ({}) - {}", lang.native_name, lang.name, lang.code))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // default_language 测试
    // ========================================================================

    #[test]
    fn test_default_language() {
        let lang = SupportedLanguage::default_language();
        assert_eq!(lang.code, "en");
        assert_eq!(lang.name, "English");
    }

    // ========================================================================
    // get 测试
    // ========================================================================

    #[test]
    fn test_get_returns_all_languages() {
        let languages = SupportedLanguage::get();
        assert!(!languages.is_empty());
        assert!(languages.len() >= 10); // 至少有 10 种语言
    }

    #[test]
    fn test_get_contains_major_languages() {
        let languages = SupportedLanguage::get();
        let codes: Vec<&str> = languages.iter().map(|l| l.code).collect();

        assert!(codes.contains(&"en"));
        assert!(codes.contains(&"zh-CN"));
        assert!(codes.contains(&"zh-TW"));
        assert!(codes.contains(&"ja"));
        assert!(codes.contains(&"ko"));
    }

    // ========================================================================
    // find 测试
    // ========================================================================

    #[test]
    fn test_find_exact_match() {
        let lang = SupportedLanguage::find("en");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().code, "en");

        let lang = SupportedLanguage::find("zh-CN");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().code, "zh-CN");
    }

    #[test]
    fn test_find_zh_alias() {
        // "zh" 应该匹配简体中文
        let lang = SupportedLanguage::find("zh");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().code, "zh-CN");

        // "zh-cn"（小写）也应该匹配
        let lang = SupportedLanguage::find("zh-cn");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().code, "zh-CN");
    }

    #[test]
    fn test_find_case_insensitive() {
        let lang = SupportedLanguage::find("EN");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().code, "en");

        let lang = SupportedLanguage::find("ZH-CN");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().code, "zh-CN");
    }

    #[test]
    fn test_find_not_found() {
        assert!(SupportedLanguage::find("invalid").is_none());
        assert!(SupportedLanguage::find("").is_none());
        assert!(SupportedLanguage::find("xyz").is_none());
    }

    // ========================================================================
    // get_instruction 测试
    // ========================================================================

    #[test]
    fn test_get_instruction_found() {
        let instruction = SupportedLanguage::get_instruction("zh-CN");
        assert!(!instruction.is_empty());
        assert!(instruction.contains("简体中文"));
    }

    #[test]
    fn test_get_instruction_default_fallback() {
        // 找不到时返回英文的默认 instruction
        let instruction = SupportedLanguage::get_instruction("invalid");
        assert!(!instruction.is_empty());
        assert!(instruction.contains("English"));
    }

    // ========================================================================
    // supported_codes 测试
    // ========================================================================

    #[test]
    fn test_supported_codes() {
        let codes = SupportedLanguage::supported_codes();
        assert!(!codes.is_empty());
        assert!(codes.contains(&"en"));
        assert!(codes.contains(&"zh-CN"));
        assert!(codes.contains(&"ja"));
    }

    // ========================================================================
    // supported_display_names 测试
    // ========================================================================

    #[test]
    fn test_supported_display_names() {
        let names = SupportedLanguage::supported_display_names();
        assert!(!names.is_empty());

        // 检查格式："{native_name} ({name}) - {code}"
        let english_name = names.iter().find(|n| n.contains("en")).unwrap();
        assert!(english_name.contains("English"));
        assert!(english_name.contains("(English)"));
        assert!(english_name.contains("- en"));

        let chinese_name = names.iter().find(|n| n.contains("zh-CN")).unwrap();
        assert!(chinese_name.contains("简体中文"));
        assert!(chinese_name.contains("(Simplified Chinese)"));
    }

    #[test]
    fn test_find_with_whitespace() {
        // 包含空白字符的输入应该无法匹配
        assert!(SupportedLanguage::find(" en ").is_none());
        assert!(SupportedLanguage::find("zh-CN ").is_none());
        assert!(SupportedLanguage::find(" en").is_none());
    }

    #[test]
    fn test_find_with_special_chars() {
        // 特殊字符应该无法匹配
        assert!(SupportedLanguage::find("en@").is_none());
        assert!(SupportedLanguage::find("zh#CN").is_none());
        assert!(SupportedLanguage::find("en!").is_none());
    }

    #[test]
    fn test_get_instruction_with_empty_string() {
        // 空字符串应该返回默认英文 instruction
        let instruction = SupportedLanguage::get_instruction("");
        assert!(!instruction.is_empty());
        assert!(instruction.contains("English"));
    }

    #[test]
    fn test_supported_codes_contains_all_languages() {
        let codes = SupportedLanguage::supported_codes();
        assert!(codes.contains(&"en"));
        assert!(codes.contains(&"zh-CN"));
        assert!(codes.contains(&"zh-TW"));
        assert!(codes.contains(&"ja"));
        assert!(codes.contains(&"ko"));
        assert!(codes.contains(&"de"));
        assert!(codes.contains(&"fr"));
        assert!(codes.contains(&"es"));
        assert!(codes.contains(&"pt"));
        assert!(codes.contains(&"ru"));
    }

    #[test]
    fn test_supported_codes_length_matches_get() {
        let codes = SupportedLanguage::supported_codes();
        let languages = SupportedLanguage::get();
        assert_eq!(codes.len(), languages.len());
    }

    #[test]
    fn test_get_instruction_all_supported_languages() {
        // 验证所有支持的语言都能返回有效的 instruction
        for code in SupportedLanguage::supported_codes() {
            let instruction = SupportedLanguage::get_instruction(code);
            assert!(
                !instruction.is_empty(),
                "Language {} should have instruction",
                code
            );
        }
    }

    #[test]
    fn test_find_zh_tw_case_variations() {
        // zh-TW 应该精确匹配，不匹配 zh
        let lang = SupportedLanguage::find("zh-TW");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().code, "zh-TW");

        let lang = SupportedLanguage::find("zh-tw");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().code, "zh-TW");
    }
}
