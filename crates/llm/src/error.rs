//! LLM 错误类型

use thiserror::Error;

/// LLM 服务错误
#[derive(Error, Debug, Clone)]
pub enum LLMError {
    #[error("LLM API 调用失败: {0}")]
    ApiError(String),

    #[error("认证失败")]
    AuthenticationFailed,

    #[error("生成失败: {0}")]
    GenerationFailed(String),

    #[error("速率限制: {0}")]
    RateLimitExceeded(String),

    #[error("空响应: {0}")]
    EmptyResponse(String),
}
