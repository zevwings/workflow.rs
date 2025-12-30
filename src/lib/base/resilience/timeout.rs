//! 超时工具模块
//!
//! 提供通用的超时机制，用于防止操作卡住。
//! 主要用于 release/update 命令中的文件下载、解压、文件系统操作等。

use color_eyre::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 全局活跃的超时操作数量
///
/// 用于限制并发超时操作的数量，防止创建过多线程导致资源泄漏。
#[cfg(test)]
pub(crate) static ACTIVE_TIMEOUT_OPERATIONS: AtomicUsize = AtomicUsize::new(0);
#[cfg(not(test))]
static ACTIVE_TIMEOUT_OPERATIONS: AtomicUsize = AtomicUsize::new(0);

/// 最大并发超时操作数量
///
/// 限制同时进行的超时操作数量，防止线程泄漏。
/// 这个值可以根据系统资源调整。
/// - 生产环境：默认设置为 10
/// - 测试环境：设置为 50（允许更多并发以支持并行测试）
#[cfg(test)]
const MAX_CONCURRENT_TIMEOUT_OPERATIONS: usize = 50;
#[cfg(not(test))]
const MAX_CONCURRENT_TIMEOUT_OPERATIONS: usize = 10;

/// 超时配置
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// 超时时间
    pub timeout: Duration,
    /// 是否使用平台特定超时（如果为 true，会根据平台自动调整）
    pub platform_specific: bool,
}

impl TimeoutConfig {
    /// 创建新的超时配置
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            platform_specific: false,
        }
    }

    /// 启用平台特定超时
    pub fn with_platform_specific(mut self) -> Self {
        self.platform_specific = true;
        self
    }

    /// 获取实际的超时时间（考虑平台特定调整）
    pub fn actual_timeout(&self) -> Duration {
        if self.platform_specific {
            // Windows 上使用 1.5 倍超时时间
            #[cfg(target_os = "windows")]
            {
                self.timeout * 3 / 2
            }
            #[cfg(not(target_os = "windows"))]
            {
                self.timeout
            }
        } else {
            self.timeout
        }
    }
}

/// 带超时执行操作
///
/// 在独立线程中执行操作，主线程监控超时。如果操作超时，返回错误。
///
/// # 参数
///
/// * `config` - 超时配置
/// * `operation` - 要执行的操作（返回 `Result<T>`）
///
/// # 返回
///
/// 成功时返回操作的结果，超时时返回错误
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::resilience::{execute_with_timeout, TimeoutConfig, default_download_timeout};
/// use std::time::Duration;
///
/// # fn main() -> color_eyre::Result<()> {
/// let result = execute_with_timeout(
///     TimeoutConfig::new(default_download_timeout()).with_platform_specific(),
///     || -> color_eyre::Result<String> {
///         // 可能卡住的操作
///         Ok("success".to_string())
///     }
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn execute_with_timeout<T, F>(config: TimeoutConfig, operation: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    // 检查并发限制
    let current = ACTIVE_TIMEOUT_OPERATIONS.fetch_add(1, Ordering::SeqCst);
    if current >= MAX_CONCURRENT_TIMEOUT_OPERATIONS {
        // 超过限制，立即减少计数并返回错误
        ACTIVE_TIMEOUT_OPERATIONS.fetch_sub(1, Ordering::SeqCst);
        return Err(color_eyre::eyre::eyre!(
            "Too many concurrent timeout operations (max: {}). \
            Please wait for some operations to complete or reduce concurrent operations.",
            MAX_CONCURRENT_TIMEOUT_OPERATIONS
        ));
    }

    // 确保在函数返回时减少计数（无论成功或失败）
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            ACTIVE_TIMEOUT_OPERATIONS.fetch_sub(1, Ordering::SeqCst);
        }
    }
    let _guard = Guard;

    let timeout = config.actual_timeout();
    let result: Arc<Mutex<Option<Result<T>>>> = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    // 尝试创建线程，捕获资源不足的错误
    let handle = thread::Builder::new()
        .name("timeout-worker".to_string())
        .spawn(move || {
            let op_result = operation();
            *result_clone.lock().unwrap() = Some(op_result);
        })
        .map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to create timeout thread: {}. \
                This may indicate system resource limits (threads, memory, or ulimit). \
                Try reducing concurrent operations or check system limits with 'ulimit -a'.",
                e
            )
        })?;

    // 等待操作完成或超时
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Ok(guard) = result.lock() {
            if guard.is_some() {
                break;
            }
        }
        thread::sleep(Duration::from_millis(10));
    }

    // 检查是否超时（在获取结果前检查，减少竞态条件）
    let timed_out = !handle.is_finished();

    // 如果超时，尝试等待一小段时间（最多100ms），看看操作是否能在短时间内完成
    // 这样可以减少线程泄漏，同时不会显著影响超时行为
    // 改进：每 10ms 检查一次结果，如果完成则立即返回
    if timed_out {
        let mut waited = Duration::from_millis(0);
        let max_wait = Duration::from_millis(100);
        let mut operation_completed = false;

        while waited < max_wait && !handle.is_finished() {
            thread::sleep(Duration::from_millis(10));
            waited += Duration::from_millis(10);

            // 每 10ms 检查一次结果，如果完成则立即退出等待循环
            // 这样可以更快地检测到操作完成，减少不必要的等待
            if let Ok(guard) = result.lock() {
                if guard.is_some() {
                    operation_completed = true;
                    break;
                }
            }
        }

        // 如果操作仍未完成，返回超时错误
        if !operation_completed && !handle.is_finished() {
            // 线程句柄会被 drop，线程会被标记为 detached
            // 这是必要的权衡，因为等待线程完成可能会导致无限期阻塞
            return Err(color_eyre::eyre::eyre!(
                "Operation timed out after {:?} seconds. \
                This may indicate a network issue, slow file system, or the operation is taking longer than expected.",
                timeout.as_secs()
            ));
        }
        // 如果操作在等待期间完成了，继续处理结果
    }

    // 无论是否超时，都尝试获取结果（可能操作已经完成）
    // 先检查结果，再检查线程状态，减少竞态条件
    let op_result = result
        .lock()
        .map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to acquire result lock: {:?}. This may indicate a panic in the operation thread.",
                e
            )
        })?
        .take();

    // 操作已完成（未超时或超时后完成），等待线程完成以确保资源被正确清理
    // 这对于防止线程泄漏非常重要，特别是在重试场景中
    if let Err(e) = handle.join() {
        return Err(color_eyre::eyre::eyre!("Thread panicked: {:?}", e));
    }

    // 获取结果：先解包 Option，然后返回内部的 Result
    match op_result {
        Some(result) => result,
        None => Err(color_eyre::eyre::eyre!(
            "Failed to get operation result. This may indicate a race condition or the operation did not complete properly."
        )),
    }
}

/// 平台特定的默认下载超时时间
///
/// 用于文件下载操作（大文件需要更长时间）。
/// 这是整个下载过程的最大超时时间。
pub fn default_download_timeout() -> Duration {
    #[cfg(target_os = "windows")]
    {
        Duration::from_secs(600) // 10 分钟（大文件下载可能需要更长时间）
    }
    #[cfg(not(target_os = "windows"))]
    {
        Duration::from_secs(600) // 10 分钟（大文件下载可能需要更长时间）
    }
}

/// 平台特定的单次读取超时时间
///
/// 用于检测单次读取操作是否卡住。
/// 如果单次读取超过这个时间，认为网络可能有问题。
pub fn default_read_timeout() -> Duration {
    #[cfg(target_os = "windows")]
    {
        Duration::from_secs(30) // 30 秒
    }
    #[cfg(not(target_os = "windows"))]
    {
        Duration::from_secs(30) // 30 秒
    }
}

/// 平台特定的默认解压超时时间
///
/// 用于文件解压操作（大文件解压需要时间）。
pub fn default_extract_timeout() -> Duration {
    #[cfg(target_os = "windows")]
    {
        Duration::from_secs(60) // 1 分钟
    }
    #[cfg(not(target_os = "windows"))]
    {
        Duration::from_secs(30) // 30 秒
    }
}

/// 平台特定的默认文件系统操作超时时间
///
/// 用于文件系统操作（创建目录、删除文件等）。
pub fn default_filesystem_timeout() -> Duration {
    #[cfg(target_os = "windows")]
    {
        Duration::from_secs(10) // 10 秒
    }
    #[cfg(not(target_os = "windows"))]
    {
        Duration::from_secs(5) // 5 秒
    }
}

/// 平台特定的默认脚本执行超时时间
///
/// 用于脚本执行操作（如 `./install`）。
pub fn default_script_timeout() -> Duration {
    #[cfg(target_os = "windows")]
    {
        Duration::from_secs(60) // 1 分钟
    }
    #[cfg(not(target_os = "windows"))]
    {
        Duration::from_secs(30) // 30 秒
    }
}

// 注意：所有 public 方法的测试已迁移到 tests/base/resilience/timeout.rs
