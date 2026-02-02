//! Commit management subcommands
//!
//! 提交管理子命令结构定义

use clap::{Args, Subcommand};

/// Commit 管理命令
///
/// 可以直接使用 `workflow commit -m "message"` 创建提交，
/// 或使用 `workflow commit amend` 修改最后一次提交。
#[derive(Args)]
pub struct CommitCommand {
    /// 提交消息（用于创建新提交）
    /// Commit message (for creating new commit)
    #[arg(short, long)]
    pub message: Option<String>,

    /// 自动暂存所有变更（类似 git commit -a）
    /// Automatically stage all changes (similar to git commit -a)
    #[arg(short, long)]
    pub all: bool,

    /// 子命令
    #[command(subcommand)]
    pub subcommand: Option<CommitSubcommand>,
}

/// Commit 子命令
#[derive(Subcommand)]
pub enum CommitSubcommand {
    /// 修改最后一次提交（amend）
    ///
    /// Amend the last commit, optionally with a new message.
    /// By default, pre-commit hooks are skipped. Use --verify to enable them.
    Amend(AmendArgs),
}

/// Amend 命令参数
#[derive(Args, Debug, Clone)]
pub struct AmendArgs {
    /// 新的提交消息
    /// New commit message
    #[arg(short, long)]
    pub message: Option<String>,
    /// 不编辑提交消息，保留原消息
    /// Don't edit the commit message, keep the original
    #[arg(long)]
    pub no_edit: bool,
    /// 启用 pre-commit hooks（默认跳过）
    /// Enable pre-commit hooks (skipped by default)
    #[arg(long)]
    pub verify: bool,
}

