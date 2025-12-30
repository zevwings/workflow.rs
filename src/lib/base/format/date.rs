//! 日期时间格式化工具
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
/// use workflow::base::format::date::{format_document_timestamp, DateFormat, Timezone};
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
/// use workflow::base::format::date::format_last_updated;
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
/// use workflow::base::format::date::format_last_updated_with_time;
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
/// use workflow::base::format::date::format_filename_timestamp;
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
/// use workflow::base::format::date::get_unix_timestamp;
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
/// use workflow::base::format::date::get_unix_timestamp_nanos;
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
    use color_eyre::Result;
    use regex::Regex;
    use rstest::rstest;
    use std::thread;

    /// 测试基本时间戳格式
    ///
    /// ## 测试目的
    /// 验证时间戳生成功能的基本功能，包括文档时间戳格式和 Unix 时间戳。
    ///
    /// ## 测试场景
    /// 1. 测试文档时间戳格式化（DateOnly 格式，本地时区）
    /// 2. 测试 Unix 时间戳获取
    ///
    /// ## 预期结果
    /// - 文档时间戳格式为 YYYY-MM-DD（10个字符，包含连字符）
    /// - Unix 时间戳大于 2020-01-01 的时间戳（1577836800）
    #[test]
    fn test_basic_timestamp_formats() {
        // Arrange: 准备时间戳阈值（2020年）
        let min_timestamp = 1577836800;

        // Act: 格式化文档时间戳（DateOnly 格式，本地时区）
        let date = format_document_timestamp(DateFormat::DateOnly, Timezone::Local);

        // Assert: 验证文档时间戳格式为 YYYY-MM-DD
        assert!(date.contains('-') && date.len() == 10); // YYYY-MM-DD

        // Act: 获取 Unix 时间戳
        let timestamp = get_unix_timestamp();

        // Assert: 验证 Unix 时间戳合理（在2020年之后）
        assert!(timestamp > min_timestamp); // After 2020-01-01
    }

    // ==================== 日期时间格式化测试 ====================

    /// 测试日期格式化功能
    ///
    /// ## 测试目的
    /// 验证 format_document_timestamp() 能够使用 DateFormat::DateOnly 正确格式化日期。
    ///
    /// ## 测试场景
    /// 测试 Local 和 UTC 时区的日期格式化
    ///
    /// ## 预期结果
    /// - 格式为 YYYY-MM-DD
    /// - Local 和 UTC 时区都能正确格式化
    #[test]
    fn test_date_format_patterns_with_date_format_returns_formatted_date() -> Result<()> {
        // Arrange: 准备日期格式正则表达式
        let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$")
            .map_err(|e| color_eyre::eyre::eyre!("Date regex pattern should be valid: {}", e))?;

        // Act: 格式化日期（Local和UTC时区）
        let date_local = format_document_timestamp(DateFormat::DateOnly, Timezone::Local);
        let date_utc = format_document_timestamp(DateFormat::DateOnly, Timezone::Utc);

        // Assert: 验证格式为YYYY-MM-DD
        assert!(date_regex.is_match(&date_local));
        assert!(date_regex.is_match(&date_utc));
        Ok(())
    }

    /// 测试日期时间格式化功能
    ///
    /// ## 测试目的
    /// 验证 format_document_timestamp() 能够使用 DateFormat::DateTime 正确格式化日期时间。
    ///
    /// ## 测试场景
    /// 测试 Local 和 UTC 时区的日期时间格式化
    ///
    /// ## 预期结果
    /// - 格式为 YYYY-MM-DD HH:MM:SS
    /// - Local 和 UTC 时区都能正确格式化
    #[test]
    fn test_datetime_format_patterns_with_datetime_format_returns_formatted_datetime() -> Result<()>
    {
        // Arrange: 准备日期时间格式正则表达式
        let datetime_regex =
            regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").map_err(|e| {
                color_eyre::eyre::eyre!("DateTime regex pattern should be valid: {}", e)
            })?;

        // Act: 格式化日期时间（Local和UTC时区）
        let datetime_local = format_document_timestamp(DateFormat::DateTime, Timezone::Local);
        let datetime_utc = format_document_timestamp(DateFormat::DateTime, Timezone::Utc);

        // Assert: 验证格式为YYYY-MM-DD HH:MM:SS
        assert!(datetime_regex.is_match(&datetime_local));
        assert!(datetime_regex.is_match(&datetime_utc));
        Ok(())
    }

    /// 测试ISO 8601格式化功能
    ///
    /// ## 测试目的
    /// 验证 format_document_timestamp() 能够使用 DateFormat::Iso8601 正确格式化ISO 8601时间戳。
    ///
    /// ## 测试场景
    /// 测试 Local 和 UTC 时区的ISO 8601格式化
    ///
    /// ## 预期结果
    /// - 格式符合ISO 8601标准
    /// - UTC时区以Z结尾或包含时区偏移
    #[test]
    fn test_iso8601_format_patterns_with_iso8601_format_returns_formatted_string() {
        // Arrange: 准备ISO 8601格式

        // Act: 格式化ISO 8601时间戳（Local和UTC时区）
        let iso_local = format_document_timestamp(DateFormat::Iso8601, Timezone::Local);
        let iso_utc = format_document_timestamp(DateFormat::Iso8601, Timezone::Utc);

        // Assert: 验证ISO 8601格式特征
        assert!(iso_local.contains('T'));
        assert!(iso_utc.contains('T'));
        assert!(iso_utc.ends_with('Z') || iso_utc.contains('+') || iso_utc.contains('-'));
    }

    /// 测试日期格式化的便利函数
    ///
    /// ## 测试目的
    /// 验证 format_last_updated() 和 format_last_updated_with_time() 等便利函数能够返回有效格式。
    ///
    /// ## 预期结果
    /// - format_last_updated() 返回日期格式（YYYY-MM-DD）
    /// - format_last_updated_with_time() 返回日期时间格式（YYYY-MM-DD HH:MM:SS）
    #[test]
    fn test_convenience_functions_return_valid_format() -> Result<()> {
        // Arrange: 准备正则表达式模式
        let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$")
            .map_err(|e| color_eyre::eyre::eyre!("Date regex pattern should be valid: {}", e))?;
        let datetime_regex =
            regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").map_err(|e| {
                color_eyre::eyre::eyre!("DateTime regex pattern should be valid: {}", e)
            })?;

        // Act: 调用便利函数
        let last_updated = format_last_updated();
        let last_updated_with_time = format_last_updated_with_time();

        // Assert: 验证格式正确
        assert!(date_regex.is_match(&last_updated));
        assert!(datetime_regex.is_match(&last_updated_with_time));
        Ok(())
    }

    /// 测试文件名时间戳格式化功能
    ///
    /// ## 测试目的
    /// 验证 format_filename_timestamp() 能够返回文件名友好的时间戳格式。
    ///
    /// ## 预期结果
    /// - 格式为 YYYY-MM-DD_HH-MM-SS
    /// - 不包含空格和冒号（文件名友好）
    #[test]
    fn test_filename_timestamp_format_returns_filename_friendly_string() -> Result<()> {
        // Arrange: 准备正则表达式模式
        let filename_regex =
            regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$").map_err(|e| {
                color_eyre::eyre::eyre!("Filename regex pattern should be valid: {}", e)
            })?;

        // Act: 调用文件名时间戳格式化函数
        let filename_timestamp = format_filename_timestamp();

        // Assert: 验证格式正确且文件名友好
        assert!(filename_regex.is_match(&filename_timestamp));
        assert!(!filename_timestamp.contains(' '));
        assert!(!filename_timestamp.contains(':'));
        Ok(())
    }

    /// 测试不同日期格式的一致性
    ///
    /// ## 测试目的
    /// 验证同一时刻的不同日期格式应该包含相同的日期部分。
    ///
    /// ## 测试场景
    /// 比较 DateOnly、DateTime 和 filename_timestamp 格式的日期部分
    ///
    /// ## 预期结果
    /// - 所有格式的日期部分（YYYY-MM-DD）一致
    #[test]
    fn test_date_consistency_across_formats_has_same_date_part() {
        // Arrange: 准备不同格式的时间戳函数
        // 注意：测试同一时刻的不同格式应该包含相同的日期部分

        // Act: 调用不同格式的时间戳函数
        let date_only = format_document_timestamp(DateFormat::DateOnly, Timezone::Local);
        let datetime = format_document_timestamp(DateFormat::DateTime, Timezone::Local);
        let filename_ts = format_filename_timestamp();

        // Assert: 验证日期部分一致
        let date_part_from_datetime = &datetime[..10];
        let date_part_from_filename = &filename_ts[..10];
        assert_eq!(date_only, date_part_from_datetime);
        assert_eq!(date_only, date_part_from_filename);
    }

    /// 测试日期格式模式（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证不同日期格式的模式匹配。
    ///
    /// ## 测试场景
    /// 测试 DateOnly 和 DateTime 格式的正则表达式匹配
    ///
    /// ## 预期结果
    /// - 所有格式都能正确匹配对应的正则表达式
    #[rstest]
    #[case(DateFormat::DateOnly, r"^\d{4}-\d{2}-\d{2}$")]
    #[case(DateFormat::DateTime, r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")]
    fn test_format_patterns_parametrized(
        #[case] format: DateFormat,
        #[case] pattern: &str,
    ) -> Result<()> {
        let result_local = format_document_timestamp(format, Timezone::Local);
        let result_utc = format_document_timestamp(format, Timezone::Utc);

        let regex = regex::Regex::new(pattern)
            .map_err(|e| color_eyre::eyre::eyre!("Regex pattern should be valid: {}", e))?;
        assert!(regex.is_match(&result_local));
        assert!(regex.is_match(&result_utc));
        Ok(())
    }

    // ==================== Filename Timestamp Tests ====================

    /// 测试格式化文件名时间戳（无参数）
    ///
    /// ## 测试目的
    /// 验证 `format_filename_timestamp()` 函数能够返回正确格式的文件名时间戳（YYYY-MM-DD_HH-MM-SS格式，适合文件名）。
    ///
    /// ## 测试场景
    /// 1. 调用 `format_filename_timestamp()` 格式化时间戳
    /// 2. 使用正则表达式验证格式
    ///
    /// ## 预期结果
    /// - 返回的时间戳格式为 YYYY-MM-DD_HH-MM-SS
    /// - 格式适合用作文件名（不包含空格和冒号）
    #[test]
    fn test_format_filename_timestamp_with_no_parameters_returns_formatted_string() -> Result<()> {
        // Arrange: 准备文件名时间戳格式的正则表达式
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$").map_err(|e| {
            color_eyre::eyre::eyre!("Filename timestamp regex should be valid: {}", e)
        })?;

        // Act: 格式化文件名时间戳
        let result = format_filename_timestamp();

        // Assert: 验证格式为YYYY-MM-DD_HH-MM-SS（适合文件名）
        assert!(
            re.is_match(&result),
            "Filename timestamp format should match YYYY-MM-DD_HH-MM-SS"
        );
        Ok(())
    }

    // ==================== Unix Timestamp Tests ====================

    /// 测试获取Unix时间戳（无参数）
    ///
    /// ## 测试目的
    /// 验证 `get_unix_timestamp()` 函数能够返回合理的Unix时间戳，并且时间戳会递增。
    ///
    /// ## 测试场景
    /// 1. 获取Unix时间戳
    /// 2. 验证时间戳合理（在2020年之后）
    /// 3. 等待一小段时间后再次获取
    /// 4. 验证时间戳递增
    ///
    /// ## 预期结果
    /// - 时间戳大于2020年的时间戳（1577836800）
    /// - 第二次获取的时间戳大于等于第一次
    #[test]
    fn test_get_unix_timestamp_with_no_parameters_returns_timestamp() {
        // Arrange: 准备时间戳阈值（2020年）
        let min_timestamp = 1577836800;

        // Act: 获取Unix时间戳
        let timestamp1 = get_unix_timestamp();

        // Assert: 验证时间戳是合理的（应该在2020年之后）
        assert!(timestamp1 > min_timestamp);

        // Act: 等待一小段时间后再次获取
        thread::sleep(std::time::Duration::from_millis(10));
        let timestamp2 = get_unix_timestamp();

        // Assert: 验证时间戳递增
        assert!(timestamp2 >= timestamp1);
    }

    // ==================== Document Timestamp Format Tests ====================

    /// 测试格式化文档时间戳所有格式（UTC时区）
    ///
    /// ## 测试目的
    /// 验证 `format_document_timestamp()` 函数能够使用所有日期格式（DateOnly, DateTime, Iso8601, Filename）和UTC时区正确格式化时间戳。
    ///
    /// ## 测试场景
    /// 1. 使用各种格式和UTC时区格式化时间戳
    /// 2. 验证每种格式的输出符合预期
    ///
    /// ## 预期结果
    /// - DateOnly格式：YYYY-MM-DD
    /// - DateTime格式：YYYY-MM-DD HH:MM:SS
    /// - Iso8601格式：包含'T'和时区标识符（Z或+/-）
    /// - Filename格式：YYYY-MM-DD_HH-MM-SS（不包含空格和冒号）
    #[test]
    fn test_format_document_timestamp_all_formats_utc_with_all_formats_returns_formatted_strings(
    ) -> Result<()> {
        // Arrange: 准备各种格式的正则表达式
        let re_date = Regex::new(r"^\d{4}-\d{2}-\d{2}$")
            .map_err(|e| color_eyre::eyre::eyre!("Date only regex should be valid: {}", e))?;
        let re_datetime = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
            .map_err(|e| color_eyre::eyre::eyre!("DateTime regex should be valid: {}", e))?;
        let re_filename = Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$")
            .map_err(|e| color_eyre::eyre::eyre!("Filename regex should be valid: {}", e))?;

        // Act: 格式化各种格式的时间戳（UTC时区）
        let date_only = format_document_timestamp(DateFormat::DateOnly, Timezone::Utc);
        let datetime = format_document_timestamp(DateFormat::DateTime, Timezone::Utc);
        let iso8601 = format_document_timestamp(DateFormat::Iso8601, Timezone::Utc);
        let filename = format_document_timestamp(DateFormat::Filename, Timezone::Utc);

        // Assert: 验证所有格式正确
        assert!(re_date.is_match(&date_only));
        assert!(re_datetime.is_match(&datetime));
        assert!(iso8601.contains('T'));
        assert!(iso8601.contains('Z') || iso8601.contains('+') || iso8601.contains('-'));
        assert!(re_filename.is_match(&filename));
        assert!(!filename.contains(' '));
        assert!(!filename.contains(':'));
        Ok(())
    }
}
