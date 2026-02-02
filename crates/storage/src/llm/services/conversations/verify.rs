//! 验证对话
//!
//! 用于验证 LLM 配置是否有效。

use domain::{LLMError, SupportedLanguage};

use crate::llm::services::{parsers::TextParser, LLMConversation};

/// 验证对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(crate) struct VerifyConversation;

impl VerifyConversation {
    /// 创建新的验证对话实例
    pub fn new() -> Self {
        Self
    }
}

impl LLMConversation for VerifyConversation {
    type Input = String;
    type Output = String;

    fn get_system_prompt(&self, language_code: &str) -> String {
        let base_prompt = "You are a helpful assistant.";
        SupportedLanguage::get_requirement(base_prompt, language_code)
    }

    fn get_user_prompt(&self, language_code: &str) -> String {
        let lang = match SupportedLanguage::find(language_code) {
            Some(lang) => lang,
            None => SupportedLanguage::default_language(),
        };

        let greeting = format!(
            "Say hello, and you should respond in {}({}) only.",
            lang.name, lang.code
        );
        greeting.to_string()
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.5) // max_tokens 由 LLM 自动决定
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        TextParser::clean_and_validate(response)
            .map(|s| s.trim().to_string())
            .map_err(|e| LLMError::ApiError(format!("Failed to parse response: {}", e)))
    }
}
