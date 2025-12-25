//! Base Util Date 模块补充测试
//!
//! 测试日期时间工具中未完全覆盖的功能。
//!
//! ## 测试策略
//!
//! - 使用 `expect()` 替代 `unwrap()` 提供清晰的错误消息
//! - 测试各种日期时间格式化功能

use regex::Regex;
use workflow::base::format::date::{
    format_document_timestamp, format_filename_timestamp, get_unix_timestamp, DateFormat, Timezone,
};

// ==================== Filename Timestamp Tests ====================

#[test]
fn test_format_filename_timestamp_with_no_parameters_returns_formatted_string() {
    // Arrange: 准备文件名时间戳格式的正则表达式
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$")
        .expect("Filename timestamp regex should be valid");

    // Act: 格式化文件名时间戳
    let result = format_filename_timestamp();

    // Assert: 验证格式为YYYY-MM-DD_HH-MM-SS（适合文件名）
    assert!(
        re.is_match(&result),
        "Filename timestamp format should match YYYY-MM-DD_HH-MM-SS"
    );
}

// ==================== Unix Timestamp Tests ====================

#[test]
fn test_get_unix_timestamp_with_no_parameters_returns_timestamp() {
    // Arrange: 准备时间戳阈值（2020年）
    let min_timestamp = 1577836800;

    // Act: 获取Unix时间戳
    let timestamp1 = get_unix_timestamp();

    // Assert: 验证时间戳是合理的（应该在2020年之后）
    assert!(timestamp1 > min_timestamp);

    // Act: 等待一小段时间后再次获取
    std::thread::sleep(std::time::Duration::from_millis(10));
    let timestamp2 = get_unix_timestamp();

    // Assert: 验证时间戳递增
    assert!(timestamp2 >= timestamp1);
}

// ==================== Document Timestamp Format Tests ====================

#[test]
fn test_format_document_timestamp_all_formats_utc_with_all_formats_returns_formatted_strings() {
    // Arrange: 准备各种格式的正则表达式
    let re_date = Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("Date only regex should be valid");
    let re_datetime = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
        .expect("DateTime regex should be valid");
    let re_filename = Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$")
        .expect("Filename regex should be valid");

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
}
