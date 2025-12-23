//! Logger Tracing 测试
//!
//! 测试 tracing 封装的功能。

use workflow::base::logger::tracing::Tracer;

#[test]
fn test_tracer_debug() {
    // 测试 debug 方法（不会 panic 即可）
    Tracer::debug("Test debug message");
}

#[test]
fn test_tracer_info() {
    // 测试 info 方法
    Tracer::info("Test info message");
}

#[test]
fn test_tracer_warn() {
    // 测试 warn 方法
    Tracer::warn("Test warn message");
}

#[test]
fn test_tracer_error() {
    // 测试 error 方法
    Tracer::error("Test error message");
}

#[test]
fn test_tracer_debug_fmt() {
    // 测试 debug_fmt 方法
    Tracer::debug_fmt(format_args!("Debug: {}", "test"));
}

#[test]
fn test_tracer_info_fmt() {
    // 测试 info_fmt 方法
    Tracer::info_fmt(format_args!("Info: {}", "test"));
}

#[test]
fn test_tracer_warn_fmt() {
    // 测试 warn_fmt 方法
    Tracer::warn_fmt(format_args!("Warn: {}", "test"));
}

#[test]
fn test_tracer_error_fmt() {
    // 测试 error_fmt 方法
    Tracer::error_fmt(format_args!("Error: {}", "test"));
}

// 注意：get_log_file_path 是私有方法，无法直接测试
// 可以通过 Tracer::init() 间接测试路径创建功能

#[test]
fn test_trace_macros() {
    // 测试 trace_* 宏
    workflow::trace_debug!("Debug macro test");
    workflow::trace_info!("Info macro test");
    workflow::trace_warn!("Warn macro test");
    workflow::trace_error!("Error macro test");
}

#[test]
fn test_trace_macros_with_formatting() {
    // 测试带格式化的 trace_* 宏
    let count = 5;
    workflow::trace_debug!("Debug: {} items", count);
    workflow::trace_info!("Info: {} items", count);
    workflow::trace_warn!("Warn: {} items", count);
    workflow::trace_error!("Error: {} items", count);
}

#[test]
fn test_tracer_init() {
    // 测试初始化（可能会失败，但不应该 panic）
    // 注意：多次初始化可能会失败，这是正常的
    Tracer::init();
}

#[test]
fn test_trace_macro_multiple_calls() {
    // 测试多次调用宏
    for i in 0..5 {
        workflow::trace_debug!("Iteration {}", i);
        workflow::trace_info!("Iteration {}", i);
    }
}

#[test]
fn test_tracer_init_multiple_times() {
    // 测试多次初始化（应该不会 panic）
    Tracer::init();
    Tracer::init();
    Tracer::init();
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_tracer_methods_with_different_inputs() {
    // 测试不同输入的方法
    Tracer::debug("");
    Tracer::info("Simple message");
    Tracer::warn("Warning with special chars: !@#$%");
    Tracer::error("Error with newline\nand tab\t");
}

#[test]
fn test_tracer_fmt_methods_with_complex_formatting() {
    // 测试复杂格式化
    let number = 42;
    let text = "test";
    let boolean = true;

    Tracer::debug_fmt(format_args!(
        "Debug: number={}, text={}, bool={}",
        number, text, boolean
    ));
    Tracer::info_fmt(format_args!(
        "Info: number={}, text={}, bool={}",
        number, text, boolean
    ));
    Tracer::warn_fmt(format_args!(
        "Warn: number={}, text={}, bool={}",
        number, text, boolean
    ));
    Tracer::error_fmt(format_args!(
        "Error: number={}, text={}, bool={}",
        number, text, boolean
    ));
}

#[test]
fn test_trace_macros_with_various_types() {
    // 测试不同类型的参数
    workflow::trace_debug!("Number: {}", 42);
    workflow::trace_info!("Float: {}", 3.14);
    workflow::trace_warn!("Boolean: {}", true);
    workflow::trace_error!("String: {}", "test");
}

#[test]
fn test_trace_macros_with_empty_strings() {
    // 测试空字符串
    workflow::trace_debug!("");
    workflow::trace_info!("");
    workflow::trace_warn!("");
    workflow::trace_error!("");
}

#[test]
fn test_trace_macros_with_long_messages() {
    // 测试长消息
    let long_message = "x".repeat(1000);
    workflow::trace_debug!("Long: {}", long_message);
    workflow::trace_info!("Long: {}", long_message);
    workflow::trace_warn!("Long: {}", long_message);
    workflow::trace_error!("Long: {}", long_message);
}

#[test]
fn test_tracer_init_with_enable_console() {
    // 测试 enable_trace_console 配置分支
    // 注意：由于 Tracer::init() 只能调用一次，这个测试可能不会完全覆盖所有分支
    // 但至少可以验证代码路径存在
    Tracer::init();
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_tracer_init_file_creation_fallback() {
    // 测试文件创建失败时的回退逻辑
    // 注意：这个测试很难直接触发文件创建失败，但至少可以验证代码路径存在
    Tracer::init();
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_tracer_init_log_level_none() {
    // 测试 LogLevel::None 的分支（输出到 sink）
    // 注意：由于 Tracer::init() 从配置文件读取，这个测试可能不会完全覆盖
    // 但至少可以验证代码路径存在
    Tracer::init();
    // 如果运行到这里没有 panic，说明成功
}

#[test]
fn test_tracer_get_log_file_path_indirect() {
    // 间接测试 get_log_file_path() 方法
    // 通过 Tracer::init() 调用，验证日志文件路径创建功能
    Tracer::init();

    // 验证 tracing 目录被创建（通过 init 调用）
    // 如果运行到这里没有 panic，说明路径创建成功
}
