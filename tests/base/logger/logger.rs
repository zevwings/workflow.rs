//! Logger 模块测试
//!
//! 测试日志相关的功能，包括日志级别、日志输出格式和 tracing 宏等。
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 使用 `ok_or_else()` 替代 `unwrap()` 处理 Option 类型
//! - 测试日志级别解析、日志输出格式和 tracing 宏

use color_eyre::Result;
use pretty_assertions::assert_eq;
use workflow::base::logger::console::{debug, error, info, success, warning};
use workflow::base::logger::LogLevel;
use workflow::trace_debug;
use workflow::trace_error;
use workflow::trace_info;
use workflow::trace_warn;

// ==================== Console Logger 测试 ====================

#[test]
fn test_logger_output() -> Result<()> {
    // 测试成功消息（使用 ASCII 字符 ✓）
    let success_msg = success("Test");
    assert!(success_msg.contains("Test"));
    assert!(success_msg.contains("✓"));

    // 测试错误消息（使用 ASCII 字符 ✗）
    let error_msg = error("Test");
    assert!(error_msg.contains("Test"));
    assert!(error_msg.contains("✗"));

    // 测试警告消息（使用 ASCII 字符 ⚠）
    let warning_msg = warning("Test");
    assert!(warning_msg.contains("Test"));
    assert!(warning_msg.contains("⚠"));

    // 测试信息消息（使用 ASCII 字符 ℹ）
    let info_msg = info("Test");
    assert!(info_msg.contains("Test"));
    assert!(info_msg.contains("ℹ"));

    // 测试调试消息（使用 ASCII 字符 ⚙）
    let debug_msg = debug("Test");
    assert!(debug_msg.contains("Test"));
    assert!(debug_msg.contains("⚙"));
Ok(())
}

#[test]
fn test_colors_have_space() {
    // 测试所有颜色函数都在 Emoji 和文本之间有空格
    let success_msg = success("Test");
    let error_msg = error("Test");
    let warning_msg = warning("Test");
    let info_msg = info("Test");
    let debug_msg = debug("Test");

    // 检查是否包含空格
    assert!(
        success_msg.contains(' '),
        "Success message should contain a space"
    );
    assert!(
        error_msg.contains(' '),
        "Error message should contain a space"
    );
    assert!(
        warning_msg.contains(' '),
        "Warning message should contain a space"
    );
    assert!(
        info_msg.contains(' '),
        "Info message should contain a space"
    );
    assert!(
        debug_msg.contains(' '),
        "Debug message should contain a space"
    );

    // 检查空格的位置（应该在 Emoji 之后）
    let info_space_pos = info_msg.find(' ').expect("No space in info message");
    let success_space_pos = success_msg.find(' ').expect("No space in success message");

    // 打印调试信息
    println!("Info message: '{}'", info_msg);
    println!("Info message length: {}", info_msg.len());
    println!("Info space position: {}", info_space_pos);
    println!("Success message: '{}'", success_msg);
    println!("Success message length: {}", success_msg.len());
    println!("Success space position: {}", success_space_pos);

    // 验证格式：图标 + 空格 + 文本
    // 去除 ANSI 转义码后再比较（在 CI 环境中可能有颜色代码）
    let strip_ansi = |s: &str| -> String {
        // 简单的 ANSI 转义码去除（匹配 \u{1b}[...m 格式）
        s.replace("\u{1b}[0m", "")
            .replace("\u{1b}[31m", "")
            .replace("\u{1b}[32m", "")
            .replace("\u{1b}[33m", "")
            .replace("\u{1b}[34m", "")
            .replace("\u{1b}[90m", "")
            .replace("\u{1b}[1m", "")
            .replace("\u{1b}[22m", "")
    };

    let info_clean = strip_ansi(&info_msg);
    let info_parts: Vec<&str> = info_clean.splitn(2, ' ').collect();
    assert_eq!(
        info_parts.len(),
        2,
        "Info message should split into 2 parts at space"
    );
    assert!(
        info_parts[0].contains("ℹ"),
        "First part should contain icon: '{}'",
        info_parts[0]
    );
    assert_eq!(
        info_parts[1], "Test",
        "Second part should be the text: '{}'",
        info_parts[1]
    );
}

// ==================== LogLevel 测试 ====================

#[test]
fn test_log_level_from_str() {
    // 测试有效的日志级别字符串（不区分大小写）
    // 支持 "off"（新格式）和 "none"（向后兼容）
    assert_eq!(
        "off".parse::<LogLevel>()
            .expect("'off' should parse to LogLevel"),
        LogLevel::None
    );
    assert_eq!(
        "OFF".parse::<LogLevel>()
            .expect("'OFF' should parse to LogLevel"),
        LogLevel::None
    );
    assert_eq!(
        "none".parse::<LogLevel>()
            .expect("'none' should parse to LogLevel"),
        LogLevel::None
    );
    assert_eq!(
        "NONE".parse::<LogLevel>()
            .expect("'NONE' should parse to LogLevel"),
        LogLevel::None
    );
    assert_eq!(
        "error".parse::<LogLevel>()
            .expect("'error' should parse to LogLevel"),
        LogLevel::Error
    );
    assert_eq!(
        "ERROR".parse::<LogLevel>()
            .expect("'ERROR' should parse to LogLevel"),
        LogLevel::Error
    );
    assert_eq!(
        "warn".parse::<LogLevel>()
            .expect("'warn' should parse to LogLevel"),
        LogLevel::Warn
    );
    assert_eq!(
        "WARN".parse::<LogLevel>()
            .expect("'WARN' should parse to LogLevel"),
        LogLevel::Warn
    );
    assert_eq!(
        "info".parse::<LogLevel>()
            .expect("'info' should parse to LogLevel"),
        LogLevel::Info
    );
    assert_eq!(
        "INFO".parse::<LogLevel>()
            .expect("'INFO' should parse to LogLevel"),
        LogLevel::Info
    );
    assert_eq!(
        "debug".parse::<LogLevel>()
            .expect("'debug' should parse to LogLevel"),
        LogLevel::Debug
    );
    assert_eq!(
        "DEBUG".parse::<LogLevel>()
            .expect("'DEBUG' should parse to LogLevel"),
        LogLevel::Debug
    );

    // 测试无效的日志级别字符串
    assert!("invalid".parse::<LogLevel>().is_err());
    assert!("".parse::<LogLevel>().is_err());
    assert!("trace".parse::<LogLevel>().is_err());
}

#[test]
fn test_log_level_as_str() -> Result<()> {
    assert_eq!(LogLevel::None.as_str(), "off");
    assert_eq!(LogLevel::Error.as_str(), "error");
    assert_eq!(LogLevel::Warn.as_str(), "warn");
    assert_eq!(LogLevel::Info.as_str(), "info");
    assert_eq!(LogLevel::Debug.as_str(), "debug");
Ok(())
}

#[test]
fn test_log_level_ordering() -> Result<()> {
    // 测试日志级别的顺序
    assert!(LogLevel::None < LogLevel::Error);
    assert!(LogLevel::Error < LogLevel::Warn);
    assert!(LogLevel::Warn < LogLevel::Info);
    assert!(LogLevel::Info < LogLevel::Debug);

    // 测试 should_log 方法
    let debug_level = LogLevel::Debug;
    assert!(debug_level.should_log(LogLevel::None));
    assert!(debug_level.should_log(LogLevel::Error));
    assert!(debug_level.should_log(LogLevel::Warn));
    assert!(debug_level.should_log(LogLevel::Info));
    assert!(debug_level.should_log(LogLevel::Debug));

    let info_level = LogLevel::Info;
    assert!(info_level.should_log(LogLevel::None));
    assert!(info_level.should_log(LogLevel::Error));
    assert!(info_level.should_log(LogLevel::Warn));
    assert!(info_level.should_log(LogLevel::Info));
    assert!(!info_level.should_log(LogLevel::Debug));

    let warn_level = LogLevel::Warn;
    assert!(warn_level.should_log(LogLevel::None));
    assert!(warn_level.should_log(LogLevel::Error));
    assert!(warn_level.should_log(LogLevel::Warn));
    assert!(!warn_level.should_log(LogLevel::Info));
    assert!(!warn_level.should_log(LogLevel::Debug));

    let error_level = LogLevel::Error;
    assert!(error_level.should_log(LogLevel::None));
    assert!(error_level.should_log(LogLevel::Error));
    assert!(!error_level.should_log(LogLevel::Warn));
    assert!(!error_level.should_log(LogLevel::Info));
    assert!(!error_level.should_log(LogLevel::Debug));

    let none_level = LogLevel::None;
    assert!(none_level.should_log(LogLevel::None));
    assert!(!none_level.should_log(LogLevel::Error));
    assert!(!none_level.should_log(LogLevel::Warn));
    assert!(!none_level.should_log(LogLevel::Info));
    assert!(!none_level.should_log(LogLevel::Debug));
Ok(())
}

#[test]
fn test_log_level_set_and_get() -> Result<()> {
    // 保存原始级别
    let original_level = LogLevel::get_level();

    // 测试设置和获取不同的日志级别
    LogLevel::set_level(LogLevel::Debug);
    assert_eq!(LogLevel::get_level(), LogLevel::Debug);

    LogLevel::set_level(LogLevel::Info);
    assert_eq!(LogLevel::get_level(), LogLevel::Info);

    LogLevel::set_level(LogLevel::Warn);
    assert_eq!(LogLevel::get_level(), LogLevel::Warn);

    LogLevel::set_level(LogLevel::Error);
    assert_eq!(LogLevel::get_level(), LogLevel::Error);

    LogLevel::set_level(LogLevel::None);
    assert_eq!(LogLevel::get_level(), LogLevel::None);

    // 恢复原始级别
    LogLevel::set_level(original_level);
Ok(())
}

#[test]
fn test_log_level_default() -> Result<()> {
    // 测试默认级别（根据编译模式）
    let default = LogLevel::default_level();

    // 在 debug 模式下应该是 Debug，在 release 模式下应该是 Info
    if cfg!(debug_assertions) {
        assert_eq!(default, LogLevel::Debug);
    } else {
        assert_eq!(default, LogLevel::Info);
    }
Ok(())
}

#[test]
fn test_log_level_round_trip() {
    // 测试字符串转换的往返一致性
    let levels = vec![
        LogLevel::None,
        LogLevel::Error,
        LogLevel::Warn,
        LogLevel::Info,
        LogLevel::Debug,
    ];

    for level in levels {
        let level_str = level.as_str();
        let parsed = level_str
            .parse::<LogLevel>()
            .expect("LogLevel round trip should succeed");
        assert_eq!(level, parsed, "Round trip failed for level: {}", level_str);
    }
}

// ==================== Tracing 宏测试 ====================

#[test]
fn test_tracing_macros() -> Result<()> {
    // 这些宏应该可以编译和运行（即使不输出）
    trace_debug!("Test debug message");
    trace_info!("Test info message");
    trace_warn!("Test warn message");
    trace_error!("Test error message");
Ok(())
}
