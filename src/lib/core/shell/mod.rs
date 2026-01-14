//! Shell 检测与管理工具
//!
//! 本模块提供了 Shell 相关的检测和管理功能，包括：
//! - 检测当前 shell 类型（zsh、bash、fish、powershell、elvish）
//! - 重新加载 shell 配置
//! - Shell 配置文件管理（环境变量、source 语句等）

pub(crate) mod block;
pub mod detect;
pub mod env;
pub(crate) mod file;
pub mod paths;
pub mod reload;
pub mod source;
