//! 日志配置提供者接口
//!
//! 定义日志配置的抽象接口，实现依赖倒置原则。
//! Logger 模块定义此接口，由其他模块（如 infra）实现。

use crate::LogLevel;
use color_eyre::Result;
use std::path::PathBuf;

/// 日志配置提供者 trait
///
/// 提供日志相关的配置信息，包括日志级别、格式、控制台输出设置和日志目录路径。
/// 通过此 trait，Logger 模块可以独立于具体的配置实现（如 Settings）。
///
/// # 实现者
///
/// 此 trait 应该由基础设施层（如 `infra::adapters`）实现，将不同的配置源
/// （如 Settings、环境变量等）适配为统一的接口。
pub trait LogConfigProvider {
    /// 获取日志级别
    ///
    /// # 返回
    ///
    /// 返回配置的日志级别，如果未配置则返回 `None`。
    fn get_log_level(&self) -> Option<LogLevel>;

    /// 获取日志格式
    ///
    /// # 返回
    ///
    /// 返回日志格式字符串（如 "json" 或 "text"），如果未配置则返回 `None`。
    fn get_log_format(&self) -> Option<String>;

    /// 是否启用控制台输出
    ///
    /// # 返回
    ///
    /// 返回 `true` 表示启用控制台输出，`false` 表示不启用。
    /// 如果未配置，默认返回 `false`。
    fn get_enable_console(&self) -> bool;

    /// 获取日志目录路径
    ///
    /// # 返回
    ///
    /// 返回日志目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法获取日志目录，返回相应的错误信息。
    fn get_logs_dir(&self) -> Result<PathBuf>;
}
