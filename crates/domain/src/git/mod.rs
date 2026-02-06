//! Git 业务域
//!
//! 包含 Git 相关的实体、仓储接口和错误类型

pub mod entity;
pub mod error;
pub mod repository;

// Re-export public types
pub use entity::{
    BlameLineInfo, BranchFilter, BranchInfo, CodePlatform, CommitChangeType, CommitFileChange,
    CommitInfo, FileStatusInfo, FileStatusType, MergeStrategy, RemoteInfo, RepoInfo,
    StashApplyResult, StashEntry, StashPopResult, StashStat, TagCreateInfo, TagCreateScope,
    TagDeleteInfo, TagDeleteScope, WorkingTreeStatus,
};
pub use error::GitError;
pub use repository::{GitRepoRepository, GitRepository};
