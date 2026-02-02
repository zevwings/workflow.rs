//! CNB account management subcommands
//!
//! CNB 账号管理子命令结构定义

use clap::Subcommand;

/// CNB 账号管理子命令
#[derive(Subcommand)]
pub enum CNBCommand {
    /// 列出所有 CNB 账号
    Check,
    /// 设置 CNB 账号（添加/切换/更新/删除账号）
    Setup,
}
