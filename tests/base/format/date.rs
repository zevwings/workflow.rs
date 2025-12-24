//! Base Util Date 模块补充测试
//!
//! 测试日期时间工具中未完全覆盖的功能。

use regex::Regex;
use workflow::base::format::date::{
    format_document_timestamp, format_filename_timestamp, get_unix_timestamp, DateFormat, Timezone,
};

#[test]
fn test_format_filename_timestamp() {
    let result = format_filename_timestamp();
    // 验证格式：YYYY-MM-DD_HH-MM-SS（适合文件名）
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$").unwrap();
    assert!(
        re.is_match(&result),
        "Filename timestamp format should match YYYY-MM-DD_HH-MM-SS"
    );
}

#[test]
fn test_get_unix_timestamp() {
    let timestamp1 = get_unix_timestamp();
    // 验证时间戳是合理的（应该在 2020 年之后，即 > 1577836800）
    assert!(timestamp1 > 1577836800);

    // 等待一小段时间后再次获取，验证时间戳递增
    std::thread::sleep(std::time::Duration::from_millis(10));
    let timestamp2 = get_unix_timestamp();
    assert!(timestamp2 >= timestamp1);
}

#[test]
fn test_format_document_timestamp_all_formats_utc() {
    // 测试所有格式在 UTC 时区下的行为
    let date_only = format_document_timestamp(DateFormat::DateOnly, Timezone::Utc);
    let re_date = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    assert!(re_date.is_match(&date_only));

    let datetime = format_document_timestamp(DateFormat::DateTime, Timezone::Utc);
    let re_datetime = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
    assert!(re_datetime.is_match(&datetime));

    let iso8601 = format_document_timestamp(DateFormat::Iso8601, Timezone::Utc);
    assert!(iso8601.contains('T'));
    assert!(iso8601.contains('Z') || iso8601.contains('+') || iso8601.contains('-'));

    let filename = format_document_timestamp(DateFormat::Filename, Timezone::Utc);
    let re_filename = Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$").unwrap();
    assert!(re_filename.is_match(&filename));

    // 确保 UTC 时区的 Filename 格式被覆盖（覆盖 date.rs:71）
    // 验证格式不包含空格和冒号
    assert!(!filename.contains(' '));
    assert!(!filename.contains(':'));
}
