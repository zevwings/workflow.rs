//! Shell 检测与管理工具
//!
//! 本模块提供了 Shell 相关的检测和管理功能，包括：
//! - 检测当前 shell 类型（zsh、bash、fish、powershell、elvish）
//! - 重新加载 shell 配置
//!
//! 注意：shell 配置文件路径和 completion 目录路径的管理已迁移到 `settings::paths` 模块：
//! - `Paths::config_file()` - 获取 shell 配置文件路径
//! - `Paths::completion_dir()` - 获取 completion 目录路径

mod detect;
mod reload;

pub use detect::Detect;
pub use reload::Reload;
