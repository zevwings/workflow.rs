//! HTTP 客户端模块

pub mod config;
pub mod error;
pub mod method;
pub mod response;

#[allow(clippy::module_inception)]
mod client;

pub use client::HttpClient;
pub use config::HttpClientConfig;
pub use error::HttpError;
pub use method::{HttpMethod, HttpMethodError};
pub use response::HttpResponse;
