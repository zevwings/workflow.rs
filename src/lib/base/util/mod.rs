//! Util 模块
//!
//! 本模块提供了各种工具函数和实用工具，包括：
//! - 日志输出（带颜色的日志宏）
//! - 字符串处理（敏感值隐藏）
//! - 浏览器和剪贴板操作
//! - 文件解压和校验和验证
//! - 重试机制
//!
//! ## 模块结构
//!
//! - `logger` - 日志输出（`Logger`）
//! - `string` - 字符串处理工具
//! - `format` - 格式化工具（文件大小格式化等）
//! - `platform` - 平台检测工具（操作系统和架构检测）
//! - `browser` - 浏览器操作（`Browser`）
//! - `clipboard` - 剪贴板操作（`Clipboard`）
//! - `unzip` - 解压工具（tar.gz 文件解压）
//! - `checksum` - 校验和工具（SHA256 计算和验证）
//!
//! 注意：以下模块已迁移到独立的目录：
//! - `lib/completion` - Completion 管理
//! - `lib/rollback` - 回滚工具
//! - `lib/uninstall` - 卸载工具
//! - `lib/proxy/env` - 代理环境变量管理（仅用于代理功能）

pub mod browser;
pub mod checksum;
pub mod clipboard;
pub mod dialog;
pub mod format;
pub mod logger;
pub mod platform;
pub mod string;
pub mod unzip;

// 重新导出 Logger 和 LogLevel
pub use logger::{LogLevel, Logger};

// 重新导出 string 模块的函数，保持向后兼容
pub use string::mask_sensitive_value;

// 重新导出 format 模块的函数
pub use format::format_size;

// 重新导出 platform 模块的函数
pub use platform::detect_release_platform;

// 重新导出 browser 和 clipboard
pub use browser::Browser;
pub use clipboard::Clipboard;

// 重新导出 dialog
pub use dialog::{ConfirmDialog, InputDialog, MultiSelectDialog, SelectDialog};

// 重新导出 unzip
pub use unzip::Unzip;

// 重新导出 checksum
pub use checksum::Checksum;
