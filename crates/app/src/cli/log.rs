//! Log management subcommands
//!
//! 日志管理子命令结构定义

use clap::Subcommand;

/// 日志管理子命令
#[derive(Subcommand)]
pub enum LogCommand {
    /// 设置日志级别（交互式选择：none/error/warn/info/debug）
    Setup,
    /// 检查当前日志级别（显示当前、默认和配置文件中的级别）
    Check,
}
