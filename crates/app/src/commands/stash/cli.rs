//! Stash 子命令定义
//!
//! 定义 stash 相关的子命令和参数。

use clap::Subcommand;

/// Stash 子命令
#[derive(Subcommand)]
pub enum StashSubcommand {
    /// 保存当前更改到 stash
    Push,
    /// 应用最新的 stash 并删除
    Pop,
    /// 应用 stash（不删除）
    Apply,
    /// 删除 stash 条目
    Drop,
    /// 列出所有 stash 条目
    List,
}
