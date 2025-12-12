//! PR Reword
//!
//! 基于 PR diff 生成简洁的 PR 标题和描述，用于更新现有 PR。

use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    Result,
};
use serde_json::Value;

use crate::base::llm::{LLMClient, LLMRequestParams};
use crate::base::prompt::REWORD_PR_SYSTEM_PROMPT;

use super::helpers::extract_json_from_markdown;

/// PR Reword 结果，包含标题和描述
///
/// 由 LLM 基于当前 PR 标题和 PR diff 生成的标题和完整描述，用于更新现有 PR。
/// 与 create 流程保持一致：标题主要基于当前标题，PR diff 用于验证和细化。
#[derive(Debug, Clone)]
pub struct PullRequestReword {
    /// PR 标题（简洁，不超过 8 个单词，主要基于当前标题，如果当前标题包含 markdown 格式如 `#` 会保留）
    pub pr_title: String,
    /// PR 描述（基于 PR diff 生成的完整描述列表，包含所有重要变更，可选）
    pub description: Option<String>,
}

/// PR Reword 生成器
pub struct RewordGenerator;

impl RewordGenerator {
    /// 基于当前 PR 标题和 PR diff 生成更新的 PR 标题和描述
    ///
    /// 根据当前 PR 标题和 PR diff 内容生成更新的标题和完整的描述，用于更新现有 PR。
    /// 与 `create` 流程保持一致：标题主要基于当前标题，PR diff 用于验证和细化。
    /// 与 `summarize_pr()` 不同，这个方法生成的是适合作为 PR 元数据的标题和描述列表，
    /// 而不是详细的总结文档。
    ///
    /// # 参数
    ///
    /// * `pr_diff` - PR 的 diff 内容（用于验证和细化标题）
    /// * `current_title` - 当前 PR 标题（主要输入，如果包含 markdown 格式如 `#` 会保留）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestReword` 结构体，包含：
    /// - `pr_title` - PR 标题（简洁，不超过 8 个单词，主要基于当前标题，如果当前标题包含 markdown 格式会保留）
    /// - `description` - PR 描述（基于 PR diff 生成的完整描述列表，包含所有重要变更，可选）
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
            max_tokens: None, // 增加 token 限制以支持更完整的描述
            temperature: 0.5,
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).wrap_err_with(|| {
            format!(
                "Failed to call LLM API for rewording PR from diff (current title: {:?})",
                current_title
            )
        })?;

        // 解析响应
        Self::parse_reword_response(response).wrap_err_with(|| {
            format!(
                "Failed to parse LLM response for PR reword (current title: {:?})",
                current_title
            )
        })
    }

    /// 生成 PR reword 的 user prompt
    ///
    /// 与 create 流程保持一致：当前标题作为主要输入，PR diff 用于验证和细化。
    fn reword_user_prompt(pr_diff: &str, current_title: Option<&str>) -> String {
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
            parts.push("PR Diff (for verification only):".to_string());
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
        let json: Value = serde_json::from_str(&json_str).wrap_err_with(|| {
            format!(
                "Failed to parse LLM response as JSON. Raw response: {}",
                json_str
            )
        })?;

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

        Ok(PullRequestReword {
            pr_title: pr_title.trim().to_string(),
            description,
        })
    }
}
