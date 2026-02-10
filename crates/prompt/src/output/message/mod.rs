//! 消息输出器
//!
//! 提供消息输出功能和格式化宏

#[allow(clippy::module_inception)]
mod message;
mod message_ref;

pub use message::Message;
pub use message_ref::MessageRef;

// ============================================================================
// 宏定义
// ============================================================================

/// 格式化并输出成功消息
///
/// # Examples
///
/// ```
/// use prompt::success;
///
/// success!("Operation completed");
/// let count = 5;
/// success!("Found {} items", count);
/// ```
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        let _ = $crate::Message::global().success(&format!($($arg)*));
    };
}

/// 格式化并输出错误消息
///
/// # Examples
///
/// ```
/// use prompt::error;
///
/// error!("Operation failed");
/// let code = 404;
/// let message = "Not Found";
/// error!("Error: {} - {}", code, message);
/// ```
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let _ = $crate::Message::global().error(&format!($($arg)*));
    };
}

/// 格式化并输出警告消息
///
/// # Examples
///
/// ```
/// use prompt::warning;
///
/// warning!("This is a warning");
/// let count = 3;
/// warning!("Warning: {} items missing", count);
/// ```
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        let _ = $crate::Message::global().warning(&format!($($arg)*));
    };
}

/// 格式化并输出信息消息
///
/// # Examples
///
/// ```
/// use prompt::info;
///
/// info!("Processing data");
/// let count = 10;
/// info!("Processing {} items", count);
/// ```
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let _ = $crate::Message::global().info(&format!($($arg)*));
    };
}

/// 格式化并输出调试消息
///
/// # Examples
///
/// ```
/// use prompt::debug;
///
/// debug!("Debug information");
/// let key = "version";
/// let value = "1.0.0";
/// debug!("Debug: {} = {}", key, value);
/// ```
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        let _ = $crate::Message::global().debug(&format!($($arg)*));
    };
}

/// 格式化并输出纯文本（无 emoji 前缀）
///
/// # Examples
///
/// ```
/// use prompt::print;
///
/// print!("Plain text message");
/// let name = "Alice";
/// print!("Hello, {}!", name);
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = $crate::Message::global().print(&format!($($arg)*));
    };
}

/// 输出换行
///
/// # Examples
///
/// ```
/// use prompt::br;
///
/// // 输出换行符
/// br!();
/// ```
#[macro_export]
macro_rules! br {
    () => {
        let _ = $crate::Message::global().break_line();
    };
}

/// 输出分隔线
///
/// # Examples
///
/// ```
/// use prompt::separator;
///
/// // 使用默认分隔符（80个 '─'）
/// separator!();
///
/// // 指定分隔符字符和长度
/// separator!('─', 80);
///
/// // 在分隔线中间插入文本
/// separator!('─', 80, "GitHub Configuration (Required)");
/// // 输出: ───────────────────────  GitHub Configuration (Required) ───────────────────────
/// ```
#[macro_export]
macro_rules! separator {
    () => {
        let _ = $crate::Message::global().separator('─', 80);
    };
    ($char:expr) => {
        let _ = $crate::Message::global().separator($char, 80);
    };
    ($char:expr, $length:expr) => {
        let _ = $crate::Message::global().separator($char, $length);
    };
    ($char:expr, $length:expr, $text:expr) => {
        let _ = $crate::Message::global().separator_with_text($char, $length, $text);
    };
}
