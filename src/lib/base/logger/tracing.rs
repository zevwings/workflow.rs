//! Tracing 封装模块
//!
//! 本模块提供了对 tracing 库的封装，用于 lib 层的结构化日志记录。
//! 通过封装，如果未来需要替换为其他日志库，只需要修改本模块即可。
//!
//! ## 设计原则
//!
//! 1. **职责分离**：
//!    - Lib 层使用 `trace_*!` 宏进行结构化日志记录（不直接输出到控制台）
//!    - Commands 层使用 `log_*!` 宏进行用户友好的控制台输出
//!
//! 2. **默认行为**：
//!    - 默认情况下，tracing 不输出到控制台（通过配置控制）
//!    - 可以通过环境变量 `RUST_LOG` 启用调试输出到 stderr
//!
//! 3. **可替换性**：
//!    - 所有 lib 层代码使用 `trace_*!` 宏，而不是直接使用 `tracing::*`
//!    - 如果未来需要替换日志库，只需要修改本模块的实现
//!
//! ## 使用示例
//!
//! ```rust
//! use workflow::trace_debug;
//!
//! trace_debug!("Processing data: {}", data);
//! trace_info!("Operation completed");
//! trace_warn!("Retrying operation");
//! trace_error!("Operation failed: {}", error);
//! ```
//!
//! ## 初始化
//!
//! ```rust
//! use workflow::Tracer;
//!
//! // 从配置文件读取日志级别并初始化
//! Tracer::init();
//! ```

use crate::base::settings::paths::Paths;
use crate::base::settings::Settings;
use crate::base::LogLevel;
use anyhow::Context;
use chrono::Local;
use std::fs;
use std::fs::OpenOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Tracing 封装结构体
///
/// 提供统一的 tracing 接口，内部使用 tracing crate。
/// 如果未来需要替换为其他日志库，只需要修改本结构体的实现。
pub struct Tracer;

impl Tracer {
    /// 初始化 tracing subscriber（从配置读取日志级别）
    ///
    /// 根据配置的日志级别决定是否输出到文件或完全丢弃。
    /// 如果日志级别为 "off"，则输出到 sink（/dev/null）。
    /// 否则，输出到日志文件（`~/.workflow/logs/tracing/workflow-YYYY-MM-DD.log`）。
    ///
    /// 如果启用了 `enable_trace_console` 配置（为 `true`），tracing 日志会同时输出到文件和控制台（stderr）。
    /// 如果配置文件中不存在此字段（为 `None`），默认为 `false`（只输出到文件）。
    ///
    /// 日志级别从 `~/.workflow/config/workflow.toml` 配置文件中的 `log.level` 字段读取。
    /// 如果配置文件中未设置，则默认使用 "off"（不输出）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::Tracer;
    ///
    /// // 从配置文件读取并初始化
    /// Tracer::init();
    /// ```
    pub fn init() {
        let settings = Settings::get();

        // 从配置文件读取日志级别并解析为 LogLevel
        let log_level = settings
            .log
            .level
            .as_deref()
            .and_then(|s| s.parse::<LogLevel>().ok())
            .unwrap_or(LogLevel::None);

        // 将 LogLevel 转换为 tracing 格式字符串
        let tracing_filter = log_level.as_str();

        // 根据配置决定输出目标
        if log_level != LogLevel::None {
            // 决定是否同时输出到控制台
            // 如果配置文件中设置了 enable_trace_console 为 true，则启用；否则默认为 false
            let enable_console = settings.log.enable_trace_console.unwrap_or(false);

            // 总是尝试输出到文件
            if let Ok(file_path) = Self::get_log_file_path() {
                if let Ok(file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)
                {
                    // 构建 registry，先添加 EnvFilter
                    let registry =
                        tracing_subscriber::registry().with(EnvFilter::new(tracing_filter));

                    // 添加文件 layer
                    let registry =
                        registry.with(tracing_subscriber::fmt::layer().with_writer(file));

                    // 如果启用了控制台输出，同时添加 console layer
                    if enable_console {
                        let _ = registry
                            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
                            .try_init();
                    } else {
                        let _ = registry.try_init();
                    }
                    return;
                }
            }

            // 如果文件创建失败，回退到 stderr
            let _ = tracing_subscriber::registry()
                .with(EnvFilter::new(tracing_filter))
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
                .try_init();
        } else {
            // 否则输出到 /dev/null（完全丢弃）
            let _ = tracing_subscriber::registry()
                .with(EnvFilter::new(tracing_filter))
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::sink))
                .try_init();
        }
    }

    /// 获取日志文件路径
    ///
    /// 返回格式：`~/.workflow/logs/tracing/workflow-YYYY-MM-DD.log`
    ///
    /// 日志文件存储在应用配置目录下，强制本地存储（不使用 iCloud 同步）。
    fn get_log_file_path() -> anyhow::Result<std::path::PathBuf> {
        // 获取日志目录（~/.workflow/logs/），强制本地存储
        let logs_dir = Paths::logs_dir().context("Failed to get logs directory")?;

        // 创建 tracing 子目录
        let tracing_dir = logs_dir.join("tracing");
        fs::create_dir_all(&tracing_dir).with_context(|| {
            format!("Failed to create tracing log directory: {:?}", tracing_dir)
        })?;

        // 生成带日期的日志文件名
        let date = Local::now().format("%Y-%m-%d");
        let log_file = tracing_dir.join(format!("workflow-{}.log", date));

        Ok(log_file)
    }
    /// 记录调试级别的日志
    ///
    /// 注意：这里直接使用 tracing crate。
    /// 如果未来需要替换为其他日志库，只需要修改这里的实现即可。
    #[inline]
    pub fn debug(message: &str) {
        // 直接使用 tracing，如果未来替换日志库，只需要修改这里
        tracing::debug!("{}", message);
    }

    /// 记录信息级别的日志
    #[inline]
    pub fn info(message: &str) {
        tracing::info!("{}", message);
    }

    /// 记录警告级别的日志
    #[inline]
    pub fn warn(message: &str) {
        tracing::warn!("{}", message);
    }

    /// 记录错误级别的日志
    #[inline]
    pub fn error(message: &str) {
        tracing::error!("{}", message);
    }

    /// 记录带格式化的调试级别日志
    #[inline]
    pub fn debug_fmt(args: std::fmt::Arguments) {
        tracing::debug!("{}", args);
    }

    /// 记录带格式化的信息级别日志
    #[inline]
    pub fn info_fmt(args: std::fmt::Arguments) {
        tracing::info!("{}", args);
    }

    /// 记录带格式化的警告级别日志
    #[inline]
    pub fn warn_fmt(args: std::fmt::Arguments) {
        tracing::warn!("{}", args);
    }

    /// 记录带格式化的错误级别日志
    #[inline]
    pub fn error_fmt(args: std::fmt::Arguments) {
        tracing::error!("{}", args);
    }
}

/// 格式化并记录调试级别的日志
///
/// 用于 lib 层的内部调试信息记录，不直接输出到控制台。
/// 默认情况下不输出，可以通过环境变量 `RUST_LOG=debug` 启用。
///
/// # Examples
///
/// ```
/// use workflow::trace_debug;
///
/// trace_debug!("Processing data");
/// trace_debug!("Found {} attachment(s)", count);
/// ```
#[macro_export]
macro_rules! trace_debug {
    ($($arg:tt)*) => {
        $crate::base::logger::tracing::Tracer::debug_fmt(format_args!($($arg)*));
    };
}

/// 格式化并记录信息级别的日志
///
/// 用于 lib 层的内部操作记录，不直接输出到控制台。
/// 默认情况下不输出，可以通过环境变量 `RUST_LOG=info` 启用。
///
/// # Examples
///
/// ```
/// use workflow::trace_info;
///
/// trace_info!("Starting download");
/// trace_info!("Downloaded {} files", count);
/// ```
#[macro_export]
macro_rules! trace_info {
    ($($arg:tt)*) => {
        $crate::base::logger::tracing::Tracer::info_fmt(format_args!($($arg)*));
    };
}

/// 格式化并记录警告级别的日志
///
/// 用于 lib 层的内部警告记录，不直接输出到控制台。
/// 默认情况下不输出，可以通过环境变量 `RUST_LOG=warn` 启用。
///
/// # Examples
///
/// ```
/// use workflow::trace_warn;
///
/// trace_warn!("Retrying operation");
/// trace_warn!("Failed to download {}: {}", filename, error);
/// ```
#[macro_export]
macro_rules! trace_warn {
    ($($arg:tt)*) => {
        $crate::base::logger::tracing::Tracer::warn_fmt(format_args!($($arg)*));
    };
}

/// 格式化并记录错误级别的日志
///
/// 用于 lib 层的内部错误记录，不直接输出到控制台。
/// 默认情况下不输出，可以通过环境变量 `RUST_LOG=error` 启用。
///
/// # Examples
///
/// ```
/// use workflow::trace_error;
///
/// trace_error!("Operation failed");
/// trace_error!("Error: {} - {}", code, message);
/// ```
#[macro_export]
macro_rules! trace_error {
    ($($arg:tt)*) => {
        $crate::base::logger::tracing::Tracer::error_fmt(format_args!($($arg)*));
    };
}
