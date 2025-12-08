use crate::base::logger::log_level::LogLevel;
use crate::base::util::colors::{
    debug, error, info, separator, separator_with_text, success, warning,
};
use std::fmt;

/// Logger 模块
/// 提供带颜色的日志输出功能
/// 颜色输出的工具函数
pub struct Logger;

impl Logger {
    /// 打印成功消息（总是输出，不受日志级别限制）
    ///
    /// 成功消息是命令执行结果的重要反馈，应该始终显示给用户。
    pub fn print_success(message: impl fmt::Display) {
        println!("{}", success(message));
    }

    /// 打印错误消息（仅在日志级别 >= ERROR 时输出）
    pub fn print_error(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        if current_level.should_log(LogLevel::Error) {
            eprintln!("{}", error(message));
        }
    }

    /// 打印警告消息（仅在日志级别 >= WARN 时输出）
    pub fn print_warning(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        if current_level.should_log(LogLevel::Warn) {
            println!("{}", warning(message));
        }
    }

    /// 打印信息消息（仅在日志级别 >= INFO 时输出）
    pub fn print_info(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        if current_level.should_log(LogLevel::Info) {
            println!("{}", info(message));
        }
    }

    /// 打印调试消息（仅在日志级别 >= DEBUG 时输出）
    pub fn print_debug(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        if current_level.should_log(LogLevel::Debug) {
            println!("{}", debug(message));
        }
    }

    /// 打印说明信息（总是输出，不受日志级别限制）
    ///
    /// 用于输出 setup/check 等命令的说明信息，这些信息是指令性的，
    /// 用户需要看到，不应该被日志级别过滤。
    pub fn print_message(message: impl fmt::Display) {
        println!("{}", message);
    }

    /// 打印分隔线
    pub fn print_separator(char: Option<char>, length: Option<usize>) {
        let char = char.unwrap_or('-');
        let length = length.unwrap_or(80);
        println!("{}", separator(char, length));
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
        println!("{}", separator_with_text(char, length, text));
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

/// 格式化并打印说明信息
///
/// 总是输出，不受日志级别限制。
/// 用于输出 setup/check 等命令的说明信息，这些信息是指令性的，
/// 用户需要看到，不应该被日志级别过滤。
///
/// # Examples
///
/// ```
/// use workflow::log_message;
///
/// log_message!("Running environment checks...");
/// log_message!("[1/2] Checking Git repository status...");
/// log_message!("  User Configuration (Required)");
/// ```
#[macro_export]
macro_rules! log_message {
    ($($arg:tt)*) => {
        $crate::Logger::print_message(format!($($arg)*));
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
