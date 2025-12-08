use workflow::trace_debug;
use workflow::trace_error;
use workflow::trace_info;
use workflow::trace_warn;

#[test]
fn test_tracing_macros() {
    // 这些宏应该可以编译和运行（即使不输出）
    trace_debug!("Test debug message");
    trace_info!("Test info message");
    trace_warn!("Test warn message");
    trace_error!("Test error message");
}
