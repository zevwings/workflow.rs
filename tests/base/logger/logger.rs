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

// ==================== Console Logger Tests ====================

/// 测试所有日志级别的输出格式
///
/// ## 测试目的
/// 验证各个日志级别函数（success, error, warning, info, debug）能够返回正确格式化的消息，包含文本和对应的图标。
///
/// ## 测试场景
/// 1. 调用各个日志级别的函数（success, error, warning, info, debug）
/// 2. 验证每个消息都包含测试文本和对应的图标
///
/// ## 预期结果
/// - success消息包含 "✓" 图标
/// - error消息包含 "✗" 图标
/// - warning消息包含 "⚠" 图标
/// - info消息包含 "ℹ" 图标
/// - debug消息包含 "⚙" 图标
/// - 所有消息都包含原始文本
#[test]
fn test_logger_output_with_all_levels_return_collect() -> Result<()> {
    // Arrange: 准备测试消息
    let test_message = "Test";

    // Act: 调用各个日志级别的函数
    let success_msg = success(test_message);
    let error_msg = error(test_message);
    let warning_msg = warning(test_message);
    let info_msg = info(test_message);
    let debug_msg = debug(test_message);

    // Assert: 验证每个消息都包含文本和对应的图标
    assert!(success_msg.contains(test_message));
    assert!(success_msg.contains("✓"));
    assert!(error_msg.contains(test_message));
    assert!(error_msg.contains("✗"));
    assert!(warning_msg.contains(test_message));
    assert!(warning_msg.contains("⚠"));
    assert!(info_msg.contains(test_message));
    assert!(info_msg.contains("ℹ"));
    assert!(debug_msg.contains(test_message));
    assert!(debug_msg.contains("⚙"));
    Ok(())
}

/// 测试日志消息格式：图标和文本之间有空格
///
/// ## 测试目的
/// 验证所有日志级别的消息格式正确，图标和文本之间包含空格，格式为"图标 + 空格 + 文本"。
///
/// ## 测试场景
/// 1. 调用各个日志级别的函数
/// 2. 验证所有消息都包含空格
/// 3. 去除ANSI转义码后验证info消息的格式
///
/// ## 预期结果
/// - 所有消息都在图标和文本之间包含空格
/// - info消息去除ANSI码后可以分割为两部分：图标部分和文本部分
/// - 图标部分包含 "ℹ"，文本部分为原始消息
#[test]
fn test_colors_have_space_between_icon_and_text() {
    // Arrange: 准备测试消息
    let test_message = "Test";

    // Act: 调用各个日志级别的函数
    let success_msg = success(test_message);
    let error_msg = error(test_message);
    let warning_msg = warning(test_message);
    let info_msg = info(test_message);
    let debug_msg = debug(test_message);

    // Assert: 验证所有消息都在图标和文本之间包含空格
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

    // Assert: 验证格式（图标 + 空格 + 文本）
    // 去除 ANSI 转义码后再验证格式（在 CI 环境中可能有颜色代码）
    let strip_ansi = |s: &str| -> String {
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

// ==================== LogLevel Tests ====================

/// 测试从字符串解析日志级别（不区分大小写）
///
/// ## 测试目的
/// 验证 `LogLevel` 的 `FromStr` 实现能够正确解析各种格式的日志级别字符串，支持不区分大小写，并支持 "off"（新格式）和 "none"（向后兼容）。
///
/// ## 测试场景
/// 1. 测试 "off"/"OFF" 和 "none"/"NONE" 解析为 LogLevel::None
/// 2. 测试 "error"/"ERROR" 解析为 LogLevel::Error
/// 3. 测试 "warn"/"WARN" 解析为 LogLevel::Warn
/// 4. 测试 "info"/"INFO" 解析为 LogLevel::Info
/// 5. 测试 "debug"/"DEBUG" 解析为 LogLevel::Debug
/// 6. 测试无效字符串返回错误
///
/// ## 预期结果
/// - 所有有效字符串（不区分大小写）都能正确解析
/// - 无效字符串（"invalid", "", "trace"）返回错误
#[test]
fn test_log_level_from_str_with_valid_strings_parses_correctly() {
    // Arrange: 准备有效的日志级别字符串（不区分大小写）
    // 支持 "off"（新格式）和 "none"（向后兼容）

    // Act & Assert: 验证各种格式的字符串都能正确解析
    assert_eq!(
        "off".parse::<LogLevel>().expect("'off' should parse to LogLevel"),
        LogLevel::None
    );
    assert_eq!(
        "OFF".parse::<LogLevel>().expect("'OFF' should parse to LogLevel"),
        LogLevel::None
    );
    assert_eq!(
        "none".parse::<LogLevel>().expect("'none' should parse to LogLevel"),
        LogLevel::None
    );
    assert_eq!(
        "NONE".parse::<LogLevel>().expect("'NONE' should parse to LogLevel"),
        LogLevel::None
    );
    assert_eq!(
        "error".parse::<LogLevel>().expect("'error' should parse to LogLevel"),
        LogLevel::Error
    );
    assert_eq!(
        "ERROR".parse::<LogLevel>().expect("'ERROR' should parse to LogLevel"),
        LogLevel::Error
    );
    assert_eq!(
        "warn".parse::<LogLevel>().expect("'warn' should parse to LogLevel"),
        LogLevel::Warn
    );
    assert_eq!(
        "WARN".parse::<LogLevel>().expect("'WARN' should parse to LogLevel"),
        LogLevel::Warn
    );
    assert_eq!(
        "info".parse::<LogLevel>().expect("'info' should parse to LogLevel"),
        LogLevel::Info
    );
    assert_eq!(
        "INFO".parse::<LogLevel>().expect("'INFO' should parse to LogLevel"),
        LogLevel::Info
    );
    assert_eq!(
        "debug".parse::<LogLevel>().expect("'debug' should parse to LogLevel"),
        LogLevel::Debug
    );
    assert_eq!(
        "DEBUG".parse::<LogLevel>().expect("'DEBUG' should parse to LogLevel"),
        LogLevel::Debug
    );

    // Arrange: 准备测试无效的日志级别字符串
    assert!("invalid".parse::<LogLevel>().is_err());
    assert!("".parse::<LogLevel>().is_err());
    assert!("trace".parse::<LogLevel>().is_err());
}

/// 测试日志级别转换为字符串
///
/// ## 测试目的
/// 验证 `LogLevel::as_str()` 方法能够将各个日志级别正确转换为对应的字符串表示。
///
/// ## 测试场景
/// 1. 测试各个日志级别转换为字符串
///
/// ## 预期结果
/// - LogLevel::None -> "off"
/// - LogLevel::Error -> "error"
/// - LogLevel::Warn -> "warn"
/// - LogLevel::Info -> "info"
/// - LogLevel::Debug -> "debug"
#[test]
fn test_log_level_as_str_return_ok() -> Result<()> {
    assert_eq!(LogLevel::None.as_str(), "off");
    assert_eq!(LogLevel::Error.as_str(), "error");
    assert_eq!(LogLevel::Warn.as_str(), "warn");
    assert_eq!(LogLevel::Info.as_str(), "info");
    assert_eq!(LogLevel::Debug.as_str(), "debug");
    Ok(())
}

/// 测试日志级别的顺序和should_log方法
///
/// ## 测试目的
/// 验证日志级别的顺序关系（None < Error < Warn < Info < Debug）以及 `should_log()` 方法的过滤逻辑。
///
/// ## 测试场景
/// 1. 验证日志级别的顺序关系
/// 2. 测试各个日志级别的 `should_log()` 方法：
///    - Debug级别：应该记录所有级别
///    - Info级别：应该记录None、Error、Warn、Info，不记录Debug
///    - Warn级别：应该记录None、Error、Warn，不记录Info、Debug
///    - Error级别：应该记录None、Error，不记录其他级别
///    - None级别：只记录None，不记录其他级别
///
/// ## 预期结果
/// - 日志级别顺序正确
/// - `should_log()` 方法按照预期过滤日志
#[test]
fn test_log_level_ordering_return_ok() -> Result<()> {
    // Arrange: 准备测试日志级别的顺序
    assert!(LogLevel::None < LogLevel::Error);
    assert!(LogLevel::Error < LogLevel::Warn);
    assert!(LogLevel::Warn < LogLevel::Info);
    assert!(LogLevel::Info < LogLevel::Debug);

    // Arrange: 准备测试 should_log 方法
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

/// 测试设置和获取全局日志级别
///
/// ## 测试目的
/// 验证 `LogLevel::set_level()` 和 `LogLevel::get_level()` 方法能够正确设置和获取全局日志级别。
///
/// ## 测试场景
/// 1. 保存原始日志级别
/// 2. 依次设置各个日志级别（Debug, Info, Warn, Error, None）
/// 3. 验证每次设置后都能正确获取
/// 4. 恢复原始日志级别
///
/// ## 预期结果
/// - 每次设置后，`get_level()` 返回正确的级别
/// - 测试结束后恢复原始级别，不影响其他测试
#[test]
fn test_log_level_set_and_get_return_ok() -> Result<()> {
    // 保存原始级别
    let original_level = LogLevel::get_level();

    // Arrange: 准备测试设置和获取不同的日志级别
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

/// 测试默认日志级别（根据编译模式）
///
/// ## 测试目的
/// 验证 `LogLevel::default_level()` 方法根据编译模式返回正确的默认日志级别。
///
/// ## 测试场景
/// 1. 调用 `default_level()` 获取默认级别
/// 2. 根据编译模式验证级别
///
/// ## 预期结果
/// - Debug模式（debug_assertions）：返回 LogLevel::Debug
/// - Release模式：返回 LogLevel::Info
#[test]
fn test_log_level_default_return_ok() -> Result<()> {
    // Arrange: 准备测试默认级别（根据编译模式）
    let default = LogLevel::default_level();

    // 在 debug 模式下应该是 Debug，在 release 模式下应该是 Info
    if cfg!(debug_assertions) {
        assert_eq!(default, LogLevel::Debug);
    } else {
        assert_eq!(default, LogLevel::Info);
    }
    Ok(())
}

/// 测试日志级别的往返转换一致性
///
/// ## 测试目的
/// 验证日志级别通过字符串转换的往返一致性，即 `as_str()` 和 `parse()` 的组合应该保持原始值不变。
///
/// ## 测试场景
/// 1. 遍历所有日志级别
/// 2. 将级别转换为字符串，再解析回级别
/// 3. 验证解析后的级别与原始级别相同
///
/// ## 预期结果
/// - 所有级别的往返转换都成功
/// - 解析后的级别与原始级别完全一致
#[test]
fn test_log_level_round_trip() {
    // Arrange: 准备测试字符串转换的往返一致性
    let levels = vec![
        LogLevel::None,
        LogLevel::Error,
        LogLevel::Warn,
        LogLevel::Info,
        LogLevel::Debug,
    ];

    for level in levels {
        let level_str = level.as_str();
        let parsed = level_str.parse::<LogLevel>().expect("LogLevel round trip should succeed");
        assert_eq!(level, parsed, "Round trip failed for level: {}", level_str);
    }
}

// ==================== Tracing 宏测试 ====================

/// 测试tracing宏可以编译和运行
///
/// ## 测试目的
/// 验证所有tracing宏（trace_debug!, trace_info!, trace_warn!, trace_error!）能够正常编译和运行，即使不输出日志。
///
/// ## 测试场景
/// 1. 调用各个tracing宏
/// 2. 验证不会产生编译错误或运行时错误
///
/// ## 预期结果
/// - 所有宏都能正常编译和运行
/// - 不会panic或产生错误
#[test]
fn test_tracing_macros_return_ok() -> Result<()> {
    // 这些宏应该可以编译和运行（即使不输出）
    trace_debug!("Test debug message");
    trace_info!("Test info message");
    trace_warn!("Test warn message");
    trace_error!("Test error message");
    Ok(())
}
