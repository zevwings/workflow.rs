//! Base Util Date 模块补充测试
//!
//! 测试日期时间工具中未完全覆盖的功能。
//!
//! ## 测试策略
//!
//! - 测试函数返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 测试各种日期时间格式化功能

use color_eyre::Result;
use regex::Regex;
use workflow::base::format::date::{
    format_document_timestamp, format_filename_timestamp, get_unix_timestamp, DateFormat, Timezone,
};

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
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$")
        .map_err(|e| color_eyre::eyre::eyre!("Filename timestamp regex should be valid: {}", e))?;

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
    std::thread::sleep(std::time::Duration::from_millis(10));
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
