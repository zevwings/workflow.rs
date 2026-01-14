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

/// 日期格式化器
///
/// 提供日期时间格式化功能，支持配置默认格式和时区。
///
/// # 示例
///
/// ```rust
/// use workflow::util::date::{DateFormatter, DateFormat, Timezone};
///
/// // 使用默认配置（DateOnly, Local）
/// let formatter = DateFormatter::new();
/// let date = formatter.last_updated();
///
/// // 使用自定义配置
/// let formatter = DateFormatter::with_config(DateFormat::Iso8601, Timezone::Utc);
/// let date = formatter.format_with_defaults();
/// ```
pub struct DateFormatter {
    default_format: DateFormat,
    default_timezone: Timezone,
}

impl DateFormatter {
    /// 创建新的日期格式化器，使用默认配置
    ///
    /// 默认格式：`DateFormat::DateOnly`
    /// 默认时区：`Timezone::Local`
    pub fn new() -> Self {
        Self {
            default_format: DateFormat::DateOnly,
            default_timezone: Timezone::Local,
        }
    }

    /// 创建新的日期格式化器，指定默认格式和时区
    pub fn with_config(format: DateFormat, timezone: Timezone) -> Self {
        Self {
            default_format: format,
            default_timezone: timezone,
        }
    }

    /// 格式化文档时间戳
    ///
    /// # 参数
    ///
    /// * `format` - 日期格式选项
    /// * `timezone` - 时区选项
    ///
    /// # 示例
    ///
    /// ```rust
    /// use workflow::util::date::{DateFormatter, DateFormat, Timezone};
    ///
    /// let formatter = DateFormatter::new();
    /// let date = formatter.format(DateFormat::DateOnly, Timezone::Local);
    /// ```
    pub fn format(&self, format: DateFormat, timezone: Timezone) -> String {
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

    /// 使用默认配置格式化时间戳
    pub fn format_with_defaults(&self) -> String {
        self.format(self.default_format, self.default_timezone)
    }

    /// 生成文档"最后更新"时间戳（默认格式：YYYY-MM-DD）
    ///
    /// 使用默认时区，格式为 `DateFormat::DateOnly`。
    pub fn last_updated(&self) -> String {
        self.format(DateFormat::DateOnly, self.default_timezone)
    }

    /// 生成文档"最后更新"时间戳（带时间）
    ///
    /// 使用默认时区，格式为 `DateFormat::DateTime`。
    pub fn last_updated_with_time(&self) -> String {
        self.format(DateFormat::DateTime, self.default_timezone)
    }

    /// 生成文件名时间戳（格式：YYYY-MM-DD_HH-MM-SS）
    ///
    /// 使用默认时区，格式为 `DateFormat::Filename`。
    pub fn filename_timestamp(&self) -> String {
        self.format(DateFormat::Filename, self.default_timezone)
    }

    /// 获取当前 Unix 时间戳（秒）
    ///
    /// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的秒数。
    pub fn unix_timestamp(&self) -> color_eyre::Result<u64> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .map_err(|_| color_eyre::eyre::eyre!("System time is before Unix epoch"))
    }

    /// 获取当前 Unix 时间戳（纳秒）
    ///
    /// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的纳秒数。
    pub fn unix_timestamp_nanos(&self) -> color_eyre::Result<u128> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .map_err(|_| color_eyre::eyre::eyre!("System time is before Unix epoch"))
    }
}

impl Default for DateFormatter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// 便利函数（向后兼容）
// ============================================

/// 格式化文档时间戳
///
/// 使用默认格式化器格式化时间戳。
///
/// # 参数
///
/// * `format` - 日期格式选项
/// * `timezone` - 时区选项
///
/// # 示例
///
/// ```rust
/// use workflow::util::date::{format_document_timestamp, DateFormat, Timezone};
///
/// let date = format_document_timestamp(DateFormat::DateOnly, Timezone::Local);
/// ```
pub fn format_document_timestamp(format: DateFormat, timezone: Timezone) -> String {
    DateFormatter::new().format(format, timezone)
}

/// 格式化文件名时间戳
///
/// 生成适合作为文件名的时间戳（格式：YYYY-MM-DD_HH-MM-SS）。
///
/// # 示例
///
/// ```rust
/// use workflow::util::date::format_filename_timestamp;
///
/// let timestamp = format_filename_timestamp();
/// ```
pub fn format_filename_timestamp() -> String {
    DateFormatter::new().filename_timestamp()
}

/// 格式化最后更新时间戳（仅日期）
///
/// 生成文档"最后更新"时间戳（格式：YYYY-MM-DD）。
///
/// # 示例
///
/// ```rust
/// use workflow::util::date::format_last_updated;
///
/// let date = format_last_updated();
/// ```
pub fn format_last_updated() -> String {
    DateFormatter::new().last_updated()
}

/// 格式化最后更新时间戳（带时间）
///
/// 生成文档"最后更新"时间戳（格式：YYYY-MM-DD HH:MM:SS）。
///
/// # 示例
///
/// ```rust
/// use workflow::util::date::format_last_updated_with_time;
///
/// let datetime = format_last_updated_with_time();
/// ```
pub fn format_last_updated_with_time() -> String {
    DateFormatter::new().last_updated_with_time()
}

/// 获取当前 Unix 时间戳（纳秒）
///
/// 返回自 Unix 纪元（1970-01-01 00:00:00 UTC）以来的纳秒数。
///
/// # 示例
///
/// ```rust
/// use workflow::util::date::get_unix_timestamp_nanos;
///
/// let timestamp = get_unix_timestamp_nanos().unwrap();
/// ```
pub fn get_unix_timestamp_nanos() -> color_eyre::Result<u128> {
    DateFormatter::new().unix_timestamp_nanos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date_only() {
        let formatter = DateFormatter::new();
        let result = formatter.format(DateFormat::DateOnly, Timezone::Local);
        // 验证格式：YYYY-MM-DD
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap().is_match(&result));
    }

    #[test]
    fn test_format_datetime() {
        let formatter = DateFormatter::new();
        let result = formatter.format(DateFormat::DateTime, Timezone::Local);
        // 验证格式：YYYY-MM-DD HH:MM:SS
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
            .unwrap()
            .is_match(&result));
    }

    #[test]
    fn test_format_iso8601() {
        let formatter = DateFormatter::new();
        let result = formatter.format(DateFormat::Iso8601, Timezone::Utc);
        // 验证 ISO 8601 格式
        assert!(result.contains('T'));
        assert!(result.contains('Z') || result.contains('+') || result.contains('-'));
    }

    #[test]
    fn test_format_filename() {
        let formatter = DateFormatter::new();
        let result = formatter.format(DateFormat::Filename, Timezone::Local);
        // 验证格式：YYYY-MM-DD_HH-MM-SS（适合文件名）
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$")
            .unwrap()
            .is_match(&result));
    }

    // DateFormatter 方法测试
    #[test]
    fn test_date_formatter_new() {
        let formatter = DateFormatter::new();
        let result = formatter.last_updated();
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap().is_match(&result));
    }

    #[test]
    fn test_date_formatter_with_timezone() {
        let formatter = DateFormatter::with_config(DateFormat::DateOnly, Timezone::Utc);
        let result = formatter.last_updated();
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap().is_match(&result));
    }

    #[test]
    fn test_date_formatter_with_format() {
        let formatter = DateFormatter::with_config(DateFormat::DateTime, Timezone::Local);
        let result = formatter.format_with_defaults();
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
            .unwrap()
            .is_match(&result));
    }

    #[test]
    fn test_date_formatter_with_config() {
        let formatter = DateFormatter::with_config(DateFormat::Iso8601, Timezone::Utc);
        let result = formatter.format_with_defaults();
        assert!(result.contains('T'));
        assert!(result.contains('Z') || result.contains('+') || result.contains('-'));
    }

    #[test]
    fn test_date_formatter_last_updated() {
        let formatter = DateFormatter::new();
        let result = formatter.last_updated();
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap().is_match(&result));
    }

    #[test]
    fn test_date_formatter_last_updated_with_time() {
        let formatter = DateFormatter::new();
        let result = formatter.last_updated_with_time();
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
            .unwrap()
            .is_match(&result));
    }

    #[test]
    fn test_date_formatter_filename_timestamp() {
        let formatter = DateFormatter::new();
        let result = formatter.filename_timestamp();
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$")
            .unwrap()
            .is_match(&result));
    }

    #[test]
    fn test_date_formatter_unix_timestamp() {
        let formatter = DateFormatter::new();
        let timestamp1 = formatter.unix_timestamp().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let timestamp2 = formatter.unix_timestamp().unwrap();

        assert!(timestamp2 >= timestamp1);
        let year_2020_timestamp = 1577836800;
        assert!(timestamp1 > year_2020_timestamp);
    }

    #[test]
    fn test_date_formatter_unix_timestamp_nanos() {
        let formatter = DateFormatter::new();
        let timestamp1 = formatter.unix_timestamp_nanos().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let timestamp2 = formatter.unix_timestamp_nanos().unwrap();

        assert!(timestamp2 > timestamp1);
        let year_2020_timestamp_nanos = 1_577_836_800_000_000_000_u128;
        assert!(timestamp1 > year_2020_timestamp_nanos);
    }
}
