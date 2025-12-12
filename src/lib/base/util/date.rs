//! 日期时间工具模块
//!
//! 提供文档时间戳生成功能，支持时区和格式配置。

use chrono::{Local, Utc};

/// 文档时间戳格式选项
#[derive(Debug, Clone, Copy)]
pub enum DateFormat {
    /// 日期格式：YYYY-MM-DD（如：2024-12-19）
    DateOnly,
    /// 日期时间格式：YYYY-MM-DD HH:MM:SS（如：2024-12-19 14:30:00）
    DateTime,
    /// ISO 8601 格式：YYYY-MM-DDTHH:MM:SS+00:00（如：2024-12-19T14:30:00+08:00）
    Iso8601,
}

/// 时区选项
#[derive(Debug, Clone, Copy)]
pub enum Timezone {
    /// 使用本地时区
    Local,
    /// 使用 UTC 时区
    Utc,
}

/// 生成文档时间戳
///
/// # 参数
///
/// * `format` - 日期格式选项
/// * `timezone` - 时区选项（默认使用本地时区）
///
/// # 示例
///
/// ```rust
/// use workflow::base::util::date::{format_document_timestamp, DateFormat, Timezone};
///
/// // 生成日期格式的时间戳（本地时区）
/// let date = format_document_timestamp(DateFormat::DateOnly, Timezone::Local);
/// // 输出：2024-12-19
///
/// // 生成日期时间格式的时间戳（UTC时区）
/// let datetime = format_document_timestamp(DateFormat::DateTime, Timezone::Utc);
/// // 输出：2024-12-19 06:30:00
/// ```
pub fn format_document_timestamp(format: DateFormat, timezone: Timezone) -> String {
    match format {
        DateFormat::DateOnly => match timezone {
            Timezone::Local => Local::now().format("%Y-%m-%d").to_string(),
            Timezone::Utc => Utc::now().format("%Y-%m-%d").to_string(),
        },
        DateFormat::DateTime => match timezone {
            Timezone::Local => Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            Timezone::Utc => Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        },
        DateFormat::Iso8601 => match timezone {
            Timezone::Local => Local::now().to_rfc3339(),
            Timezone::Utc => Utc::now().to_rfc3339(),
        },
    }
}

/// 生成文档"最后更新"时间戳（默认格式：YYYY-MM-DD）
///
/// 这是最常用的函数，用于在文档末尾生成"最后更新"时间戳。
/// 默认使用本地时区和日期格式（YYYY-MM-DD）。
///
/// # 示例
///
/// ```rust
/// use workflow::base::util::date::format_last_updated;
///
/// let timestamp = format_last_updated();
/// // 输出：2024-12-19
/// ```
pub fn format_last_updated() -> String {
    format_document_timestamp(DateFormat::DateOnly, Timezone::Local)
}

/// 生成文档"最后更新"时间戳（带时间）
///
/// 生成包含时间的"最后更新"时间戳（格式：YYYY-MM-DD HH:MM:SS）。
///
/// # 示例
///
/// ```rust
/// use workflow::base::util::date::format_last_updated_with_time;
///
/// let timestamp = format_last_updated_with_time();
/// // 输出：2024-12-19 14:30:00
/// ```
pub fn format_last_updated_with_time() -> String {
    format_document_timestamp(DateFormat::DateTime, Timezone::Local)
}

/// 生成文件名时间戳（格式：YYYY-MM-DD_HH-MM-SS）
///
/// 用于在文件名中添加时间戳，格式为 `YYYY-MM-DD_HH-MM-SS`（如：2024-12-19_14-30-00）。
/// 这个格式适合作为文件名的一部分，不包含空格或冒号等特殊字符，使用下划线和连字符分隔。
///
/// **自动获取当前时间**：函数会在调用时自动获取当前系统时间，无需提前获取。
/// 每次调用都会返回最新的时间戳。
///
/// # 示例
///
/// ```rust
/// use workflow::base::util::date::format_filename_timestamp;
///
/// // 直接调用，自动获取当前时间
/// let timestamp = format_filename_timestamp();
/// // 输出：2024-12-19_14-30-00
///
/// // 用于生成带时间戳的文件名
/// let filename = format!("CHECK_REPORT_{}.md", timestamp);
/// // 输出：CHECK_REPORT_2024-12-19_14-30-00.md
/// ```
pub fn format_filename_timestamp() -> String {
    Local::now().format("%Y-%m-%d_%H-%M-%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date_only() {
        let result = format_document_timestamp(DateFormat::DateOnly, Timezone::Local);
        // 验证格式：YYYY-MM-DD
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap().is_match(&result));
    }

    #[test]
    fn test_format_datetime() {
        let result = format_document_timestamp(DateFormat::DateTime, Timezone::Local);
        // 验证格式：YYYY-MM-DD HH:MM:SS
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
            .unwrap()
            .is_match(&result));
    }

    #[test]
    fn test_format_iso8601() {
        let result = format_document_timestamp(DateFormat::Iso8601, Timezone::Utc);
        // 验证 ISO 8601 格式
        assert!(result.contains('T'));
        assert!(result.contains('Z') || result.contains('+') || result.contains('-'));
    }

    #[test]
    fn test_format_last_updated() {
        let result = format_last_updated();
        // 验证格式：YYYY-MM-DD
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap().is_match(&result));
    }

    #[test]
    fn test_format_last_updated_with_time() {
        let result = format_last_updated_with_time();
        // 验证格式：YYYY-MM-DD HH:MM:SS
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
            .unwrap()
            .is_match(&result));
    }

    #[test]
    fn test_format_filename_timestamp() {
        let result = format_filename_timestamp();
        // 验证格式：YYYY-MM-DD_HH-MM-SS（适合文件名）
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$")
            .unwrap()
            .is_match(&result));
    }
}
