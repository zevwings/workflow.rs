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

pub trait LanguageManager: Send + Sync + 'static {
    /// 获取默认语言
    fn get_default_language(&self) -> &'static SupportedLanguage;

    /// 获取所有支持的语言
    fn get_supported_languages(&self) -> &'static [SupportedLanguage];

    /// 根据语言代码查找支持的语言
    fn find_language(&self, code: &str) -> Option<&'static SupportedLanguage>;

    /// 获取语言的 instruction
    fn get_language_instruction(&self, code: &str) -> String;

    /// 获取所有支持的语言代码
    fn get_supported_codes(&self) -> Vec<&'static str>;

    /// 获取所有支持的语言显示名称
    /// 格式："{native_name} ({name}) - {code}"
    fn get_supported_display_names(&self) -> Vec<String>;
}
