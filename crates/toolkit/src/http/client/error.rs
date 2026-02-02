//! HTTP 错误类型

use thiserror::Error;

use crate::http::HttpMethodError;

/// HTTP 错误类型
///
/// 统一的 HTTP 错误类型，包含客户端、网络连接、请求执行、响应处理相关的错误。
#[derive(Debug, Error)]
pub enum HttpError {
    // ===== 客户端和网络错误 =====
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

    // ===== 响应处理错误 =====
    /// 无法读取响应体
    #[error("Unable to read response body: {0}")]
    UnableToReadBody(String),

    /// HTTP 响应错误（带状态码和响应体）
    #[error("HTTP request failed with status {status}: {body}")]
    ResponseFailed { status: u16, body: String },

    /// 解析空响应为 JSON 失败
    #[error("Failed to parse empty response as JSON")]
    ParseEmptyJsonFailed,

    /// 解析 JSON 响应失败
    #[error("Failed to parse JSON response (HTTP {status}). Response preview: {preview}")]
    ParseJsonFailed { status: u16, preview: String },

    /// HTTP 请求失败（带状态码）
    #[error("HTTP request failed with status {0}")]
    HttpRequestFailed(u16),

    /// 解码响应体为 UTF-8 文本失败
    #[error("Failed to decode response body as UTF-8 text: {0}")]
    DecodeUtf8Failed(#[from] std::string::FromUtf8Error),

    // ===== 通用错误 =====
    /// 通用错误
    #[error("{0}")]
    Other(String),
}
