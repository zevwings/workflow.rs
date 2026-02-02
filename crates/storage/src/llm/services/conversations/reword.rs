//! PR Reword 对话
//!
//! 基于 PR diff 生成简洁的 PR 标题和描述，用于更新现有 PR。

use domain::LLMError;
use domain::PullRequestReword;
use toolkit::Truncate;

use crate::llm::services::{parsers::JsonParser, prompt, LLMConversation};

/// PR Reword 对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(crate) struct RewordConversation {
    input: (String, Option<String>),
}

impl RewordConversation {
    /// 创建新的 PR Reword 对话实例
    pub fn new(input: (String, Option<String>)) -> Self {
        Self { input }
    }

    /// 构建 user prompt
    ///
    /// 与 create 流程保持一致：当前标题作为主要输入，PR diff 用于验证和细化。
    fn build_user_prompt(pr_diff: &str, current_title: Option<&str>) -> String {
        let mut parts = Vec::new();

        // 如果有当前标题，将其作为主要输入（与 create 流程一致）
        if let Some(title) = current_title {
            parts.push(format!("Current PR title (PRIMARY INPUT): {}", title));
            parts.push(String::new());
            parts.push("Instructions:".to_string());
            parts.push(
                "- Generate PR title primarily based on the current PR title above".to_string(),
            );
            parts.push("- Use PR diff below only to verify and refine, not to replace the current title's intent".to_string());
            parts.push("- Focus on the business intent expressed in the current title, not implementation details".to_string());
            parts.push(String::new());
        } else {
            // 如果没有当前标题，回退到基于 PR diff 生成
            parts.push("Instructions:".to_string());
            parts.push(
                "- Generate a PR title and description based on the PR diff below".to_string(),
            );
            parts.push(String::new());
        }

        if !pr_diff.trim().is_empty() {
            // 限制 diff 长度，避免超过 LLM token 限制
            const MAX_DIFF_LENGTH: usize = 12000;
            let diff_trimmed = pr_diff.truncate(
                MAX_DIFF_LENGTH,
                "\n... (diff truncated, {} characters total)",
            );
            parts.push("PR Diff (for verification only):".to_string());
            parts.push(diff_trimmed);
        }

        parts.join("\n")
    }
}

impl LLMConversation for RewordConversation {
    type Input = (String, Option<String>);
    type Output = PullRequestReword;

    fn get_system_prompt(&self, _language_code: &str) -> String {
        prompt::reword().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        let (pr_diff, current_title) = &self.input;
        Self::build_user_prompt(pr_diff, current_title.as_deref())
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.5) // max_tokens 由 LLM 自动决定
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        let mut model: PullRequestReword = JsonParser::to_model(response)
            .map_err(|e| LLMError::ApiError(format!("JSON parse error: {}", e)))?;

        // 验证必需字段
        if model.pr_title.trim().is_empty() {
            return Err(LLMError::ApiError("pr_title is empty".to_string()));
        }

        // 清理标题
        model.pr_title = model.pr_title.trim().to_string();

        Ok(model)
    }
}
