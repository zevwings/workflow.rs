use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use crate::http::{HttpClient, HttpResponse};
use crate::settings::Settings;

/// LLM 请求参数
pub struct LLMRequestParams {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub model: String,
}

/// 使用代理 API 调用 LLM
pub fn call_llm(params: LLMRequestParams) -> Result<String> {
    let settings = Settings::load();
    let api_key = settings
        .llm_proxy_key
        .as_ref()
        .context("LLM_PROXY_KEY environment variable not set")?;
    let base_url = settings
        .llm_proxy_url
        .as_ref()
        .context("LLM_PROXY_URL environment variable not set")?;

    let client = HttpClient::new()?;
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

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
        .post(&url, &payload, None, Some(&headers))
        .context("Failed to send request to proxy API")?;

    if !response.is_success() {
        let error_text = serde_json::to_string(&response.data).unwrap_or_default();
        anyhow::bail!(
            "Proxy API request failed: {} - {}",
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
        .context("Failed to extract content from proxy API response")?;

    Ok(content.trim().to_string())
}

