//! 分支名对话
//!
//! 用于根据标题生成分支名。

use llm::LLMConversation;

/// 分支名对话
///
/// 负责构建 prompt 和业务逻辑，不直接调用 LLM API。
pub(super) struct BranchNameConversation {
    title: Option<String>,
    exists_branches: Vec<String>,
}

impl BranchNameConversation {
    /// 创建新的分支名对话实例
    ///
    /// # 参数
    ///
    /// * `title` - 可选的标题，用于生成分支名
    /// * `exists_branches` - 已存在的分支列表，用于避免重名
    pub fn new(title: Option<&str>, exists_branches: &[String]) -> Self {
        Self {
            title: title.map(String::from),
            exists_branches: exists_branches.to_vec(),
        }
    }
}

impl LLMConversation for BranchNameConversation {
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
        // 构建输入文本
        let mut input_parts = Vec::with_capacity(1);
        if let Some(title) = &self.title {
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
        if !self.exists_branches.is_empty() {
            let filtered_branches: Vec<&str> = self
                .exists_branches
                .iter()
                .map(|s| s.as_str())
                .filter(|b| !b.is_empty())
                .collect();
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
}
