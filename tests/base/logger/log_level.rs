//! Base Logger LogLevel 模块测试
//!
//! 测试日志级别枚举的核心功能。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use std::str::FromStr;
use workflow::base::LogLevel;

// ==================== LogLevel Parsing Tests ====================

/// 测试从字符串解析日志级别（off/none）
///
/// ## 测试目的
/// 验证 LogLevel::from_str() 能够解析各种大小写的 "off" 和 "none" 字符串。
///
/// ## 测试场景
/// 1. 使用各种大小写的 "off" 和 "none" 字符串解析
/// 2. 验证都解析为 LogLevel::None
///
/// ## 预期结果
/// - 所有变体都解析为 LogLevel::None
#[test]
fn test_log_level_from_str_off_with_various_cases_returns_none() -> Result<()> {
    // Arrange: 准备各种大小写的 "off" 和 "none" 字符串
    let inputs = ["off", "OFF", "none", "NONE"];

    // Act & Assert: 验证所有变体都解析为 None
    for input in inputs.iter() {
        let level = LogLevel::from_str(input)
            .map_err(|e| color_eyre::eyre::eyre!("'{}' should parse: {}", input, e))?;
        assert_eq!(level, LogLevel::None);
    }
    Ok(())
}

/// 测试从字符串解析日志级别（error）
///
/// ## 测试目的
/// 验证 LogLevel::from_str() 能够解析各种大小写的 "error" 字符串。
///
/// ## 测试场景
/// 1. 使用各种大小写的 "error" 字符串解析
/// 2. 验证都解析为 LogLevel::Error
///
/// ## 预期结果
/// - 所有变体都解析为 LogLevel::Error
#[test]
fn test_log_level_from_str_error_with_various_cases_returns_error() -> Result<()> {
    // Arrange: 准备各种大小写的 "error" 字符串
    let inputs = ["error", "ERROR", "Error"];

    // Act & Assert: 验证所有变体都解析为 Error
    for input in inputs.iter() {
        let level = LogLevel::from_str(input)
            .map_err(|e| color_eyre::eyre::eyre!("'{}' should parse: {}", input, e))?;
        assert_eq!(level, LogLevel::Error);
    }
    Ok(())
}

/// 测试从字符串解析日志级别（warn）
///
/// ## 测试目的
/// 验证 LogLevel::from_str() 能够解析各种大小写的 "warn" 字符串。
///
/// ## 测试场景
/// 1. 使用各种大小写的 "warn" 字符串解析
/// 2. 验证都解析为 LogLevel::Warn
///
/// ## 预期结果
/// - 所有变体都解析为 LogLevel::Warn
#[test]
fn test_log_level_from_str_warn_with_various_cases_returns_warn() -> Result<()> {
    // Arrange: 准备各种大小写的 "warn" 字符串
    let inputs = ["warn", "WARN"];

    // Act & Assert: 验证所有变体都解析为 Warn
    for input in inputs.iter() {
        let level = LogLevel::from_str(input)
            .map_err(|e| color_eyre::eyre::eyre!("'{}' should parse: {}", input, e))?;
        assert_eq!(level, LogLevel::Warn);
    }
    Ok(())
}

/// 测试从字符串解析日志级别（info）
///
/// ## 测试目的
/// 验证 LogLevel::from_str() 能够解析各种大小写的 "info" 字符串。
///
/// ## 测试场景
/// 1. 使用各种大小写的 "info" 字符串解析
/// 2. 验证都解析为 LogLevel::Info
///
/// ## 预期结果
/// - 所有变体都解析为 LogLevel::Info
#[test]
fn test_log_level_from_str_info_with_various_cases_returns_info() -> Result<()> {
    // Arrange: 准备各种大小写的 "info" 字符串
    let inputs = ["info", "INFO"];

    // Act & Assert: 验证所有变体都解析为 Info
    for input in inputs.iter() {
        let level = LogLevel::from_str(input)
            .map_err(|e| color_eyre::eyre::eyre!("'{}' should parse: {}", input, e))?;
        assert_eq!(level, LogLevel::Info);
    }
    Ok(())
}

/// 测试从字符串解析日志级别（debug）
///
/// ## 测试目的
/// 验证 LogLevel::from_str() 能够解析各种大小写的 "debug" 字符串。
///
/// ## 测试场景
/// 1. 使用各种大小写的 "debug" 字符串解析
/// 2. 验证都解析为 LogLevel::Debug
///
/// ## 预期结果
/// - 所有变体都解析为 LogLevel::Debug
#[test]
fn test_log_level_from_str_debug_with_various_cases_returns_debug() -> Result<()> {
    // Arrange: 准备各种大小写的 "debug" 字符串
    let inputs = ["debug", "DEBUG"];

    // Act & Assert: 验证所有变体都解析为 Debug
    for input in inputs.iter() {
        let level = LogLevel::from_str(input)
            .map_err(|e| color_eyre::eyre::eyre!("'{}' should parse: {}", input, e))?;
        assert_eq!(level, LogLevel::Debug);
    }
    Ok(())
}

/// 测试从无效字符串解析日志级别
///
/// ## 测试目的
/// 验证 LogLevel::from_str() 对无效字符串返回错误。
///
/// ## 测试场景
/// 1. 使用无效的字符串解析日志级别
/// 2. 验证返回错误
///
/// ## 预期结果
/// - 无效输入返回解析错误
#[test]
fn test_log_level_from_str_invalid_with_invalid_strings_returns_error() {
    // Arrange: 准备无效的字符串
    let invalid_inputs = ["invalid", "", "trace"];

    // Act & Assert: 验证所有无效输入都返回错误
    for input in invalid_inputs.iter() {
        assert!(
            LogLevel::from_str(input).is_err(),
            "Input '{}' should fail to parse",
            input
        );
    }
}

// ==================== LogLevel String Conversion Tests ====================

/// 测试获取日志级别的字符串表示
///
/// ## 测试目的
/// 验证 LogLevel::as_str() 返回正确的字符串表示。
///
/// ## 测试场景
/// 1. 获取所有日志级别的字符串表示
/// 2. 验证字符串表示正确
///
/// ## 预期结果
/// - 所有级别都返回正确的字符串表示
#[test]
fn test_log_level_as_str_with_all_levels_returns_string_representation() {
    // Arrange: 准备所有日志级别和对应的字符串表示
    let levels = [
        (LogLevel::None, "off"),
        (LogLevel::Error, "error"),
        (LogLevel::Warn, "warn"),
        (LogLevel::Info, "info"),
        (LogLevel::Debug, "debug"),
    ];

    // Act & Assert: 验证每个级别的字符串表示正确
    for (level, expected_str) in levels.iter() {
        assert_eq!(level.as_str(), *expected_str);
    }
}

// ==================== LogLevel Should Log Tests ====================

/// 测试日志级别是否应该记录
///
/// ## 测试目的
/// 验证 LogLevel::should_log() 能够正确判断是否应该记录某个级别的日志。
///
/// ## 测试场景
/// 1. 测试不同日志级别组合的 should_log 结果
/// 2. 验证逻辑正确（高级别应该记录低级别）
///
/// ## 预期结果
/// - Debug 记录所有级别，Info 记录 Info 及以上，Warn 记录 Warn 及以上，Error 只记录 Error，None 不记录任何级别
#[test]
fn test_log_level_should_log_with_various_levels_returns_correct_result() {
    // Arrange: 准备各种日志级别组合

    // Act & Assert: 验证 Debug 级别应该记录所有级别
    assert!(LogLevel::Debug.should_log(LogLevel::Debug));
    assert!(LogLevel::Debug.should_log(LogLevel::Info));
    assert!(LogLevel::Debug.should_log(LogLevel::Warn));
    assert!(LogLevel::Debug.should_log(LogLevel::Error));

    // Act & Assert: 验证 Info 级别应该记录 Info 及以上级别
    assert!(LogLevel::Info.should_log(LogLevel::Info));
    assert!(LogLevel::Info.should_log(LogLevel::Warn));
    assert!(LogLevel::Info.should_log(LogLevel::Error));
    assert!(!LogLevel::Info.should_log(LogLevel::Debug));

    // Act & Assert: 验证 Warn 级别应该记录 Warn 及以上级别
    assert!(LogLevel::Warn.should_log(LogLevel::Warn));
    assert!(LogLevel::Warn.should_log(LogLevel::Error));
    assert!(!LogLevel::Warn.should_log(LogLevel::Info));
    assert!(!LogLevel::Warn.should_log(LogLevel::Debug));

    // Act & Assert: 验证 Error 级别只应该记录 Error
    assert!(LogLevel::Error.should_log(LogLevel::Error));
    assert!(!LogLevel::Error.should_log(LogLevel::Warn));
    assert!(!LogLevel::Error.should_log(LogLevel::Info));
    assert!(!LogLevel::Error.should_log(LogLevel::Debug));

    // Act & Assert: 验证 None 级别不应该记录任何级别
    assert!(!LogLevel::None.should_log(LogLevel::Error));
}

// ==================== LogLevel Ordering Tests ====================

/// 测试日志级别排序
///
/// ## 测试目的
/// 验证 LogLevel 的排序关系正确。
///
/// ## 测试场景
/// 1. 比较不同日志级别的大小关系
/// 2. 验证排序正确
///
/// ## 预期结果
/// - Debug > Info > Warn > Error > None
#[test]
fn test_log_level_ordering_with_various_levels_returns_correct_order() {
    // Arrange: 准备各种日志级别

    // Act & Assert: 验证级别顺序（Debug > Info > Warn > Error > None）
    assert!(LogLevel::Debug > LogLevel::Info);
    assert!(LogLevel::Info > LogLevel::Warn);
    assert!(LogLevel::Warn > LogLevel::Error);
    assert!(LogLevel::Error > LogLevel::None);
    assert!(LogLevel::Debug >= LogLevel::Info);
    assert!(LogLevel::Info >= LogLevel::Info);
}

// ==================== LogLevel Default and Management Tests ====================

/// 测试获取默认日志级别
///
/// ## 测试目的
/// 验证 LogLevel::default_level() 根据编译模式返回正确的默认级别。
///
/// ## 测试场景
/// 1. 调用 default_level() 获取默认级别
/// 2. 根据编译模式验证返回值
///
/// ## 预期结果
/// - Debug 模式返回 Debug，Release 模式返回 Info
#[test]
fn test_log_level_default_level_with_no_parameters_returns_default() {
    // Arrange: 准备获取默认级别

    // Act: 获取默认级别
    let level = LogLevel::default_level();

    // Assert: 根据编译模式验证返回值
    if cfg!(debug_assertions) {
        assert_eq!(level, LogLevel::Debug);
    } else {
        assert_eq!(level, LogLevel::Info);
    }
}

/// 测试设置和获取日志级别
///
/// ## 测试目的
/// 验证 LogLevel::set_level() 和 get_level() 能够正确设置和获取全局日志级别。
///
/// ## 测试场景
/// 1. 保存原始级别
/// 2. 设置新级别
/// 3. 验证级别已设置
/// 4. 恢复原始级别
///
/// ## 预期结果
/// - 级别能够正确设置和获取
#[test]
fn test_log_level_set_and_get_with_valid_level_sets_and_gets_level() {
    // Arrange: 保存原始级别
    let original = LogLevel::get_level();

    // Act: 设置新级别
    LogLevel::set_level(LogLevel::Warn);

    // Assert: 验证级别已设置
    assert_eq!(LogLevel::get_level(), LogLevel::Warn);

    // 恢复原始级别
    LogLevel::set_level(original);
}

/// 测试初始化日志级别
///
/// ## 测试目的
/// 验证 LogLevel::init() 只在未初始化时设置级别。
///
/// ## 测试场景
/// 1. 设置一个已知级别
/// 2. 调用 init(None) 和 init(Some(level))
/// 3. 验证已初始化后 init 不会改变级别
///
/// ## 预期结果
/// - init 只在未初始化时设置级别，已初始化后不会改变
#[test]
fn test_log_level_init() {
    // Arrange: 准备测试 init 函数的行为：只在未初始化时设置级别
    // 先清理状态（通过设置一个已知值，然后重置）
    LogLevel::set_level(LogLevel::Error);

    // Arrange: 准备测试 init(None) - 应该使用默认级别
    // 由于已经初始化过，init 不会改变当前级别
    let before_init = LogLevel::get_level();
    LogLevel::init(None);
    let after_init = LogLevel::get_level();
    // init 不会改变已初始化的级别
    assert_eq!(before_init, after_init);

    // Arrange: 准备测试 init(Some(level)) - 如果已初始化，不会改变
    LogLevel::init(Some(LogLevel::Warn));
    let after_init_with_level = LogLevel::get_level();
    // 由于已经初始化，init 不会改变级别
    assert_eq!(after_init, after_init_with_level);
}

/// 测试日志级别克隆和复制
///
/// ## 测试目的
/// 验证 LogLevel 的 Clone 和 Copy trait 实现正确。
///
/// ## 测试场景
/// 1. 创建日志级别
/// 2. 复制和克隆级别
/// 3. 验证值相等
///
/// ## 预期结果
/// - 复制和克隆后的值与原值相等
#[test]
fn test_log_level_clone_copy() {
    let level1 = LogLevel::Debug;
    let level2 = level1; // Copy
    let level3 = level1; // Clone
    assert_eq!(level1, level2);
    assert_eq!(level1, level3);
}

/// 测试日志级别 Debug 格式化
///
/// ## 测试目的
/// 验证 LogLevel 的 Debug trait 实现正确。
///
/// ## 测试场景
/// 1. 格式化日志级别为 Debug 字符串
/// 2. 验证输出格式正确
///
/// ## 预期结果
/// - Debug 输出包含级别名称
#[test]
fn test_log_level_debug_format() {
    let level = LogLevel::Info;
    let debug_str = format!("{:?}", level);
    assert_eq!(debug_str, "Info");
}

/// 测试默认日志级别（Release 模式）
///
/// ## 测试目的
/// 验证 default_level() 在 Release 模式下返回 Info。
///
/// ## 测试场景
/// 1. 调用 default_level() 获取默认级别
/// 2. 根据编译模式验证返回值
///
/// ## 预期结果
/// - Debug 模式返回 Debug，Release 模式返回 Info
#[test]
fn test_log_level_default_level_release() {
    // Arrange: 准备测试 default_level() 函数的行为
    // 在 debug 模式下返回 Debug，在 release 模式下返回 Info
    let level = LogLevel::default_level();

    // 根据编译模式验证返回值
    if cfg!(debug_assertions) {
        assert_eq!(level, LogLevel::Debug);
    } else {
        // 覆盖 log_level.rs:32 - release 模式下的默认级别
        assert_eq!(level, LogLevel::Info);
    }

    // Assert: 验证返回值是有效的日志级别
    assert!(matches!(level, LogLevel::Debug | LogLevel::Info));
}
