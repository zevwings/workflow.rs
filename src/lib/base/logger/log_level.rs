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

    /// 初始化日志级别（从参数或使用默认值）
    ///
    /// 优先级：
    /// 1. 如果提供了 `level` 参数，使用参数值
    /// 2. 否则使用默认级别（根据编译模式决定）
    ///
    /// # 参数
    ///
    /// * `level` - 可选的日志级别，如果为 `Some`，则使用该值；如果为 `None`，则使用默认级别
    pub fn init(level: Option<LogLevel>) {
        let mut current = LOG_LEVEL.lock().unwrap();
        if current.is_none() {
            let final_level = level.unwrap_or_else(Self::default_level);
            *current = Some(final_level);
        }
    }

    /// 获取当前日志级别
    ///
    /// 如果之前没有设置过，则返回默认级别（根据编译模式决定）
    pub(crate) fn current() -> Self {
        let mut level = LOG_LEVEL.lock().unwrap();
        level.unwrap_or_else(|| {
            let default = Self::default_level();
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
