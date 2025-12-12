//! LLM 客户端共享类型和工具

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// LLM 请求参数
///
/// 包含调用 LLM API 所需的所有参数。
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct LLMRequestParams {
    /// 系统提示词
    pub system_prompt: String,
    /// 用户提示词
    pub user_prompt: String,
    /// 最大 token 数（None 表示不限制，使用模型默认最大值）
    pub max_tokens: Option<u32>,
    /// 温度参数（控制输出的随机性）
    pub temperature: f32,
    /// 模型名称（如 "gpt-3.5-turbo"）
    pub model: String,
}

impl Default for LLMRequestParams {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            user_prompt: String::new(),
            max_tokens: None,
            temperature: 0.5,
            model: "gpt-3.5-turbo".to_string(),
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
