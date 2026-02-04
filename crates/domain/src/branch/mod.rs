//! Branch 业务域
//!
//! 包含 Branch 相关的实体和服务接口

pub mod entity;
pub mod service;

// Re-export public types
pub use entity::{
    sanitize_branch_name, BranchSyncCallbacks, BranchSyncOptions, BranchSyncResult, BranchType,
    SourceBranchInfo, SyncStrategy,
};
pub use service::BranchService;
