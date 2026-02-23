//! 卸载命令模块
//!
//! 提供卸载 Workflow CLI 的功能，包括：
//! - 删除二进制文件
//! - 删除 shell completion 脚本
//! - 删除配置文件
//!
//! ## 功能
//!
//! - 支持交互式确认
//! - 支持 Unix 和 Windows 平台
//! - 自动使用 sudo（Unix）处理权限问题

mod command;

pub use command::UninstallCommand;
