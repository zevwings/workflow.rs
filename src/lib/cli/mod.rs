//! CLI 命令结构定义
//!
//! 这个模块定义了 Workflow CLI 的命令结构，供 `main.rs` 和补全生成器使用。
//! 这样可以确保补全脚本与实际命令结构保持同步。

use clap::Parser;

// 导入所有子命令枚举
mod branch;
mod commands;
mod commit;
mod common;
mod config;
mod github;
mod jira;
mod llm;
mod log;
mod pr;
mod proxy;
mod stash;

// 重新导出所有子命令枚举和主结构体，保持向后兼容
// 这些导出是必需的，因为 bin/workflow.rs 需要使用它们进行命令分发
pub use branch::{BranchSubcommand, IgnoreSubcommand, PrefixSubcommand};
pub use commands::Commands;
pub use commit::CommitSubcommand;
pub use common::{DryRunArgs, JiraIdArg, OutputFormatArgs};
pub use config::{CompletionSubcommand, ConfigSubcommand, LogLevelSubcommand};
pub use github::GitHubSubcommand;
pub use jira::JiraSubcommand;
pub use llm::LLMSubcommand;
pub use log::LogSubcommand;
pub use pr::PRCommands;
pub use proxy::ProxySubcommand;
pub use stash::StashSubcommand;

/// CLI 主结构体
///
/// 使用 clap 进行命令行参数解析，支持子命令模式。
#[derive(Parser)]
#[command(name = "workflow")]
#[command(about = "Workflow CLI tool", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}
