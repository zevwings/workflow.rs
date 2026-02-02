//! PR 创建内容对话
//!
//! 用于创建 PR 时生成分支名、PR 标题、描述和 scope。
//! 注意：详细总结（summary）由服务层在调用此对话后自动生成。

use domain::LLMError;
use domain::{BranchNaming, PullRequestContent};
use toolkit::Truncate;

use crate::llm::services::{parsers::JsonParser, prompt, LLMConversation};

/// PR 创建内容对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(crate) struct CreateConversation {
    input: (String, Option<Vec<String>>, Option<String>),
}

impl CreateConversation {
    /// 创建新的 PR 创建内容对话实例
    pub fn new(input: (String, Option<Vec<String>>, Option<String>)) -> Self {
        Self { input }
    }

    /// 构建 user prompt
    fn build_user_prompt(
        commit_title: &str,
        exists_branches: Option<&[String]>,
        git_diff: Option<&str>,
    ) -> String {
        // 提取分支列表，过滤空字符串
        let base_branch_names: Vec<String> = exists_branches
            .map(|branches| branches.iter().filter(|b| !b.is_empty()).cloned().collect())
            .unwrap_or_default();

        // 组装 prompt 内容
        let mut parts = vec![
            format!("Commit title (PRIMARY INPUT): {}", commit_title),
            String::new(),
            "Instructions:".to_string(),
            "- Generate PR title primarily based on the commit title above".to_string(),
            "- Use git changes below only to verify and refine, not to replace the commit title's intent".to_string(),
            "- Focus on the business intent expressed in the commit title, not implementation details".to_string(),
        ];

        if !base_branch_names.is_empty() {
            parts.push(String::new());
            parts.push(format!(
                "Existing base branch names: {}",
                base_branch_names.join(", ")
            ));
        }

        if let Some(diff) = git_diff {
            if !diff.trim().is_empty() {
                // 限制 git diff 长度，避免超过 LLM token 限制
                const MAX_DIFF_LENGTH: usize = 10000;
                let diff_trimmed = diff.truncate(
                    MAX_DIFF_LENGTH,
                    "\n... (git diff truncated, {} characters total)",
                );
                parts.push(String::new());
                parts.push("Git changes (for verification only):".to_string());
                parts.push(diff_trimmed);
            }
        }

        parts.join("\n")
    }
}

impl LLMConversation for CreateConversation {
    type Input = (String, Option<Vec<String>>, Option<String>);
    type Output = PullRequestContent;

    fn get_system_prompt(&self, _language_code: &str) -> String {
        prompt::create().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        let (commit_title, exists_branches, git_diff) = &self.input;
        Self::build_user_prompt(
            commit_title,
            exists_branches.as_deref(),
            git_diff.as_deref(),
        )
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.5) // max_tokens 由 LLM 自动决定
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        let mut model: PullRequestContent = JsonParser::to_model(response)
            .map_err(|e| LLMError::ApiError(format!("JSON parse error: {}", e)))?;

        // 验证必需字段
        if model.branch_name.trim().is_empty() {
            return Err(LLMError::ApiError("branch_name is empty".to_string()));
        }
        if model.pr_title.trim().is_empty() {
            return Err(LLMError::ApiError("pr_title is empty".to_string()));
        }

        // 清理分支名，确保只保留 ASCII 字符
        model.branch_name = BranchNaming::sanitize(model.branch_name.trim());
        model.pr_title = model.pr_title.trim().to_string();

        Ok(model)
    }
}
