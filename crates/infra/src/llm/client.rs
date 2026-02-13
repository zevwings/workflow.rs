//! LLM 客户端
//!
//! 本模块提供了 LLM 客户端实现，支持所有遵循 OpenAI 兼容格式的提供商。
//!
//! ## 配置提供者
//!
//! LLM 客户端通过 `LLMConfigContext` trait 获取配置，实现了依赖倒置原则。
//! 通过 `LLMClient::new()` 方法传入配置上下文来创建客户端实例。

use std::sync::Arc;

use client::{HttpClient, HttpClientHolder};

use serde_json::{json, Value};

use client::{
    ChatCompletionResponse, IntoLLMConfig, LLMClient, LLMConfigContext, LLMError,
    LLMRequestParameters,
};
use toolkit::log_debug;

pub struct LLMClientImpl {
    pub context: Arc<dyn LLMConfigContext>,
    pub holder: HttpClientHolder,
}

impl LLMClientImpl {
    pub fn new(http_client: Arc<dyn HttpClient>, context: Arc<dyn LLMConfigContext>) -> Self {
        let holder = HttpClientHolder::new(http_client);
        Self { holder, context }
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
    fn call(&self, params: &LLMRequestParameters) -> Result<ChatCompletionResponse, LLMError> {
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

        // // 创建带超时的 HTTP 客户端
        // let http_config = HttpClientConfig::new().timeout(self.timeout);
        // let client = self.http_client.with_config(http_config).map_err(|e| {
        //     LLMError::ApiError(format!("Failed to create HTTP client with timeout: {}", e))
        // })?;

        // 发送请求
        let response =
            self.holder.post(&llm_config.url).auth(llm_config.auth).body(&payload)?.send()?;

        // 检查 HTTP 状态码
        if !response.is_success() {
            let error_message = response
                .get_error_message()
                .map_err(|e| LLMError::ApiError(format!("Failed to get error message: {}", e)))?;
            return Err(LLMError::ApiError(format!(
                "LLM API request failed ({}): {} - {}",
                llm_config.provider, response.status, error_message
            )));
        }

        let completion: ChatCompletionResponse = response.json()?;

        Ok(completion)
    }
}
