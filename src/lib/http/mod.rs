//! HTTP 客户端模块
//!
//! 本模块提供了 HTTP 请求的完整功能，包括：
//! - HTTP 客户端封装（GET、POST、PUT、DELETE、PATCH）
//! - Basic Authentication 支持
//! - 自定义 Headers 支持
//! - HTTP 响应封装和解析
//!
//! ## 模块结构
//!
//! - `client` - HTTP 客户端（`HttpClient`、`Authorization`）
//! - `response` - HTTP 响应（`HttpResponse`）

pub mod client;
pub mod response;

pub use client::{Authorization, HttpClient};
pub use response::HttpResponse;
