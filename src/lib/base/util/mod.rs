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
//! 本模块按功能领域分为一个子模块：
//! - `data` - 数据处理（字符串、日期时间、校验和）
//!
//! 注意：以下模块已迁移到独立的目录：
//! - `lib/base/system` - 系统交互（平台检测、浏览器、剪贴板）
//! - `lib/base/fs` - 文件系统操作（文件、路径、目录）
//! - `lib/base/zip` - 解压工具（tar.gz 和 zip 文件解压）
//! - `lib/base/logger` - 日志相关功能（`LogLevel`、`Logger`、`Tracer`、`colors`）
//! - `lib/base/indicator` - 进度指示器（`Spinner`、`Progress`）
//! - `lib/base/dialog` - 交互式对话框（`InputDialog`、`SelectDialog`、`MultiSelectDialog`、`ConfirmDialog`）
//! - `lib/base/table` - 表格输出工具（`TableBuilder`、`TableStyle`）
//! - `lib/completion` - Completion 管理
//! - `lib/rollback` - 回滚工具
//! - `lib/uninstall` - 卸载工具
//! - `lib/proxy/env` - 代理环境变量管理（仅用于代理功能）

pub mod data;

// 向后兼容：重新导出子模块，使 `util::file::FileReader` 等路径仍然可用
pub mod file {
    pub use crate::base::fs::file::*;
}
pub mod path {
    pub use crate::base::fs::path::*;
}
pub mod directory {
    pub use crate::base::fs::directory::*;
}
pub mod unzip {
    pub use crate::base::zip::*;
}
pub mod platform {
    pub use crate::base::system::platform::*;
}
pub mod browser {
    pub use crate::base::system::browser::*;
}
pub mod clipboard {
    pub use crate::base::system::clipboard::*;
}
pub mod string {
    pub use crate::base::format::sensitive::*;
}
pub mod date {
    pub use crate::base::format::date::*;
}
pub mod checksum {
    pub use crate::base::checksum::*;
}

// 重新导出所有公共 API，保持向后兼容
pub use crate::base::checksum::Checksum;
pub use crate::base::format::{
    format_document_timestamp, format_last_updated, format_last_updated_with_time, DateFormat,
    Sensitive, Timezone,
};
pub use crate::base::fs::{DirectoryWalker, FileReader, FileWriter, PathAccess};
pub use crate::base::system::{Browser, Clipboard, Platform};
pub use crate::base::zip::Unzip;

// 重新导出 colors 函数（从 logger::console 模块，保持向后兼容）
pub use crate::base::logger::console::{
    debug, error, info, separator, separator_with_text, success, warning,
};
