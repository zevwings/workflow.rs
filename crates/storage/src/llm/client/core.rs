//! LLM 客户端
//!
//! 本模块提供了 LLM 客户端实现，支持所有遵循 OpenAI 兼容格式的提供商。
//!
//! ## 配置提供者
//!
//! LLM 客户端通过 `LLMConfigContext` trait 获取配置，实现了依赖倒置原则。
//! 通过 `LLMClient::new()` 方法传入配置上下文来创建客户端实例。

use std::sync::Arc;
use std::time::Duration;

use domain::LLMError;
use serde::Serialize;
use serde_json::{json, Value};

use domain::LLMConfigContext;
use toolkit::{Authorization, HttpClient, HttpClientConfig};

use crate::llm::client::response::ChatCompletionResponse;

/// LLM 请求参数
///
/// 包含调用 LLM API 所需的所有参数。
/// 注意：模型名称从配置上下文自动获取，不在此结构体中。
#[derive(Debug, Clone, Serialize)]
pub struct LLMRequestParameters {
    /// 系统提示词
    pub system_prompt: String,
    /// 用户提示词
    pub user_prompt: String,
    /// 最大 token 数（None 表示不限制，使用模型默认最大值）
    pub max_tokens: Option<u32>,
    /// 温度参数（控制输出的随机性）
    pub temperature: f32,
}

impl Default for LLMRequestParameters {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            user_prompt: String::new(),
            max_tokens: None,
            temperature: 0.5,
        }
    }
}

pub trait LLMClient: Send + Sync {
    fn call(&self, params: &LLMRequestParameters) -> Result<String, LLMError>;
}

/// LLM 客户端
///
/// 所有 LLM 提供商使用同一个客户端实现，通过配置上下文区分不同的提供商。
/// 所有配置（URL、API key、model）都从配置上下文动态获取。
pub struct LLMClientImpl {
    context: Arc<dyn LLMConfigContext>,
}

impl LLMClientImpl {
    pub fn new(context: Arc<dyn LLMConfigContext>) -> Self {
        Self { context }
    }

    /// 构建 API URL
    ///
    /// 从配置提供者获取 URL：
    /// - openai: `https://api.openai.com/v1/chat/completions`
    /// - deepseek: `https://api.deepseek.com/chat/completions`
    /// - proxy: 从配置提供者获取 URL，拼接 `/chat/completions`
    fn build_url(&self) -> Result<String, LLMError> {
        let provider = self.context.get_provider();

        match provider.as_str() {
            "openai" => Ok("https://api.openai.com/v1/chat/completions".to_string()),
            "deepseek" => Ok("https://api.deepseek.com/chat/completions".to_string()),
            "proxy" => {
                let base_url = self.context.get_current_provider_url();
                if base_url.is_empty() {
                    return Err(LLMError::ApiError("URL is empty in settings".to_string()));
                }
                Ok(format!(
                    "{}/chat/completions",
                    base_url.trim_end_matches('/')
                ))
            }
            _ => Err(LLMError::ApiError(format!(
                "Unsupported LLM provider: {}",
                provider
            ))),
        }
    }

    /// 构建认证信息
    fn build_auth(&self) -> Result<Authorization, LLMError> {
        let llm_key = self.context.get_current_provider_key();
        if llm_key.is_empty() {
            return Err(LLMError::ApiError(
                "LLM key is empty in settings".to_string(),
            ));
        }
        Ok(Authorization::bearer(llm_key))
    }

    /// 构建模型名称
    ///
    /// 从配置提供者获取模型名称：
    /// - openai/deepseek: 如果配置中不存在，使用默认值
    /// - proxy: 如果配置中不存在，报错
    fn build_model(&self) -> Result<String, LLMError> {
        let provider = self.context.get_provider();
        let model = self.context.get_current_provider_model();

        if model.is_empty() {
            return Err(LLMError::ApiError(format!(
                "Model is required for {} provider",
                provider
            )));
        }

        match provider.as_str() {
            "openai" | "deepseek" | "proxy" => Ok(model),
            _ => Err(LLMError::ApiError(format!(
                "Unsupported LLM provider: {}",
                provider
            ))),
        }
    }

    /// 构建请求体
    fn build_payload(&self, params: &LLMRequestParameters) -> Result<Value, LLMError> {
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
    pub fn extract_content(&self, response: &Value) -> Result<String, LLMError> {
        // 解析为标准结构体
        let completion: ChatCompletionResponse =
            serde_json::from_value(response.clone()).map_err(|e| {
                LLMError::ApiError(format!(
                    "Failed to parse response as OpenAI ChatCompletion format: {}",
                    e
                ))
            })?;

        // 提取内容
        let content = completion
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| {
                LLMError::ApiError(
                    "No content in response: choices array is empty or content is null".to_string(),
                )
            })?;

        Ok(content.trim().to_string())
    }
}

impl LLMClient for LLMClientImpl {
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
    fn call(&self, params: &LLMRequestParameters) -> Result<String, LLMError> {
        // 创建带超时的 HTTP 客户端（60秒）
        let config = HttpClientConfig::new().timeout(Duration::from_secs(60));
        let client = HttpClient::with_config(config).map_err(|e| {
            LLMError::ApiError(format!("Failed to create HTTP client with timeout: {}", e))
        })?;

        // 构建请求体（统一格式）
        let payload = self.build_payload(params)?;

        // 构建认证信息
        let auth = self.build_auth()?;

        // 构建 URL（统一格式）
        let url = self.build_url()?;

        // 获取 provider 名称用于错误消息
        let provider = self.context.get_provider();

        toolkit::log_debug!("LLM url: {}", url);
        toolkit::log_debug!("LLM payload: {}", payload);
        toolkit::log_debug!("LLM provider: {}", provider);

        // 发送请求
        let response = client.post(&url).auth(auth).body(&payload).send().map_err(|e| {
            LLMError::ApiError(format!("Failed to send LLM request to {}: {}", provider, e))
        })?;

        // 检查 HTTP 状态码
        if !response.is_success() {
            let error_message = response.extract_error_message();
            return Err(LLMError::ApiError(format!(
                "LLM API request failed ({}): {} - {}",
                provider, response.status, error_message
            )));
        }

        // 解析 JSON 响应
        let data: Value = response
            .json()
            .map_err(|e| LLMError::ApiError(format!("Failed to parse JSON response: {}", e)))?;

        // 根据配置的响应格式提取内容
        self.extract_content(&data)
    }
}
