//! 顶层 CLI 结构定义
//!
//! 定义 Workflow CLI 的顶层命令结构（Cli 和 Command）。

use clap::{Parser, Subcommand};

use super::branch::BranchSubcommand;
use super::cnb::CNBCommand;
use super::commit::CommitCommand;
use super::github::GithubCommand;
use super::jira::JiraCommand;
use super::llm::LlmCommand;
use super::log::LogCommand;
use super::pr::PrSubcommand;
use super::repo::RepoCommand;
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
    /// CNB 账号管理命令
    #[command(subcommand)]
    Cnb(CNBCommand),
    /// 分支管理命令
    #[command(subcommand)]
    Branch(BranchSubcommand),
    /// 提交管理命令
    Commit(CommitCommand),
    /// Tag 管理命令
    #[command(subcommand)]
    Tag(TagSubcommand),
    /// Pull Request 管理命令
    #[command(subcommand)]
    Pr(PrSubcommand),
    /// 推送分支到远程
    ///
    /// Push the current branch to remote repository.
    Push,
}
