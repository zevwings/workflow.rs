//! LLM 客户端
//!
//! 本模块提供了 LLM 客户端实现，支持所有遵循 OpenAI 兼容格式的提供商。

use std::sync::OnceLock;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};

use super::types::{ChatCompletionResponse, LLMRequestParams};
use crate::{base::http::HttpResponse, base::settings::defaults::default_llm_model, Settings};

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

        crate::trace_debug!("LLM url: {}", url);
        crate::trace_debug!("LLM payload: {}", payload);
        crate::trace_debug!("LLM headers: {:?}", headers);
        crate::trace_debug!("LLM provider: {}", provider);

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

        // 检查错误（使用 ensure_success_with 统一处理）
        let http_response = http_response.ensure_success_with(|r| {
            let provider = self
                .get_provider_name()
                .unwrap_or_else(|_| "unknown".to_string());
            let error_message = r.extract_error_message();
            anyhow::anyhow!(
                "LLM API request failed ({}): {} - {}",
                provider,
                r.status,
                error_message
            )
        })?;

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
    /// 使用 OpenAI 标准格式解析响应，提取消息内容。
    /// 支持所有遵循 OpenAI Chat Completions API 标准的响应格式。
    pub fn extract_content(&self, response: &Value) -> Result<String> {
        // 解析为标准结构体
        // 先将 Value 转换为 JSON 字符串，再反序列化（这样可以处理 DeserializeOwned 的要求）
        let json_str = serde_json::to_string(response)
            .context("Failed to serialize response to JSON string")?;
        let completion: ChatCompletionResponse = serde_json::from_str(&json_str)
            .context("Failed to parse response as OpenAI ChatCompletion format")?;

        // 提取内容
        let content = completion
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .context("No content in response: choices array is empty or content is null")?;

        Ok(content.trim().to_string())
    }

    /// 获取 provider 名称
    fn get_provider_name(&self) -> Result<String> {
        let settings: &Settings = Settings::get();
        Ok(settings.llm.provider.clone())
    }
}
