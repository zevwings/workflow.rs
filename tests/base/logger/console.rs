//! Logger Console 测试
//!
//! 测试控制台日志输出的功能。

use workflow::base::logger::console;
use workflow::base::logger::log_level::LogLevel;
use workflow::base::Logger;

// ==================== Console Formatting Tests ====================

/// 测试格式化成功消息
///
/// ## 测试目的
/// 验证 `console::success()` 函数能够正确格式化成功消息，包含成功图标和消息内容。
///
/// ## 测试场景
/// 1. 准备成功消息
/// 2. 调用 `console::success()` 格式化消息
///
/// ## 预期结果
/// - 返回的字符串包含成功图标（✓）
/// - 返回的字符串包含消息内容
#[test]
fn test_success_formatting_with_message_returns_formatted_string() {
    // Arrange: 准备成功消息

    // Act: 格式化成功消息
    let formatted = console::success("Operation completed");

    // Assert: 验证包含成功图标和消息
    assert!(formatted.contains("✓"));
    assert!(formatted.contains("Operation completed"));
}

/// 测试格式化错误消息
///
/// ## 测试目的
/// 验证 `console::error()` 函数能够正确格式化错误消息，包含错误图标和消息内容。
///
/// ## 测试场景
/// 1. 准备错误消息
/// 2. 调用 `console::error()` 格式化消息
///
/// ## 预期结果
/// - 返回的字符串包含错误图标（✗）
/// - 返回的字符串包含消息内容
#[test]
fn test_error_formatting_with_message_returns_formatted_string() {
    // Arrange: 准备错误消息

    // Act: 格式化错误消息
    let formatted = console::error("Operation failed");

    // Assert: 验证包含错误图标和消息
    assert!(formatted.contains("✗"));
    assert!(formatted.contains("Operation failed"));
}

/// 测试格式化警告消息
///
/// ## 测试目的
/// 验证 `console::warning()` 函数能够正确格式化警告消息，包含警告图标和消息内容。
///
/// ## 测试场景
/// 1. 准备警告消息
/// 2. 调用 `console::warning()` 格式化消息
///
/// ## 预期结果
/// - 返回的字符串包含警告图标（⚠）
/// - 返回的字符串包含消息内容
#[test]
fn test_warning_formatting_with_message_returns_formatted_string() {
    // Arrange: 准备警告消息

    // Act: 格式化警告消息
    let formatted = console::warning("This is a warning");

    // Assert: 验证包含警告图标和消息
    assert!(formatted.contains("⚠"));
    assert!(formatted.contains("This is a warning"));
}

/// 测试格式化信息消息
///
/// ## 测试目的
/// 验证 `console::info()` 函数能够正确格式化信息消息，包含信息图标和消息内容。
///
/// ## 测试场景
/// 1. 准备信息消息
/// 2. 调用 `console::info()` 格式化消息
///
/// ## 预期结果
/// - 返回的字符串包含信息图标（ℹ）
/// - 返回的字符串包含消息内容
#[test]
fn test_info_formatting_with_message_returns_formatted_string() {
    // Arrange: 准备信息消息

    // Act: 格式化信息消息
    let formatted = console::info("Processing data");

    // Assert: 验证包含信息图标和消息
    assert!(formatted.contains("ℹ"));
    assert!(formatted.contains("Processing data"));
}

/// 测试格式化调试消息
///
/// ## 测试目的
/// 验证 `console::debug()` 函数能够正确格式化调试消息，包含调试图标和消息内容。
///
/// ## 测试场景
/// 1. 准备调试消息
/// 2. 调用 `console::debug()` 格式化消息
///
/// ## 预期结果
/// - 返回的字符串包含调试图标（⚙）
/// - 返回的字符串包含消息内容
#[test]
fn test_debug_formatting_with_message_returns_formatted_string() {
    // Arrange: 准备调试消息

    // Act: 格式化调试消息
    let formatted = console::debug("Debug information");

    // Assert: 验证包含调试图标和消息
    assert!(formatted.contains("⚙"));
    assert!(formatted.contains("Debug information"));
}

// ==================== Console Separator Tests ====================

/// 测试生成分隔符字符串
///
/// ## 测试目的
/// 验证 `console::separator()` 函数能够根据指定字符和长度生成分隔符字符串。
///
/// ## 测试场景
/// 1. 准备分隔符字符和长度
/// 2. 调用 `console::separator()` 生成分隔符
///
/// ## 预期结果
/// - 返回的字符串包含指定字符
/// - 字符数量等于指定长度
#[test]
fn test_separator_with_char_and_length_returns_separator_string() {
    // Arrange: 准备分隔符字符和长度
    let char = '-';
    let length = 10;

    // Act: 生成分隔符
    let sep = console::separator(char, length);

    // Assert: 验证包含指定字符和数量（去除 ANSI 码后）
    assert!(sep.contains(char));
    let dash_count = sep.matches(char).count();
    assert_eq!(dash_count, length);
}

/// 测试生成带文本的分隔符
///
/// ## 测试目的
/// 验证 `console::separator_with_text()` 函数能够生成包含文本的分隔符。
///
/// ## 测试场景
/// 1. 准备分隔符参数和文本
/// 2. 调用 `console::separator_with_text()` 生成分隔符
///
/// ## 预期结果
/// - 返回的字符串包含文本
/// - 返回的字符串包含分隔符字符
#[test]
fn test_separator_with_text_with_valid_params_returns_separator_with_text() {
    // Arrange: 准备分隔符参数和文本
    let char = '=';
    let length = 20;
    let text = "Title";

    // Act: 生成带文本的分隔符
    let sep = console::separator_with_text(char, length, text);

    // Assert: 验证包含文本和分隔符字符
    assert!(sep.contains(text));
    assert!(sep.contains(char));
}

/// 测试生成带长文本的分隔符（文本长度大于分隔符长度）
///
/// ## 测试目的
/// 验证当文本长度大于分隔符长度时，`console::separator_with_text()` 函数能够正确处理。
///
/// ## 测试场景
/// 1. 准备长文本（长度大于分隔符长度）
/// 2. 调用 `console::separator_with_text()` 生成分隔符
///
/// ## 预期结果
/// - 直接输出文本（如果文本长度大于等于总长度）
#[test]
fn test_separator_with_text_long_with_long_text_returns_text_only() {
    // Arrange: 准备长文本（长度大于分隔符长度）
    let long_text = "x".repeat(30);
    let char = '-';
    let length = 20;

    // Act: 生成带长文本的分隔符
    let sep = console::separator_with_text(char, length, &long_text);

    // Assert: 验证直接输出文本（如果文本长度大于等于总长度）
    assert!(sep.contains(&long_text));
}

// ==================== Logger Print Tests ====================

/// 测试打印成功消息
///
/// ## 测试目的
/// 验证 `Logger::print_success()` 方法能够正确打印成功消息。
///
/// ## 测试场景
/// 1. 准备成功消息
/// 2. 调用 `Logger::print_success()` 打印消息
///
/// ## 预期结果
/// - 不会panic（print_success 总是输出，不受日志级别限制）
#[test]
fn test_logger_print_success_with_message() {
    // Arrange: 准备成功消息
    // print_success 总是输出，不受日志级别限制

    // Act: 打印成功消息
    Logger::print_success("Test success message");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试打印普通消息
///
/// ## 测试目的
/// 验证 `Logger::print_message()` 方法能够正确打印普通消息。
///
/// ## 测试场景
/// 1. 准备消息
/// 2. 调用 `Logger::print_message()` 打印消息
///
/// ## 预期结果
/// - 不会panic（print_message 总是输出，不受日志级别限制）
#[test]
fn test_logger_print_message_with_message() {
    // Arrange: 准备消息
    // print_message 总是输出，不受日志级别限制

    // Act: 打印消息
    Logger::print_message("Test message");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试打印分隔符（使用不同参数）
///
/// ## 测试目的
/// 验证 `Logger::print_separator()` 方法能够使用不同参数正确打印分隔符。
///
/// ## 测试场景
/// 1. 使用不同的分隔符参数调用 `print_separator()`
///
/// ## 预期结果
/// - 不会panic（无返回值）
#[test]
fn test_logger_print_separator_with_various_params() {
    // Arrange: 准备不同的分隔符参数

    // Act: 打印分隔符（使用不同参数）
    Logger::print_separator(None, None);
    Logger::print_separator(Some('-'), None);
    Logger::print_separator(Some('='), Some(50));

    // Assert: 验证不会 panic（无返回值）
}

/// 测试打印带文本的分隔符
///
/// ## 测试目的
/// 验证 `Logger::print_separator_with_text()` 方法能够正确打印带文本的分隔符。
///
/// ## 测试场景
/// 1. 准备分隔符参数和文本
/// 2. 调用 `Logger::print_separator_with_text()` 打印分隔符
///
/// ## 预期结果
/// - 不会panic（无返回值）
#[test]
fn test_logger_print_separator_with_text_with_params() {
    // Arrange: 准备分隔符参数和文本

    // Act: 打印带文本的分隔符
    Logger::print_separator_with_text('=', 80, "Test Section");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试打印换行
///
/// ## 测试目的
/// 验证 `Logger::print_newline()` 方法能够正确打印换行。
///
/// ## 测试场景
/// 1. 调用 `Logger::print_newline()` 打印换行
///
/// ## 预期结果
/// - 不会panic（无返回值）
#[test]
fn test_logger_print_newline_with_no_params() {
    // Arrange: 准备测试（无需额外准备）

    // Act: 打印换行
    Logger::print_newline();

    // Assert: 验证不会 panic（无返回值）
}

/// 测试打印错误消息（Error级别）
///
/// ## 测试目的
/// 验证在 Error 日志级别下，`Logger::print_error()` 方法能够正确打印错误消息。
///
/// ## 测试场景
/// 1. 设置日志级别为 Error
/// 2. 调用 `Logger::print_error()` 打印错误消息
///
/// ## 预期结果
/// - 不会panic（错误消息应该输出）
#[test]
fn test_logger_print_error_with_error_level() {
    // Arrange: 设置日志级别为 Error
    LogLevel::set_level(LogLevel::Error);

    // Act: 打印错误消息（应该输出）
    Logger::print_error("Test error message");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试打印警告消息（Warn级别）
///
/// ## 测试目的
/// 验证在 Warn 日志级别下，`Logger::print_warning()` 方法能够正确打印警告消息。
///
/// ## 测试场景
/// 1. 设置日志级别为 Warn
/// 2. 调用 `Logger::print_warning()` 打印警告消息
///
/// ## 预期结果
/// - 不会panic（警告消息应该输出）
#[test]
fn test_logger_print_warning_with_warn_level() {
    // Arrange: 设置日志级别为 Warn
    LogLevel::set_level(LogLevel::Warn);

    // Act: 打印警告消息（应该输出）
    Logger::print_warning("Test warning message");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试打印信息消息（Info级别）
///
/// ## 测试目的
/// 验证在 Info 日志级别下，`Logger::print_info()` 方法能够正确打印信息消息。
///
/// ## 测试场景
/// 1. 设置日志级别为 Info
/// 2. 调用 `Logger::print_info()` 打印信息消息
///
/// ## 预期结果
/// - 不会panic（信息消息应该输出）
#[test]
fn test_logger_print_info_with_info_level() {
    // Arrange: 设置日志级别为 Info
    LogLevel::set_level(LogLevel::Info);

    // Act: 打印信息消息（应该输出）
    Logger::print_info("Test info message");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试打印调试消息（Debug级别）
///
/// ## 测试目的
/// 验证在 Debug 日志级别下，`Logger::print_debug()` 方法能够正确打印调试消息。
///
/// ## 测试场景
/// 1. 设置日志级别为 Debug
/// 2. 调用 `Logger::print_debug()` 打印调试消息
///
/// ## 预期结果
/// - 不会panic（调试消息应该输出）
#[test]
fn test_logger_print_debug_with_debug_level() {
    // Arrange: 设置日志级别为 Debug
    LogLevel::set_level(LogLevel::Debug);

    // Act: 打印调试消息（应该输出）
    Logger::print_debug("Test debug message");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试日志级别过滤（Info级别过滤Debug消息）
///
/// ## 测试目的
/// 验证在 Info 日志级别下，Debug 消息会被过滤，不会输出。
///
/// ## 测试场景
/// 1. 设置日志级别为 Info
/// 2. 调用 `Logger::print_debug()` 打印调试消息
///
/// ## 预期结果
/// - 不会panic（调试消息应该被过滤）
#[test]
fn test_logger_log_level_filtering_with_info_level_filters_debug() {
    // Arrange: 设置日志级别为 Info（Debug 消息不应该输出）
    LogLevel::set_level(LogLevel::Info);

    // Act: 打印调试消息（应该被过滤）
    Logger::print_debug("This debug message should not appear");

    // Assert: 验证不会 panic（虽然我们无法直接验证输出）
}

/// 测试使用不同字符生成分隔符
///
/// ## 测试目的
/// 验证使用不同字符生成的分隔符字符串是不同的。
///
/// ## 测试场景
/// 1. 使用不同字符，相同长度生成分隔符
///
/// ## 预期结果
/// - 不同字符产生不同的字符串
#[test]
fn test_separator_with_different_chars_returns_different_strings() {
    // Arrange: 准备不同的字符，相同长度

    // Act: 生成不同字符的分隔符
    let sep1 = console::separator('-', 10);
    let sep2 = console::separator('=', 10);
    let sep3 = console::separator('*', 10);

    // Assert: 验证不同字符产生不同的字符串
    assert_ne!(sep1, sep2);
    assert_ne!(sep2, sep3);
}

/// 测试使用不同长度生成分隔符
///
/// ## 测试目的
/// 验证使用不同长度生成的分隔符字符串长度不同。
///
/// ## 测试场景
/// 1. 使用相同字符，不同长度生成分隔符
///
/// ## 预期结果
/// - 长度不同
#[test]
fn test_separator_with_different_lengths_returns_different_lengths() {
    // Arrange: 准备相同字符，不同长度

    // Act: 生成不同长度的分隔符
    let sep1 = console::separator('-', 10);
    let sep2 = console::separator('-', 20);

    // Assert: 验证长度不同
    assert_ne!(sep1.len(), sep2.len());
}

/// 测试生成居中文本的分隔符
///
/// ## 测试目的
/// 验证 `console::separator_with_text()` 函数能够生成文本居中的分隔符。
///
/// ## 测试场景
/// 1. 准备分隔符参数和文本
/// 2. 调用 `console::separator_with_text()` 生成分隔符
///
/// ## 预期结果
/// - 文本在中间，左右两侧有分隔符
#[test]
fn test_separator_with_text_centering_returns_centered_text() {
    // Arrange: 准备分隔符参数和文本

    // Act: 生成带文本的分隔符
    let sep = console::separator_with_text('-', 20, "Test");

    // Assert: 验证文本在中间，左右两侧有分隔符
    assert!(sep.contains("Test"));
    let parts: Vec<&str> = sep.split("Test").collect();
    assert_eq!(parts.len(), 2);
}

/// 测试打印分隔符（使用默认参数）
///
/// ## 测试目的
/// 验证 `Logger::print_separator()` 方法使用默认参数时能够正确打印分隔符。
///
/// ## 测试场景
/// 1. 调用 `Logger::print_separator()` 使用默认参数（None, None）
///
/// ## 预期结果
/// - 不会panic（应该使用默认的 '-' 和 80）
#[test]
fn test_logger_print_separator_with_defaults() {
    // Arrange: 准备测试（无需额外准备）
    // 测试 print_separator 的默认参数

    // Act: 打印分隔符（使用默认参数）
    Logger::print_separator(None, None);

    // Assert: 验证不会 panic（应该使用默认的 '-' 和 80）
}

// ==================== Logger Log Level Filtering Tests ====================

/// 测试日志级别过滤（None级别过滤错误消息）
///
/// ## 测试目的
/// 验证在 None 日志级别下，错误消息会被过滤，不会输出。
///
/// ## 测试场景
/// 1. 设置日志级别为 None
/// 2. 调用 `Logger::print_error()` 打印错误消息
///
/// ## 预期结果
/// - 不会panic（错误消息应该被过滤）
#[test]
fn test_logger_print_error_with_none_level_filters_error() {
    // Arrange: 设置日志级别为 None（错误消息不应该输出）
    LogLevel::set_level(LogLevel::None);

    // Act: 打印错误消息（应该被过滤）
    Logger::print_error("This error should not appear");

    // Assert: 验证不会 panic（虽然我们无法直接验证输出）
}

/// 测试日志级别过滤（Error级别过滤警告消息）
///
/// ## 测试目的
/// 验证在 Error 日志级别下，警告消息会被过滤，不会输出。
///
/// ## 测试场景
/// 1. 设置日志级别为 Error
/// 2. 调用 `Logger::print_warning()` 打印警告消息
///
/// ## 预期结果
/// - 不会panic（警告消息应该被过滤）
#[test]
fn test_logger_print_warning_with_error_level_filters_warning() {
    // Arrange: 设置日志级别为 Error（警告消息不应该输出）
    LogLevel::set_level(LogLevel::Error);

    // Act: 打印警告消息（应该被过滤）
    Logger::print_warning("This warning should not appear");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试日志级别过滤（Warn级别过滤信息消息）
///
/// ## 测试目的
/// 验证在 Warn 日志级别下，信息消息会被过滤，不会输出。
///
/// ## 测试场景
/// 1. 设置日志级别为 Warn
/// 2. 调用 `Logger::print_info()` 打印信息消息
///
/// ## 预期结果
/// - 不会panic（信息消息应该被过滤）
#[test]
fn test_logger_print_info_with_warn_level_filters_info() {
    // Arrange: 设置日志级别为 Warn（信息消息不应该输出）
    LogLevel::set_level(LogLevel::Warn);

    // Act: 打印信息消息（应该被过滤）
    Logger::print_info("This info should not appear");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试日志级别过滤（Info级别过滤调试消息）
///
/// ## 测试目的
/// 验证在 Info 日志级别下，调试消息会被过滤，不会输出。
///
/// ## 测试场景
/// 1. 设置日志级别为 Info
/// 2. 调用 `Logger::print_debug()` 打印调试消息
///
/// ## 预期结果
/// - 不会panic（调试消息应该被过滤）
#[test]
fn test_logger_print_debug_with_info_level_filters_debug() {
    // Arrange: 设置日志级别为 Info（调试消息不应该输出）
    LogLevel::set_level(LogLevel::Info);

    // Act: 打印调试消息（应该被过滤）
    Logger::print_debug("This debug should not appear");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试日志级别过滤（Warn级别过滤调试消息）
///
/// ## 测试目的
/// 验证在 Warn 日志级别下，调试消息会被过滤，不会输出。
///
/// ## 测试场景
/// 1. 设置日志级别为 Warn
/// 2. 调用 `Logger::print_debug()` 打印调试消息
///
/// ## 预期结果
/// - 不会panic（调试消息应该被过滤）
#[test]
fn test_logger_print_debug_with_warn_level_filters_debug() {
    // Arrange: 设置日志级别为 Warn（调试消息不应该输出）
    LogLevel::set_level(LogLevel::Warn);

    // Act: 打印调试消息（应该被过滤）
    Logger::print_debug("This debug should not appear");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试日志级别过滤（Error级别过滤调试消息）
///
/// ## 测试目的
/// 验证在 Error 日志级别下，调试消息会被过滤，不会输出。
///
/// ## 测试场景
/// 1. 设置日志级别为 Error
/// 2. 调用 `Logger::print_debug()` 打印调试消息
///
/// ## 预期结果
/// - 不会panic（调试消息应该被过滤）
#[test]
fn test_logger_print_debug_with_error_level_filters_debug() {
    // Arrange: 设置日志级别为 Error（调试消息不应该输出）
    LogLevel::set_level(LogLevel::Error);

    // Act: 打印调试消息（应该被过滤）
    Logger::print_debug("This debug should not appear");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试日志级别过滤（Error级别过滤信息消息）
///
/// ## 测试目的
/// 验证在 Error 日志级别下，信息消息会被过滤，不会输出。
///
/// ## 测试场景
/// 1. 设置日志级别为 Error
/// 2. 调用 `Logger::print_info()` 打印信息消息
///
/// ## 预期结果
/// - 不会panic（信息消息应该被过滤）
#[test]
fn test_logger_print_info_with_error_level_filters_info() {
    // Arrange: 设置日志级别为 Error（信息消息不应该输出）
    LogLevel::set_level(LogLevel::Error);

    // Act: 打印信息消息（应该被过滤）
    Logger::print_info("This info should not appear");

    // Assert: 验证不会 panic（无返回值）
}

/// 测试生成分隔符（文本长度等于总长度）
///
/// ## 测试目的
/// 验证当文本长度等于总长度时，`console::separator_with_text()` 函数能够正确处理。
///
/// ## 测试场景
/// 1. 准备文本长度等于总长度的情况
/// 2. 调用 `console::separator_with_text()` 生成分隔符
///
/// ## 预期结果
/// - 返回的字符串包含文本
#[test]
fn test_separator_with_text_exact_length_returns_text_only() {
    // Arrange: 准备文本长度等于总长度的情况
    let text = "x".repeat(20);

    // Act: 生成带文本的分隔符
    let sep = console::separator_with_text('-', 20, &text);

    // Assert: 验证包含文本
    assert!(sep.contains(&text));
}

/// 测试生成带短文本的分隔符
///
/// ## 测试目的
/// 验证当文本长度小于总长度时，`console::separator_with_text()` 函数能够生成带文本的分隔符。
///
/// ## 测试场景
/// 1. 准备短文本
/// 2. 调用 `console::separator_with_text()` 生成分隔符
///
/// ## 预期结果
/// - 返回的字符串包含文本和分隔符字符
#[test]
fn test_separator_with_text_short_returns_separator_with_text() {
    // Arrange: 准备短文本

    // Act: 生成带短文本的分隔符
    let sep = console::separator_with_text('=', 50, "Short");

    // Assert: 验证包含文本和分隔符字符
    assert!(sep.contains("Short"));
    assert!(sep.contains('='));
}

/// 测试生成长度为0的分隔符
///
/// ## 测试目的
/// 验证当长度为0时，`console::separator()` 函数能够正确处理。
///
/// ## 测试场景
/// 1. 准备长度为 0
/// 2. 调用 `console::separator()` 生成分隔符
///
/// ## 预期结果
/// - 不包含分隔符字符
#[test]
fn test_separator_with_zero_length_returns_empty_string() {
    // Arrange: 准备长度为 0

    // Act: 生成长度为 0 的分隔符
    let sep = console::separator('-', 0);

    // Assert: 验证不包含分隔符字符
    let dash_count = sep.matches('-').count();
    assert_eq!(dash_count, 0);
}

/// 测试生成带文本的分隔符（总长度为0）
///
/// ## 测试目的
/// 验证当总长度为0时，`console::separator_with_text()` 函数能够正确处理。
///
/// ## 测试场景
/// 1. 准备总长度为 0 的情况
/// 2. 调用 `console::separator_with_text()` 生成分隔符
///
/// ## 预期结果
/// - 直接输出文本（因为文本长度大于总长度）
#[test]
fn test_separator_with_text_zero_length_returns_text_only() {
    // Arrange: 准备总长度为 0 的情况

    // Act: 生成带文本的分隔符（总长度为 0）
    let sep = console::separator_with_text('-', 0, "Test");

    // Assert: 验证直接输出文本（因为文本长度大于总长度）
    assert!(sep.contains("Test"));
}
