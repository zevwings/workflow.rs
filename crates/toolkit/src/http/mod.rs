//! HTTP 客户端模块
//!
//! 本模块提供了 HTTP 请求的完整功能，包括：
//! - HTTP 客户端封装（GET、POST、PUT、DELETE、PATCH）
//! - 多种认证方式支持（Basic Auth、Bearer Token）
//! - 自定义 Headers 支持
//! - HTTP 响应封装和解析
//! - HTTP 请求重试机制（支持 Retry-After header）
//! - 连接池管理和性能优化
//! - 请求/响应大小限制
//! - 请求 ID 追踪
//!
//! ## 模块结构
//!
//! - `auth` - HTTP 认证（`Authorization`）
//! - `client` - HTTP 客户端（`HttpClient`、`HttpClientConfig`、`HttpError`、`HttpResponse`）
//! - `request` - HTTP 请求配置（`RequestConfig`、`MultipartRequestConfig`）
//! - `retry` - HTTP 重试工具（`HttpRetry`、`HttpRetryConfig`、`HttpRetryError`、`RetryResult`）

pub mod auth;
pub mod client;
pub mod request;
pub mod retry;

#[cfg(test)]
mod mock;

#[cfg(test)]
pub use mock::HttpMockServer;

// 重新导出主要类型（便利性导出）
pub use auth::Authorization;
pub use client::{
    HttpClient, HttpClientConfig, HttpError, HttpMethod, HttpMethodError, HttpResponse,
};
pub use request::{IntoHeaderMap, MultipartRequestConfig, RequestConfig};
pub use retry::{HttpRetry, HttpRetryConfig, HttpRetryError, RetryResult};
