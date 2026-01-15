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
//! - `client` - HTTP 客户端（`HttpClient`、`HttpClientConfig`、`ClientHttpError`）
//! - `headers` - HTTP Headers 工具（`IntoHeaderMap`）
//! - `method` - HTTP 方法（`HttpMethod`、`HttpMethodError`）
//! - `parser` - HTTP 响应解析器（`ResponseParser`、`JsonParser`、`TextParser`）
//! - `request` - HTTP 请求配置（`RequestConfig`、`MultipartRequestConfig`）
//! - `response` - HTTP 响应（`HttpResponse`、`HttpResponseError`）
//! - `retry` - HTTP 重试工具（`HttpRetry`、`HttpRetryConfig`、`HttpRetryError`、`RetryResult`）

pub mod auth;
pub mod client;
pub mod request;
pub mod response;
pub mod retry;

// 重新导出主要类型（便利性导出）
pub use auth::Authorization;
pub use client::{HttpClient, HttpClientConfig, HttpClientError, HttpMethod, HttpMethodError};
pub use request::{IntoHeaderMap, MultipartRequestConfig, RequestConfig};
pub use response::{HttpResponse, HttpResponseError, JsonParser, ResponseParser, TextParser};
pub use retry::{HttpRetry, HttpRetryConfig, HttpRetryError, RetryResult};
