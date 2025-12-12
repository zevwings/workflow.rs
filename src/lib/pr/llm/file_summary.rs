//! 单个文件修改总结
//!
//! 用于生成单个文件的修改总结。

use color_eyre::{eyre::WrapErr, Result};

use crate::base::llm::{LLMClient, LLMRequestParams};
use crate::base::prompt::generate_summarize_file_change_system_prompt;

use super::helpers::extract_json_from_markdown;

/// 文件修改总结生成器
pub struct FileSummaryGenerator;

impl FileSummaryGenerator {
    /// 生成单个文件的修改总结
    ///
    /// 根据文件的 diff 内容生成该文件的修改总结。
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    /// * `file_diff` - 文件的 diff 内容
    /// * `language` - 可选的语言代码（如 "en", "zh", "zh-CN", "zh-TW"），如果为 None，则从配置文件读取
    ///
    /// # 返回
    ///
    /// 返回文件的修改总结（纯文本）
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败，返回相应的错误信息。
    pub fn summarize_file_change(file_path: &str, file_diff: &str) -> Result<String> {
        // 使用统一的 v2 客户端
        let client = LLMClient::global();

        // 构建请求参数
        let user_prompt = Self::summarize_file_change_user_prompt(file_path, file_diff);
        // 根据语言生成 system prompt（语言选择逻辑在 prompt 生成函数内部处理）
        let system_prompt = generate_summarize_file_change_system_prompt();

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: None, // 单个文件的总结应该比较简短
            temperature: 0.3,
            model: String::new(), // model 会从 Settings 自动获取，这里可以留空
        };

        // 调用 LLM API
        let response = client.call(&params).wrap_err_with(|| {
            format!(
                "Failed to call LLM API for summarizing file change: '{}'",
                file_path
            )
        })?;

        // 清理响应（移除可能的 markdown 代码块包装）
        let summary = Self::clean_file_change_summary_response(response);

        Ok(summary)
    }

    /// 生成单个文件修改总结的 user prompt
    fn summarize_file_change_user_prompt(file_path: &str, file_diff: &str) -> String {
        // 限制单个文件的 diff 长度，避免超过 LLM token 限制
        const MAX_FILE_DIFF_LENGTH: usize = 8000; // 单个文件的总结不需要太多上下文
        let diff_trimmed = {
            let char_count = file_diff.chars().count();
            if char_count > MAX_FILE_DIFF_LENGTH {
                // 使用字符边界安全截取
                let mut char_boundary = file_diff.len();
                for (idx, _) in file_diff.char_indices().take(MAX_FILE_DIFF_LENGTH + 1) {
                    char_boundary = idx;
                }
                let truncated = &file_diff[..char_boundary];
                // 尝试在最后一个换行符处截断
                let last_newline = truncated.rfind('\n').unwrap_or(0);
                let truncated_diff = if last_newline > 0 {
                    &file_diff[..last_newline]
                } else {
                    truncated
                };
                format!(
                    "{}\n... (file diff truncated, {} characters total)",
                    truncated_diff, char_count
                )
            } else {
                file_diff.to_string()
            }
        };
        format!("File path: {}\n\nFile diff:\n{}", file_path, diff_trimmed)
    }

    /// 清理文件修改总结响应
    ///
    /// 移除可能的 markdown 代码块包装，返回纯文本。
    fn clean_file_change_summary_response(response: String) -> String {
        extract_json_from_markdown(response)
    }
}
