//! HTTP 实现层
//!
//! 使用 reqwest 实现 `client::HttpClient` trait。
//! 链式 API（get/post/multipart）由 client 定义，通过 `client::http` 访问。
//!
//! # 使用方式
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use client::http::{HttpClient, HttpClientHolder, MultipartRequest};
//! use infra::http::ReqwestHttpClient;
//!
//! // DI 注入 Arc<dyn HttpClient>
//! let client: Arc<dyn HttpClient> = Arc::new(ReqwestHttpClient::new()?);
//! let holder = HttpClientHolder::new(client);
//!
//! // GET
//! let response = holder.get("/users").send()?;
//!
//! // POST with JSON
//! let response = holder.post("/users").body(&payload)?.send()?;
//!
//! // Multipart
//! let mp = MultipartRequest::new()
//!     .text("name", "value")
//!     .file("file", path);
//! let response = holder.post("/upload").multipart(mp).send()?;
//! ```

mod auth;
mod client;
mod error;
mod multipart;
mod response;
mod rest;
mod retry;

pub use client::ReqwestHttpClient;
pub use rest::RestRequestBuilder;
pub use retry::{execute_with_retry, RetryConfig};
