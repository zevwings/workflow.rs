//! LLM 客户端
//!
//! 本模块提供了 LLM 客户端实现，支持所有遵循 OpenAI 兼容格式的提供商。
//!
//! ## 配置提供者
//!
//! LLM 客户端通过 `LLMConfigContext` trait 获取配置，实现了依赖倒置原则。
//! 通过 `LLMClient::new()` 方法传入配置上下文来创建客户端实例。

use std::{sync::Arc, time::Duration};

use http::{HttpClient, HttpClientConfig};

use serde_json::{json, Value};

use crate::{
    ChatCompletionResponse, IntoLLMConfig, LLMClient, LLMConfigContext, LLMError,
    LLMRequestParameters,
};

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

pub struct LLMClientImpl {
    pub context: Arc<dyn LLMConfigContext>,
    pub timeout: Duration,
}

impl LLMClientImpl {
    pub fn new(context: Arc<dyn LLMConfigContext>, timeout: Duration) -> Self {
        Self { context, timeout }
    }

    /// 构建请求体
    fn build_payload(
        &self,
        model: impl AsRef<str>,
        params: &LLMRequestParameters,
    ) -> Result<Value, LLMError> {
        let mut payload = json!({
            "model": model.as_ref(),
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
    fn extract_content(&self, response: Value) -> Result<String, LLMError> {
        // 解析为标准结构体 - 直接消费 Value，避免 clone
        let completion: ChatCompletionResponse = serde_json::from_value(response).map_err(|e| {
            LLMError::ApiError(format!(
                "Failed to parse response as OpenAI ChatCompletion format: {}",
                e
            ))
        })?;

        log_debug!("LLM response: {:?}", completion);

        // 提取内容
        let first = completion.choices.first();
        let content = first
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| {
                let reason = first
                    .map(|c| c.finish_reason.as_str())
                    .unwrap_or("unknown");
                let msg = if reason == "length" {
                    "No content in response: 响应因达到最大 token 限制被截断 (finish_reason=length)，API 未返回内容。请减少输入或为此次调用设置较小的 max_tokens 以获取完整输出。".to_string()
                } else {
                    format!(
                        "No content in response: choices array is empty or content is null (finish_reason={})",
                        reason
                    )
                };
                LLMError::ApiError(msg)
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
        let llm_config = self.context.as_ref().to_config()?;
        log_debug!("LLM config: {:?}", llm_config);

        // 构建请求体（统一格式）
        let payload = self.build_payload(llm_config.model, params)?;

        /// DEBUG 时单条 payload 最多打印的字符数，避免超大 diff 刷屏导致终端卡顿
        const MAX_PAYLOAD_LOG_LEN: usize = 800;
        let payload_str = payload.to_string();
        log_debug!("LLM url: {}", llm_config.url);
        if payload_str.len() <= MAX_PAYLOAD_LOG_LEN {
            log_debug!("LLM payload: {}", payload_str);
        } else {
            let trunc_at = payload_str
                .char_indices()
                .nth(MAX_PAYLOAD_LOG_LEN)
                .map(|(i, _)| i)
                .unwrap_or(payload_str.len());
            log_debug!(
                "LLM payload (truncated, {} chars total): {}...",
                payload_str.len(),
                &payload_str[..trunc_at]
            );
        }
        log_debug!("LLM provider: {}", llm_config.provider);

        // 创建带超时的 HTTP 客户端
        let http_config = HttpClientConfig::new().timeout(self.timeout);
        let client = HttpClient::with_config(http_config).map_err(|e| {
            LLMError::ApiError(format!("Failed to create HTTP client with timeout: {}", e))
        })?;

        // 发送请求
        let response = client
            .post(&llm_config.url)
            .auth(llm_config.auth)
            .body(&payload)
            .send()
            .map_err(|e| {
                LLMError::ApiError(format!(
                    "Failed to send LLM request to {}: {}",
                    llm_config.provider, e
                ))
            })?;

        // 检查 HTTP 状态码
        if !response.is_success() {
            let error_message = response.extract_error_message();
            return Err(LLMError::ApiError(format!(
                "LLM API request failed ({}): {} - {}",
                llm_config.provider, response.status, error_message
            )));
        }

        // 解析 JSON 响应
        let data: Value = response
            .json()
            .map_err(|e| LLMError::ApiError(format!("Failed to parse JSON response: {}", e)))?;

        // 根据配置的响应格式提取内容（直接传递 Value，避免引用）
        self.extract_content(data)
    }
}
