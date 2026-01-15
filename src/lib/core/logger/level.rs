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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // ==================== as_str() 方法测试 ====================

    #[rstest]
    #[case(LogLevel::None, "off")]
    #[case(LogLevel::Error, "error")]
    #[case(LogLevel::Warn, "warn")]
    #[case(LogLevel::Info, "info")]
    #[case(LogLevel::Debug, "debug")]
    fn test_as_str_all_levels(#[case] level: LogLevel, #[case] expected: &str) {
        assert_eq!(level.as_str(), expected);
    }

    // ==================== Display trait 测试 ====================

    #[rstest]
    #[case(LogLevel::None, "off")]
    #[case(LogLevel::Error, "error")]
    #[case(LogLevel::Warn, "warn")]
    #[case(LogLevel::Info, "info")]
    #[case(LogLevel::Debug, "debug")]
    fn test_display_all_levels(#[case] level: LogLevel, #[case] expected: &str) {
        assert_eq!(level.to_string(), expected);
    }

    #[rstest]
    #[case(LogLevel::None)]
    #[case(LogLevel::Error)]
    #[case(LogLevel::Warn)]
    #[case(LogLevel::Info)]
    #[case(LogLevel::Debug)]
    fn test_display_matches_as_str(#[case] level: LogLevel) {
        // 验证 Display 输出与 as_str() 一致
        assert_eq!(format!("{}", level), level.as_str());
    }

    // ==================== FromStr trait 测试 ====================

    #[rstest]
    // 小写
    #[case("off", LogLevel::None)]
    #[case("error", LogLevel::Error)]
    #[case("warn", LogLevel::Warn)]
    #[case("info", LogLevel::Info)]
    #[case("debug", LogLevel::Debug)]
    // 大写
    #[case("OFF", LogLevel::None)]
    #[case("ERROR", LogLevel::Error)]
    #[case("WARN", LogLevel::Warn)]
    #[case("INFO", LogLevel::Info)]
    #[case("DEBUG", LogLevel::Debug)]
    // 混合大小写
    #[case("Off", LogLevel::None)]
    #[case("ErRoR", LogLevel::Error)]
    #[case("WaRn", LogLevel::Warn)]
    #[case("InFo", LogLevel::Info)]
    #[case("DeBuG", LogLevel::Debug)]
    fn test_from_str_valid(#[case] input: &str, #[case] expected: LogLevel) {
        assert_eq!(input.parse::<LogLevel>().unwrap(), expected);
    }

    #[rstest]
    #[case("invalid")]
    #[case("")]
    #[case("trace")]
    #[case("fatal")]
    #[case("warning")]
    fn test_from_str_invalid(#[case] input: &str) {
        assert!(input.parse::<LogLevel>().is_err());
    }

    #[rstest]
    #[case("invalid")]
    #[case("trace")]
    #[case("fatal")]
    fn test_from_str_error_message(#[case] input: &str) {
        let result = input.parse::<LogLevel>();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Invalid log level"));
        assert!(error.contains(input));
        assert!(error.contains("off, error, warn, info, debug"));
    }

    // ==================== PartialOrd 和 Ord trait 测试 ====================

    #[rstest]
    #[case(LogLevel::None, LogLevel::Error)]
    #[case(LogLevel::Error, LogLevel::Warn)]
    #[case(LogLevel::Warn, LogLevel::Info)]
    #[case(LogLevel::Info, LogLevel::Debug)]
    fn test_partial_ord(#[case] smaller: LogLevel, #[case] larger: LogLevel) {
        assert!(smaller < larger);
        assert!(larger > smaller);
    }

    #[test]
    fn test_ord_sorting() {
        let mut levels = vec![
            LogLevel::Debug,
            LogLevel::None,
            LogLevel::Info,
            LogLevel::Error,
            LogLevel::Warn,
        ];
        levels.sort();

        assert_eq!(
            levels,
            vec![
                LogLevel::None,
                LogLevel::Error,
                LogLevel::Warn,
                LogLevel::Info,
                LogLevel::Debug,
            ]
        );
    }

    #[rstest]
    #[case(LogLevel::None)]
    #[case(LogLevel::Error)]
    #[case(LogLevel::Warn)]
    #[case(LogLevel::Info)]
    #[case(LogLevel::Debug)]
    fn test_partial_eq_same(#[case] level: LogLevel) {
        assert_eq!(level, level);
    }

    #[rstest]
    #[case(LogLevel::None, LogLevel::Error)]
    #[case(LogLevel::None, LogLevel::Warn)]
    #[case(LogLevel::None, LogLevel::Info)]
    #[case(LogLevel::None, LogLevel::Debug)]
    #[case(LogLevel::Error, LogLevel::Warn)]
    #[case(LogLevel::Error, LogLevel::Info)]
    #[case(LogLevel::Error, LogLevel::Debug)]
    #[case(LogLevel::Warn, LogLevel::Info)]
    #[case(LogLevel::Warn, LogLevel::Debug)]
    #[case(LogLevel::Info, LogLevel::Debug)]
    fn test_partial_eq_different(#[case] level1: LogLevel, #[case] level2: LogLevel) {
        assert_ne!(level1, level2);
    }

    // ==================== Default trait 测试 ====================

    #[test]
    fn test_default() {
        let default = LogLevel::default();
        // 在测试环境中，debug_assertions 通常是启用的
        #[cfg(debug_assertions)]
        assert_eq!(default, LogLevel::Debug);
        #[cfg(not(debug_assertions))]
        assert_eq!(default, LogLevel::Info);
    }
}
