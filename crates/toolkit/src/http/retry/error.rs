//! HTTP 重试错误类型

use thiserror::Error;

/// HTTP 重试错误类型
///
/// 用于重试逻辑相关的错误。
#[derive(Debug, Error)]
pub enum HttpRetryError {
    /// 重试检查失败但没有可用错误
    #[error("No error available but retryable check failed")]
    NoErrorAvailable,

    /// 所有重试都失败但没有可用错误
    #[error("All retries failed but no error available")]
    AllRetriesFailedNoError,

    /// 操作在多次重试后失败
    #[error("{operation} failed after {retries} retries: {error_message}")]
    OperationFailedAfterRetries {
        operation: String,
        retries: u32,
        error_message: String,
    },

    /// 其他错误
    #[error("{0}")]
    Other(String),

    /// HTTP 错误
    #[error("HTTP error: {0}")]
    Http(#[from] crate::http::HttpError),
}
