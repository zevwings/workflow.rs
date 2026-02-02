//! Tag management subcommands
//!
//! Tag 管理子命令结构定义

use clap::Subcommand;

use super::args::{DryRunArgs, ForceArgs};

/// Tag management subcommands
///
/// 用于管理 Git Tag。
#[derive(Subcommand)]
pub enum TagSubcommand {
    /// 创建 Tag
    ///
    /// Create a new tag, optionally with a message (annotated tag).
    Create {
        /// Tag 名称
        /// Tag name
        tag_name: String,
        /// 目标 commit（可选，默认为 HEAD）
        /// Target commit (optional, defaults to HEAD)
        #[arg(long, short = 't')]
        target: Option<String>,
        /// Tag 消息（可选，提供则创建 annotated tag）
        /// Tag message (optional, creates annotated tag if provided)
        #[arg(long, short = 'm')]
        message: Option<String>,
        /// 只创建本地 tag（不推送到远程）
        /// Create only local tag (do not push to remote)
        #[arg(long)]
        local: bool,
        /// 强制创建（如果 tag 已存在则覆盖）
        /// Force create (overwrite if tag already exists)
        #[command(flatten)]
        force: ForceArgs,
    },
    /// 删除 Tag
    ///
    /// Remove a tag locally and/or remotely.
    Remove {
        /// Tag 名称（可选，不提供时交互式选择）
        /// Tag name (optional, will enter interactive mode if not provided)
        tag_name: Option<String>,
        /// 只删除本地 tag
        /// Delete only local tag
        #[arg(long)]
        local: bool,
        /// 只删除远程 tag
        /// Delete only remote tag
        #[arg(long)]
        remote: bool,
        /// 删除匹配模式的 tag
        /// Delete tags matching pattern
        #[arg(long, short = 'p')]
        pattern: Option<String>,
        #[command(flatten)]
        dry_run: DryRunArgs,
        #[command(flatten)]
        force: ForceArgs,
    },
}
