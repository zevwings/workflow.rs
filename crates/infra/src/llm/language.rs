//! LLM 支持的语言定义
//!
//! 定义了支持的语言列表及其对应的 instruction，用于增强 LLM prompt 中的语言要求。

use client::{LanguageManager, SupportedLanguage};

pub struct LanguageManagerImpl;

impl Default for LanguageManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageManagerImpl {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageManager for LanguageManagerImpl {
    /// 获取默认语言（英文）
    ///
    /// # 返回
    ///
    /// 返回英文语言的静态引用
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use llm::SupportedLanguage;
    ///
    /// let default_lang = SupportedLanguage::default_language();
    /// assert_eq!(default_lang.code, "en");
    /// ```
    fn get_default_language(&self) -> &'static SupportedLanguage {
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
    /// use llm::SupportedLanguage;
    ///
    /// let languages = SupportedLanguage::get();
    /// assert!(!languages.is_empty());
    /// ```
    fn get_supported_languages(&self) -> &'static [SupportedLanguage] {
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
    /// use llm::SupportedLanguage;
    ///
    /// let lang = SupportedLanguage::find("zh-CN");
    /// assert!(lang.is_some());
    /// ```
    fn find_language(&self, code: &str) -> Option<&'static SupportedLanguage> {
        let code_lower = code.to_lowercase();

        // 特殊处理：zh 和 zh-cn 都匹配简体中文
        if code_lower == "zh" || code_lower == "zh-cn" {
            return self.get_supported_languages().iter().find(|lang| lang.code == "zh-CN");
        }

        // 精确匹配
        self.get_supported_languages()
            .iter()
            .find(|lang| lang.code.to_lowercase() == code_lower)
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
    /// use llm::SupportedLanguage;
    ///
    /// let instruction = SupportedLanguage::get_instruction("zh-CN");
    /// assert!(!instruction.is_empty());
    /// ```
    fn get_language_instruction(&self, code: &str) -> String {
        self.find_language(code)
            .map(|lang| lang.instruction_template.to_string())
            .unwrap_or_else(|| {
                // 如果找不到匹配的语言，使用英文的默认 instruction
                self.get_supported_languages()[0].instruction_template.to_string()
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
    /// use llm::SupportedLanguage;
    ///
    /// let codes = SupportedLanguage::supported_codes();
    /// assert!(!codes.is_empty());
    /// ```
    fn get_supported_codes(&self) -> Vec<&'static str> {
        self.get_supported_languages().iter().map(|lang| lang.code).collect()
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
    /// use llm::SupportedLanguage;
    ///
    /// let names = SupportedLanguage::supported_display_names();
    /// assert!(!names.is_empty());
    /// ```
    fn get_supported_display_names(&self) -> Vec<String> {
        self.get_supported_languages()
            .iter()
            .map(|lang| format!("{} ({}) - {}", lang.native_name, lang.name, lang.code))
            .collect()
    }
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
