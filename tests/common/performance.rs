//! 测试性能测量工具
//!
//! 提供用于测量和监控测试执行时间的工具函数，包括：
//! - 执行时间测量
//! - 调用栈分析
//! - 内存使用分析（基础版本）

#![allow(dead_code)] // 这些函数供测试使用，可能暂时未被引用

use color_eyre::Result;
use std::backtrace::Backtrace;

/// 测量测试执行时间
///
/// 如果测试执行时间超过5秒，会输出警告信息。
///
/// # 参数
///
/// * `name` - 测试名称，用于标识和报告
/// * `test_fn` - 要执行的测试函数
///
/// # 返回
///
/// 返回测试函数的执行结果
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::performance::measure_test_time;
/// use color_eyre::Result;
///
/// #[test]
/// fn test_slow_operation() -> Result<()> {
///     measure_test_time("test_slow_operation", || {
///         // 执行一些耗时操作
///         std::thread::sleep(std::time::Duration::from_secs(2));
///         Ok(())
///     })
/// }
/// ```
pub fn measure_test_time<F>(name: &str, test_fn: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let start = std::time::Instant::now();
    let result = test_fn();
    let duration = start.elapsed();

    if duration.as_secs() > 5 {
        eprintln!("Warning: Test '{}' took {:?}", name, duration);
    }

    result
}

/// 测量测试执行时间并返回持续时间
///
/// 与 `measure_test_time` 类似，但返回测试执行的时间，而不是只输出警告。
///
/// # 参数
///
/// * `name` - 测试名称，用于标识和报告
/// * `test_fn` - 要执行的测试函数
///
/// # 返回
///
/// 返回测试函数的执行结果和执行时间
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::performance::measure_test_time_with_duration;
/// use color_eyre::Result;
///
/// #[test]
/// fn test_with_timing() -> Result<()> {
///     let (result, duration) = measure_test_time_with_duration("test_with_timing", || {
///         // 执行操作
///         Ok(())
///     })?;
///
///     println!("Test completed in {:?}", duration);
///     result
/// }
/// ```
pub fn measure_test_time_with_duration<F>(
    name: &str,
    test_fn: F,
) -> Result<(Result<()>, std::time::Duration)>
where
    F: FnOnce() -> Result<()>,
{
    let start = std::time::Instant::now();
    let result = test_fn();
    let duration = start.elapsed();

    if duration.as_secs() > 5 {
        eprintln!("Warning: Test '{}' took {:?}", name, duration);
    }

    Ok((result, duration))
}

/// 测量测试执行时间，使用自定义阈值
///
/// 允许指定自定义的时间阈值，而不是固定的5秒。
///
/// # 参数
///
/// * `name` - 测试名称，用于标识和报告
/// * `threshold` - 时间阈值，超过此时间会输出警告
/// * `test_fn` - 要执行的测试函数
///
/// # 返回
///
/// 返回测试函数的执行结果
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::performance::measure_test_time_with_threshold;
/// use color_eyre::Result;
/// use std::time::Duration;
///
/// #[test]
/// fn test_with_custom_threshold() -> Result<()> {
///     measure_test_time_with_threshold(
///         "test_with_custom_threshold",
///         Duration::from_secs(10),
///         || {
///             // 执行操作
///             Ok(())
///         }
///     )
/// }
/// ```
pub fn measure_test_time_with_threshold<F>(
    name: &str,
    threshold: std::time::Duration,
    test_fn: F,
) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let start = std::time::Instant::now();
    let result = test_fn();
    let duration = start.elapsed();

    if duration > threshold {
        eprintln!(
            "Warning: Test '{}' took {:?}, exceeding threshold of {:?}",
            name, duration, threshold
        );
    }

    result
}

/// 性能分析结果
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    /// 测试名称
    pub name: String,
    /// 执行时间
    pub duration: std::time::Duration,
    /// 调用栈信息（如果启用）
    pub stack_trace: Option<String>,
    /// 内存使用估算（字节，如果可用）
    pub memory_usage_bytes: Option<usize>,
    /// 是否超过阈值
    pub exceeded_threshold: bool,
}

/// 测量测试执行时间并进行性能分析
///
/// 提供详细的性能分析，包括执行时间、调用栈和内存使用。
///
/// # 参数
///
/// * `name` - 测试名称，用于标识和报告
/// * `enable_stack_trace` - 是否启用调用栈分析（可能影响性能）
/// * `enable_memory_tracking` - 是否启用内存跟踪（基础版本，使用估算）
/// * `test_fn` - 要执行的测试函数
///
/// # 返回
///
/// 返回测试函数的执行结果和性能分析结果
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::performance::measure_test_with_analysis;
/// use color_eyre::Result;
///
/// #[test]
/// fn test_with_analysis() -> Result<()> {
///     let (result, analysis) = measure_test_with_analysis(
///         "test_with_analysis",
///         false, // 不启用调用栈（避免性能开销）
///         true,  // 启用内存跟踪
///         || {
///             // 执行操作
///             Ok(())
///         }
///     )?;
///
///     println!("Duration: {:?}", analysis.duration);
///     if let Some(mem) = analysis.memory_usage_bytes {
///         println!("Memory: {} bytes", mem);
///     }
///     result
/// }
/// ```
pub fn measure_test_with_analysis<F>(
    name: &str,
    enable_stack_trace: bool,
    enable_memory_tracking: bool,
    test_fn: F,
) -> Result<(Result<()>, PerformanceAnalysis)>
where
    F: FnOnce() -> Result<()>,
{
    let start = std::time::Instant::now();

    // 记录开始时的内存使用（如果启用）
    let memory_before = if enable_memory_tracking {
        Some(estimate_memory_usage())
    } else {
        None
    };

    // 捕获调用栈（如果启用）
    let stack_trace = if enable_stack_trace {
        let backtrace = Backtrace::capture();
        Some(format!("{}", backtrace))
    } else {
        None
    };

    let result = test_fn();
    let duration = start.elapsed();

    // 记录结束时的内存使用（如果启用）
    let memory_after = if enable_memory_tracking {
        Some(estimate_memory_usage())
    } else {
        None
    };

    // 计算内存使用差异
    let memory_usage_bytes = if let (Some(before), Some(after)) = (memory_before, memory_after) {
        if after > before {
            Some(after - before)
        } else {
            Some(0)
        }
    } else {
        None
    };

    let analysis = PerformanceAnalysis {
        name: name.to_string(),
        duration,
        stack_trace,
        memory_usage_bytes,
        exceeded_threshold: duration.as_secs() > 5,
    };

    if analysis.exceeded_threshold {
        eprintln!("Warning: Test '{}' took {:?}", name, duration);
        if let Some(mem) = memory_usage_bytes {
            eprintln!(
                "  Memory usage: {} bytes ({:.2} KB)",
                mem,
                mem as f64 / 1024.0
            );
        }
        if let Some(ref stack) = analysis.stack_trace {
            eprintln!("  Stack trace:\n{}", stack);
        }
    }

    Ok((result, analysis))
}

/// 估算当前内存使用（基础版本）
///
/// 这是一个简化的内存估算，使用系统信息。
/// 对于更精确的内存分析，建议使用专门的工具如 `dhat` 或 `heaptrack`。
///
/// # 返回
///
/// 返回估算的内存使用量（字节）
fn estimate_memory_usage() -> usize {
    // 基础实现：使用系统信息估算
    // 注意：这是一个简化的实现，实际内存使用可能更复杂

    // 尝试从 /proc/self/status 读取（Linux）
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<usize>() {
                            return kb * 1024; // 转换为字节
                        }
                    }
                }
            }
        }
    }

    // macOS 和 Windows 需要更复杂的实现
    // 这里返回 0 表示无法估算（实际项目中可以使用更专业的库）
    #[cfg(not(target_os = "linux"))]
    {
        // macOS: 可以使用 libc::proc_pidinfo
        // Windows: 可以使用 winapi::GetProcessMemoryInfo
        // 为了简化，这里返回 0
    }

    0
}

/// 测量测试执行时间并收集调用栈信息
///
/// 当测试执行时间超过阈值时，自动收集调用栈信息以便分析性能瓶颈。
///
/// # 参数
///
/// * `name` - 测试名称
/// * `threshold` - 时间阈值，超过此时间会收集调用栈
/// * `test_fn` - 要执行的测试函数
///
/// # 返回
///
/// 返回测试函数的执行结果和性能分析
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::performance::measure_test_with_stack_trace;
/// use color_eyre::Result;
/// use std::time::Duration;
///
/// #[test]
/// fn test_slow_with_stack() -> Result<()> {
///     let (result, analysis) = measure_test_with_stack_trace(
///         "test_slow_with_stack",
///         Duration::from_secs(3),
///         || {
///             // 执行可能较慢的操作
///             Ok(())
///         }
///     )?;
///
///     if analysis.exceeded_threshold {
///         println!("Test exceeded threshold!");
///         if let Some(stack) = analysis.stack_trace {
///             println!("Stack trace: {}", stack);
///         }
///     }
///     result
/// }
/// ```
pub fn measure_test_with_stack_trace<F>(
    name: &str,
    threshold: std::time::Duration,
    test_fn: F,
) -> Result<(Result<()>, PerformanceAnalysis)>
where
    F: FnOnce() -> Result<()>,
{
    let start = std::time::Instant::now();
    let result = test_fn();
    let duration = start.elapsed();

    let exceeded_threshold = duration > threshold;
    let stack_trace = if exceeded_threshold {
        let backtrace = Backtrace::capture();
        Some(format!("{}", backtrace))
    } else {
        None
    };

    let analysis = PerformanceAnalysis {
        name: name.to_string(),
        duration,
        stack_trace: stack_trace.clone(),
        memory_usage_bytes: None,
        exceeded_threshold,
    };

    if exceeded_threshold {
        eprintln!(
            "Warning: Test '{}' took {:?}, exceeding threshold of {:?}",
            name, duration, threshold
        );
        if let Some(ref stack) = stack_trace {
            eprintln!("Stack trace:\n{}", stack);
        }
    }

    Ok((result, analysis))
}

/// 测量测试的内存使用情况
///
/// 使用闭包前后的内存差异来估算测试的内存使用。
/// 注意：这是一个简化的实现，实际内存使用可能更复杂。
///
/// # 参数
///
/// * `name` - 测试名称
/// * `test_fn` - 要执行的测试函数
///
/// # 返回
///
/// 返回测试函数的执行结果和内存使用估算（字节）
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::performance::measure_test_memory;
/// use color_eyre::Result;
///
/// #[test]
/// fn test_memory_usage() -> Result<()> {
///     let (result, memory_bytes) = measure_test_memory("test_memory_usage", || {
///         // 执行可能分配内存的操作
///         let _vec = vec![0u8; 1024 * 1024]; // 1MB
///         Ok(())
///     })?;
///
///     println!("Estimated memory usage: {} bytes", memory_bytes);
///     result
/// }
/// ```
pub fn measure_test_memory<F>(name: &str, test_fn: F) -> Result<(Result<()>, usize)>
where
    F: FnOnce() -> Result<()>,
{
    let memory_before = estimate_memory_usage();
    let result = test_fn();
    let memory_after = estimate_memory_usage();

    let memory_used = if memory_after > memory_before {
        memory_after - memory_before
    } else {
        0
    };

    if memory_used > 10 * 1024 * 1024 {
        // 超过 10MB
        eprintln!(
            "Warning: Test '{}' used approximately {} bytes ({:.2} MB) of memory",
            name,
            memory_used,
            memory_used as f64 / (1024.0 * 1024.0)
        );
    }

    Ok((result, memory_used))
}
