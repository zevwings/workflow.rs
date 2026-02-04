//! Commit management subcommands
//!
//! 提交管理子命令结构定义

use clap::Args;

/// Commit 管理命令
///
/// 使用 `workflow commit -m "message"` 创建提交。
#[derive(Args)]
pub struct CommitCommand {
    /// 提交消息（用于创建新提交）
    /// Commit message (for creating new commit)
    #[arg(short, long)]
    pub message: Option<String>,

    /// 自动暂存所有变更（类似 git commit -a）
    /// Automatically stage all changes (similar to git commit -a)
    #[arg(short, long, default_value_t = true)]
    pub all: bool,
}
