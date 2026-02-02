//! PR 内容对话
//!
//! 用于根据分支名和提交列表生成 PR 标题和描述。

use domain::LLMError;
use serde::Deserialize;

use domain::PrContent;

use crate::llm::services::{parsers::JsonParser, prompt, LLMConversation};

#[derive(Deserialize)]
struct PrContentModel {
    pr_title: String,
    description: Option<String>,
}

/// PR 内容对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(crate) struct PrContentConversation {
    input: (String, Vec<String>),
}

impl PrContentConversation {
    /// 创建新的 PR 内容对话实例
    pub fn new(input: (String, Vec<String>)) -> Self {
        Self { input }
    }

    /// 构建 user prompt
    fn build_user_prompt(branch_name: &str, commits: &[String]) -> String {
        // 构建提交信息
        let commits_text = if commits.is_empty() {
            "No commits".to_string()
        } else {
            commits.join("\n")
        };

        // 组装 prompt 内容
        let mut parts = vec![
            format!("Branch name (PRIMARY INPUT): {}", branch_name),
            String::new(),
            "Instructions:".to_string(),
            "- Generate PR title and description based on the branch name above".to_string(),
            "- Use commit messages below to understand the changes and generate a comprehensive description".to_string(),
            "- Focus on the business intent expressed in the branch name and commits".to_string(),
        ];

        if !commits_text.is_empty() {
            parts.push(String::new());
            parts.push("Commit messages:".to_string());
            parts.push(commits_text);
        }

        parts.join("\n")
    }
}

impl LLMConversation for PrContentConversation {
    type Input = (String, Vec<String>);
    type Output = PrContent;

    fn get_system_prompt(&self, _language_code: &str) -> String {
        prompt::create().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        let (branch_name, commits) = &self.input;
        Self::build_user_prompt(branch_name, commits)
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.5) // max_tokens 由 LLM 自动决定
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        let model: PrContentModel = JsonParser::to_model(response)
            .map_err(|e| LLMError::ApiError(format!("JSON parse error: {}", e)))?;

        // 验证必需字段
        if model.pr_title.trim().is_empty() {
            return Err(LLMError::ApiError("pr_title is empty".to_string()));
        }

        Ok(PrContent {
            title: model.pr_title.trim().to_string(),
            description: model.description.unwrap_or_default(),
        })
    }
}
