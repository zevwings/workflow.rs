//! 日期时间工具模块
//!
//! 提供文档时间戳生成功能，支持时区和格式配置。

use chrono::{Local, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// 文档时间戳格式选项
#[derive(Debug, Clone, Copy)]
pub enum DateFormat {
    /// 日期格式：YYYY-MM-DD（如：2024-12-19）
    DateOnly,
    /// 日期时间格式：YYYY-MM-DD HH:MM:SS（如：2024-12-19 14:30:00）
    DateTime,
    /// ISO 8601 格式：YYYY-MM-DDTHH:MM:SS+00:00（如：2024-12-19T14:30:00+08:00）
    Iso8601,
    /// 文件名时间戳格式：YYYY-MM-DD_HH-MM-SS（如：2024-12-19_14-30-00）
    /// 适合作为文件名的一部分，不包含空格或冒号等特殊字符
    Filename,
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
///
/// // 生成文件名格式的时间戳（本地时区）
/// let filename = format_document_timestamp(DateFormat::Filename, Timezone::Local);
/// // 输出：2024-12-19_14-30-00
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
        DateFormat::Filename => match timezone {
            Timezone::Local => Local::now().format("%Y-%m-%d_%H-%M-%S").to_string(),
            Timezone::Utc => Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string(),
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
/// 每次调用都会返回最新的时间戳。默认使用本地时区。
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
    format_document_timestamp(DateFormat::Filename, Timezone::Local)
}

/// 获取当前 Unix 时间戳（秒）
///
/// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的秒数。
/// 这是一个常用的时间戳格式，用于版本控制、缓存键等场景。
///
/// # Returns
///
/// * `u64` - Unix 时间戳（秒）
///
/// # Examples
///
/// ```rust
/// use workflow::base::util::date::get_unix_timestamp;
///
/// let timestamp = get_unix_timestamp();
/// println!("Current timestamp: {}", timestamp);
/// // 输出类似：Current timestamp: 1703001234
/// ```
///
/// # Panics
///
/// 如果系统时间在 Unix 纪元之前，此函数会 panic。在正常情况下这不应该发生。
pub fn get_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch")
        .as_secs()
}

/// 获取当前 Unix 时间戳（纳秒）
///
/// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的纳秒数。
/// 提供更高精度的时间戳，适用于需要高精度时间测量的场景。
///
/// # Returns
///
/// * `u128` - Unix 时间戳（纳秒）
///
/// # Examples
///
/// ```rust
/// use workflow::base::util::date::get_unix_timestamp_nanos;
///
/// let timestamp = get_unix_timestamp_nanos();
/// println!("Current timestamp (nanos): {}", timestamp);
/// // 输出类似：Current timestamp (nanos): 1703001234567890123
/// ```
///
/// # Panics
///
/// 如果系统时间在 Unix 纪元之前，此函数会 panic。在正常情况下这不应该发生。
pub fn get_unix_timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch")
        .as_nanos()
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
    fn test_format_filename() {
        let result = format_document_timestamp(DateFormat::Filename, Timezone::Local);
        // 验证格式：YYYY-MM-DD_HH-MM-SS（适合文件名）
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$")
            .unwrap()
            .is_match(&result));
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

    #[test]
    fn test_get_unix_timestamp() {
        let timestamp1 = get_unix_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let timestamp2 = get_unix_timestamp();

        // 验证时间戳是递增的
        assert!(timestamp2 >= timestamp1);

        // 验证时间戳在合理范围内（2020年之后）
        let year_2020_timestamp = 1577836800; // 2020-01-01 00:00:00 UTC
        assert!(timestamp1 > year_2020_timestamp);
    }

    #[test]
    fn test_get_unix_timestamp_nanos() {
        let timestamp1 = get_unix_timestamp_nanos();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let timestamp2 = get_unix_timestamp_nanos();

        // 验证纳秒时间戳是递增的
        assert!(timestamp2 > timestamp1);

        // 验证时间戳在合理范围内（2020年之后）
        let year_2020_timestamp_nanos = 1_577_836_800_000_000_000_u128; // 2020-01-01 00:00:00 UTC in nanos
        assert!(timestamp1 > year_2020_timestamp_nanos);

        // 验证纳秒时间戳比秒时间戳精度更高
        let timestamp_secs = get_unix_timestamp() as u128;
        let timestamp_nanos = get_unix_timestamp_nanos();
        assert!(timestamp_nanos > timestamp_secs * 1_000_000_000);
    }
}
