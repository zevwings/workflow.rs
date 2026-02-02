//! Commit 业务域
//!
//! 包含 Commit 相关的实体和服务接口

pub mod entity;
pub mod service;

// Re-export public types
pub use entity::{AmendPreview, CommitAmend, CommitReword, CommitSquash};
pub use service::CommitService;
