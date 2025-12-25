//! Base Logger LogLevel 模块测试
//!
//! 测试日志级别枚举的核心功能。

use pretty_assertions::assert_eq;
use std::str::FromStr;
use workflow::base::LogLevel;

#[test]
fn test_log_level_from_str_off() {
    assert_eq!(
        LogLevel::from_str("off").expect("'off' should parse"),
        LogLevel::None
    );
    assert_eq!(
        LogLevel::from_str("OFF").expect("'OFF' should parse"),
        LogLevel::None
    );
    assert_eq!(
        LogLevel::from_str("none").expect("'none' should parse"),
        LogLevel::None
    );
    assert_eq!(
        LogLevel::from_str("NONE").expect("'NONE' should parse"),
        LogLevel::None
    );
}

#[test]
fn test_log_level_from_str_error() {
    assert_eq!(
        LogLevel::from_str("error").expect("'error' should parse"),
        LogLevel::Error
    );
    assert_eq!(
        LogLevel::from_str("ERROR").expect("'ERROR' should parse"),
        LogLevel::Error
    );
    assert_eq!(
        LogLevel::from_str("Error").expect("'Error' should parse"),
        LogLevel::Error
    );
}

#[test]
fn test_log_level_from_str_warn() {
    assert_eq!(
        LogLevel::from_str("warn").expect("'warn' should parse"),
        LogLevel::Warn
    );
    assert_eq!(
        LogLevel::from_str("WARN").expect("'WARN' should parse"),
        LogLevel::Warn
    );
}

#[test]
fn test_log_level_from_str_info() {
    assert_eq!(
        LogLevel::from_str("info").expect("'info' should parse"),
        LogLevel::Info
    );
    assert_eq!(
        LogLevel::from_str("INFO").expect("'INFO' should parse"),
        LogLevel::Info
    );
}

#[test]
fn test_log_level_from_str_debug() {
    assert_eq!(
        LogLevel::from_str("debug").expect("'debug' should parse"),
        LogLevel::Debug
    );
    assert_eq!(
        LogLevel::from_str("DEBUG").expect("'DEBUG' should parse"),
        LogLevel::Debug
    );
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
    // 根据编译模式验证返回值
    if cfg!(debug_assertions) {
        assert_eq!(level, LogLevel::Debug);
    } else {
        assert_eq!(level, LogLevel::Info);
    }
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
    // 测试 init 函数的行为：只在未初始化时设置级别
    // 先清理状态（通过设置一个已知值，然后重置）
    LogLevel::set_level(LogLevel::Error);

    // 测试 init(None) - 应该使用默认级别
    // 由于已经初始化过，init 不会改变当前级别
    let before_init = LogLevel::get_level();
    LogLevel::init(None);
    let after_init = LogLevel::get_level();
    // init 不会改变已初始化的级别
    assert_eq!(before_init, after_init);

    // 测试 init(Some(level)) - 如果已初始化，不会改变
    LogLevel::init(Some(LogLevel::Warn));
    let after_init_with_level = LogLevel::get_level();
    // 由于已经初始化，init 不会改变级别
    assert_eq!(after_init, after_init_with_level);
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
