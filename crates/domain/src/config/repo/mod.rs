//! 仓库配置模块
//!
//! 包含项目级配置（ProjectConfig）和用户级配置（UserConfig）

pub mod branch;
pub mod config;
pub mod mcp;
pub mod project;
pub mod repository;
pub mod template;
pub mod user;

pub use branch::BranchConfig;
pub use config::RepoConfig;
pub use mcp::{MCPConfig, MCPServerConfig};
pub use project::ProjectConfig;
pub use repository::RepoConfigRepository;
pub use template::{BranchTemplates, CommitTemplates, PullRequestsTemplates, TemplateConfig};
pub use user::UserConfig;
