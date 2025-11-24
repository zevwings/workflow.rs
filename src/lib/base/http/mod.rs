//! HTTP 客户端模块
//!
//! 本模块提供了 HTTP 请求的完整功能，包括：
//! - HTTP 客户端封装（GET、POST、PUT、DELETE、PATCH）
//! - Basic Authentication 支持
//! - 自定义 Headers 支持
//! - HTTP 响应封装和解析
//! - HTTP 请求重试机制
//!
//! ## 模块结构
//!
//! - `auth` - Basic Authentication（`Authorization`）
//! - `client` - HTTP 客户端（`HttpClient`）
//! - `method` - HTTP 方法（`HttpMethod`）
//! - `config` - HTTP 请求配置（`RequestConfig`）
//! - `response` - HTTP 响应（`HttpResponse`）
//! - `parser` - HTTP 响应解析器（`ResponseParser`、`JsonParser`、`TextParser`）
//! - `retry` - HTTP 重试工具（`HttpRetry`、`HttpRetryConfig`）

pub mod auth;
pub mod client;
pub mod config;
pub mod method;
pub mod parser;
pub mod response;
pub mod retry;

pub use auth::Authorization;
pub use client::HttpClient;
pub use config::RequestConfig;
pub use method::HttpMethod;
pub use parser::{JsonParser, ResponseParser, TextParser};
pub use response::HttpResponse;
pub use retry::{HttpRetry, HttpRetryConfig};
