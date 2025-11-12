use colored::*;
use std::env;
use std::fmt;
use std::sync::OnceLock;

/// 日志级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// 不输出任何日志
    Off = 0,
    /// 只输出错误
    Error = 1,
    /// 输出警告和错误
    Warn = 2,
    /// 输出信息、警告和错误（默认）
    Info = 3,
    /// 输出所有日志（包括调试）
    Debug = 4,
}

impl LogLevel {
    /// 从字符串解析日志级别
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "off" | "0" => LogLevel::Off,
            "error" | "1" => LogLevel::Error,
            "warn" | "warning" | "2" => LogLevel::Warn,
            "info" | "3" => LogLevel::Info,
            "debug" | "4" => LogLevel::Debug,
            _ => LogLevel::Info, // 默认值
        }
    }

    /// 获取当前日志级别（从环境变量读取）
    fn current() -> Self {
        static LOG_LEVEL: OnceLock<LogLevel> = OnceLock::new();
        *LOG_LEVEL.get_or_init(|| {
            env::var("WORKFLOW_LOG_LEVEL")
                .map(|s| LogLevel::from_str(&s))
                .unwrap_or(LogLevel::Info) // 默认 INFO，不输出 DEBUG
        })
    }

    /// 检查是否应该输出指定级别的日志
    fn should_log(&self, level: LogLevel) -> bool {
        *self >= level
    }
}

/// Logger 模块
/// 提供带颜色的日志输出功能
/// 颜色输出的工具函数
pub struct Logger;

impl Logger {
    /// 成功消息（绿色 ✅）
    pub fn success(message: impl fmt::Display) -> String {
        format!("{} {}", "✅".green(), message)
    }

    /// 错误消息（红色 ❌）
    pub fn error(message: impl fmt::Display) -> String {
        format!("{} {}", "❌".red(), message)
    }

    /// 警告消息（黄色 ⚠️）
    pub fn warning(message: impl fmt::Display) -> String {
        format!("{} {}", "⚠️".yellow(), message)
    }

    /// 信息消息（蓝色 ℹ️）
    pub fn info(message: impl fmt::Display) -> String {
        format!("{} {}", "ℹ️".blue(), message)
    }

    /// 调试消息（灰色 ⚙️）
    pub fn debug(message: impl fmt::Display) -> String {
        format!("{} {}", "⚙️".bright_black(), message)
    }

    /// 打印成功消息
    pub fn print_success(message: impl fmt::Display) {
        println!("{}", Self::success(message));
    }

    /// 打印错误消息
    pub fn print_error(message: impl fmt::Display) {
        eprintln!("{}", Self::error(message));
    }

    /// 打印警告消息
    pub fn print_warning(message: impl fmt::Display) {
        println!("{}", Self::warning(message));
    }

    /// 打印信息消息
    pub fn print_info(message: impl fmt::Display) {
        println!("{}", Self::info(message));
    }

    /// 打印调试消息（仅在日志级别 >= DEBUG 时输出）
    pub fn print_debug(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        if current_level.should_log(LogLevel::Debug) {
            println!("{}", Self::debug(message));
        }
    }

    /// 打印分隔线
    pub fn print_separator(char: Option<char>, length: Option<usize>) {
        let char = char.unwrap_or('-');
        let length = length.unwrap_or(80);
        let separator = char.to_string().repeat(length);
        println!("{}", separator.bright_black());
    }

    /// 打印换行符
    pub fn print_newline() {
        println!();
    }
}

/// 格式化并打印成功消息
///
/// # Examples
///
/// ```
/// use workflow::log_success;
///
/// log_success!("Operation completed");
/// log_success!("Found {} items", count);
/// ```
#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        $crate::Logger::print_success(format!($($arg)*));
    };
}

/// 格式化并打印错误消息
///
/// # Examples
///
/// ```
/// use workflow::log_error;
///
/// log_error!("Operation failed");
/// log_error!("Error: {} - {}", code, message);
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::Logger::print_error(format!($($arg)*));
    };
}

/// 格式化并打印警告消息
///
/// # Examples
///
/// ```
/// use workflow::log_warning;
///
/// log_warning!("This is a warning");
/// log_warning!("Warning: {} items missing", count);
/// ```
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        $crate::Logger::print_warning(format!($($arg)*));
    };
}

/// 格式化并打印信息消息
///
/// # Examples
///
/// ```
/// use workflow::log_info;
///
/// log_info!("Processing data");
/// log_info!("Processing {} items", count);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::Logger::print_info(format!($($arg)*));
    };
}

/// 格式化并打印调试消息
///
/// 仅在日志级别 >= DEBUG 时输出。
/// 日志级别通过环境变量 `WORKFLOW_LOG_LEVEL` 控制：
/// - `off` 或 `0`: 不输出任何日志
/// - `error` 或 `1`: 只输出错误
/// - `warn` 或 `2`: 输出警告和错误
/// - `info` 或 `3`: 输出信息、警告和错误（默认）
/// - `debug` 或 `4`: 输出所有日志（包括调试）
///
/// # Examples
///
/// ```
/// use workflow::log_debug;
///
/// // 默认情况下（WORKFLOW_LOG_LEVEL=info），这些不会输出
/// log_debug!("Debug information");
/// log_debug!("Debug: {} = {}", key, value);
///
/// // 设置 WORKFLOW_LOG_LEVEL=debug 后才会输出
/// ```
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::Logger::print_debug(format!($($arg)*));
    };
}

/// 打印分隔线或换行
///
/// # Examples
///
/// ```
/// use workflow::log_break;
///
/// // 输出换行符
/// log_break!();
///
/// // 使用默认分隔符（80个 '-'）
/// log_break!('-');
///
/// // 指定分隔符字符
/// log_break!('=');
///
/// // 指定分隔符字符和长度
/// log_break!('=', 100);
/// ```
#[macro_export]
macro_rules! log_break {
    () => {
      $crate::Logger::print_newline();
    };
    ($char:expr) => {
        $crate::Logger::print_separator(Some($char), None);
    };
    ($char:expr, $length:expr) => {
        $crate::Logger::print_separator(Some($char), Some($length));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_output() {
        assert!(Logger::success("Test").contains("✅"));
        assert!(Logger::error("Test").contains("❌"));
        assert!(Logger::warning("Test").contains("⚠️"));
        assert!(Logger::info("Test").contains("ℹ️"));
        assert!(Logger::debug("Test").contains("⚙️"));
    }
}
