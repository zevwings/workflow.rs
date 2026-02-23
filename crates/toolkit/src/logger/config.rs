//! 日志配置结构体
//!
//! 定义日志配置的数据结构。

use std::path::PathBuf;

/// 日志配置
///
/// 提供日志相关的配置信息，包括日志级别、格式、控制台输出设置和日志目录路径。
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// 日志级别（如 "off", "error", "warn", "info", "debug"）
    /// 如果为 `None`，等同于 "off"
    pub level: Option<String>,

    /// 日志格式（"json" 或 "text"）
    /// 如果为 `None`，使用文本格式
    pub format: Option<String>,

    /// 是否启用控制台输出
    pub enable_console: bool,

    /// 日志目录路径
    pub logs_dir: PathBuf,
}

impl LoggerConfig {
    /// 创建新的日志配置
    pub fn new(
        level: Option<String>,
        format: Option<String>,
        enable_console: bool,
        logs_dir: PathBuf,
    ) -> Self {
        Self {
            level,
            format,
            enable_console,
            logs_dir,
        }
    }
}
