//! Logger Console 测试
//!
//! 测试控制台日志输出的功能。

use workflow::base::logger::console;
use workflow::base::logger::log_level::LogLevel;
use workflow::base::Logger;

#[test]
fn test_success_formatting() {
    let formatted = console::success("Operation completed");
    assert!(formatted.contains("✓"));
    assert!(formatted.contains("Operation completed"));
}

#[test]
fn test_error_formatting() {
    let formatted = console::error("Operation failed");
    assert!(formatted.contains("✗"));
    assert!(formatted.contains("Operation failed"));
}

#[test]
fn test_warning_formatting() {
    let formatted = console::warning("This is a warning");
    assert!(formatted.contains("⚠"));
    assert!(formatted.contains("This is a warning"));
}

#[test]
fn test_info_formatting() {
    let formatted = console::info("Processing data");
    assert!(formatted.contains("ℹ"));
    assert!(formatted.contains("Processing data"));
}

#[test]
fn test_debug_formatting() {
    let formatted = console::debug("Debug information");
    assert!(formatted.contains("⚙"));
    assert!(formatted.contains("Debug information"));
}

#[test]
fn test_separator() {
    let sep = console::separator('-', 10);
    // separator 函数返回的字符串包含 ANSI 转义码（用于颜色），所以实际长度会大于10
    // 我们检查内容而不是长度，或者检查去除 ANSI 码后的内容
    assert!(sep.contains('-'));
    // 检查是否包含10个 '-' 字符（去除 ANSI 码后）
    let dash_count = sep.matches('-').count();
    assert_eq!(dash_count, 10);
}

#[test]
fn test_separator_with_text() {
    let sep = console::separator_with_text('=', 20, "Title");
    assert!(sep.contains("Title"));
    assert!(sep.contains('='));
}

#[test]
fn test_separator_with_text_long() {
    // 如果文本长度大于等于总长度，直接输出文本
    let long_text = "x".repeat(30);
    let sep = console::separator_with_text('-', 20, &long_text);
    assert!(sep.contains(&long_text));
}

#[test]
fn test_logger_print_success() {
    // print_success 总是输出，不受日志级别限制
    Logger::print_success("Test success message");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_message() {
    // print_message 总是输出，不受日志级别限制
    Logger::print_message("Test message");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_separator() {
    Logger::print_separator(None, None);
    Logger::print_separator(Some('-'), None);
    Logger::print_separator(Some('='), Some(50));
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_separator_with_text() {
    Logger::print_separator_with_text('=', 80, "Test Section");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_newline() {
    Logger::print_newline();
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_error() {
    // 设置日志级别为 Error，应该输出
    LogLevel::set_level(LogLevel::Error);
    Logger::print_error("Test error message");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_warning() {
    // 设置日志级别为 Warn，应该输出
    LogLevel::set_level(LogLevel::Warn);
    Logger::print_warning("Test warning message");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_info() {
    // 设置日志级别为 Info，应该输出
    LogLevel::set_level(LogLevel::Info);
    Logger::print_info("Test info message");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_debug() {
    // 设置日志级别为 Debug，应该输出
    LogLevel::set_level(LogLevel::Debug);
    Logger::print_debug("Test debug message");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_log_level_filtering() {
    // 测试日志级别过滤
    // 设置为 Info 级别，Debug 消息不应该输出
    LogLevel::set_level(LogLevel::Info);
    Logger::print_debug("This debug message should not appear");
    // 如果运行到这里没有 panic，说明成功（虽然我们无法直接验证输出）
}

#[test]
fn test_separator_different_chars() {
    let sep1 = console::separator('-', 10);
    let sep2 = console::separator('=', 10);
    let sep3 = console::separator('*', 10);

    assert_ne!(sep1, sep2);
    assert_ne!(sep2, sep3);
}

#[test]
fn test_separator_different_lengths() {
    let sep1 = console::separator('-', 10);
    let sep2 = console::separator('-', 20);

    assert_ne!(sep1.len(), sep2.len());
}

#[test]
fn test_separator_with_text_centering() {
    let sep = console::separator_with_text('-', 20, "Test");
    // 文本应该在中间
    assert!(sep.contains("Test"));
    // 左右两侧应该有分隔符
    let parts: Vec<&str> = sep.split("Test").collect();
    assert_eq!(parts.len(), 2);
}

#[test]
fn test_logger_print_separator_defaults() {
    // 测试 print_separator 的默认参数
    Logger::print_separator(None, None);
    // 应该使用默认的 '-' 和 80
}

#[test]
fn test_logger_print_error_filtered() {
    // 测试日志级别过滤：设置为 None，错误消息不应该输出
    LogLevel::set_level(LogLevel::None);
    Logger::print_error("This error should not appear");
    // 如果运行到这里没有 panic，说明成功（虽然我们无法直接验证输出）
}

#[test]
fn test_logger_print_warning_filtered() {
    // 测试日志级别过滤：设置为 Error，警告消息不应该输出
    LogLevel::set_level(LogLevel::Error);
    Logger::print_warning("This warning should not appear");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_info_filtered() {
    // 测试日志级别过滤：设置为 Warn，信息消息不应该输出
    LogLevel::set_level(LogLevel::Warn);
    Logger::print_info("This info should not appear");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_debug_filtered_at_info() {
    // 测试日志级别过滤：设置为 Info，调试消息不应该输出
    LogLevel::set_level(LogLevel::Info);
    Logger::print_debug("This debug should not appear");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_debug_filtered_at_warn() {
    // 测试日志级别过滤：设置为 Warn，调试消息不应该输出
    LogLevel::set_level(LogLevel::Warn);
    Logger::print_debug("This debug should not appear");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_debug_filtered_at_error() {
    // 测试日志级别过滤：设置为 Error，调试消息不应该输出
    LogLevel::set_level(LogLevel::Error);
    Logger::print_debug("This debug should not appear");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_info_filtered_at_error() {
    // 测试日志级别过滤：设置为 Error，信息消息不应该输出
    LogLevel::set_level(LogLevel::Error);
    Logger::print_info("This info should not appear");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_logger_print_warning_filtered_at_error() {
    // 测试日志级别过滤：设置为 Error，警告消息不应该输出
    LogLevel::set_level(LogLevel::Error);
    Logger::print_warning("This warning should not appear");
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_separator_with_text_exact_length() {
    // 测试文本长度等于总长度的情况
    let text = "x".repeat(20);
    let sep = console::separator_with_text('-', 20, &text);
    assert!(sep.contains(&text));
}

#[test]
fn test_separator_with_text_short() {
    // 测试短文本的情况
    let sep = console::separator_with_text('=', 50, "Short");
    assert!(sep.contains("Short"));
    assert!(sep.contains('='));
}

#[test]
fn test_separator_zero_length() {
    // 测试长度为 0 的情况
    let sep = console::separator('-', 0);
    let dash_count = sep.matches('-').count();
    assert_eq!(dash_count, 0);
}

#[test]
fn test_separator_with_text_zero_length() {
    // 测试总长度为 0 的情况
    let sep = console::separator_with_text('-', 0, "Test");
    // 应该直接输出文本（因为文本长度大于总长度）
    assert!(sep.contains("Test"));
}
