//! 日志配置提供者接口
//!
//! 定义日志配置的抽象接口，实现依赖倒置原则。

use super::level::LogLevel;
use color_eyre::Result;
use std::path::PathBuf;

/// 日志配置提供者 trait
///
/// 提供日志相关的配置信息，包括日志级别、格式、控制台输出设置和日志目录路径。
/// 通过此 trait，Logger 模块可以独立于具体的配置实现。
pub trait ConfigProvider {
    /// 获取日志级别
    fn log_level(&self) -> Option<LogLevel>;

    /// 获取日志格式
    ///
    /// 返回日志格式字符串（如 "json" 或 "text"），如果未配置则返回 `None`。
    fn log_format(&self) -> Option<String>;

    /// 是否启用控制台输出
    ///
    /// 返回 `true` 表示启用控制台输出，`false` 表示不启用。
    fn enable_console(&self) -> bool;

    /// 获取日志目录路径
    ///
    /// # 错误
    ///
    /// 如果无法获取日志目录，返回相应的错误信息。
    fn logs_dir(&self) -> Result<PathBuf>;
}
