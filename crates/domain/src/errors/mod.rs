//! 错误类型模块
//!
//! 包含领域层的错误类型定义

pub mod service;

// Re-export public types
pub use service::ServiceError;
