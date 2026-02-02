//! Completion 业务域
//!
//! 包含 Completion 相关的实体和服务接口

pub mod entity;
pub mod service;

// Re-export public types
pub use entity::{Completion, CompletionConfigResult, CompletionRemovalResult};
pub use service::CompletionService;
