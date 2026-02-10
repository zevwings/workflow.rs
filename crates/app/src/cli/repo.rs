//! Repository management subcommands
//!
//! 仓库管理子命令结构定义

use clap::Subcommand;

/// 仓库管理子命令
#[derive(Subcommand)]
pub enum RepoCommand {
    /// 交互式初始化仓库配置
    Setup,
    /// 查看仓库状态
    Status,
    /// 验证仓库配置
    Check,
}
