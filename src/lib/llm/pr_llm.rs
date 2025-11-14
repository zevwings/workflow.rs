//! Pull Request LLM 服务
//!
//! 本模块提供了使用 LLM 生成 Pull Request 内容的功能。
//! 根据 commit 标题自动生成符合规范的分支名和 PR 标题。

use anyhow::{Context, Result};
use serde_json::Value;

use crate::pr::helpers::transform_to_branch_name;

use super::client::{LLMClient, LLMRequestParams};

/// PR 内容，包含分支名、PR 标题和描述
///
/// 由 LLM 生成的分支名、PR 标题和描述，用于创建 Pull Request。
#[derive(Debug, Clone)]
pub struct PullRequestContent {
    /// 分支名称（小写，使用连字符分隔）
    pub branch_name: String,
    /// PR 标题（简洁，不超过 8 个单词）
    pub pr_title: String,
    /// PR 描述（基于 Git 修改内容生成）
    pub description: Option<String>,
}

/// Pull Request LLM 服务
///
/// 提供使用 LLM 生成分支名和 PR 标题的功能。
/// 支持多种 LLM 提供商：OpenAI、DeepSeek、代理 API。
pub struct PullRequestLLM;

impl PullRequestLLM {
    /// 同时生成分支名和 PR 标题（通过一个 LLM 请求）
    ///
    /// 根据 commit 标题生成符合规范的分支名和 PR 标题。
    /// 分支名和 PR 标题都会自动翻译为英文（如果输入包含非英文内容）。
    ///
    /// # 参数
    ///
    /// * `commit_title` - commit 标题或描述
    /// * `exists_branches` - 已存在的分支列表（可选）
    /// * `git_diff` - Git 工作区和暂存区的修改内容（可选）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体，包含：
    /// - `branch_name` - 分支名称（小写，使用连字符分隔）
    /// - `pr_title` - PR 标题（简洁，不超过 8 个单词）
    /// - `description` - PR 描述（基于 Git 修改内容生成，可选）
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败或响应格式不正确，返回相应的错误信息。
    pub fn generate(
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> Result<PullRequestContent> {
        // 使用统一的 v2 客户端
        let client = LLMClient::new();

        // 构建请求参数
        let params = LLMRequestParams {
            system_prompt: Self::system_prompt(),
            user_prompt: Self::user_prompt(commit_title, exists_branches, git_diff),
            max_tokens: 100,
            temperature: 0.5,
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params)?;

        // 解析响应
        Self::parse_llm_response(response)
    }

    /// 生成同时生成分支名和 PR 标题的 system prompt
    fn system_prompt() -> String {
        "You're a git assistant that generates a branch name, PR title, and description based on the commit title and git changes.

IMPORTANT: All outputs MUST be in English only. If the commit title contains non-English text (like Chinese), translate it to English first.

For the branch name:
- Must be all lowercase
- Use hyphens to separate words
- Be under 50 characters
- Follow git branch naming conventions (no spaces, no special characters except hyphens, ASCII characters only)
- Generate only the base branch name without prefix (e.g., 'feature-name' not 'prefix/feature-name')
- If existing base branch names are provided, ensure the generated base branch name does not duplicate any of them
- Consider the git changes when generating the branch name to make it more accurate
- Examples:
  * \"Fix login bug\" → \"fix-login-bug\"
  * \"修复登录问题\" → \"fix-login-issue\"
  * \"Add user authentication\" → \"feature-add-user-authentication\"
  * \"新功能：用户认证\" → \"feature-user-authentication\"
  * \"Refactor code structure\" → \"refactoring-code-structure\"
  * \"重构代码结构\" → \"refactoring-code-structure\"
  * \"Update documentation\" → \"update-documentation\"
  * \"更新文档\" → \"update-documentation\"
  * \"Improve performance\" → \"improve-performance\"
  * \"优化性能\" → \"performance-optimization\"

For the PR title:
- Must be concise, within 8 words
- No punctuation
- In English only

For the description:
- Generate a concise description based on the git changes provided
- Summarize what was changed, added, or fixed
- If no git changes are provided, you can omit this field or provide a brief description based on the commit title
- Keep it brief (2-4 sentences)
- In English only

Return your response in JSON format with three fields: \"branch_name\", \"pr_title\", and \"description\" (optional). Example:
{
  \"branch_name\": \"add-user-authentication\",
  \"pr_title\": \"Add user authentication\",
  \"description\": \"This PR adds user authentication functionality including login and registration features.\"
}".to_string()
    }

    /// 生成同时生成分支名和 PR 标题的 user prompt
    fn user_prompt(
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> String {
        // 提取分支列表，如果没有或为空则使用空数组
        // 注意：exists_branches 已经通过 get_all_branches(true) 获取，已经去掉了前缀
        let base_branch_names: Vec<String> = exists_branches
            .filter(|b| !b.is_empty())
            .unwrap_or_default();

        // 组装 prompt 内容
        let mut parts = vec![format!("Commit title: {}", commit_title)];

        if !base_branch_names.is_empty() {
            parts.push(format!(
                "Existing base branch names: {}",
                base_branch_names.join(", ")
            ));
        }

        if let Some(diff) = git_diff {
            if !diff.trim().is_empty() {
                parts.push(format!("Git changes:\n{}", diff));
            }
        }

        parts.join("\n")
    }

    /// 解析 LLM 返回的 JSON 响应，提取分支名和 PR 标题
    ///
    /// 从 LLM 的 JSON 响应中提取 `branch_name` 和 `pr_title` 字段。
    /// 支持处理包含 markdown 代码块的响应格式。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串（可能是 JSON 或包含 JSON 的 markdown 代码块）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体，包含清理后的分支名和 PR 标题。
    ///
    /// # 错误
    ///
    /// 如果响应格式不正确或缺少必要字段，返回相应的错误信息。
    fn parse_llm_response(response: String) -> Result<PullRequestContent> {
        let trimmed = response.trim();

        // 尝试提取 JSON（可能包含 markdown 代码块）
        let json_str = if trimmed.starts_with("```json") {
            // 移除 ```json 开头和 ``` 结尾
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim()
        } else if trimmed.starts_with("```") {
            // 移除 ``` 开头和 ``` 结尾
            let start = trimmed.find('\n').unwrap_or(0);
            let end = trimmed.rfind("```").unwrap_or(trimmed.len());
            trimmed[start..end].trim()
        } else {
            trimmed
        };

        // 解析 JSON
        let json: Value =
            serde_json::from_str(json_str).context("Failed to parse LLM response as JSON")?;

        let branch_name = json
            .get("branch_name")
            .and_then(|v| v.as_str())
            .context("Missing 'branch_name' field in LLM response")?
            .to_string();

        let pr_title = json
            .get("pr_title")
            .and_then(|v| v.as_str())
            .context("Missing 'pr_title' field in LLM response")?
            .to_string();

        // description 是可选的
        let description = json
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // 清理分支名，确保只保留 ASCII 字符
        let cleaned_branch_name = transform_to_branch_name(branch_name.trim());

        Ok(PullRequestContent {
            branch_name: cleaned_branch_name,
            pr_title: pr_title.trim().to_string(),
            description,
        })
    }
}
