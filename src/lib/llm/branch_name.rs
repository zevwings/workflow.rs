use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use crate::http::{HttpClient, HttpResponse};
use crate::pr::helpers::transform_to_branch_name;
use crate::settings::Settings;

/// 使用 LLM 根据 commit_title 生成分支名
pub fn generate_branch_name_with_llm(commit_title: &str) -> Result<String> {
    let settings = Settings::load();
    let provider = settings.llm_provider.clone();

    // 先检查对应的 API key 是否设置
    let api_key_set = match provider.as_str() {
        "openai" => settings.openai_key.is_some(),
        "deepseek" => settings.deepseek_key.is_some(),
        "proxy" => settings.llm_proxy_key.is_some() && settings.llm_proxy_url.is_some(),
        _ => settings.openai_key.is_some(), // 默认检查 OpenAI
    };

    if !api_key_set {
        let error_msg = match provider.as_str() {
            "openai" => "LLM_OPENAI_KEY environment variable not set",
            "deepseek" => "LLM_DEEPSEEK_KEY environment variable not set",
            "proxy" => {
                if settings.llm_proxy_key.is_none() && settings.llm_proxy_url.is_none() {
                    "LLM_PROXY_KEY and LLM_PROXY_URL environment variables not set"
                } else if settings.llm_proxy_key.is_none() {
                    "LLM_PROXY_KEY environment variable not set"
                } else {
                    "LLM_PROXY_URL environment variable not set"
                }
            }
            _ => "LLM_OPENAI_KEY environment variable not set",
        };
        anyhow::bail!("{} (provider: {})", error_msg, provider);
    }

    match provider.as_str() {
        "openai" => generate_branch_name_with_openai(commit_title),
        "deepseek" => generate_branch_name_with_deepseek(commit_title),
        "proxy" => generate_branch_name_with_proxy(commit_title),
        _ => generate_branch_name_with_openai(commit_title), // 默认使用 OpenAI
    }
}

/// 使用 OpenAI API 生成分支名
fn generate_branch_name_with_openai(commit_title: &str) -> Result<String> {
    let settings = Settings::load();
    let api_key = settings
        .openai_key
        .as_ref()
        .context("LLM_OPENAI_KEY environment variable not set")?;

    let client = HttpClient::new()?;
    let url = "https://api.openai.com/v1/chat/completions";

    let payload = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "You're a git branch naming assistant. Generate a concise, descriptive git branch name based on the commit title. IMPORTANT: The branch name MUST be in English only. If the commit title contains non-English text (like Chinese), translate it to English first. The branch name should be all lowercase, use hyphens to separate words, be under 50 characters, and follow git branch naming conventions (no spaces, no special characters except hyphens, ASCII characters only). Only return the branch name, nothing else."
            },
            {
                "role": "user",
                "content": format!("Generate an English-only git branch name for this commit title: {}", commit_title)
            }
        ],
        "max_tokens": 50,
        "temperature": 0.3
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

    let branch_name = response
        .data
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .context("Failed to extract branch name from OpenAI response")?;

    // 清理分支名，确保只保留 ASCII 字符
    let cleaned_branch_name = transform_to_branch_name(branch_name.trim());
    Ok(cleaned_branch_name)
}

/// 使用 DeepSeek API 生成分支名
fn generate_branch_name_with_deepseek(commit_title: &str) -> Result<String> {
    let settings = Settings::load();
    let api_key = settings
        .deepseek_key
        .as_ref()
        .context("LLM_DEEPSEEK_KEY environment variable not set")?;

    let client = HttpClient::new()?;
    let url = "https://api.deepseek.com/v1/chat/completions";

    let payload = json!({
        "model": "deepseek-chat",
        "messages": [
            {
                "role": "system",
                "content": "You're a git branch naming assistant. Generate a concise, descriptive git branch name based on the commit title. IMPORTANT: The branch name MUST be in English only. If the commit title contains non-English text (like Chinese), translate it to English first. The branch name should be all lowercase, use hyphens to separate words, be under 50 characters, and follow git branch naming conventions (no spaces, no special characters except hyphens, ASCII characters only). Only return the branch name, nothing else."
            },
            {
                "role": "user",
                "content": format!("Generate an English-only git branch name for this commit title: {}", commit_title)
            }
        ],
        "max_tokens": 50,
        "temperature": 0.3
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
        .context("Failed to send request to DeepSeek API")?;

    if !response.is_success() {
        let error_text = serde_json::to_string(&response.data).unwrap_or_default();
        anyhow::bail!(
            "DeepSeek API request failed: {} - {}",
            response.status,
            error_text
        );
    }

    let branch_name = response
        .data
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .context("Failed to extract branch name from DeepSeek response")?;

    // 清理分支名，确保只保留 ASCII 字符
    let cleaned_branch_name = transform_to_branch_name(branch_name.trim());
    Ok(cleaned_branch_name)
}

/// 使用代理 API 生成分支名
fn generate_branch_name_with_proxy(commit_title: &str) -> Result<String> {
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
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "You're a git branch naming assistant. Generate a concise, descriptive git branch name based on the commit title. IMPORTANT: The branch name MUST be in English only. If the commit title contains non-English text (like Chinese), translate it to English first. The branch name should be all lowercase, use hyphens to separate words, be under 50 characters, and follow git branch naming conventions (no spaces, no special characters except hyphens, ASCII characters only). Only return the branch name, nothing else."
            },
            {
                "role": "user",
                "content": format!("Generate an English-only git branch name for this commit title: {}", commit_title)
            }
        ],
        "max_tokens": 50,
        "temperature": 0.3
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

    let branch_name = response
        .data
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .context("Failed to extract branch name from proxy API response")?;

    // 清理分支名，确保只保留 ASCII 字符
    let cleaned_branch_name = transform_to_branch_name(branch_name.trim());
    Ok(cleaned_branch_name)
}
