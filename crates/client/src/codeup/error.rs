//! Codeup 客户端错误类型

use thiserror::Error;

/// Codeup API 错误
#[derive(Error, Debug)]
pub enum CodeupClientError {
    #[error("Codeup API 调用失败: {0}")]
    ApiError(String),

    #[error("认证失败，请检查 CSRF Token 和 Cookie")]
    AuthenticationFailed,

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("HTTP 错误: {0}")]
    HttpError(String),

    #[error("JSON 解析错误: {0}")]
    JsonError(String),
}

impl From<crate::HttpError> for CodeupClientError {
    fn from(err: crate::HttpError) -> Self {
        CodeupClientError::HttpError(err.to_string())
    }
}
