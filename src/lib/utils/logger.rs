use colored::*;
use std::fmt;
use std::str::FromStr;
use std::sync::Mutex;

/// 全局日志级别存储（使用 Mutex 保证线程安全）
static LOG_LEVEL: Mutex<Option<LogLevel>> = Mutex::new(None);

/// 日志级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// 不输出任何日志
    None = 0,
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
    /// 获取默认日志级别（根据编译模式自动决定）
    ///
    /// - 在 debug 模式下（`cargo build` 或 `make dev`）使用 `Debug` 级别
    /// - 在 release 模式下（`cargo build --release`）使用 `Info` 级别
    pub fn default_level() -> Self {
        // 在 debug 模式下自动启用 DEBUG 日志级别
        if cfg!(debug_assertions) {
            LogLevel::Debug
        } else {
            LogLevel::Info
        }
    }

    /// 初始化日志级别（从配置文件加载）
    ///
    /// 如果配置文件中设置了日志级别，则使用配置文件中的值；
    /// 否则使用默认级别（根据编译模式决定）
    pub fn init_from_config() {
        use crate::settings::Settings;

        let mut level = LOG_LEVEL.lock().unwrap();
        if level.is_none() {
            // 尝试从配置文件加载
            let config_level = Settings::get()
                .log
                .level
                .as_ref()
                .and_then(|s| s.parse::<LogLevel>().ok());

            let final_level = config_level.unwrap_or(Self::default_level());
            *level = Some(final_level);
        }
    }

    /// 获取当前日志级别
    ///
    /// 如果之前没有设置过，则从配置文件加载或返回默认级别（根据编译模式决定）
    fn current() -> Self {
        let mut level = LOG_LEVEL.lock().unwrap();
        level.unwrap_or_else(|| {
            // 尝试从配置文件加载
            use crate::settings::Settings;
            let config_level = Settings::get()
                .log
                .level
                .as_ref()
                .and_then(|s| s.parse::<LogLevel>().ok());

            let default = config_level.unwrap_or(Self::default_level());
            *level = Some(default);
            default
        })
    }

    /// 设置日志级别
    ///
    /// # Arguments
    ///
    /// * `level` - 要设置的日志级别
    pub fn set_level(level: Self) {
        let mut current = LOG_LEVEL.lock().unwrap();
        *current = Some(level);
    }

    /// 获取当前日志级别（公开方法）
    pub fn get_level() -> Self {
        Self::current()
    }
}

impl FromStr for LogLevel {
    type Err = String;

    /// 从字符串转换为 LogLevel
    ///
    /// # Arguments
    ///
    /// * `s` - 日志级别字符串（不区分大小写）："none", "error", "warn", "info", "debug"
    ///
    /// # Returns
    ///
    /// 如果字符串有效，返回对应的 LogLevel；否则返回错误信息
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(LogLevel::None),
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            _ => Err(format!("Invalid log level: {}. Expected: none, error, warn, info, debug", s)),
        }
    }
}

impl LogLevel {

    /// 将 LogLevel 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::None => "none",
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        }
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

    /// 打印成功消息（总是输出，不受日志级别限制）
    ///
    /// 成功消息是命令执行结果的重要反馈，应该始终显示给用户。
    pub fn print_success(message: impl fmt::Display) {
        println!("{}", Self::success(message));
    }

    /// 打印错误消息（仅在日志级别 >= ERROR 时输出）
    pub fn print_error(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        if current_level.should_log(LogLevel::Error) {
            eprintln!("{}", Self::error(message));
        }
    }

    /// 打印警告消息（仅在日志级别 >= WARN 时输出）
    pub fn print_warning(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        if current_level.should_log(LogLevel::Warn) {
            println!("{}", Self::warning(message));
        }
    }

    /// 打印信息消息（仅在日志级别 >= INFO 时输出）
    pub fn print_info(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        if current_level.should_log(LogLevel::Info) {
            println!("{}", Self::info(message));
        }
    }

    /// 打印调试消息（仅在日志级别 >= DEBUG 时输出）
    pub fn print_debug(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        if current_level.should_log(LogLevel::Debug) {
            println!("{}", Self::debug(message));
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

    #[test]
    fn test_log_level_from_str() {
        // 测试有效的日志级别字符串（不区分大小写）
        assert_eq!("none".parse::<LogLevel>().unwrap(), LogLevel::None);
        assert_eq!("NONE".parse::<LogLevel>().unwrap(), LogLevel::None);
        assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert_eq!("ERROR".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("WARN".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("INFO".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("DEBUG".parse::<LogLevel>().unwrap(), LogLevel::Debug);

        // 测试无效的日志级别字符串
        assert!("invalid".parse::<LogLevel>().is_err());
        assert!("".parse::<LogLevel>().is_err());
        assert!("trace".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::None.as_str(), "none");
        assert_eq!(LogLevel::Error.as_str(), "error");
        assert_eq!(LogLevel::Warn.as_str(), "warn");
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Debug.as_str(), "debug");
    }

    #[test]
    fn test_log_level_ordering() {
        // 测试日志级别的顺序
        assert!(LogLevel::None < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);

        // 测试 should_log 方法
        let debug_level = LogLevel::Debug;
        assert!(debug_level.should_log(LogLevel::None));
        assert!(debug_level.should_log(LogLevel::Error));
        assert!(debug_level.should_log(LogLevel::Warn));
        assert!(debug_level.should_log(LogLevel::Info));
        assert!(debug_level.should_log(LogLevel::Debug));

        let info_level = LogLevel::Info;
        assert!(info_level.should_log(LogLevel::None));
        assert!(info_level.should_log(LogLevel::Error));
        assert!(info_level.should_log(LogLevel::Warn));
        assert!(info_level.should_log(LogLevel::Info));
        assert!(!info_level.should_log(LogLevel::Debug));

        let warn_level = LogLevel::Warn;
        assert!(warn_level.should_log(LogLevel::None));
        assert!(warn_level.should_log(LogLevel::Error));
        assert!(warn_level.should_log(LogLevel::Warn));
        assert!(!warn_level.should_log(LogLevel::Info));
        assert!(!warn_level.should_log(LogLevel::Debug));

        let error_level = LogLevel::Error;
        assert!(error_level.should_log(LogLevel::None));
        assert!(error_level.should_log(LogLevel::Error));
        assert!(!error_level.should_log(LogLevel::Warn));
        assert!(!error_level.should_log(LogLevel::Info));
        assert!(!error_level.should_log(LogLevel::Debug));

        let none_level = LogLevel::None;
        assert!(none_level.should_log(LogLevel::None));
        assert!(!none_level.should_log(LogLevel::Error));
        assert!(!none_level.should_log(LogLevel::Warn));
        assert!(!none_level.should_log(LogLevel::Info));
        assert!(!none_level.should_log(LogLevel::Debug));
    }

    #[test]
    fn test_log_level_set_and_get() {
        // 保存原始级别
        let original_level = LogLevel::get_level();

        // 测试设置和获取不同的日志级别
        LogLevel::set_level(LogLevel::Debug);
        assert_eq!(LogLevel::get_level(), LogLevel::Debug);

        LogLevel::set_level(LogLevel::Info);
        assert_eq!(LogLevel::get_level(), LogLevel::Info);

        LogLevel::set_level(LogLevel::Warn);
        assert_eq!(LogLevel::get_level(), LogLevel::Warn);

        LogLevel::set_level(LogLevel::Error);
        assert_eq!(LogLevel::get_level(), LogLevel::Error);

        LogLevel::set_level(LogLevel::None);
        assert_eq!(LogLevel::get_level(), LogLevel::None);

        // 恢复原始级别
        LogLevel::set_level(original_level);
    }

    #[test]
    fn test_log_level_default() {
        // 测试默认级别（根据编译模式）
        let default = LogLevel::default_level();

        // 在 debug 模式下应该是 Debug，在 release 模式下应该是 Info
        if cfg!(debug_assertions) {
            assert_eq!(default, LogLevel::Debug);
        } else {
            assert_eq!(default, LogLevel::Info);
        }
    }

    #[test]
    fn test_log_level_round_trip() {
        // 测试字符串转换的往返一致性
        let levels = vec![
            LogLevel::None,
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
        ];

        for level in levels {
            let level_str = level.as_str();
            let parsed = level_str.parse::<LogLevel>().unwrap();
            assert_eq!(level, parsed, "Round trip failed for level: {}", level_str);
        }
    }
}
