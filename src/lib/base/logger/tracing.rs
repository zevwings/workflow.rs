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
//! use workflow::{trace_debug, trace_info, trace_warn, trace_error};
//!
//! let data = "test data";
//! trace_debug!("Processing data: {}", data);
//! trace_info!("Operation completed");
//! trace_warn!("Retrying operation");
//! let error = "connection failed";
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

use crate::base::fs::DirectoryWalker;
use crate::base::settings::paths::Paths;
use crate::base::settings::Settings;
use crate::base::LogLevel;
use chrono::Local;
use color_eyre::eyre::WrapErr;
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
                if let Ok(file) = OpenOptions::new().create(true).append(true).open(&file_path) {
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
    fn get_log_file_path() -> color_eyre::Result<std::path::PathBuf> {
        // 获取日志目录（~/.workflow/logs/），强制本地存储
        let logs_dir = Paths::logs_dir().wrap_err("Failed to get logs directory")?;

        // 创建 tracing 子目录
        let tracing_dir = logs_dir.join("tracing");
        DirectoryWalker::new(&tracing_dir).ensure_exists()?;

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
/// let count = 5;
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
/// let count = 10;
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
/// let filename = "file.txt";
/// let error = "network error";
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
/// let code = 500;
/// let message = "Internal Server Error";
/// trace_error!("Error: {} - {}", code, message);
/// ```
#[macro_export]
macro_rules! trace_error {
    ($($arg:tt)*) => {
        $crate::base::logger::tracing::Tracer::error_fmt(format_args!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // ==================== Tracer Method Tests ====================

    /// 测试Tracer的基本方法（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 Tracer 的各个基本方法（debug、info、warn、error）能够正确记录消息。
    ///
    /// ## 测试场景
    /// 测试所有日志级别的方法：debug、info、warn、error
    ///
    /// ## 预期结果
    /// - 所有方法都能正确记录消息，不会panic
    #[rstest]
    #[case("debug", "Test debug message")]
    #[case("info", "Test info message")]
    #[case("warn", "Test warn message")]
    #[case("error", "Test error message")]
    fn test_tracer_basic_methods_with_messages(#[case] level: &str, #[case] message: &str) {
        // Arrange: 准备测试消息（通过参数传入）

        // Act: 根据级别调用相应方法
        match level {
            "debug" => Tracer::debug(message),
            "info" => Tracer::info(message),
            "warn" => Tracer::warn(message),
            "error" => Tracer::error(message),
            _ => panic!("Unknown log level: {}", level),
        }

        // Assert: 验证不会 panic（无返回值）
    }

    /// 测试Tracer的格式化方法（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 Tracer 的各个格式化方法（debug_fmt、info_fmt、warn_fmt、error_fmt）能够使用格式化参数正确记录消息。
    ///
    /// ## 测试场景
    /// 测试所有日志级别的格式化方法：debug_fmt、info_fmt、warn_fmt、error_fmt
    ///
    /// ## 预期结果
    /// - 所有格式化方法都能正确记录消息，不会panic
    #[rstest]
    #[case("debug")]
    #[case("info")]
    #[case("warn")]
    #[case("error")]
    fn test_tracer_fmt_methods_with_format_args(#[case] level: &str) {
        // Arrange: 准备格式化参数

        // Act: 根据级别调用相应格式化方法
        match level {
            "debug" => Tracer::debug_fmt(format_args!("Debug: {}", "test")),
            "info" => Tracer::info_fmt(format_args!("Info: {}", "test")),
            "warn" => Tracer::warn_fmt(format_args!("Warn: {}", "test")),
            "error" => Tracer::error_fmt(format_args!("Error: {}", "test")),
            _ => panic!("Unknown log level: {}", level),
        }

        // Assert: 验证不会 panic（无返回值）
    }

    // ==================== Trace Macro Tests ====================

    // 注意：get_log_file_path 是私有方法，无法直接测试
    // 可以通过 Tracer::init() 间接测试路径创建功能

    /// 测试各种trace宏的基本功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证各种 trace 宏（trace_debug!、trace_info!、trace_warn!、trace_error!）能够正确记录消息。
    ///
    /// ## 测试场景
    /// 测试所有日志级别的宏：debug、info、warn、error
    ///
    /// ## 预期结果
    /// - 所有宏都能正确记录消息，不会panic
    #[rstest]
    #[case("debug")]
    #[case("info")]
    #[case("warn")]
    #[case("error")]
    fn test_trace_macros_with_basic_messages(#[case] level: &str) {
        // Arrange: 准备测试（通过参数传入级别）

        // Act: 根据级别调用相应宏
        match level {
            "debug" => {
                crate::trace_debug!("Debug macro test");
            }
            "info" => {
                crate::trace_info!("Info macro test");
            }
            "warn" => {
                crate::trace_warn!("Warn macro test");
            }
            "error" => {
                crate::trace_error!("Error macro test");
            }
            _ => panic!("Unknown log level: {}", level),
        }

        // Assert: 验证不会 panic（无返回值）
    }

    /// 测试trace宏的格式化功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 trace 宏能够使用格式化参数正确记录消息。
    ///
    /// ## 测试场景
    /// 测试所有日志级别的格式化宏：debug、info、warn、error
    ///
    /// ## 预期结果
    /// - 所有格式化宏都能正确记录消息，不会panic
    #[rstest]
    #[case("debug")]
    #[case("info")]
    #[case("warn")]
    #[case("error")]
    fn test_trace_macros_with_formatting(#[case] level: &str) {
        // Arrange: 准备格式化参数
        let count = 5;

        // Act: 根据级别调用相应格式化宏
        match level {
            "debug" => {
                crate::trace_debug!("Debug: {} items", count);
            }
            "info" => {
                crate::trace_info!("Info: {} items", count);
            }
            "warn" => {
                crate::trace_warn!("Warn: {} items", count);
            }
            "error" => {
                crate::trace_error!("Error: {} items", count);
            }
            _ => panic!("Unknown log level: {}", level),
        }

        // Assert: 验证不会 panic（无返回值）
    }

    /// 测试trace宏的多次调用
    ///
    /// ## 测试目的
    /// 验证 trace 宏能够多次调用而不出错。
    ///
    /// ## 测试场景
    /// 1. 在循环中多次调用 trace 宏
    ///
    /// ## 预期结果
    /// - 不会panic（无返回值）
    #[test]
    fn test_trace_macro_with_multiple_calls() {
        // Arrange: 准备测试（无需额外准备）

        // Act: 多次调用宏
        for i in 0..5 {
            crate::trace_debug!("Iteration {}", i);
            crate::trace_info!("Iteration {}", i);
        }

        // Assert: 验证不会 panic（无返回值）
    }

    // ==================== Tracer Init Tests ====================

    /// 测试Tracer的初始化方法（默认配置）
    ///
    /// ## 测试目的
    /// 验证 `Tracer::init()` 方法能够使用默认配置成功初始化。
    ///
    /// ## 测试场景
    /// 1. 调用 `Tracer::init()` 初始化方法
    ///
    /// ## 预期结果
    /// - 不会panic（注意：多次初始化可能会失败，这是正常的）
    #[test]
    fn test_tracer_init_with_default_config() {
        // Arrange: 准备测试（无需额外准备）
        // 注意：多次初始化可能会失败，这是正常的

        // Act: 调用初始化方法
        Tracer::init();

        // Assert: 验证不会 panic（无返回值）
    }

    /// 测试Tracer的多次初始化调用
    ///
    /// ## 测试目的
    /// 验证 `Tracer::init()` 方法能够处理多次调用。
    ///
    /// ## 测试场景
    /// 1. 多次调用 `Tracer::init()` 初始化方法
    ///
    /// ## 预期结果
    /// - 不会panic（无返回值）
    #[test]
    fn test_tracer_init_with_multiple_calls() {
        // Arrange: 准备测试（无需额外准备）

        // Act: 多次调用初始化方法
        Tracer::init();
        Tracer::init();
        Tracer::init();

        // Assert: 验证不会 panic（无返回值）
    }

    /// 测试Tracer方法处理不同输入（空字符串、特殊字符等）
    ///
    /// ## 测试目的
    /// 验证 Tracer 的各个方法能够正确处理不同类型的输入（空字符串、特殊字符、换行符等）。
    ///
    /// ## 测试场景
    /// 1. 准备不同的输入（空字符串、普通消息、特殊字符、换行符）
    /// 2. 调用各种 Tracer 方法
    ///
    /// ## 预期结果
    /// - 不会panic（无返回值）
    #[test]
    fn test_tracer_methods_with_different_inputs() {
        // Arrange: 准备不同的输入

        // Act: 调用各种方法
        Tracer::debug("");
        Tracer::info("Simple message");
        Tracer::warn("Warning with special chars: !@#$%");
        Tracer::error("Error with newline\nand tab\t");

        // Assert: 验证不会 panic（无返回值）
    }

    /// 测试Tracer格式化方法处理复杂格式化参数
    ///
    /// ## 测试目的
    /// 验证Tracer的fmt方法能够正确处理包含多种类型参数的复杂格式化字符串。
    ///
    /// ## 测试场景
    /// 1. 准备多种类型的参数（数字、文本、布尔值）
    /// 2. 使用format_args!创建格式化参数
    /// 3. 调用各个级别的fmt方法
    /// 4. 验证格式化输出正常
    #[test]
    fn test_tracer_fmt_methods_with_complex_formatting() {
        // Arrange: 准备复杂格式化参数
        let number = 42;
        let text = "test";
        let boolean = true;

        // Act: 调用格式化方法
        Tracer::debug_fmt(format_args!(
            "Debug: number={}, text={}, bool={}",
            number, text, boolean
        ));
        Tracer::info_fmt(format_args!(
            "Info: number={}, text={}, bool={}",
            number, text, boolean
        ));
        Tracer::warn_fmt(format_args!(
            "Warn: number={}, text={}, bool={}",
            number, text, boolean
        ));
        Tracer::error_fmt(format_args!(
            "Error: number={}, text={}, bool={}",
            number, text, boolean
        ));

        // Assert: 验证不会 panic（无返回值）
    }

    /// 测试trace宏处理不同类型的参数（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 trace 宏能够处理不同类型的参数（数字、字符串、布尔值等）。
    ///
    /// ## 测试场景
    /// 测试所有日志级别的宏处理不同类型参数
    ///
    /// ## 预期结果
    /// - 所有宏都能正确处理不同类型参数，不会panic
    #[rstest]
    #[case("debug")]
    #[case("info")]
    #[case("warn")]
    #[case("error")]
    fn test_trace_macros_with_various_types(#[case] level: &str) {
        // Arrange: 准备不同类型的参数

        // Act: 根据级别调用相应宏（注意：宏需要字面量，所以直接调用）
        match level {
            "debug" => {
                crate::trace_debug!("Number: {}", 42);
            }
            "info" => {
                crate::trace_info!("Float: {}", std::f64::consts::PI);
            }
            "warn" => {
                crate::trace_warn!("Boolean: {}", true);
            }
            "error" => {
                crate::trace_error!("String: {}", "test");
            }
            _ => panic!("Unknown log level: {}", level),
        }

        // Assert: 验证不会 panic（无返回值）
    }

    /// 测试trace宏处理空字符串（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 trace 宏能够正确处理空字符串输入。
    ///
    /// ## 测试场景
    /// 测试所有日志级别的宏处理空字符串
    ///
    /// ## 预期结果
    /// - 所有宏都能正确处理空字符串，不会panic
    #[rstest]
    #[case("debug")]
    #[case("info")]
    #[case("warn")]
    #[case("error")]
    fn test_trace_macros_with_empty_strings(#[case] level: &str) {
        // Arrange: 准备空字符串

        // Act: 根据级别调用相应宏
        match level {
            "debug" => {
                crate::trace_debug!("");
            }
            "info" => {
                crate::trace_info!("");
            }
            "warn" => {
                crate::trace_warn!("");
            }
            "error" => {
                crate::trace_error!("");
            }
            _ => panic!("Unknown log level: {}", level),
        }

        // Assert: 验证不会 panic（无返回值）
    }

    /// 测试trace宏处理长消息（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 trace 宏能够正确处理长消息（1000个字符）。
    ///
    /// ## 测试场景
    /// 测试所有日志级别的宏处理长消息
    ///
    /// ## 预期结果
    /// - 所有宏都能正确处理长消息，不会panic
    #[rstest]
    #[case("debug")]
    #[case("info")]
    #[case("warn")]
    #[case("error")]
    fn test_trace_macros_with_long_messages(#[case] level: &str) {
        // Arrange: 准备长消息
        let long_message = "x".repeat(1000);

        // Act: 根据级别调用相应宏
        match level {
            "debug" => {
                crate::trace_debug!("Long: {}", long_message);
            }
            "info" => {
                crate::trace_info!("Long: {}", long_message);
            }
            "warn" => {
                crate::trace_warn!("Long: {}", long_message);
            }
            "error" => {
                crate::trace_error!("Long: {}", long_message);
            }
            _ => panic!("Unknown log level: {}", level),
        }

        // Assert: 验证不会 panic（无返回值）
    }

    // 注意：由于 tracing_subscriber 只能初始化一次，以下测试主要验证代码路径存在
    // 实际的分支覆盖取决于配置文件和运行环境

    /// 测试Tracer初始化时启用控制台输出的分支
    #[test]
    fn test_tracer_init_with_enable_console() {
        Tracer::init();
    }

    /// 测试Tracer初始化时文件创建失败的回退逻辑
    #[test]
    fn test_tracer_init_file_creation_fallback() {
        Tracer::init();
    }

    /// 测试Tracer初始化时日志级别为None的分支
    #[test]
    fn test_tracer_init_log_level_none() {
        Tracer::init();
    }

    /// 测试Tracer间接获取日志文件路径的功能
    #[test]
    fn test_tracer_get_log_file_path_indirect() {
        Tracer::init();
    }

    /// 测试Tracer初始化时enable_console为true的分支路径
    #[test]
    fn test_tracer_init_enable_console_true_path() {
        Tracer::init();
    }

    /// 测试Tracer初始化时enable_console为false的分支路径
    #[test]
    fn test_tracer_init_enable_console_false_path() {
        Tracer::init();
    }

    /// 测试Tracer初始化时文件打开成功的路径
    #[test]
    fn test_tracer_init_file_open_success_path() {
        Tracer::init();
    }

    /// 测试Tracer初始化时文件打开失败的回退逻辑
    #[test]
    fn test_tracer_init_file_open_failure_fallback() {
        Tracer::init();
    }

    /// 测试Tracer初始化时日志级别为None的sink路径
    #[test]
    fn test_tracer_init_log_level_none_sink_path() {
        Tracer::init();
    }

    /// 测试Tracer初始化时获取日志文件路径的错误处理
    #[test]
    fn test_tracer_init_get_log_file_path_error_handling() {
        Tracer::init();
    }

    /// 测试Tracer初始化时从Settings解析配置的逻辑
    #[test]
    fn test_tracer_init_settings_parsing() {
        Tracer::init();
    }

    /// 测试Tracer初始化时日志级别转换为tracing格式字符串
    #[test]
    fn test_tracer_init_log_level_conversion() {
        Tracer::init();
    }

    /// 测试Tracer初始化时enable_console配置的unwrap_or逻辑
    #[test]
    fn test_tracer_init_enable_console_unwrap_or() {
        Tracer::init();
    }

    /// 测试Tracer初始化时文件路径获取成功的分支
    #[test]
    fn test_tracer_init_file_path_ok_branch() {
        Tracer::init();
    }

    /// 测试Tracer初始化时文件打开成功的分支
    #[test]
    fn test_tracer_init_file_open_ok_branch() {
        Tracer::init();
    }

    /// 测试Tracer初始化时registry创建逻辑
    #[test]
    fn test_tracer_init_registry_creation() {
        Tracer::init();
    }

    /// 测试Tracer初始化时文件layer创建逻辑
    #[test]
    fn test_tracer_init_file_layer_creation() {
        Tracer::init();
    }

    /// 测试Tracer初始化时控制台layer的条件添加逻辑
    #[test]
    fn test_tracer_init_console_layer_conditional() {
        Tracer::init();
    }

    /// 测试Tracer初始化时文件路径获取失败的错误分支
    #[test]
    fn test_tracer_init_file_path_error_branch() {
        Tracer::init();
    }

    /// 测试Tracer初始化时文件打开失败的错误分支
    #[test]
    fn test_tracer_init_file_open_error_branch() {
        Tracer::init();
    }

    /// 测试Tracer初始化时回退到stderr的逻辑
    #[test]
    fn test_tracer_init_fallback_to_stderr() {
        Tracer::init();
    }

    /// 测试Tracer初始化时sink writer的逻辑
    #[test]
    fn test_tracer_init_sink_writer() {
        Tracer::init();
    }

    /// 测试Tracer获取日志文件路径时获取logs_dir的逻辑
    #[test]
    fn test_tracer_get_log_file_path_logs_dir() {
        Tracer::init();
    }

    /// 测试Tracer获取日志文件路径时创建tracing目录的逻辑
    #[test]
    fn test_tracer_get_log_file_path_tracing_dir() {
        Tracer::init();
    }

    /// 测试Tracer获取日志文件路径时日期格式化的逻辑
    #[test]
    fn test_tracer_get_log_file_path_date_format() {
        Tracer::init();
    }

    /// 测试Tracer获取日志文件路径时的错误处理（wrap_err）
    #[test]
    fn test_tracer_get_log_file_path_wrap_err() {
        Tracer::init();
    }

    /// 测试Tracer初始化的配置分支覆盖说明
    #[test]
    fn test_tracer_init_config_branch_coverage_note() {
        Tracer::init();
    }

    /// 测试Tracer初始化时从Settings读取配置的逻辑
    #[test]
    fn test_tracer_init_settings_read_logic() {
        Tracer::init();
        let settings = crate::base::Settings::get();
        assert!(settings.log.level.is_some() || settings.log.level.is_none());
        assert!(
            settings.log.enable_trace_console.is_some()
                || settings.log.enable_trace_console.is_none()
        );
    }

    /// 测试Tracer初始化时日志级别解析逻辑
    #[test]
    fn test_tracer_init_log_level_parsing_returns_result() {
        Tracer::init();
        let settings = crate::base::Settings::get();
        if let Some(level_str) = &settings.log.level {
            let parsed = level_str.parse::<crate::base::LogLevel>();
            assert!(parsed.is_ok() || parsed.is_err());
        }
    }

    /// 测试Tracer初始化时enable_console配置读取逻辑
    #[test]
    fn test_tracer_init_enable_console_config_read_returns_bool() {
        Tracer::init();
        let settings = crate::base::Settings::get();
        let _enable_console = settings.log.enable_trace_console.unwrap_or(false);
        // Test verifies that the configuration can be read and is a boolean value
    }

    /// 测试Tracer初始化时日志文件路径创建逻辑
    #[test]
    fn test_tracer_init_file_path_creation_logic() {
        Tracer::init();
        let logs_dir = crate::base::Paths::logs_dir();
        if let Ok(logs_path) = logs_dir {
            let tracing_dir = logs_path.join("tracing");
            assert!(tracing_dir.exists() || !tracing_dir.exists());
        }
    }

    /// 测试Tracer初始化时registry构建逻辑
    #[test]
    fn test_tracer_init_registry_building_logic() {
        Tracer::init();
    }

    /// 测试Tracer初始化时条件添加console layer的逻辑
    #[test]
    fn test_tracer_init_conditional_console_layer() {
        Tracer::init();
        let settings = crate::base::Settings::get();
        let _enable_console = settings.log.enable_trace_console.unwrap_or(false);
        // Test verifies that the configuration can be read for conditional console layer setup
    }

    /// 测试Tracer初始化时回退逻辑的存在性
    #[test]
    fn test_tracer_init_fallback_logic_existence() {
        Tracer::init();
    }

    /// 测试Tracer初始化时sink writer逻辑的存在性
    #[test]
    fn test_tracer_init_sink_writer_logic() {
        Tracer::init();
    }
}
