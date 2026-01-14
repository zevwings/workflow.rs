//! GitHub API 封装模块
//!
//! 本模块提供了与 GitHub API 交互的完整功能，包括：
//! - Pull Request 操作（创建、合并、查询、更新等）
//! - 用户信息管理
//! - 仓库信息查询
//! - 错误处理
//!
//! ## 架构设计
//!
//! 本模块是独立的 GitHub API 封装，不依赖 PR 模块。
//! 通过 `infra::adapters` 层适配为 `PlatformProvider` trait，实现依赖倒置。

pub mod api;
pub mod errors;
pub mod types;

// 重新导出主要类型
pub use api::GitHub;
pub use errors::{
    format_error, handle_github_error, GitHubApiError, GitHubError, GitHubErrorResponse,
};
pub use types::GitHubUser;
