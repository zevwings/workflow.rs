//! PR 总结文档对话
//!
//! 用于生成详细的 PR 总结文档并保存到文件。

use domain::LLMError;
use domain::PullRequestSummary;
use toolkit::Truncate;

use crate::llm::services::{parsers::JsonParser, prompt, LLMConversation};

/// PR 总结对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(crate) struct SummaryConversation {
    input: (String, String),
}

impl SummaryConversation {
    /// 创建新的 PR 总结对话实例
    pub fn new(input: (String, String)) -> Self {
        Self { input }
    }

    /// 构建 user prompt
    fn build_user_prompt(pr_title: &str, pr_diff: &str) -> String {
        let mut parts = vec![format!("PR Title: {}", pr_title)];

        if !pr_diff.trim().is_empty() {
            // 限制 diff 长度，避免请求过大
            const MAX_DIFF_LENGTH: usize = 15000; // 总结需要更多上下文
            let diff_trimmed = pr_diff.truncate(
                MAX_DIFF_LENGTH,
                "\n... (diff truncated, {} characters total)",
            );
            parts.push(format!("PR Diff:\n{}", diff_trimmed));
        }

        parts.join("\n\n")
    }
}

impl LLMConversation for SummaryConversation {
    type Input = (String, String);
    type Output = PullRequestSummary;

    fn get_system_prompt(&self, language_code: &str) -> String {
        prompt::summarize_with_language(language_code)
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        let (pr_title, pr_diff) = &self.input;
        Self::build_user_prompt(pr_title, pr_diff)
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3) // temperature，降低温度使输出更稳定
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        let mut model: PullRequestSummary = JsonParser::to_model(response)
            .map_err(|e| LLMError::ApiError(format!("JSON parse error: {}", e)))?;

        // 验证必需字段
        if model.summary.trim().is_empty() {
            return Err(LLMError::ApiError("summary is empty".to_string()));
        }
        if model.filename.trim().is_empty() {
            return Err(LLMError::ApiError("filename is empty".to_string()));
        }

        // 清理文件名，确保只包含有效的文件名字符
        let cleaned_filename = model
            .filename
            .trim()
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>();

        // 移除 .md 扩展名（如果存在），因为我们会自动添加
        let cleaned_filename = cleaned_filename.trim_end_matches(".md").to_string();

        if cleaned_filename.is_empty() {
            return Err(LLMError::ApiError(
                "Generated filename is empty after cleaning".to_string(),
            ));
        }

        model.summary = model.summary.trim().to_string();
        model.filename = cleaned_filename;

        Ok(model)
    }
}
