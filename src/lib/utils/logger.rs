use colored::*;
use std::fmt;

/// Logger 模块
/// 提供带颜色的日志输出功能
/// 颜色输出的工具函数
pub struct Logger;

impl Logger {
    /// 成功消息（绿色 ✓）
    pub fn success(message: impl fmt::Display) -> String {
        format!("{} {}", "✓".green(), message)
    }

    /// 错误消息（红色 ✗）
    pub fn error(message: impl fmt::Display) -> String {
        format!("{} {}", "✗".red(), message)
    }

    /// 警告消息（黄色 ⚠）
    pub fn warning(message: impl fmt::Display) -> String {
        format!("{} {}", "⚠".yellow(), message)
    }

    /// 信息消息（蓝色 ℹ）
    pub fn info(message: impl fmt::Display) -> String {
        format!("{} {}", "ℹ".blue(), message)
    }

    /// 调试消息（灰色）
    pub fn debug(message: impl fmt::Display) -> String {
        format!("{} {}", "•".bright_black(), message)
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

    /// 打印调试消息
    pub fn print_debug(message: impl fmt::Display) {
        println!("{}", Self::debug(message));
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
/// # Examples
///
/// ```
/// use workflow::log_debug;
///
/// log_debug!("Debug information");
/// log_debug!("Debug: {} = {}", key, value);
/// ```
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::Logger::print_debug(format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_output() {
        assert!(Logger::success("Test").contains("✓"));
        assert!(Logger::error("Test").contains("✗"));
        assert!(Logger::warning("Test").contains("⚠"));
        assert!(Logger::info("Test").contains("ℹ"));
        assert!(Logger::debug("Test").contains("•"));
    }
}
