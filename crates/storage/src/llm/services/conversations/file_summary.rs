//! 单个文件修改总结对话
//!
//! 用于生成单个文件的修改总结。

use domain::LLMError;
use toolkit::Truncate;

use crate::llm::services::{parsers::TextParser, prompt, LLMConversation};

/// 文件修改总结对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(crate) struct FileSummaryConversation {
    input: (String, String),
}

impl FileSummaryConversation {
    /// 创建新的文件修改总结对话实例
    pub fn new(input: (String, String)) -> Self {
        Self { input }
    }

    /// 构建 user prompt
    fn build_user_prompt(file_path: &str, file_diff: &str) -> String {
        // 限制单个文件的 diff 长度，避免超过 LLM token 限制
        const MAX_FILE_DIFF_LENGTH: usize = 8000;
        let diff_trimmed = file_diff.truncate(
            MAX_FILE_DIFF_LENGTH,
            "\n... (file diff truncated, {} characters total)",
        );

        format!("File path: {}\n\nFile diff:\n{}", file_path, diff_trimmed)
    }
}

impl LLMConversation for FileSummaryConversation {
    type Input = (String, String);
    type Output = String;

    fn get_system_prompt(&self, language_code: &str) -> String {
        prompt::file_summary_with_language(language_code)
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        let (file_path, file_diff) = &self.input;
        Self::build_user_prompt(file_path, file_diff)
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3) // max_tokens: None, temperature: 0.3
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        TextParser::clean_and_validate(response)
            .map_err(|e| LLMError::ApiError(format!("Failed to parse response: {}", e)))
    }
}
