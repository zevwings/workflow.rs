//! Commit management subcommands
//!
//! 提交管理子命令结构定义

use clap::{Args, Subcommand};

/// Commit 管理命令
///
/// 使用 `workflow commit -m "message"` 创建提交；
/// 启用 develop feature 时可用子命令做接口测试。
#[derive(Args)]
pub struct CommitCommand {
    /// 提交消息（用于创建新提交）
    #[arg(short, long)]
    pub message: Option<String>,

    /// 自动暂存所有变更（类似 git commit -a）
    #[arg(short, long, default_value_t = true)]
    pub all: bool,

    /// 子命令（develop feature 下可用）
    #[cfg(feature = "develop")]
    #[command(subcommand)]
    pub subcommand: Option<CommitSubcommand>,
}

/// Commit 子命令（仅 develop feature）
#[cfg(feature = "develop")]
#[derive(Subcommand)]
pub enum CommitSubcommand {
    /// 列出将源分支合并到目标分支时会引入的 commit SHA 列表
    #[command(name = "to-merge")]
    CommitToMerge {
        #[arg(value_name = "SOURCE")]
        source_branch: String,
        #[arg(value_name = "TARGET")]
        target_branch: String,
    },
    /// 获取指定 commit 变更的文件列表
    #[command(name = "files")]
    CommitFiles {
        #[arg(value_name = "REF")]
        ref_or_sha: String,
    },
    /// 获取指定 commit 的 diff 内容（patch）
    #[command(name = "diff")]
    CommitDiff {
        #[arg(value_name = "REF")]
        ref_or_sha: String,
    },
}
