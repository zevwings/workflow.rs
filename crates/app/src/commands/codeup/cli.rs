//! Codeup 配置管理子命令
//!
//! Codeup 配置管理子命令结构定义

use clap::Subcommand;

/// Codeup 配置管理子命令
#[derive(Subcommand)]
pub enum CodeupCommand {
    /// 检查 Codeup 配置（显示项目 ID、验证状态）
    Check,
    /// 设置 Codeup 配置（交互式配置项目 ID、CSRF Token、Cookie）
    Setup,
}
