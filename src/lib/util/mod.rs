//! Util 模块
//!
//! 本模块提供了各种工具函数和实用工具，包括：
//! - 基础工具（字符串、文件、目录、平台检测等）
//! - 格式化工具（显示格式化、消息格式化）
//! - 并发工具（并发任务执行）
//! - 常量定义（错误消息、Git 常量、网络常量等）
//! - 浏览器和剪贴板操作
//! - 文件解压和校验和验证
//!
//! ## 模块结构
//!
//! - `string` - 字符串处理工具
//! - `platform` - 平台检测工具（操作系统和架构检测）
//! - `browser` - 浏览器操作（`Browser`）
//! - `clipboard` - 剪贴板操作（`Clipboard`）
//! - `unzip` - 解压工具（tar.gz 文件解压）
//! - `checksum` - 校验和工具（SHA256 计算和验证）
//! - `concurrent` - 并发任务执行工具
//! - `format` - 格式化工具（消息格式化、显示格式化）
//!
//! 注意：以下模块已迁移到独立的目录：
//! - `lib/constants` - 常量定义（错误消息、Git 常量、网络常量等）
//! - `lib/base/logger` - 日志相关功能（`LogLevel`、`Logger`、`colors`）
//! - `lib/base/interactive/output` - 进度指示器（`Progress`）
//! - `lib/base/interactive` - 交互式功能（包括 `spinner`、`dialog`）
//!   - `lib/base/interactive/dialog` - 交互式对话框（`InputDialog`、`SelectDialog`、`MultiSelectDialog`、`ConfirmDialog`、`FormBuilder`）
//! - `lib/base/interactive/output/table` - 表格输出工具（`TableBuilder`、`TableStyle`）
//! - `lib/completion` - Completion 管理
//! - `lib/rollback` - 回滚工具
//! - `lib/uninstall` - 卸载工具
//! - `lib/proxy/env` - 代理环境变量管理（仅用于代理功能）

pub mod browser;
pub mod checksum;
pub mod clipboard;
pub mod concurrent;
pub mod date;
pub mod directory;
pub mod file;
pub mod format;
pub mod path;
pub mod platform;
pub mod string;
pub mod unzip;

// 重新导出 string 模块的函数，保持向后兼容
pub use string::mask_sensitive_value;

// 重新导出 platform 模块的结构体和函数
pub use platform::{detect_release_platform, Platform};

// 重新导出 browser 和 clipboard
pub use browser::Browser;
pub use clipboard::Clipboard;

// 重新导出 unzip
pub use unzip::Unzip;

// 重新导出 checksum
pub use checksum::Checksum;

// 重新导出 date
pub use date::{
    format_document_timestamp, format_last_updated, format_last_updated_with_time, DateFormat,
    Timezone,
};

// 重新导出 directory
pub use directory::DirectoryWalker;

// 重新导出 file
pub use file::{FileReader, FileWriter};

// 重新导出 path
pub use path::PathAccess;

// 重新导出 concurrent
pub use concurrent::{ConcurrentExecutor, TaskResult};

// 重新导出格式化器
pub use format::{DisplayFormatter, MessageFormatter};

// 注意：颜色格式化函数已移除，请使用 success!, error!, warning!, info!, debug!, br! 宏代替
