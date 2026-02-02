//! 提交信息对话
//!
//! 用于根据代码变更生成符合 Conventional Commits 格式的提交信息。

use domain::LLMError;

use crate::llm::services::{parsers::TextParser, LLMConversation};

/// 提交信息对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(crate) struct CommitMessageConversation {
    input: String,
}

impl CommitMessageConversation {
    /// 创建新的提交信息对话实例
    pub fn new(input: String) -> Self {
        Self { input }
    }
}

impl LLMConversation for CommitMessageConversation {
    type Input = String;
    type Output = String;

    fn get_system_prompt(&self, _language_code: &str) -> String {
        "You are a helpful assistant that generates commit messages following Conventional Commits format (type(scope): subject). Keep the message concise and clear.".to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        format!(
            "Generate a concise commit message (following Conventional Commits format) for these changes:\n\n{}",
            self.input
        )
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (Some(100), 0.3) // max_tokens，提交信息通常比较短
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        TextParser::clean_and_validate(response)
            .map(|s| s.trim().to_string())
            .map_err(|e| LLMError::ApiError(format!("Failed to parse response: {}", e)))
    }
}
