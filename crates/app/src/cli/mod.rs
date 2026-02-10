//! CLI 命令结构定义
//!
//! 这个模块定义了 Workflow CLI 的命令结构，供 `bin/workflow.rs` 使用。
//! 这样可以确保命令结构定义与命令处理逻辑分离，提高代码可维护性。

mod alias;
mod args;
mod branch;
#[cfg(feature = "develop")]
mod commit;
mod completion;
mod github;
mod jira;
mod llm;
mod log;
mod main;
mod pr;
mod repo;
#[cfg(feature = "develop")]
mod rollback;
mod stash;
mod tag;

// 重新导出所有子命令枚举和参数结构
pub use alias::AliasCommand;
pub use args::{DryRunArgs, ForceArgs, JiraIdArg};
pub use branch::{BranchSubcommand, IgnoreSubcommand};
#[cfg(feature = "develop")]
pub use commit::{CommitCommand, CommitSubcommand};
pub use completion::CompletionCommand;
pub use github::GithubCommand;
pub use jira::{AttachmentsArgs, CleanArgs, InfoArgs, JiraCommand, OutputFormat};
pub use llm::LlmCommand;
pub use log::LogCommand;
pub use main::{Cli, Command, UninstallArgs, UpdateArgs};
pub use pr::PrSubcommand;
pub use repo::RepoCommand;
#[cfg(feature = "develop")]
pub use rollback::RollbackCommand;
pub use stash::StashSubcommand;
pub use tag::TagSubcommand;
