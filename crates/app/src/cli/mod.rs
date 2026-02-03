//! CLI 命令结构定义
//!
//! 这个模块定义了 Workflow CLI 的命令结构，供 `bin/workflow.rs` 使用。
//! 这样可以确保命令结构定义与命令处理逻辑分离，提高代码可维护性。

mod args;
mod branch;
mod commit;
mod github;
mod jira;
mod llm;
mod log;
mod main;
mod pr;
mod repo;
mod stash;
mod tag;

// 重新导出所有子命令枚举和参数结构
pub use args::{DryRunArgs, ForceArgs, JiraIdArg};
pub use branch::{BranchSubcommand, IgnoreSubcommand};
pub use commit::{AmendArgs, CommitCommand, CommitSubcommand};
pub use github::GithubCommand;
pub use jira::{AttachmentsArgs, CleanArgs, InfoArgs, JiraCommand};
pub use llm::LlmCommand;
pub use log::LogCommand;
pub use main::{Cli, Command};
pub use pr::PrSubcommand;
pub use repo::RepoCommand;
pub use stash::StashSubcommand;
pub use tag::TagSubcommand;
