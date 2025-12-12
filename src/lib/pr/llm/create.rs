//! PR 创建内容生成
//!
//! 用于创建 PR 时生成分支名、PR 标题、描述和 scope。

use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    Result,
};
use serde_json::Value;

use crate::base::llm::{LLMClient, LLMRequestParams};
use crate::base::prompt::GENERATE_BRANCH_SYSTEM_PROMPT;
use crate::branch::BranchNaming;

use super::helpers::extract_json_from_markdown;

/// PR 内容，包含分支名、PR 标题、描述和 scope
///
/// 由 LLM 生成的分支名、PR 标题、描述和 scope，用于创建 Pull Request。
#[derive(Debug, Clone)]
pub struct PullRequestContent {
    /// 分支名称（小写，使用连字符分隔）
    pub branch_name: String,
    /// PR 标题（简洁，不超过 8 个单词）
    pub pr_title: String,
    /// PR 描述（基于 Git 修改内容生成）
    pub description: Option<String>,
    /// Commit scope（从 git diff 提取，用于 Conventional Commits 格式）
    ///
    /// Scope 表示变更涉及的模块或功能区域，例如 "api", "auth", "jira" 等。
    /// 如果无法确定 scope，此字段为 `None`。
    pub scope: Option<String>,
}

/// PR 创建内容生成器
pub struct CreateGenerator;

impl CreateGenerator {
    /// 同时生成分支名、PR 标题、描述和 scope（通过一个 LLM 请求）
    ///
    /// 根据 commit 标题和 git diff 生成符合规范的分支名、PR 标题、描述和 scope。
    /// 分支名和 PR 标题都会自动翻译为英文（如果输入包含非英文内容）。
    ///
    /// # 参数
    ///
    /// * `commit_title` - commit 标题或描述
    /// * `exists_branches` - 已存在的分支列表（可选）
    /// * `git_diff` - Git 工作区和暂存区的修改内容（可选，用于生成描述和提取 scope）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体，包含：
    /// - `branch_name` - 分支名称（小写，使用连字符分隔）
    /// - `pr_title` - PR 标题（简洁，不超过 8 个单词）
    /// - `description` - PR 描述（基于 Git 修改内容生成，可选）
    /// - `scope` - Commit scope（从 git diff 提取，用于 Conventional Commits 格式，可选）
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
        let client = LLMClient::global();

        // 构建请求参数
        let user_prompt = Self::user_prompt(commit_title, exists_branches, git_diff);
        // 使用编译时嵌入的 system prompt
        let system_prompt = GENERATE_BRANCH_SYSTEM_PROMPT.to_string();

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: None, // 增加到 500，确保有足够空间返回完整的 JSON（包括 description）
            temperature: 0.5,
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).wrap_err_with(|| {
            format!(
                "Failed to call LLM API for generating branch name from commit title: '{}'",
                commit_title
            )
        })?;

        // 解析响应
        Self::parse_llm_response(response).wrap_err_with(|| {
            format!(
                "Failed to parse LLM response for commit title: '{}'",
                commit_title
            )
        })
    }

    /// 生成同时生成分支名和 PR 标题的 user prompt
    fn user_prompt(
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> String {
        // 提取分支列表，如果没有或为空则使用空数组
        // 注意：exists_branches 已经通过 get_all_branches(true) 获取，已经去掉了前缀
        let base_branch_names: Vec<String> =
            exists_branches.filter(|b| !b.is_empty()).unwrap_or_default();

        // 组装 prompt 内容
        // 明确说明优先级：commit title 是主要输入，git diff 仅用于验证
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
                // create 主要用于生成标题和描述，不需要完整 diff
                const MAX_DIFF_LENGTH: usize = 10000; // create 只需要了解主要变更
                let diff_trimmed = {
                    let char_count = diff.chars().count();
                    if char_count > MAX_DIFF_LENGTH {
                        // 使用字符边界安全截取
                        let mut char_boundary = diff.len();
                        for (idx, _) in diff.char_indices().take(MAX_DIFF_LENGTH + 1) {
                            char_boundary = idx;
                        }
                        let truncated = &diff[..char_boundary];
                        // 尝试在最后一个换行符处截断
                        let last_newline = truncated.rfind('\n').unwrap_or(0);
                        let truncated_diff = if last_newline > 0 {
                            &diff[..last_newline]
                        } else {
                            truncated
                        };
                        format!(
                            "{}\n... (git diff truncated, {} characters total)",
                            truncated_diff, char_count
                        )
                    } else {
                        diff.to_string()
                    }
                };
                parts.push(String::new());
                parts.push("Git changes (for verification only):".to_string());
                parts.push(diff_trimmed);
            }
        }

        parts.join("\n")
    }

    /// 解析 LLM 返回的 JSON 响应，提取分支名、PR 标题、描述和 scope
    ///
    /// 从 LLM 的 JSON 响应中提取 `branch_name`、`pr_title`、`description` 和 `scope` 字段。
    /// 支持处理包含 markdown 代码块的响应格式。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串（可能是 JSON 或包含 JSON 的 markdown 代码块）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestContent` 结构体，包含清理后的分支名、PR 标题、描述和 scope。
    ///
    /// # 错误
    ///
    /// 如果响应格式不正确或缺少必要字段，返回相应的错误信息。
    fn parse_llm_response(response: String) -> Result<PullRequestContent> {
        // 使用公共方法提取 JSON
        let json_str = extract_json_from_markdown(response);

        // 解析 JSON
        let json: Value = serde_json::from_str(&json_str).wrap_err_with(|| {
            format!(
                "Failed to parse LLM response as JSON. Raw response: {}",
                json_str
            )
        })?;

        let branch_name = json
            .get("branch_name")
            .and_then(|v| v.as_str())
            .wrap_err("Missing 'branch_name' field in LLM response")?
            .to_string();

        let pr_title = json
            .get("pr_title")
            .and_then(|v| v.as_str())
            .wrap_err("Missing 'pr_title' field in LLM response")?
            .to_string();

        // description 是可选的
        let description = json
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // scope 是可选的
        let scope = json
            .get("scope")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // 清理分支名，确保只保留 ASCII 字符
        let cleaned_branch_name = BranchNaming::sanitize(branch_name.trim());

        Ok(PullRequestContent {
            branch_name: cleaned_branch_name,
            pr_title: pr_title.trim().to_string(),
            description,
            scope,
        })
    }
}
