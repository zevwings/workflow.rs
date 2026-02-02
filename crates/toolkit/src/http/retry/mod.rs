//! HTTP 重试模块

pub mod config;
pub mod error;

#[allow(clippy::module_inception)]
mod retry;

pub use config::{HttpRetryConfig, RetryResult};
pub use error::HttpRetryError;
pub use retry::HttpRetry;
