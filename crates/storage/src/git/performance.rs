//! 性能监控工具
//!
//! 提供性能监控、日志记录和分析工具。

#![allow(dead_code)]

use std::time::{Duration, Instant};

/// 性能计时器
pub struct PerformanceTimer {
    name: String,
    start: Instant,
    threshold: Option<Duration>,
}

impl PerformanceTimer {
    /// 创建新的计时器
    ///
    /// # 参数
    /// - `name`: 操作名称
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            threshold: None,
        }
    }

    /// 设置警告阈值
    ///
    /// 如果操作耗时超过阈值，将输出警告日志
    pub fn with_threshold(mut self, threshold: Duration) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// 停止计时并返回耗时
    pub fn stop(self) -> Duration {
        let duration = self.start.elapsed();

        // 检查是否超过阈值
        if let Some(threshold) = self.threshold {
            if duration > threshold {
                eprintln!(
                    "[WARN] {} took too long: {:?} (threshold: {:?})",
                    self.name, duration, threshold
                );
            }
        }

        duration
    }

    /// 记录中间点
    pub fn checkpoint(&self, label: &str) -> Duration {
        let duration = self.start.elapsed();
        eprintln!("[PERF] {} - {}: {:?}", self.name, label, duration);
        duration
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        if let Some(threshold) = self.threshold {
            if duration > threshold {
                eprintln!(
                    "[WARN] {} took too long: {:?} (threshold: {:?})",
                    self.name, duration, threshold
                );
            }
        }
    }
}

/// 测量函数执行时间
///
/// # 示例
/// ```rust,no_run
/// use storage::git::performance::measure;
///
/// let result = measure("get_commit_info", || {
///     // 执行操作
///     42
/// });
/// ```
pub fn measure<F, T>(name: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let timer = PerformanceTimer::new(name);
    let result = f();
    let duration = timer.stop();
    eprintln!("[PERF] {} completed, elapsed: {:?}", name, duration);
    result
}

/// 测量函数执行时间，带阈值警告
pub fn measure_with_threshold<F, T>(name: &str, threshold: Duration, f: F) -> T
where
    F: FnOnce() -> T,
{
    let timer = PerformanceTimer::new(name).with_threshold(threshold);
    let result = f();
    let duration = timer.stop();

    if duration <= threshold {
        eprintln!("[PERF] {} completed, elapsed: {:?}", name, duration);
    }

    result
}

/// 性能统计信息
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub name: String,
    pub count: usize,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
}

impl PerformanceStats {
    /// 计算平均值
    pub fn calculate_avg(&mut self) {
        if self.count > 0 {
            self.avg_duration = self.total_duration / self.count as u32;
        }
    }

    /// 打印统计信息
    pub fn print(&self) {
        println!("\n=== Performance Stats: {} ===", self.name);
        println!("  Call count: {}", self.count);
        println!("  Total time: {:?}", self.total_duration);
        println!("  Avg time: {:?}", self.avg_duration);
        println!("  Min time: {:?}", self.min_duration);
        println!("  Max time: {:?}", self.max_duration);
    }
}

/// 性能统计收集器
pub struct PerformanceCollector {
    samples: Vec<Duration>,
    name: String,
}

impl PerformanceCollector {
    /// 创建新的收集器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            samples: Vec::new(),
            name: name.into(),
        }
    }

    /// 添加样本
    pub fn add_sample(&mut self, duration: Duration) {
        self.samples.push(duration);
    }

    /// 测量并添加样本
    pub fn measure<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        self.add_sample(duration);
        result
    }

    /// 生成统计信息
    pub fn stats(&self) -> PerformanceStats {
        let count = self.samples.len();
        let total_duration = self.samples.iter().sum();
        let min_duration = self.samples.iter().min().copied().unwrap_or_default();
        let max_duration = self.samples.iter().max().copied().unwrap_or_default();
        let avg_duration = if count > 0 {
            total_duration / count as u32
        } else {
            Duration::default()
        };

        PerformanceStats {
            name: self.name.clone(),
            count,
            total_duration,
            min_duration,
            max_duration,
            avg_duration,
        }
    }

    /// 打印统计信息
    pub fn print_stats(&self) {
        self.stats().print();
    }

    /// 清空样本
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// 性能测试宏
///
/// # 示例
/// ```rust,no_run
/// use storage::perf_test;
///
/// perf_test!("my_operation", {
///     // 执行操作
/// });
/// ```
#[macro_export]
macro_rules! perf_test {
    ($name:expr, $code:block) => {{
        use std::time::Instant;
        let start = Instant::now();
        let result = $code;
        let duration = start.elapsed();
        eprintln!("[PERF] {} elapsed: {:?}", $name, duration);
        result
    }};

    ($name:expr, $threshold:expr, $code:block) => {{
        use std::time::Instant;
        let start = Instant::now();
        let result = $code;
        let duration = start.elapsed();

        if duration > $threshold {
            eprintln!(
                "[WARN] {} took too long: {:?} (threshold: {:?})",
                $name, duration, $threshold
            );
        } else {
            eprintln!("[PERF] {} elapsed: {:?}", $name, duration);
        }

        result
    }};
}
