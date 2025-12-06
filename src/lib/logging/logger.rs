use crate::base::ui::theme::Theme;
use std::fmt;
use std::str::FromStr;
use std::sync::Mutex;
use tracing;

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

    /// 初始化日志级别（从环境变量或参数）
    ///
    /// 优先级：
    /// 1. 如果提供了 `level` 参数，使用参数值
    /// 2. 如果设置了 `WORKFLOW_LOG_LEVEL` 环境变量，使用环境变量值
    /// 3. 否则使用默认级别（根据编译模式决定）
    ///
    /// # 参数
    ///
    /// * `level` - 可选的日志级别，如果为 `Some`，则使用该值；如果为 `None`，则从环境变量读取
    pub fn init(level: Option<LogLevel>) {
        let mut current = LOG_LEVEL.lock().unwrap();
        if current.is_none() {
            let final_level = level.unwrap_or_else(|| {
                // 尝试从环境变量读取
                std::env::var("WORKFLOW_LOG_LEVEL")
                    .ok()
                    .and_then(|s| s.parse::<LogLevel>().ok())
                    .unwrap_or_else(Self::default_level)
            });
            *current = Some(final_level);
        }
    }

    /// 获取当前日志级别
    ///
    /// 如果之前没有设置过，则从环境变量读取或返回默认级别（根据编译模式决定）
    fn current() -> Self {
        let mut level = LOG_LEVEL.lock().unwrap();
        level.unwrap_or_else(|| {
            // 尝试从环境变量读取
            let env_level = std::env::var("WORKFLOW_LOG_LEVEL")
                .ok()
                .and_then(|s| s.parse::<LogLevel>().ok());

            let default = env_level.unwrap_or_else(Self::default_level);
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
            _ => Err(format!(
                "Invalid log level: {}. Expected: none, error, warn, info, debug",
                s
            )),
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
    pub fn should_log(&self, level: LogLevel) -> bool {
        *self >= level
    }
}

/// Logger 模块
/// 提供带颜色的日志输出功能
/// 颜色输出的工具函数
pub struct Logger;

impl Logger {
    /// 成功消息（绿色 ✓）
    pub fn success(message: impl fmt::Display) -> String {
        let text = format!("✓ {}", message);
        format_with_style(text, Theme::success())
    }

    /// 错误消息（红色 ✗）
    pub fn error(message: impl fmt::Display) -> String {
        let text = format!("✗ {}", message);
        format_with_style(text, Theme::error())
    }

    /// 警告消息（黄色 ⚠）
    pub fn warning(message: impl fmt::Display) -> String {
        let text = format!("⚠ {}", message);
        format_with_style(text, Theme::warning())
    }

    /// 信息消息（蓝色 ℹ）
    pub fn info(message: impl fmt::Display) -> String {
        let text = format!("ℹ {}", message);
        format_with_style(text, Theme::info())
    }

    /// 调试消息（灰色 ⚙）
    pub fn debug(message: impl fmt::Display) -> String {
        let text = format!("⚙ {}", message);
        format_with_style(text, Theme::debug())
    }

    /// 打印成功消息（总是输出，不受日志级别限制）
    ///
    /// 成功消息是命令执行结果的重要反馈，应该始终显示给用户。
    pub fn print_success(message: impl fmt::Display) {
        let msg = message.to_string();
        // 使用 tracing 记录
        tracing::info!(message = %msg, "Success");
        // CLI 输出（转换为 ANSI 颜色码）
        println!("{}", Self::success(&msg));
    }

    /// 打印错误消息（仅在日志级别 >= ERROR 时输出）
    pub fn print_error(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        let msg = message.to_string();
        // 使用 tracing 记录
        tracing::error!(message = %msg, "Error");
        if current_level.should_log(LogLevel::Error) {
            eprintln!("{}", Self::error(&msg));
        }
    }

    /// 打印警告消息（仅在日志级别 >= WARN 时输出）
    pub fn print_warning(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        let msg = message.to_string();
        // 使用 tracing 记录
        tracing::warn!(message = %msg, "Warning");
        if current_level.should_log(LogLevel::Warn) {
            println!("{}", Self::warning(&msg));
        }
    }

    /// 打印信息消息（仅在日志级别 >= INFO 时输出）
    pub fn print_info(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        let msg = message.to_string();
        // 使用 tracing 记录
        tracing::info!(message = %msg, "Info");
        if current_level.should_log(LogLevel::Info) {
            println!("{}", Self::info(&msg));
        }
    }

    /// 打印调试消息（仅在日志级别 >= DEBUG 时输出）
    pub fn print_debug(message: impl fmt::Display) {
        let current_level = LogLevel::current();
        let msg = message.to_string();
        // 使用 tracing 记录
        tracing::debug!(message = %msg, "Debug");
        if current_level.should_log(LogLevel::Debug) {
            println!("{}", Self::debug(&msg));
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
        println!("{}", format_with_style(separator, Theme::debug()));
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
            println!("{}", format_with_style(text_str, Theme::debug()));
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
            format_with_style(left_sep, Theme::debug()),
            text_str,
            format_with_style(right_sep, Theme::debug())
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

/// 将 ratatui Style 转换为 ANSI 颜色码字符串
fn format_with_style(text: String, style: ratatui::style::Style) -> String {
    use std::fmt::Write;
    let mut result = String::new();

    // 前景色
    if let Some(fg) = style.fg {
        write!(&mut result, "\x1b[{}m", color_to_ansi(fg)).unwrap();
    }

    // 修饰符
    if style.add_modifier.contains(ratatui::style::Modifier::BOLD) {
        write!(&mut result, "\x1b[1m").unwrap();
    }
    if style.add_modifier.contains(ratatui::style::Modifier::DIM) {
        write!(&mut result, "\x1b[2m").unwrap();
    }
    if style
        .add_modifier
        .contains(ratatui::style::Modifier::ITALIC)
    {
        write!(&mut result, "\x1b[3m").unwrap();
    }
    if style
        .add_modifier
        .contains(ratatui::style::Modifier::UNDERLINED)
    {
        write!(&mut result, "\x1b[4m").unwrap();
    }

    // 文本内容
    write!(&mut result, "{}", text).unwrap();

    // 重置所有样式
    write!(&mut result, "\x1b[0m").unwrap();

    result
}

/// 将 ratatui Color 转换为 ANSI 颜色码
fn color_to_ansi(color: ratatui::style::Color) -> u8 {
    match color {
        ratatui::style::Color::Black => 30,
        ratatui::style::Color::Red => 31,
        ratatui::style::Color::Green => 32,
        ratatui::style::Color::Yellow => 33,
        ratatui::style::Color::Blue => 34,
        ratatui::style::Color::Magenta => 35,
        ratatui::style::Color::Cyan => 36,
        ratatui::style::Color::White => 37,
        ratatui::style::Color::Gray => 90,
        ratatui::style::Color::DarkGray => 90,
        ratatui::style::Color::LightRed => 91,
        ratatui::style::Color::LightGreen => 92,
        ratatui::style::Color::LightYellow => 93,
        ratatui::style::Color::LightBlue => 94,
        ratatui::style::Color::LightMagenta => 95,
        ratatui::style::Color::LightCyan => 96,
        _ => 37, // 默认白色（包括 LightGray 等不存在的颜色）
    }
}
