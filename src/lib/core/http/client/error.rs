//! HTTP 客户端错误类型

use crate::core::http::client::method::HttpMethodError;
use thiserror::Error;

/// HTTP 客户端错误类型
///
/// 用于客户端初始化、网络连接、请求执行相关的错误。
#[derive(Debug, Error)]
pub enum HttpClientError {
    /// 创建 HTTP 客户端失败
    #[error("Failed to create HTTP client: {0}")]
    CreateClientFailed(#[from] reqwest::Error),

    /// 网络超时
    #[error("Network timeout: {url} ({method})")]
    Timeout { url: String, method: String },

    /// 连接失败
    #[error("Connection failed: {url} ({method})")]
    ConnectionFailed { url: String, method: String },

    /// 速率限制超出
    #[error("Rate limit exceeded: {url} ({method})")]
    RateLimitExceeded { url: String, method: String },

    /// 请求失败
    #[error("Failed to send {method} request to {url}: {source}")]
    RequestFailed {
        method: String,
        url: String,
        source: reqwest::Error,
    },

    /// HTTP 方法错误
    #[error(transparent)]
    Method(#[from] HttpMethodError),
}
