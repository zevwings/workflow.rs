use std::sync::Arc;

use crate::{LLMClient, LLMConversation, LLMError, LLMRequestParameters};

pub trait LLMExecutor: Send + Sync {
    fn execute(
        &self,
        conversation: &dyn LLMConversation,
        language_code: &str,
        context: &str,
    ) -> Result<String, LLMError>;
}

pub struct LLMExecutorImpl {
    client: Arc<dyn LLMClient>,
}

impl LLMExecutorImpl {
    pub fn new(client: Arc<dyn LLMClient>) -> Self {
        Self { client }
    }
}

impl LLMExecutor for LLMExecutorImpl {
    /// 执行 LLM 调用并解析响应
    ///
    /// 从 conversation 获取 prompt 和参数，调用 LLM API，然后解析响应。
    ///
    /// # 参数
    ///
    /// * `conversation` - 实现了 `LLMConversation` trait 的对话实例
    /// * `language_code` - 语言代码（如 "en", "zh"）
    /// * `context` - 上下文信息，用于错误提示
    ///
    /// # 返回
    ///
    /// 返回解析后的结果
    ///
    /// # 错误
    ///
    /// 如果 LLM API 调用失败或响应解析失败，返回相应的错误信息。
    fn execute(
        &self,
        conversation: &dyn LLMConversation,
        language_code: &str,
        context: &str,
    ) -> Result<String, LLMError> {
        // 从 conversation 获取 prompt 和参数
        let system_prompt = conversation.get_system_prompt(language_code);
        let user_prompt = conversation.get_user_prompt(language_code);
        let (max_tokens, temperature) = conversation.get_execution_params();

        // 调用 LLM API
        let params = LLMRequestParameters {
            system_prompt,
            user_prompt,
            max_tokens,
            temperature,
        };

        let response = self.client.call(&params).map_err(|e| {
            // 提取原始错误消息，避免重复的 "LLM API 调用失败: " 前缀
            let original_msg = match &e {
                LLMError::ApiError(msg) => {
                    msg.strip_prefix("LLM API call failed: ").unwrap_or(msg)
                }
                _ => {
                    // 只在需要时分配字符串
                    return LLMError::ApiError(format!(
                        "Failed to call LLM API ({}): {}",
                        context, e
                    ));
                }
            };
            LLMError::ApiError(format!(
                "Failed to call LLM API ({}): {}",
                context, original_msg
            ))
        })?;

        Ok(response)
    }
}
