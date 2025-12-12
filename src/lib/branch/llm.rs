//! Branch LLM 服务
//!
//! 本模块提供了使用 LLM 处理分支名称的功能。
//! 包括将非英文文本翻译为英文。

use crate::base::llm::{LLMClient, LLMRequestParams};
use crate::base::prompt::TRANSLATE_SYSTEM_PROMPT;
use anyhow::Result;

/// Branch LLM 服务
///
/// 提供使用 LLM 处理分支名称的功能。
/// 支持多种 LLM 提供商：OpenAI、DeepSeek、代理 API。
pub struct BranchLLM;

impl BranchLLM {
    /// 使用 LLM 将文本翻译为英文
    ///
    /// 使用 LLM 将非英文文本（中文、俄文等）翻译为英文。
    ///
    /// # 参数
    ///
    /// * `text` - 需要翻译的文本
    ///
    /// # 返回
    ///
    /// 返回翻译后的英文文本
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败或返回空结果，返回相应的错误信息。
    pub fn translate_to_english(text: &str) -> Result<String> {
        let user_prompt = format!("Translate this text to English: {}", text);

        let params = LLMRequestParams {
            system_prompt: TRANSLATE_SYSTEM_PROMPT.to_string(),
            user_prompt,
            max_tokens: Some(100),
            temperature: 0.3,
            ..Default::default()
        };

        let client = LLMClient::global();
        let translated = client.call(&params)?;

        // Clean up the response (remove quotes, extra whitespace, etc.)
        let cleaned = translated.trim().trim_matches('"').trim_matches('\'').trim().to_string();

        if cleaned.is_empty() {
            anyhow::bail!("LLM returned empty translation");
        }

        Ok(cleaned)
    }
}
