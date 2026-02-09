//! Commit Message 生成服务
//!
//! 为单次提交场景提供轻量级的 commit message 生成功能。

mod conversation;
mod service;

pub use conversation::CommitMessageConversation;
pub use service::CommitMessageServiceImpl;
