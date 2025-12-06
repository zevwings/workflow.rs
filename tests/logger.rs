//! Logger 测试
//!
//! 测试日志相关的功能，包括日志级别、日志输出格式等。

use workflow::base::util::logger::{LogLevel, Logger};

#[test]
fn test_logger_output() {
    assert!(Logger::success("Test").contains("✓"));
    assert!(Logger::error("Test").contains("✗"));
    assert!(Logger::warning("Test").contains("⚠"));
    assert!(Logger::info("Test").contains("ℹ"));
    assert!(Logger::debug("Test").contains("⚙"));
}

#[test]
fn test_log_level_from_str() {
    // 测试有效的日志级别字符串（不区分大小写）
    assert_eq!("none".parse::<LogLevel>().unwrap(), LogLevel::None);
    assert_eq!("NONE".parse::<LogLevel>().unwrap(), LogLevel::None);
    assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
    assert_eq!("ERROR".parse::<LogLevel>().unwrap(), LogLevel::Error);
    assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
    assert_eq!("WARN".parse::<LogLevel>().unwrap(), LogLevel::Warn);
    assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
    assert_eq!("INFO".parse::<LogLevel>().unwrap(), LogLevel::Info);
    assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
    assert_eq!("DEBUG".parse::<LogLevel>().unwrap(), LogLevel::Debug);

    // 测试无效的日志级别字符串
    assert!("invalid".parse::<LogLevel>().is_err());
    assert!("".parse::<LogLevel>().is_err());
    assert!("trace".parse::<LogLevel>().is_err());
}

#[test]
fn test_log_level_as_str() {
    assert_eq!(LogLevel::None.as_str(), "none");
    assert_eq!(LogLevel::Error.as_str(), "error");
    assert_eq!(LogLevel::Warn.as_str(), "warn");
    assert_eq!(LogLevel::Info.as_str(), "info");
    assert_eq!(LogLevel::Debug.as_str(), "debug");
}

#[test]
fn test_log_level_ordering() {
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
}

#[test]
fn test_log_level_set_and_get() {
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
}

#[test]
fn test_log_level_default() {
    // 测试默认级别（根据编译模式）
    let default = LogLevel::default_level();

    // 在 debug 模式下应该是 Debug，在 release 模式下应该是 Info
    if cfg!(debug_assertions) {
        assert_eq!(default, LogLevel::Debug);
    } else {
        assert_eq!(default, LogLevel::Info);
    }
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
        let parsed = level_str.parse::<LogLevel>().unwrap();
        assert_eq!(level, parsed, "Round trip failed for level: {}", level_str);
    }
}
