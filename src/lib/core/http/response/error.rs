//! HTTP 响应错误类型

use thiserror::Error;

/// HTTP 响应错误类型
///
/// 用于响应体读取、解析、解码相关的错误。
#[derive(Debug, Error)]
pub enum HttpResponseError {
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
}
