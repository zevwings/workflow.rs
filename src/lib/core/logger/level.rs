//! 日志级别定义
//!
//! 定义日志级别枚举和相关转换方法。

use std::fmt;
use std::str::FromStr;

/// 日志级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// 不输出任何日志
    None = 0,
    /// 只输出错误
    Error = 1,
    /// 输出警告和错误
    Warn = 2,
    /// 输出信息、警告和错误
    Info = 3,
    /// 输出所有日志（包括调试）
    Debug = 4,
}

impl Default for LogLevel {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Self::Debug
        } else {
            Self::Info
        }
    }
}

impl LogLevel {
    /// 将 LogLevel 转换为 tracing 过滤器字符串
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "off",
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LogLevel {
    type Err = String;

    /// 从字符串转换为 LogLevel
    ///
    /// # 参数
    ///
    /// * `s` - 日志级别字符串（不区分大小写）："off", "error", "warn", "info", "debug"
    ///
    /// # 返回
    ///
    /// 如果字符串有效，返回对应的 LogLevel；否则返回错误信息
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(Self::None),
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            _ => Err(format!(
                "Invalid log level: {}. Expected: off, error, warn, info, debug",
                s
            )),
        }
    }
}
