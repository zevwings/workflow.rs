//! 分支名对话
//!
//! 用于根据标题生成分支名。

use domain::{sanitize_branch_name, LLMError};

use crate::llm::services::{parsers::JsonParser, LLMConversation};

/// 分支名对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(crate) struct BranchNameConversation {
    input: (Option<String>, Option<Vec<String>>),
}

impl BranchNameConversation {
    /// 创建新的分支名对话实例
    pub fn new(input: (Option<String>, Option<Vec<String>>)) -> Self {
        Self { input }
    }
}

impl LLMConversation for BranchNameConversation {
    type Input = (Option<String>, Option<Vec<String>>);
    type Output = String;

    fn get_system_prompt(&self, _language_code: &str) -> String {
        r#"You are a helpful Assistant that generates a git branch name.

Generate a branch name based on the input above, and return the branch name in JSON format.

Rules:
- All lowercase
- Use hyphens to separate words
- Maximum 50 characters
- ASCII characters and hyphens only
- Translate non-English text to English first
- Keep it concise and descriptive
- Do NOT duplicate existing branch names
- Do NOT respond with any prefix

Return JSON format:
{
  "branch_name": "your-generated-name"
}"#
        .to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        let (title, exists_branches) = &self.input;
        // 构建输入文本
        let mut input_parts = Vec::new();
        if let Some(title) = title {
            input_parts.push(title.to_string());
        }

        // 组装 prompt 内容
        let mut parts = vec![
            format!("Commit title (PRIMARY INPUT): {}", input_parts.join(" - ")),
            String::new(),
            "Instructions:".to_string(),
            "- Generate a branch name based on the input above".to_string(),
            "- The branch name should be lowercase, use hyphens to separate words".to_string(),
            "- Keep it concise and descriptive".to_string(),
            "- Do not duplicate existing branch names".to_string(),
            "- Do NOT include any prefix like 'feature/', 'bugfix/', etc.".to_string(),
        ];

        // 如果有已存在的分支列表，添加到 prompt 中
        if let Some(branches) = exists_branches {
            let filtered_branches: Vec<String> =
                branches.iter().filter(|b| !b.is_empty()).cloned().collect();
            if !filtered_branches.is_empty() {
                parts.push(String::new());
                parts.push(format!(
                    "Existing branch names (DO NOT duplicate): {}",
                    filtered_branches.join(", ")
                ));
            }
        }

        parts.join("\n")
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.8) // max_tokens 由 LLM 自动决定
    }

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        let map = JsonParser::to_map(response)
            .map_err(|e| LLMError::ApiError(format!("JSON parse error: {}", e)))?;

        // 提取分支名字段
        let branch_name = map
            .get("branch_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMError::ApiError("branch_name is missing or empty".to_string()))?;

        // 清理分支名，确保只保留 ASCII 字符
        let cleaned_branch_name: String = sanitize_branch_name(branch_name.trim());

        Ok(cleaned_branch_name)
    }
}
