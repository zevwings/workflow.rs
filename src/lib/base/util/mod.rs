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
//! 本模块按功能领域分为三个子模块：
//! - `fs` - 文件系统操作（文件、路径、目录、解压）
//! - `system` - 系统交互（平台检测、浏览器、剪贴板）
//! - `data` - 数据处理（字符串、日期时间、校验和）
//!
//! 注意：以下模块已迁移到独立的目录：
//! - `lib/base/logger` - 日志相关功能（`LogLevel`、`Logger`、`Tracer`、`colors`）
//! - `lib/base/indicator` - 进度指示器（`Spinner`、`Progress`）
//! - `lib/base/dialog` - 交互式对话框（`InputDialog`、`SelectDialog`、`MultiSelectDialog`、`ConfirmDialog`）
//! - `lib/base/table` - 表格输出工具（`TableBuilder`、`TableStyle`）
//! - `lib/completion` - Completion 管理
//! - `lib/rollback` - 回滚工具
//! - `lib/uninstall` - 卸载工具
//! - `lib/proxy/env` - 代理环境变量管理（仅用于代理功能）

pub mod data;
pub mod fs;
pub mod system;

// 向后兼容：重新导出子模块，使 `util::file::FileReader` 等路径仍然可用
pub mod file {
    pub use super::fs::file::*;
}
pub mod path {
    pub use super::fs::path::*;
}
pub mod directory {
    pub use super::fs::directory::*;
}
pub mod unzip {
    pub use super::fs::unzip::*;
}
pub mod platform {
    pub use super::system::platform::*;
}
pub mod browser {
    pub use super::system::browser::*;
}
pub mod clipboard {
    pub use super::system::clipboard::*;
}
pub mod string {
    pub use super::data::string::*;
}
pub mod date {
    pub use super::data::date::*;
}
pub mod checksum {
    pub use super::data::checksum::*;
}

// 重新导出所有公共 API，保持向后兼容
pub use data::{
    format_document_timestamp, format_last_updated, format_last_updated_with_time,
    mask_sensitive_value, Checksum, DateFormat, Timezone,
};
pub use fs::{DirectoryWalker, FileReader, FileWriter, PathAccess, Unzip};
pub use system::{Browser, Clipboard, Platform};

// 重新导出 colors 函数（从 logger::console 模块，保持向后兼容）
pub use crate::base::logger::console::{
    debug, error, info, separator, separator_with_text, success, warning,
};
