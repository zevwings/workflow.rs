//! LLM 客户端
//!
//! 本模块提供了 LLM 客户端实现，支持所有遵循 OpenAI 兼容格式的提供商。

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::sync::OnceLock;

use super::types::LLMRequestParams;
use crate::log_debug;
use crate::{
    base::http::HttpResponse,
    base::settings::defaults::{default_llm_model, default_response_format},
    Settings,
};

/// LLM 客户端
///
/// 所有 LLM 提供商使用同一个客户端实现，通过 Settings 配置区分不同的提供商。
/// 所有配置（URL、API key、model、response_format）都从 Settings 动态获取。
#[allow(dead_code)]
pub struct LLMClient;

impl LLMClient {
    /// 获取全局 LLMClient 单例
    ///
    /// 返回进程级别的 LLMClient 单例。
    /// 单例会在首次调用时初始化，后续调用会复用同一个实例。
    ///
    /// # 返回
    ///
    /// 返回 `LLMClient` 的静态引用。
    ///
    /// # 优势
    ///
    /// - 减少资源消耗：避免重复创建客户端实例
    /// - 线程安全：可以在多线程环境中安全使用
    /// - 统一管理：所有 LLM 调用使用同一个客户端实例
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::base::llm::LLMClient;
    ///
    /// let client = LLMClient::global();
    /// let params = LLMRequestParams::new("What is Rust?");
    /// let response = client.call(&params)?;
    /// ```
    pub fn global() -> &'static Self {
        static CLIENT: OnceLock<LLMClient> = OnceLock::new();
        CLIENT.get_or_init(|| LLMClient)
    }

    /// 调用 LLM API
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
    pub fn call(&self, params: &LLMRequestParams) -> Result<String> {
        // 创建带超时的 HTTP 客户端（60秒）
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client with timeout")?;

        // 构建请求体（统一格式）
        let payload = self.build_payload(params)?;

        // 构建请求头（统一格式）
        let headers = self.build_headers()?;

        // 构建 URL（统一格式）
        let url = self.build_url()?;

        // 获取 provider 名称用于错误消息
        let provider = self.get_provider_name()?;

        log_debug!("LLM url: {}", url);
        log_debug!("LLM payload: {}", payload);
        log_debug!("LLM headers: {:?}", headers);
        log_debug!("LLM provider: {}", provider);

        // 发送请求
        let mut request = client.post(&url);

        // 添加 headers
        for (key, value) in headers.iter() {
            request = request.header(key, value);
        }

        let response = request
            .json(&payload)
            .send()
            .with_context(|| format!("Failed to send LLM request to {}", provider))?;

        // 转换为 HttpResponse
        let http_response = HttpResponse::from_reqwest_response(response)?;

        // 检查错误
        if !http_response.is_success() {
            return self.handle_error(&http_response);
        }

        // 解析 JSON 响应
        let data: Value = http_response.as_json()?;

        // 根据配置的响应格式提取内容
        self.extract_content(&data)
    }

    /// 构建 API URL
    ///
    /// 从 Settings 获取 URL：
    /// - openai: `https://api.openai.com/v1/chat/completions`
    /// - deepseek: `https://api.deepseek.com/chat/completions`
    /// - proxy: 从 Settings 获取 URL，拼接 `/chat/completions`
    fn build_url(&self) -> Result<String> {
        let settings: &Settings = Settings::get();
        let provider = &settings.llm.provider;

        match provider.as_str() {
            "openai" => Ok("https://api.openai.com/v1/chat/completions".to_string()),
            "deepseek" => Ok("https://api.deepseek.com/chat/completions".to_string()),
            "proxy" => {
                let base_url = settings
                    .llm
                    .url
                    .as_ref()
                    .context("LLM proxy URL is not configured")?;
                Ok(format!(
                    "{}/chat/completions",
                    base_url.trim_end_matches('/')
                ))
            }
            _ => Err(anyhow::anyhow!("Unsupported LLM provider: {}", provider)),
        }
    }

    /// 构建请求头
    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let settings = Settings::get();
        let llm_key = settings.llm.key.as_deref().unwrap_or_default();
        if llm_key.is_empty() {
            return Err(anyhow::anyhow!("LLM key is empty in settings"));
        }
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", llm_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// 构建模型名称
    ///
    /// 从 Settings 获取模型名称：
    /// - openai/deepseek: 如果 Settings 中不存在，使用默认值
    /// - proxy: 如果 Settings 中不存在，报错
    fn build_model(&self) -> Result<String> {
        let settings: &Settings = Settings::get();
        let provider = &settings.llm.provider;

        match provider.as_str() {
            "openai" | "deepseek" => Ok(settings
                .llm
                .model
                .clone()
                .unwrap_or_else(|| default_llm_model(provider))),
            "proxy" => settings
                .llm
                .model
                .clone()
                .context("Model is required for proxy provider"),
            _ => settings.llm.model.clone().context("Model is required"),
        }
    }

    /// 构建请求体
    fn build_payload(&self, params: &LLMRequestParams) -> Result<Value> {
        let model = self.build_model()?;
        let mut payload = json!({
            "model": model,
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
            "temperature": params.temperature
        });

        // 只有当 max_tokens 有值时才添加到请求体中
        // 如果为 None，则不包含该字段，让 API 使用模型默认的最大值
        if let Some(max_tokens) = params.max_tokens {
            payload["max_tokens"] = json!(max_tokens);
        }

        Ok(payload)
    }

    /// 从响应中提取内容
    ///
    /// 根据 Settings 中的 response_format 配置决定提取方式：
    /// - 空字符串或 "choices[0].message.content": 使用 OpenAI 标准格式
    /// - 其他值: 使用自定义 JSON path 提取
    fn extract_content(&self, response: &Value) -> Result<String> {
        let settings: &Settings = Settings::get();
        let response_format = &settings.llm.response_format;

        // 如果为空或等于默认值，使用 OpenAI 格式；否则使用自定义格式
        if response_format.is_empty() || *response_format == default_response_format() {
            // 标准 OpenAI 格式
            response
                .get("choices")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|c| c.as_str())
                .context("Failed to extract content from OpenAI format response")
                .map(|s| s.trim().to_string())
        } else {
            // 通过 JSON path 提取
            self.extract_by_path(response, response_format)
        }
    }

    /// 通过 JSON path 提取内容
    ///
    /// 支持点分隔路径和数组索引，例如：
    /// - `data.result.text` - 简单路径
    /// - `candidates[0].content.parts[0].text` - 包含数组索引的路径
    fn extract_by_path(&self, json: &Value, path: &str) -> Result<String> {
        let mut current = json;

        // 解析路径：支持 "key[0].subkey[1].value" 格式
        // 使用正则表达式分割路径，同时保留数组索引
        let parts = Self::parse_path(path);

        for part in parts {
            match part {
                PathPart::Key(key) => {
                    let key_str = key.as_str();
                    current = current.get(key_str).with_context(|| {
                        format!(
                            "Path '{}' not found in response: missing key '{}'",
                            path, key_str
                        )
                    })?;
                }
                PathPart::Index(idx) => {
                    current = current
                        .as_array()
                        .with_context(|| {
                            format!(
                                "Path '{}' not found in response: expected array at index {}",
                                path, idx
                            )
                        })?
                        .get(idx)
                        .with_context(|| {
                            format!(
                                "Path '{}' not found in response: array index {} out of bounds",
                                path, idx
                            )
                        })?;
                }
            }
        }

        current
            .as_str()
            .with_context(|| format!("Value at path '{}' is not a string", path))
            .map(|s| s.trim().to_string())
    }

    /// 解析 JSON path 字符串
    ///
    /// 将路径字符串解析为路径部分序列，支持：
    /// - 点分隔的键名：`data.result`
    /// - 数组索引：`[0]`, `[1]`
    /// - 混合：`candidates[0].content.parts[0].text`
    fn parse_path(path: &str) -> Vec<PathPart> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_brackets = false;

        for ch in path.chars() {
            match ch {
                '[' => {
                    if !current.is_empty() {
                        parts.push(PathPart::Key(current.clone()));
                        current.clear();
                    }
                    in_brackets = true;
                }
                ']' => {
                    if in_brackets {
                        if let Ok(idx) = current.parse::<usize>() {
                            parts.push(PathPart::Index(idx));
                        } else {
                            // 如果不是数字，作为键名处理
                            parts.push(PathPart::Key(current.clone()));
                        }
                        current.clear();
                        in_brackets = false;
                    } else {
                        current.push(ch);
                    }
                }
                '.' => {
                    if !in_brackets {
                        if !current.is_empty() {
                            parts.push(PathPart::Key(current.clone()));
                            current.clear();
                        }
                    } else {
                        current.push(ch);
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // 处理最后一个部分
        if !current.is_empty() {
            if in_brackets {
                if let Ok(idx) = current.parse::<usize>() {
                    parts.push(PathPart::Index(idx));
                } else {
                    parts.push(PathPart::Key(current));
                }
            } else {
                parts.push(PathPart::Key(current));
            }
        }

        parts
    }

    /// 获取 provider 名称
    fn get_provider_name(&self) -> Result<String> {
        let settings: &Settings = Settings::get();
        Ok(settings.llm.provider.clone())
    }

    /// 处理错误响应
    fn handle_error(&self, response: &HttpResponse) -> Result<String> {
        let provider = self
            .get_provider_name()
            .unwrap_or_else(|_| "unknown".to_string());

        // 尝试解析错误响应为 JSON，提取详细的错误信息
        let error_message = match response.as_json::<Value>() {
            Ok(error_json) => {
                // 尝试提取常见的错误字段
                let error_detail = error_json
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .or_else(|| error_json.get("error").and_then(|e| e.as_str()))
                    .or_else(|| error_json.get("message").and_then(|m| m.as_str()));

                if let Some(detail) = error_detail {
                    format!(
                        "{} (details: {})",
                        serde_json::to_string(&error_json).unwrap_or_default(),
                        detail
                    )
                } else {
                    serde_json::to_string(&error_json).unwrap_or_default()
                }
            }
            Err(_) => {
                // 如果不是 JSON，尝试作为文本解析
                response
                    .as_text()
                    .unwrap_or_else(|_| String::from_utf8_lossy(response.as_bytes()).to_string())
            }
        };

        anyhow::bail!(
            "LLM API request failed ({}): {} - {}",
            provider,
            response.status,
            error_message
        );
    }
}

/// JSON path 部分
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum PathPart {
    /// 键名
    Key(String),
    /// 数组索引
    Index(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path_simple() {
        let parts = LLMClient::parse_path("data.result.text");
        assert_eq!(parts.len(), 3);
        match &parts[0] {
            PathPart::Key(k) => assert_eq!(k, "data"),
            _ => panic!("Expected Key"),
        }
        match &parts[1] {
            PathPart::Key(k) => assert_eq!(k, "result"),
            _ => panic!("Expected Key"),
        }
        match &parts[2] {
            PathPart::Key(k) => assert_eq!(k, "text"),
            _ => panic!("Expected Key"),
        }
    }

    #[test]
    fn test_parse_path_with_array() {
        let parts = LLMClient::parse_path("candidates[0].content.parts[0].text");
        assert_eq!(parts.len(), 6);
        match &parts[0] {
            PathPart::Key(k) => assert_eq!(k, "candidates"),
            _ => panic!("Expected Key"),
        }
        match &parts[1] {
            PathPart::Index(i) => assert_eq!(*i, 0),
            _ => panic!("Expected Index"),
        }
        match &parts[2] {
            PathPart::Key(k) => assert_eq!(k, "content"),
            _ => panic!("Expected Key"),
        }
        match &parts[3] {
            PathPart::Key(k) => assert_eq!(k, "parts"),
            _ => panic!("Expected Key"),
        }
        match &parts[4] {
            PathPart::Index(i) => assert_eq!(*i, 0),
            _ => panic!("Expected Index"),
        }
        match &parts[5] {
            PathPart::Key(k) => assert_eq!(k, "text"),
            _ => panic!("Expected Key"),
        }
    }

    #[test]
    fn test_extract_by_path_simple() {
        let json = json!({
            "data": {
                "result": {
                    "text": "Hello, World!"
                }
            }
        });

        let client = LLMClient::global();

        let result = client.extract_by_path(&json, "data.result.text").unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_extract_by_path_with_array() {
        let json = json!({
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "Hello, World!"
                            }
                        ]
                    }
                }
            ]
        });

        let client = LLMClient::global();

        let result = client
            .extract_by_path(&json, "candidates[0].content.parts[0].text")
            .unwrap();
        assert_eq!(result, "Hello, World!");
    }
}
