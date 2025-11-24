//! Pull Request LLM 服务
//!
//! 本模块提供了使用 LLM 生成 Pull Request 内容的功能。
//! 根据 commit 标题自动生成符合规范的分支名和 PR 标题。

use anyhow::{Context, Result};
use serde_json::Value;

use crate::base::llm::{LLMClient, LLMRequestParams};
use crate::base::prompt::PromptManager;
use crate::pr::helpers::transform_to_branch_name;

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
        let client = LLMClient::global();

        // 构建请求参数
        let user_prompt = Self::user_prompt(commit_title, exists_branches, git_diff);
        let system_prompt = PromptManager::load("generate_branch.system.md")
            .with_context(|| "Failed to load system prompt from file: generate_branch.system.md")?;

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: 500, // 增加到 500，确保有足够空间返回完整的 JSON（包括 description）
            temperature: 0.5,
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).with_context(|| {
            format!(
                "Failed to call LLM API for generating branch name from commit title: '{}'",
                commit_title
            )
        })?;

        // 解析响应
        Self::parse_llm_response(response).with_context(|| {
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
                // 限制 diff 长度，避免请求过大
                // 对于生成分支名和 PR 标题，不需要完整的 diff
                // 限制在 3000 字符左右（约 750-1000 tokens），足够提供上下文信息
                const MAX_DIFF_LENGTH: usize = 3000;
                let diff_trimmed = {
                    let char_count = diff.chars().count();
                    if char_count > MAX_DIFF_LENGTH {
                        // 使用字符边界安全截取，避免在多字节字符中间截断
                        let mut char_boundary = diff.len(); // 默认到字符串末尾
                        for (idx, _) in diff.char_indices().take(MAX_DIFF_LENGTH + 1) {
                            char_boundary = idx;
                        }
                        let truncated = &diff[..char_boundary];
                        // 尝试在最后一个换行符处截断，避免截断中间的行
                        let last_newline = truncated.rfind('\n').unwrap_or(0);
                        let truncated_diff = if last_newline > 0 {
                            &diff[..last_newline]
                        } else {
                            truncated
                        };
                        format!(
                            "{}\n... (diff truncated, {} characters total)",
                            truncated_diff, char_count
                        )
                    } else {
                        diff
                    }
                };
                parts.push(format!("Git changes:\n{}", diff_trimmed));
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
        let json: Value = serde_json::from_str(json_str).with_context(|| {
            format!(
                "Failed to parse LLM response as JSON. Raw response: {}",
                json_str
            )
        })?;

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
