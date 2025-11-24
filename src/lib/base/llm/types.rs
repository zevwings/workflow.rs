//! LLM 客户端共享类型和工具

use serde::Serialize;

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
    /// 最大 token 数
    pub max_tokens: u32,
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
            max_tokens: 100,
            temperature: 0.5,
            model: "gpt-3.5-turbo".to_string(),
        }
    }
}
