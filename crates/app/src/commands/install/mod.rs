//! 安装命令模块
//!
//! 提供安装二进制文件和 shell completion 的功能。
//!
//! ## 功能
//!
//! - 安装二进制文件到系统目录（通常是 /usr/local/bin）
//! - 安装 shell completion 脚本
//! - 支持 Unix 和 Windows 平台

mod command;

pub use command::InstallCommand;
