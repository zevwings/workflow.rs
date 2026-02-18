//! CLI 命令结构定义
//!
//! 这个模块定义了 Workflow CLI 的命令结构，供 `bin/workflow.rs` 使用。
//! 这样可以确保命令结构定义与命令处理逻辑分离，提高代码可维护性。

use clap::{Parser, Subcommand};

// 从 commands 重新导出所有命令（供 bin/workflow.rs 使用）
pub use crate::commands::alias::AliasCommand;
// 从 commands 重新导出共享参数（供外部使用）
pub use crate::commands::args::{DryRunArgs, ForceArgs, JiraIdArg};
#[cfg(feature = "develop")]
pub use crate::commands::commit::{CommitCommand, CommitSubcommand};
#[cfg(feature = "develop")]
pub use crate::commands::rollback::RollbackCommand;
pub use crate::commands::{
    branch::{BranchSubcommand, IgnoreSubcommand},
    completion::CompletionCommand,
    github::GithubCommand,
    jira::{AttachmentsArgs, CleanArgs, InfoArgs, JiraCommand, OutputFormat},
    llm::LlmCommand,
    log::LogCommand,
    pr::PrSubcommand,
    repo::RepoCommand,
    stash::StashSubcommand,
    tag::TagSubcommand,
};

/// 顶层 CLI 定义
#[derive(Parser)]
#[command(
    name = "workflow",
    author,
    version = env!("CARGO_PKG_VERSION"),
    about = "Workflow CLI - Git / PR / Jira / LLM workflow assistant",
    propagate_version = true
)]
pub struct Cli {
    /// Subcommands
    #[command(subcommand)]
    pub command: Command,
}

/// 支持的子命令
#[derive(Subcommand)]
pub enum Command {
    /// Show version information
    Version,
    /// Check current configuration and run environment verification
    Check,
    /// Interactive setup or update configuration
    Setup,
    /// Update Workflow CLI to latest version
    Update(UpdateArgs),
    /// Uninstall Workflow CLI
    Uninstall(UninstallArgs),
    /// Repository management commands
    #[command(subcommand)]
    Repo(RepoCommand),
    /// Log management commands
    #[command(subcommand)]
    Log(LogCommand),
    /// LLM configuration management commands
    #[command(subcommand)]
    Llm(LlmCommand),
    /// GitHub account management commands
    #[command(subcommand)]
    Github(GithubCommand),
    /// Jira configuration management commands
    #[command(subcommand)]
    Jira(JiraCommand),
    /// Branch management commands
    #[command(subcommand)]
    Branch(BranchSubcommand),
    /// Commit management commands
    #[cfg(feature = "develop")]
    Commit(CommitCommand),
    /// Stash management commands
    #[command(subcommand)]
    Stash(StashSubcommand),
    /// Tag management commands
    #[command(subcommand)]
    Tag(TagSubcommand),
    /// Pull Request management commands
    #[command(subcommand)]
    Pr(PrSubcommand),
    /// Push current branch to remote
    #[cfg(feature = "develop")]
    Push,
    /// Pull current branch from remote
    #[cfg(feature = "develop")]
    Pull,
    /// Shell completion management commands
    #[command(subcommand)]
    Completion(CompletionCommand),
    /// Alias management commands
    #[command(subcommand)]
    Alias(AliasCommand),
    /// Rollback management commands (development only)
    #[cfg(feature = "develop")]
    #[command(subcommand)]
    Rollback(RollbackCommand),
}

/// Update 命令参数
#[derive(Parser, Debug)]
pub struct UpdateArgs {
    /// Target version (e.g. 1.2.3), omit to update to latest
    #[arg(short = 't', long = "target")]
    pub target_version: Option<String>,

    /// Skip confirmation and force update
    #[arg(short, long)]
    pub force: bool,

    /// GitHub Token (for higher API rate limit, or set via GITHUB_TOKEN env var)
    #[arg(long)]
    pub github_token: Option<String>,
}

/// Uninstall 命令参数
#[derive(Parser, Debug)]
pub struct UninstallArgs {
    /// Skip confirmation and force uninstall
    #[arg(short, long)]
    pub force: bool,

    /// Keep config files (do not delete workflow.toml, etc.)
    #[arg(long)]
    pub keep_config: bool,
}
