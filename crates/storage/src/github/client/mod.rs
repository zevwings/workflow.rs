//! GitHub 客户端模块
//!
//! 提供 GitHub API 客户端和配置上下文实现

mod context;
mod core;
mod response;

pub use context::GitHubContextImpl;
pub use core::{GitHubClient, GitHubClientImpl};
