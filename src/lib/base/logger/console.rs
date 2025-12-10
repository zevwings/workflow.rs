//! Logger 模块
//!
//! 提供带颜色的日志输出功能，包括：
//! - 颜色格式化函数（success, error, warning, info, debug, separator）
//! - Logger 结构体和日志输出方法
//! - 日志宏（log_success!, log_error!, log_warning!, log_info!, log_debug!, log_message!, log_break!）
//!
//! ## 颜色输出
//!
//! 使用 console crate 实现，提供更丰富的功能和更好的终端支持：
//! - 支持多种日志级别样式（success, error, warning, info, debug）
//! - 支持分隔线样式（separator, separator_with_text）
//! - 使用 ASCII 字符作为图标（✓✗⚠ℹ⚙）

use crate::base::logger::log_level::LogLevel;
use console::style;
use std::fmt;

// ============================================================================
// 颜色格式化函数
// ============================================================================

/// 成功消息样式（绿色 ✓）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含绿色样式和成功图标
///
/// # 示例
/// ```
/// use workflow::base::logger::console::success;
/// use workflow::log_message;
/// let msg = success("Operation completed");
/// log_message!("{}", msg);
/// ```
pub fn success(text: impl fmt::Display) -> String {
    style(format!("✓ {}", text)).green().to_string()
}

/// 错误消息样式（红色 ✗）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含红色样式和错误图标
///
/// # 示例
/// ```
/// use workflow::base::logger::console::error;
/// use workflow::log_error;
/// let msg = error("Operation failed");
/// log_error!("{}", msg);
/// ```
pub fn error(text: impl fmt::Display) -> String {
    style(format!("✗ {}", text)).red().to_string()
}

/// 警告消息样式（黄色 ⚠）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含黄色样式和警告图标
///
/// # 示例
/// ```
/// use workflow::base::logger::console::warning;
/// use workflow::log_warning;
/// let msg = warning("This is a warning");
/// log_warning!("{}", msg);
/// ```
pub fn warning(text: impl fmt::Display) -> String {
    style(format!("⚠ {}", text)).yellow().to_string()
}

/// 信息消息样式（蓝色 ℹ）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含蓝色样式和信息图标
///
/// # 示例
/// ```
/// use workflow::base::logger::console::info;
/// use workflow::log_info;
/// let msg = info("Processing data");
/// log_info!("{}", msg);
/// ```
pub fn info(text: impl fmt::Display) -> String {
    style(format!("ℹ {}", text)).blue().to_string()
}

/// 调试消息样式（灰色 ⚙）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含灰色样式和调试图标
///
/// # 示例
/// ```
/// use workflow::base::logger::console::debug;
/// use workflow::log_debug;
/// let msg = debug("Debug information");
/// log_debug!("{}", msg);
/// ```
pub fn debug(text: impl fmt::Display) -> String {
    style(format!("⚙ {}", text)).bright().black().to_string()
}

/// 分隔线样式（灰色）
///
/// # 参数
/// * `char` - 分隔符字符
/// * `length` - 分隔线长度
///
/// # 返回
/// 格式化后的分隔线字符串
///
/// # 示例
/// ```
/// use workflow::base::logger::console::separator;
/// use workflow::log_message;
/// let sep = separator('-', 80);
/// log_message!("{}", sep);
/// ```
pub fn separator(char: char, length: usize) -> String {
    style(char.to_string().repeat(length)).bright().black().to_string()
}

/// 带文本的分隔线样式
///
/// 在分隔线中间插入文本，文本前后用分隔符字符填充。
/// 文本前后会自动添加空格。
///
/// # 参数
/// * `char` - 分隔符字符
/// * `length` - 总长度
/// * `text` - 要插入的文本
///
/// # 返回
/// 格式化后的带文本分隔线字符串
///
/// # 示例
/// ```
/// use workflow::base::logger::console::separator_with_text;
/// use workflow::log_message;
/// let sep = separator_with_text('=', 80, "Section Title");
/// log_message!("{}", sep);
/// ```
pub fn separator_with_text(char: char, length: usize, text: impl fmt::Display) -> String {
    let text_str = format!("  {} ", text);
    let text_len = text_str.chars().count();

    // 如果文本长度大于等于总长度，直接输出文本
    if text_len >= length {
        return style(text_str).bright().black().to_string();
    }

    // 计算左右两侧需要填充的字符数
    let remaining = length - text_len;
    let left_padding = remaining / 2;
    let right_padding = remaining - left_padding;

    // 生成分隔线
    let left_sep = char.to_string().repeat(left_padding);
    let right_sep = char.to_string().repeat(right_padding);

    format!(
        "{}{}{}",
        style(left_sep).bright().black(),
        text_str,
        style(right_sep).bright().black()
    )
}

// ============================================================================
// Logger 结构体和实现
// ============================================================================

/// Logger 结构体
///
/// 提供带颜色的日志输出功能，用于 Commands 层。
/// 所有方法都会根据当前日志级别决定是否输出。
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

// ============================================================================
// 日志宏
// ============================================================================

/// 格式化并打印成功消息
///
/// # Examples
///
/// ```
/// use workflow::log_success;
///
/// log_success!("Operation completed");
/// let count = 5;
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
/// let code = 404;
/// let message = "Not Found";
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
/// let count = 3;
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
/// let count = 10;
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
/// let key = "version";
/// let value = "1.0.0";
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
