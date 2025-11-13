//! OpenAI API 客户端
//!
//! 本模块提供了调用 OpenAI API 的功能。

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use crate::http::{HttpClient, HttpResponse};
use crate::settings::Settings;

/// LLM 请求参数
///
/// 包含调用 LLM API 所需的所有参数。
#[derive(Debug, Clone)]
pub struct LLMRequestParams {
    /// 系统提示词
    pub system_prompt: String,
    /// 用户提示词
    pub user_prompt: String,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 温度参数（控制输出的随机性）
    pub temperature: f32,
    /// 模型名称（如 "gpt-3.5-turbo"）
    pub model: String,
}

/// 使用 OpenAI API 调用 LLM
///
/// 调用 OpenAI 的 Chat Completions API 生成文本。
///
/// # 参数
///
/// * `params` - LLM 请求参数
///
/// # 返回
///
/// 返回 LLM 生成的文本内容（去除首尾空白）。
///
/// # 错误
///
/// 如果 API 调用失败或响应格式不正确，返回相应的错误信息。
pub fn call_llm(params: LLMRequestParams) -> Result<String> {
    let settings = Settings::get();
    let llm_settings = settings
        .llm
        .as_ref()
        .context("LLM settings not configured")?;
    let api_key = llm_settings
        .openai_key
        .as_ref()
        .context("LLM_OPENAI_KEY environment variable not set")?;

    let client = HttpClient::new()?;
    let url = "https://api.openai.com/v1/chat/completions";

    let payload = json!({
        "model": params.model,
        "messages": [
            {
                "role": "system",
                "content": params.system_prompt
            },
            {
                "role": "user",
                "content": params.user_prompt
            }
        ],
        "max_tokens": params.max_tokens,
        "temperature": params.temperature
    });

    // 构建 headers
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .context("Failed to create Authorization header")?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let response: HttpResponse<serde_json::Value> = client
        .post(url, &payload, None, Some(&headers))
        .context("Failed to send request to OpenAI API")?;

    if !response.is_success() {
        let error_text = serde_json::to_string(&response.data).unwrap_or_default();
        anyhow::bail!(
            "OpenAI API request failed: {} - {}",
            response.status,
            error_text
        );
    }

    let content = response
        .data
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .context("Failed to extract content from OpenAI response")?;

    Ok(content.trim().to_string())
}
