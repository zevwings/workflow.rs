//! PR Reword
//!
//! 基于 PR diff 生成简洁的 PR 标题和描述，用于更新现有 PR。

use anyhow::{Context, Result};
use serde_json::Value;

use crate::base::llm::{LLMClient, LLMRequestParams};
use crate::base::prompt::REWORD_PR_SYSTEM_PROMPT;

use super::helpers::extract_json_from_markdown;

/// PR Reword 结果，包含标题和描述
///
/// 由 LLM 基于 PR diff 生成的简洁标题和描述，用于更新现有 PR。
#[derive(Debug, Clone)]
pub struct PullRequestReword {
    /// PR 标题（简洁，不超过 8 个单词）
    pub pr_title: String,
    /// PR 描述（基于 PR diff 生成的简洁描述，可选）
    pub description: Option<String>,
}

/// PR Reword 生成器
pub struct RewordGenerator;

impl RewordGenerator {
    /// 基于 PR diff 生成 PR 标题和描述
    ///
    /// 根据 PR 的 diff 内容生成简洁的标题和描述，用于更新现有 PR。
    /// 与 `summarize_pr()` 不同，这个方法生成的是简洁的标题和描述（适合作为 PR 的元数据），
    /// 而不是详细的总结文档。
    ///
    /// # 参数
    ///
    /// * `pr_diff` - PR 的 diff 内容
    /// * `current_title` - 当前 PR 标题（可选，用于参考）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestReword` 结构体，包含：
    /// - `pr_title` - PR 标题（简洁，不超过 8 个单词）
    /// - `description` - PR 描述（基于 PR diff 生成的简洁描述，可选）
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败或响应格式不正确，返回相应的错误信息。
    pub fn reword_from_diff(
        pr_diff: &str,
        current_title: Option<&str>,
    ) -> Result<PullRequestReword> {
        // 使用统一的 v2 客户端
        let client = LLMClient::global();

        // 构建请求参数
        let user_prompt = Self::reword_user_prompt(pr_diff, current_title);
        // 使用专门的 reword prompt
        let system_prompt = REWORD_PR_SYSTEM_PROMPT.to_string();

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: Some(500), // 限制 token 数量，因为只需要简洁的标题和描述
            temperature: 0.5,
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).with_context(|| {
            format!(
                "Failed to call LLM API for rewording PR from diff (current title: {:?})",
                current_title
            )
        })?;

        // 解析响应
        Self::parse_reword_response(response).with_context(|| {
            format!(
                "Failed to parse LLM response for PR reword (current title: {:?})",
                current_title
            )
        })
    }

    /// 生成 PR reword 的 user prompt
    fn reword_user_prompt(pr_diff: &str, current_title: Option<&str>) -> String {
        let mut parts = Vec::new();

        // 如果有当前标题，先显示它作为参考
        if let Some(title) = current_title {
            parts.push(format!("Current PR title: {}", title));
            parts.push(String::new());
            parts.push("Instructions:".to_string());
            parts.push("- Generate a new PR title based on the PR diff below".to_string());
            parts.push(
                "- The new title should accurately reflect the actual changes in the PR"
                    .to_string(),
            );
            parts.push("- You can use the current title as a reference, but prioritize accuracy based on the diff".to_string());
            parts.push(String::new());
        } else {
            parts.push("Instructions:".to_string());
            parts.push(
                "- Generate a PR title and description based on the PR diff below".to_string(),
            );
            parts.push(String::new());
        }

        if !pr_diff.trim().is_empty() {
            // 限制 diff 长度，避免超过 LLM token 限制
            // reword 只需要了解主要变更，不需要完整 diff
            const MAX_DIFF_LENGTH: usize = 12000; // reword 需要比 summary 少一些上下文
            let diff_trimmed = {
                let char_count = pr_diff.chars().count();
                if char_count > MAX_DIFF_LENGTH {
                    // 使用字符边界安全截取
                    let mut char_boundary = pr_diff.len();
                    for (idx, _) in pr_diff.char_indices().take(MAX_DIFF_LENGTH + 1) {
                        char_boundary = idx;
                    }
                    let truncated = &pr_diff[..char_boundary];
                    // 尝试在最后一个换行符处截断
                    let last_newline = truncated.rfind('\n').unwrap_or(0);
                    let truncated_diff = if last_newline > 0 {
                        &pr_diff[..last_newline]
                    } else {
                        truncated
                    };
                    format!(
                        "{}\n... (diff truncated, {} characters total)",
                        truncated_diff, char_count
                    )
                } else {
                    pr_diff.to_string()
                }
            };
            parts.push("PR Diff:".to_string());
            parts.push(diff_trimmed);
        }

        parts.join("\n")
    }

    /// 解析 LLM 返回的 JSON 响应，提取 PR 标题和描述
    ///
    /// 从 LLM 的 JSON 响应中提取 `pr_title` 和 `description` 字段。
    /// 支持处理包含 markdown 代码块的响应格式。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串（可能是 JSON 或包含 JSON 的 markdown 代码块）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestReword` 结构体，包含清理后的 PR 标题和描述。
    ///
    /// # 错误
    ///
    /// 如果响应格式不正确或缺少必要字段，返回相应的错误信息。
    fn parse_reword_response(response: String) -> Result<PullRequestReword> {
        // 使用公共方法提取 JSON
        let json_str = extract_json_from_markdown(response);

        // 解析 JSON
        let json: Value = serde_json::from_str(&json_str).with_context(|| {
            format!(
                "Failed to parse LLM response as JSON. Raw response: {}",
                json_str
            )
        })?;

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

        Ok(PullRequestReword {
            pr_title: pr_title.trim().to_string(),
            description,
        })
    }
}
