//! GitHub account management subcommands
//!
//! GitHub 账号管理子命令结构定义

use clap::Subcommand;

/// GitHub 账号管理子命令
#[derive(Subcommand)]
pub enum GithubCommand {
    /// 列出所有 GitHub 账号
    Check,
    /// 设置 GitHub 账号（添加/切换/更新/删除账号）
    Setup,
}
