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
}
