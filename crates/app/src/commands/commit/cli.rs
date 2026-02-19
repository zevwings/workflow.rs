//! Commit management subcommands
//!
//! 提交管理子命令结构定义

use clap::Args;
#[cfg(feature = "develop")]
use clap::Subcommand;

use crate::commands::args::DryRunArgs;

/// Commit 管理命令
///
/// 使用 `workflow commit` 由 AI 生成并提交，或 `workflow commit -m "message"` 使用自定义消息；
/// 启用 develop feature 时可用子命令做接口测试。
#[derive(Args)]
pub struct CommitCommand {
    /// 提交消息（可选，未提供时由 AI 根据暂存变更生成）
    #[arg(short, long)]
    pub message: Option<String>,

    /// 自动暂存所有变更（类似 git commit -a）
    #[arg(short, long, default_value_t = true)]
    pub all: bool,

    /// 提交后自动推送到远端
    #[arg(long, short = 'p', default_value_t = false)]
    pub push: bool,

    /// Dry run 模式（仅生成 commit message，不实际提交）
    #[command(flatten)]
    pub dry_run: DryRunArgs,

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
