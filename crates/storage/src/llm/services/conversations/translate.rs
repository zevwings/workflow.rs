//! 翻译对话
//!
//! 本模块提供了使用 LLM 将非英文文本翻译为英文的功能。

use domain::LLMError;

use crate::llm::services::{parsers::TextParser, prompt, LLMConversation};

/// 翻译对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(crate) struct TranslateConversation {
    input: String,
}

impl TranslateConversation {
    /// 创建新的翻译对话实例
    pub fn new(input: String) -> Self {
        Self { input }
    }
}

impl LLMConversation for TranslateConversation {
    type Input = String;
    type Output = String;

    fn get_system_prompt(&self, _language_code: &str) -> String {
        prompt::translate().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        format!("Translate this text to English: {}", self.input)
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (Some(100), 0.3) // max_tokens，翻译通常比较短
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        TextParser::clean_and_validate(response)
            .map_err(|e| LLMError::ApiError(format!("LLM returned empty translation: {}", e)))
    }
}
