//! Tag 管理子命令
//!
//! Tag 管理子命令结构定义

use clap::Subcommand;

use super::super::args::{DryRunArgs, ForceArgs};

/// Tag 管理子命令
///
/// 用于管理 Git Tag。
#[derive(Subcommand)]
pub enum TagSubcommand {
    /// 创建 Tag（可选附带消息创建 annotated tag）
    Create {
        /// Tag 名称
        tag_name: String,
        /// 目标 commit（可选，默认为 HEAD）
        #[arg(long, short = 't')]
        target: Option<String>,
        /// Tag 消息（可选，提供则创建 annotated tag）
        #[arg(long, short = 'm')]
        message: Option<String>,
        /// 只创建本地 tag（不推送到远程）
        #[arg(long)]
        local: bool,
        /// 强制创建（如果 tag 已存在则覆盖）
        #[command(flatten)]
        force: ForceArgs,
    },
    /// 删除本地和/或远程 Tag
    Remove {
        /// Tag 名称（可选，不提供时交互式选择）
        tag_name: Option<String>,
        /// 只删除本地 tag
        #[arg(long)]
        local: bool,
        /// 只删除远程 tag
        #[arg(long)]
        remote: bool,
        /// 删除匹配模式的 tag
        #[arg(long, short = 'p')]
        pattern: Option<String>,
        #[command(flatten)]
        dry_run: DryRunArgs,
        #[command(flatten)]
        force: ForceArgs,
    },
}
