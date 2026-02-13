//! LLM 错误类型

use thiserror::Error;

use crate::http::HttpError;

/// LLM 服务错误
#[derive(Error, Debug, Clone)]
pub enum LLMError {
    #[error("创建 LLM 客户端失败: {0}")]
    ClientCreationFailed(String),

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

impl From<HttpError> for LLMError {
    fn from(e: HttpError) -> Self {
        LLMError::ApiError(e.to_string())
    }
}
