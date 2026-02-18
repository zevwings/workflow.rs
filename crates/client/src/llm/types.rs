//! LLM 类型定义

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::{
    llm::{JsonParser, TextParser},
    LLMError,
};

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
            system_prompt: String::with_capacity(512),
            user_prompt: String::with_capacity(512),
            max_tokens: None,
            temperature: 0.5,
        }
    }
}

// ==================== OpenAI 响应数据模型 ====================

/// OpenAI Chat Completions API 响应
///
/// 完整的 OpenAI 标准响应格式，支持所有标准字段和扩展字段。
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// 响应唯一标识符
    pub id: String,
    /// 对象类型，固定为 "chat.completion"
    pub object: String,
    /// 创建时间戳（Unix 时间戳）
    pub created: u64,
    /// 使用的模型名称
    pub model: String,
    /// 系统指纹（可选）
    pub system_fingerprint: Option<String>,
    /// 选择列表
    pub choices: Vec<ChatCompletionChoice>,
    /// Token 使用统计
    pub usage: Usage,
}

impl ChatCompletionResponse {
    /// 从响应中提取内容
    ///
    /// 使用 OpenAI 标准格式解析响应，提取消息内容。
    /// 支持所有遵循 OpenAI Chat Completions API 标准的响应格式。
    pub fn get_content(&self) -> Result<String, LLMError> {
        let first = self.choices.first();
        let content = first
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| {
                let reason = first
                    .map(|c| c.finish_reason.as_str())
                    .unwrap_or("unknown");
                let msg = if reason == "length" {
                    "No content in response, the response was truncated due to reaching the maximum token limit (finish_reason=length), and the API did not return any content. Please reduce the input or set a smaller max_tokens for this call to get the complete output.".to_string()
                } else {
                    format!(
                        "No content in response, the choices array is empty or the content is null (finish_reason={})",
                        reason
                    )
                };
                LLMError::ApiError(msg)
            })?;

        Ok(content.trim().to_string())
    }

    pub fn to_model<T>(&self) -> Result<T, LLMError>
    where
        T: for<'de> Deserialize<'de>,
    {
        JsonParser::to_model(&self.get_content()?)
    }

    pub fn to_text(&self) -> Result<String, LLMError> {
        TextParser::clean_and_validate(self.get_content()?)
    }

    pub fn to_map(&self) -> Result<Map<String, Value>, LLMError> {
        JsonParser::to_map(&self.get_content()?)
    }
}

/// Chat Completion 选择项
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    /// 选择索引
    pub index: u32,
    /// 消息对象
    pub message: ChatMessage,
    /// 对数概率（可选）
    pub logprobs: Option<serde_json::Value>, // 使用 Value 以支持各种格式
    /// 完成原因
    pub finish_reason: String,
}

/// Chat 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 消息角色
    pub role: String,
    /// 消息内容（可能为 null）
    pub content: Option<String>,
}

/// Token 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// 提示词 token 数
    pub prompt_tokens: u32,
    /// 完成 token 数
    pub completion_tokens: u32,
    /// 总 token 数
    pub total_tokens: u32,
}
