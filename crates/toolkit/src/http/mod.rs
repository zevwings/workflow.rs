//! HTTP 客户端模块
//!
//! 提供现代化的 HTTP 客户端 API，支持链式调用和增强的错误上下文。
//!
//! # 使用示例
//!
//! ```ignore
//! use toolkit::http::{HttpClient, Authorization};
//!
//! // 使用全局客户端
//! let response = HttpClient::global()?
//!     .get("https://api.example.com/users")
//!     .query(&[("page", "1")])
//!     .auth(Authorization::bearer("token"))
//!     .send()?;
//!
//! // 自定义客户端
//! let client = HttpClient::with_config(
//!     HttpClientConfig::new().timeout(Duration::from_secs(60))
//! )?;
//!
//! let response = client
//!     .post("https://api.example.com/users")
//!     .body(&serde_json::json!({"name": "test"}))
//!     .send()?;
//! ```

mod auth;
mod client;
mod config;
mod error;
mod headers;
mod method;
mod multipart;
mod request;
mod response;
mod retry;

pub use auth::Authorization;
pub use client::HttpClient;
pub use config::HttpClientConfig;
pub use error::{ErrorContext, HttpError};
pub use headers::IntoHeaderMap;
pub use method::HttpMethod;
pub use multipart::MultipartRequest;
pub use request::Request;
pub use response::Response;
pub use retry::{RetryConfig, RetryResult};

#[cfg(test)]
pub(crate) mod mock;
