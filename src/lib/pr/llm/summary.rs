//! PR 总结文档生成
//!
//! 用于生成详细的 PR 总结文档并保存到文件。

use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    Result,
};
use serde_json::Value;

use crate::base::llm::{LLMClient, LLMRequestParams};
use crate::base::prompt::generate_summarize_pr_system_prompt;

use super::helpers::extract_json_from_markdown;

/// PR 总结结果，包含总结文档和文件名
///
/// 由 LLM 生成的 PR 总结文档和对应的文件名。
#[derive(Debug, Clone)]
pub struct PullRequestSummary {
    /// PR 总结文档（Markdown 格式）
    pub summary: String,
    /// 文件名（不含路径和扩展名）
    pub filename: String,
}

/// PR 总结生成器
pub struct SummaryGenerator;

impl SummaryGenerator {
    /// 生成 PR 总结文档和文件名
    ///
    /// 根据 PR 的 diff 内容生成总结文档和合适的文件名。
    ///
    /// # 参数
    ///
    /// * `pr_title` - PR 标题
    /// * `pr_diff` - PR 的 diff 内容
    /// * `language` - 可选的语言代码（如 "en", "zh", "zh-CN", "zh-TW"），如果为 None，则从配置文件读取
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestSummary` 结构体，包含：
    /// - `summary` - PR 总结文档（Markdown 格式）
    /// - `filename` - 文件名（不含路径和扩展名）
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败或响应格式不正确，返回相应的错误信息。
    pub fn summarize_pr(pr_title: &str, pr_diff: &str) -> Result<PullRequestSummary> {
        // 使用统一的 v2 客户端
        let client = LLMClient::global();

        // 构建请求参数
        let user_prompt = Self::summarize_user_prompt(pr_title, pr_diff);
        // 根据语言生成 system prompt（语言选择逻辑在 prompt 生成函数内部处理）
        let system_prompt = generate_summarize_pr_system_prompt();

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: None, // 增加 token 数量，确保有足够空间返回完整的总结文档
            temperature: 0.3, // 降低温度，使输出更稳定
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).wrap_err_with(|| {
            format!("Failed to call LLM API for summarizing PR: '{}'", pr_title)
        })?;

        // 解析响应
        Self::parse_summary_response(response).wrap_err_with(|| {
            format!(
                "Failed to parse LLM response for PR summary: '{}'",
                pr_title
            )
        })
    }

    /// 生成 PR 总结的 user prompt
    fn summarize_user_prompt(pr_title: &str, pr_diff: &str) -> String {
        let mut parts = vec![format!("PR Title: {}", pr_title)];

        if !pr_diff.trim().is_empty() {
            // 限制 diff 长度，避免请求过大
            // 对于总结，我们需要更多的 diff 内容，但也要避免超过 token 限制
            const MAX_DIFF_LENGTH: usize = 15000; // 增加长度，因为总结需要更多上下文
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
            parts.push(format!("PR Diff:\n{}", diff_trimmed));
        }

        parts.join("\n\n")
    }

    /// 解析 LLM 返回的 JSON 响应，提取总结文档和文件名
    ///
    /// 从 LLM 的 JSON 响应中提取 `summary` 和 `filename` 字段。
    /// 支持处理包含 markdown 代码块的响应格式。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串（可能是 JSON 或包含 JSON 的 markdown 代码块）
    ///
    /// # 返回
    ///
    /// 返回 `PullRequestSummary` 结构体，包含清理后的总结文档和文件名。
    ///
    /// # 错误
    ///
    /// 如果响应格式不正确或缺少必要字段，返回相应的错误信息。
    fn parse_summary_response(response: String) -> Result<PullRequestSummary> {
        // 使用公共方法提取 JSON
        let json_str = extract_json_from_markdown(response);

        // 解析 JSON
        let json: Value = serde_json::from_str(&json_str).wrap_err_with(|| {
            format!(
                "Failed to parse LLM response as JSON. Raw response: {}",
                json_str
            )
        })?;

        let summary = json
            .get("summary")
            .and_then(|v| v.as_str())
            .wrap_err("Missing 'summary' field in LLM response")?
            .to_string();

        let filename = json
            .get("filename")
            .and_then(|v| v.as_str())
            .wrap_err("Missing 'filename' field in LLM response")?
            .to_string();

        // 清理文件名，确保只包含有效的文件名字符
        let cleaned_filename = filename
            .trim()
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>();

        // 移除 .md 扩展名（如果存在），因为我们会自动添加
        let cleaned_filename = cleaned_filename.trim_end_matches(".md").to_string();

        if cleaned_filename.is_empty() {
            color_eyre::eyre::bail!("Generated filename is empty after cleaning");
        }

        Ok(PullRequestSummary {
            summary: summary.trim().to_string(),
            filename: cleaned_filename,
        })
    }
}
