//! 顶层 CLI 结构定义
//!
//! 定义 Workflow CLI 的顶层命令结构（Cli 和 Command）。

use clap::{Parser, Subcommand};

use super::alias::AliasCommand;
use super::branch::BranchSubcommand;
use super::commit::CommitCommand;
use super::completion::CompletionCommand;
use super::github::GithubCommand;
use super::jira::JiraCommand;
use super::llm::LlmCommand;
use super::log::LogCommand;
use super::pr::PrSubcommand;
use super::repo::RepoCommand;
#[cfg(feature = "develop")]
use super::rollback::RollbackCommand;
use super::stash::StashSubcommand;
use super::tag::TagSubcommand;

/// 顶层 CLI 定义
#[derive(Parser)]
#[command(
    name = "workflow",
    author,
    version = env!("CARGO_PKG_VERSION"),
    about = "Workflow CLI - Git / PR / Jira / LLM 工作流助手",
    propagate_version = true
)]
pub struct Cli {
    /// 子命令
    #[command(subcommand)]
    pub command: Command,
}

/// 支持的子命令
#[derive(Subcommand)]
pub enum Command {
    /// 显示版本信息
    Version,
    /// 查看当前配置并执行环境检查
    Check,
    /// 交互式初始化或更新配置
    Setup,
    /// 更新 Workflow CLI 到最新版本
    Update(UpdateArgs),
    /// 卸载 Workflow CLI
    Uninstall(UninstallArgs),
    /// 仓库管理命令
    #[command(subcommand)]
    Repo(RepoCommand),
    /// 日志管理命令
    #[command(subcommand)]
    Log(LogCommand),
    /// LLM 配置管理命令
    #[command(subcommand)]
    Llm(LlmCommand),
    /// GitHub 账号管理命令
    #[command(subcommand)]
    Github(GithubCommand),
    /// Jira 配置管理命令
    #[command(subcommand)]
    Jira(JiraCommand),
    /// 分支管理命令
    #[command(subcommand)]
    Branch(BranchSubcommand),
    /// 提交管理命令
    Commit(CommitCommand),
    /// Stash 管理命令
    #[command(subcommand)]
    Stash(StashSubcommand),
    /// Tag 管理命令
    #[command(subcommand)]
    Tag(TagSubcommand),
    /// Pull Request 管理命令
    #[command(subcommand)]
    Pr(PrSubcommand),
    /// 推送当前分支到远程
    Push,
    /// 从远程拉取当前分支
    Pull,
    /// Shell Completion 管理命令
    #[command(subcommand)]
    Completion(CompletionCommand),
    /// 别名管理命令
    #[command(subcommand)]
    Alias(AliasCommand),
    /// 回滚管理命令（开发用）
    #[cfg(feature = "develop")]
    #[command(subcommand)]
    Rollback(RollbackCommand),
}

/// Update 命令参数
#[derive(Parser, Debug)]
pub struct UpdateArgs {
    /// 目标版本号（如 1.2.3），不指定则更新到最新版本
    #[arg(short = 't', long = "target")]
    pub target_version: Option<String>,

    /// 跳过确认，直接执行更新
    #[arg(short, long)]
    pub force: bool,

    /// GitHub Token（用于提高 API 速率限制，也可通过 GITHUB_TOKEN 环境变量设置）
    #[arg(long)]
    pub github_token: Option<String>,
}

/// Uninstall 命令参数
#[derive(Parser, Debug)]
pub struct UninstallArgs {
    /// 跳过确认，直接执行卸载
    #[arg(short, long)]
    pub force: bool,

    /// 保留配置文件（不删除 workflow.toml 等）
    #[arg(long)]
    pub keep_config: bool,
}
