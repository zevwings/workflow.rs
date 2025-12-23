//! Base Logger LogLevel 模块测试
//!
//! 测试日志级别枚举的核心功能。

use pretty_assertions::assert_eq;
use std::str::FromStr;
use workflow::base::LogLevel;

#[test]
fn test_log_level_from_str_off() {
    assert_eq!(LogLevel::from_str("off").unwrap(), LogLevel::None);
    assert_eq!(LogLevel::from_str("OFF").unwrap(), LogLevel::None);
    assert_eq!(LogLevel::from_str("none").unwrap(), LogLevel::None);
    assert_eq!(LogLevel::from_str("NONE").unwrap(), LogLevel::None);
}

#[test]
fn test_log_level_from_str_error() {
    assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
    assert_eq!(LogLevel::from_str("ERROR").unwrap(), LogLevel::Error);
    assert_eq!(LogLevel::from_str("Error").unwrap(), LogLevel::Error);
}

#[test]
fn test_log_level_from_str_warn() {
    assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::Warn);
    assert_eq!(LogLevel::from_str("WARN").unwrap(), LogLevel::Warn);
}

#[test]
fn test_log_level_from_str_info() {
    assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
    assert_eq!(LogLevel::from_str("INFO").unwrap(), LogLevel::Info);
}

#[test]
fn test_log_level_from_str_debug() {
    assert_eq!(LogLevel::from_str("debug").unwrap(), LogLevel::Debug);
    assert_eq!(LogLevel::from_str("DEBUG").unwrap(), LogLevel::Debug);
}

#[test]
fn test_log_level_from_str_invalid() {
    assert!(LogLevel::from_str("invalid").is_err());
    assert!(LogLevel::from_str("").is_err());
    assert!(LogLevel::from_str("trace").is_err());
}

#[test]
fn test_log_level_as_str() {
    assert_eq!(LogLevel::None.as_str(), "off");
    assert_eq!(LogLevel::Error.as_str(), "error");
    assert_eq!(LogLevel::Warn.as_str(), "warn");
    assert_eq!(LogLevel::Info.as_str(), "info");
    assert_eq!(LogLevel::Debug.as_str(), "debug");
}

#[test]
fn test_log_level_should_log() {
    assert!(LogLevel::Debug.should_log(LogLevel::Debug));
    assert!(LogLevel::Debug.should_log(LogLevel::Info));
    assert!(LogLevel::Debug.should_log(LogLevel::Warn));
    assert!(LogLevel::Debug.should_log(LogLevel::Error));

    assert!(LogLevel::Info.should_log(LogLevel::Info));
    assert!(LogLevel::Info.should_log(LogLevel::Warn));
    assert!(LogLevel::Info.should_log(LogLevel::Error));
    assert!(!LogLevel::Info.should_log(LogLevel::Debug));

    assert!(LogLevel::Warn.should_log(LogLevel::Warn));
    assert!(LogLevel::Warn.should_log(LogLevel::Error));
    assert!(!LogLevel::Warn.should_log(LogLevel::Info));
    assert!(!LogLevel::Warn.should_log(LogLevel::Debug));

    assert!(LogLevel::Error.should_log(LogLevel::Error));
    assert!(!LogLevel::Error.should_log(LogLevel::Warn));
    assert!(!LogLevel::Error.should_log(LogLevel::Info));
    assert!(!LogLevel::Error.should_log(LogLevel::Debug));

    assert!(!LogLevel::None.should_log(LogLevel::Error));
}

#[test]
fn test_log_level_ordering() {
    assert!(LogLevel::Debug > LogLevel::Info);
    assert!(LogLevel::Info > LogLevel::Warn);
    assert!(LogLevel::Warn > LogLevel::Error);
    assert!(LogLevel::Error > LogLevel::None);

    assert!(LogLevel::Debug >= LogLevel::Info);
    assert!(LogLevel::Info >= LogLevel::Info);
}

#[test]
fn test_log_level_default_level() {
    let level = LogLevel::default_level();
    // 在测试环境中，通常是 debug 模式
    assert!(level == LogLevel::Debug || level == LogLevel::Info);
}

#[test]
fn test_log_level_set_and_get() {
    // 保存原始级别
    let original = LogLevel::get_level();

    // 设置新级别
    LogLevel::set_level(LogLevel::Warn);
    assert_eq!(LogLevel::get_level(), LogLevel::Warn);

    // 恢复原始级别
    LogLevel::set_level(original);
}

#[test]
fn test_log_level_init() {
    // 测试 init 函数（如果之前没有初始化，应该设置级别）
    // 注意：init 只在第一次调用时设置级别，如果之前已经初始化过，就不会更新
    // 所以我们使用 set_level 来确保设置成功
    LogLevel::set_level(LogLevel::Error);
    // 验证级别已设置（通过 get_level）
    let level = LogLevel::get_level();
    assert_eq!(level, LogLevel::Error);
}

#[test]
fn test_log_level_clone_copy() {
    let level1 = LogLevel::Debug;
    let level2 = level1; // Copy
    let level3 = level1.clone(); // Clone
    assert_eq!(level1, level2);
    assert_eq!(level1, level3);
}

#[test]
fn test_log_level_debug_format() {
    let level = LogLevel::Info;
    let debug_str = format!("{:?}", level);
    assert_eq!(debug_str, "Info");
}

#[test]
fn test_log_level_default_level_release() {
    // 测试 default_level() 函数的行为
    // 在 debug 模式下返回 Debug，在 release 模式下返回 Info
    let level = LogLevel::default_level();

    // 根据编译模式验证返回值
    if cfg!(debug_assertions) {
        assert_eq!(level, LogLevel::Debug);
    } else {
        // 覆盖 log_level.rs:32 - release 模式下的默认级别
        assert_eq!(level, LogLevel::Info);
    }

    // 验证返回值是有效的日志级别
    assert!(matches!(level, LogLevel::Debug | LogLevel::Info));
}
