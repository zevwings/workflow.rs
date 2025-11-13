use colored::*;
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
    /// 获取当前日志级别（根据编译模式自动决定）
    ///
    /// - 在 debug 模式下（`cargo build` 或 `make dev`）使用 `Debug` 级别
    /// - 在 release 模式下（`cargo build --release`）使用 `Info` 级别
    fn current() -> Self {
        static LOG_LEVEL: OnceLock<LogLevel> = OnceLock::new();
        *LOG_LEVEL.get_or_init(|| {
            // 在 debug 模式下自动启用 DEBUG 日志级别
            if cfg!(debug_assertions) {
                LogLevel::Debug
            } else {
                LogLevel::Info
            }
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

    /// 打印带文本的分隔线
    ///
    /// 在分隔线中间插入文本，文本前后用分隔符字符填充
    /// 文本前后会自动添加空格
    ///
    /// # 参数
    ///
    /// * `char` - 分隔符字符
    /// * `length` - 总长度
    /// * `text` - 要插入的文本
    pub fn print_separator_with_text(char: char, length: usize, text: impl fmt::Display) {
        let text_str = format!("  {} ", text); // 文本前后添加空格
        let text_len = text_str.chars().count();

        // 如果文本长度大于等于总长度，直接输出文本
        if text_len >= length {
            println!("{}", text_str.bright_black());
            return;
        }

        // 计算左右两侧需要填充的字符数
        let remaining = length - text_len;
        let left_padding = remaining / 2;
        let right_padding = remaining - left_padding;

        // 生成分隔线
        let left_sep = char.to_string().repeat(left_padding);
        let right_sep = char.to_string().repeat(right_padding);

        println!(
            "{}{}{}",
            left_sep.bright_black(),
            text_str,
            right_sep.bright_black()
        );
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
/// 日志级别根据编译模式自动决定：
/// - 在 debug 模式下（`cargo build` 或 `make dev`）使用 `Debug` 级别，会输出调试信息
/// - 在 release 模式下（`cargo build --release`）使用 `Info` 级别，不会输出调试信息
///
/// # Examples
///
/// ```
/// use workflow::log_debug;
///
/// log_debug!("Debug information");
/// log_debug!("Debug: {} = {}", key, value);
///
/// // 在 debug 模式下会自动输出
/// // 在 release 模式下不会输出
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
///
/// // 在分隔线中间插入文本
/// log_break!('=', 20, "flutter-api.log");
/// // 输出: ===========  flutter-api.log ===========
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
    ($char:expr, $length:expr, $text:expr) => {
        $crate::Logger::print_separator_with_text($char, $length, $text);
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
