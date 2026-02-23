//! LLM 错误类型

use thiserror::Error;

use crate::http::HttpError;

/// LLM 服务错误
#[derive(Error, Debug, Clone)]
pub enum LLMError {
    #[error("Failed to create LLM client: {0}")]
    ClientCreationFailed(String),

    #[error("LLM API call failed: {0}")]
    ApiError(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Empty response: {0}")]
    EmptyResponse(String),
}

impl From<HttpError> for LLMError {
    fn from(e: HttpError) -> Self {
        LLMError::ApiError(e.to_string())
    }
}
