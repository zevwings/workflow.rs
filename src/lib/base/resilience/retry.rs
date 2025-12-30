//! 重试工具模块
//!
//! 提供通用的重试机制，用于临时性错误的重试。
//! 主要用于 release/update 命令中的文件系统操作、文件解压、脚本执行等。

use color_eyre::{eyre::eyre, Result};
use std::time::Duration;

use crate::base::dialog::ConfirmDialog;
use crate::{trace_debug, trace_info, trace_warn};

/// 重试结果
#[derive(Debug, Clone)]
pub struct RetryResult<T> {
    /// 操作结果
    pub result: T,
    /// 重试次数
    pub retry_count: u32,
    /// 是否成功（第一次尝试就成功）
    pub succeeded_on_first_attempt: bool,
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: usize,
    /// 重试之间的延迟
    pub retry_delay: Duration,
    /// 是否使用指数退避
    pub exponential_backoff: bool,
    /// 是否启用交互式确认（默认：false）
    /// 如果为 true，在重试前会询问用户是否继续
    pub interactive: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            exponential_backoff: false,
            interactive: false,
        }
    }
}

impl RetryConfig {
    /// 创建新的重试配置
    pub fn new(max_retries: usize, retry_delay: Duration) -> Self {
        Self {
            max_retries,
            retry_delay,
            exponential_backoff: false,
            interactive: false,
        }
    }

    /// 启用指数退避
    pub fn with_exponential_backoff(mut self) -> Self {
        self.exponential_backoff = true;
        self
    }

    /// 启用交互式确认
    pub fn with_interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    /// 平台特定的默认配置（Windows 需要更多重试）
    pub fn platform_default() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                max_retries: 5,
                retry_delay: Duration::from_millis(300),
                exponential_backoff: true,
                interactive: false,
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self {
                max_retries: 3,
                retry_delay: Duration::from_millis(100),
                exponential_backoff: false,
                interactive: false,
            }
        }
    }
}

/// 带重试执行操作
///
/// 执行一个可能失败的操作，如果失败且错误可重试，则按照配置的重试策略进行重试。
///
/// # 参数
///
/// * `config` - 重试配置
/// * `operation` - 要执行的操作（闭包）
/// * `operation_name` - 操作名称（用于日志输出）
///
/// # 返回
///
/// 返回操作的结果和重试信息。如果所有重试都失败，返回最后一次的错误。
///
/// # 错误处理
///
/// - 如果错误不可重试，立即返回错误，不进行重试
/// - 如果所有重试都失败，返回最后一次的错误，并附加重试信息
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::resilience::{execute_with_retry, RetryConfig};
/// use std::time::Duration;
///
/// # fn main() -> color_eyre::Result<()> {
/// let config = RetryConfig::platform_default();
/// let result = execute_with_retry(
///     config,
///     || -> color_eyre::Result<String> {
///         // 可能失败的操作
///         Ok("success".to_string())
///     },
///     "File operation",
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn execute_with_retry<F, T>(
    config: RetryConfig,
    mut operation: F,
    operation_name: &str,
) -> Result<RetryResult<T>>
where
    F: FnMut() -> Result<T>,
{
    let mut delay = config.retry_delay;
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation() {
            Ok(result) => {
                if attempt > 0 {
                    trace_info!(
                        "{} succeeded after {} retry attempts",
                        operation_name,
                        attempt
                    );
                }
                return Ok(RetryResult {
                    result,
                    retry_count: attempt as u32,
                    succeeded_on_first_attempt: attempt == 0,
                });
            }
            Err(e) => {
                let error = e;
                let error_desc = get_error_description(&error);
                last_error = Some(error);

                // 检查是否可重试
                if let Some(ref err) = last_error {
                    if !is_retryable_error(err) {
                        // 错误不可重试，立即返回
                        if attempt == 0 {
                            trace_warn!(
                                "{} failed: {} (not retryable)",
                                operation_name,
                                error_desc
                            );
                        }
                        return Err(last_error.ok_or_else(|| {
                            eyre!("No error available but retryable check failed")
                        })?);
                    }
                }

                // 如果还有重试机会
                if attempt < config.max_retries {
                    trace_warn!(
                        "{} failed: {} (attempt {}/{})",
                        operation_name,
                        error_desc,
                        attempt + 1,
                        config.max_retries + 1
                    );

                    // 交互式确认：询问用户是否继续重试
                    if config.interactive && attempt > 0 {
                        let prompt = format!(
                            "是否在 {} 秒后重试？(尝试 {}/{})",
                            delay.as_secs(),
                            attempt + 1,
                            config.max_retries + 1
                        );
                        match ConfirmDialog::new(&prompt).with_default(true).prompt() {
                            Ok(true) => {
                                // 用户选择继续，等待
                                std::thread::sleep(delay);
                            }
                            Ok(false) => {
                                // 用户选择取消
                                trace_warn!("User cancelled operation");
                                return Err(eyre!("User cancelled operation"));
                            }
                            Err(e) => {
                                // 交互失败，可能是非交互式终端，直接继续
                                trace_debug!("Failed to get user input, auto-continuing: {}", e);
                                std::thread::sleep(delay);
                            }
                        }
                    } else {
                        // 非交互模式，直接等待
                        std::thread::sleep(delay);
                    }

                    // 更新延迟（指数退避）
                    if config.exponential_backoff {
                        delay *= 2;
                    }
                } else {
                    // 所有重试都失败了
                    trace_warn!(
                        "{} failed: {} (retried {} times)",
                        operation_name,
                        error_desc,
                        config.max_retries
                    );
                }
            }
        }
    }

    // 所有重试都失败，返回最后一次的错误，并添加上下文信息
    let final_error =
        last_error.ok_or_else(|| eyre!("All retries failed but no error available"))?;
    Err(final_error.wrap_err(format!(
        "{} failed after {} retries",
        operation_name, config.max_retries
    )))
}

/// 检查错误消息中是否包含资源不足相关的关键词
///
/// 资源不足错误不应该重试，因为重试会消耗更多资源。
#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn is_resource_error_message(error_msg: &str) -> bool {
    is_resource_error_message_impl(error_msg)
}

fn is_resource_error_message_impl(error_msg: &str) -> bool {
    let error_msg_lower = error_msg.to_lowercase();
    error_msg_lower.contains("resource temporarily unavailable")
        || error_msg_lower.contains("resource limit")
        || error_msg_lower.contains("too many")
        || error_msg_lower.contains("system resource")
        || error_msg_lower.contains("failed to create timeout thread")  // 完整短语
        || (error_msg_lower.contains("failed to create") && error_msg_lower.contains("thread"))  // 组合检查
        || error_msg.contains("(os error 35)")  // macOS EAGAIN - 括号格式（保持原样，因为数字不受大小写影响）
        || error_msg.contains("os error 35")  // macOS EAGAIN
        || error_msg.contains("(os error 11)")  // Linux EAGAIN - 括号格式
        || error_msg.contains("os error 11")  // Linux EAGAIN
        || error_msg.contains("error 10035") // Windows WSAEWOULDBLOCK
}

/// 检查错误代码是否是资源不足错误（EAGAIN, EWOULDBLOCK）
///
/// 这些错误不应该重试，因为重试会消耗更多资源。
#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn is_resource_error_code(error_code: Option<i32>) -> bool {
    is_resource_error_code_impl(error_code)
}

fn is_resource_error_code_impl(error_code: Option<i32>) -> bool {
    match error_code {
        #[cfg(target_os = "macos")]
        Some(35) => true, // macOS: EAGAIN = 35, EWOULDBLOCK = 35
        #[cfg(target_os = "linux")]
        Some(11) => true, // Linux: EAGAIN = 11, EWOULDBLOCK = 11
        #[cfg(target_os = "windows")]
        Some(10035) => true, // Windows: WSAEWOULDBLOCK = 10035
        _ => false,
    }
}

/// 判断错误是否可重试
///
/// 检查错误类型，判断是否应该重试。
/// 可重试的错误包括：
/// - 文件系统错误（权限、锁文件、临时不可用）
/// - IO 错误（超时、连接失败、临时性错误）
/// - 进程执行错误（临时性错误）
///
/// 不可重试的错误包括：
/// - 文件不存在（404）
/// - 权限不足（永久性）
/// - 参数错误
/// - 资源不足错误（不应该重试，因为重试会消耗更多资源）
/// - 其他永久性错误
///
/// # 参数
///
/// * `error` - 要检查的错误
///
/// # 返回
///
/// 返回 `true` 如果错误可重试，否则返回 `false`。
fn is_retryable_error(error: &color_eyre::eyre::Report) -> bool {
    // 首先检查错误消息（包括所有包装的错误）
    // 这必须在其他检查之前，因为错误可能被多层包装
    let error_msg = error.to_string().to_lowercase();

    // 检查资源不足相关的错误，这些不应该重试
    // 注意：这些检查必须在其他检查之前，因为资源不足错误不应该重试
    if is_resource_error_message_impl(&error_msg) {
        return false;
    }

    // 检查是否是标准库 IO 错误
    if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
        // 检查错误代码（资源不足错误）
        if is_resource_error_code_impl(io_error.raw_os_error()) {
            return false;
        }

        match io_error.kind() {
            // 临时性错误，可重试
            std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::Interrupted
            | std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::NotConnected
            | std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::ResourceBusy => return true,
            // 永久性错误，不可重试
            std::io::ErrorKind::NotFound
            | std::io::ErrorKind::PermissionDenied
            | std::io::ErrorKind::InvalidInput
            | std::io::ErrorKind::InvalidData => return false,
            // 其他错误，根据错误消息判断
            _ => {
                let io_error_msg = io_error.to_string().to_lowercase();

                // 检查资源不足相关的错误消息
                if is_resource_error_message_impl(&io_error_msg) {
                    return false;
                }

                // 检查错误消息中是否包含可重试的关键词
                if io_error_msg.contains("lock")
                    || io_error_msg.contains("busy")
                    || io_error_msg.contains("temporarily")
                    || io_error_msg.contains("timeout")
                {
                    return true;
                }
            }
        }
    }

    // 检查错误消息中是否包含可重试的关键词
    // 注意：只有在不是资源不足错误的情况下才检查这些
    if error_msg.contains("lock")
        || error_msg.contains("busy")
        || (error_msg.contains("temporarily")
            && !error_msg.contains("resource temporarily unavailable"))
        || error_msg.contains("timeout")
        || error_msg.contains("connection")
    {
        return true;
    }

    // 检查是否是进程执行错误（可能是临时性错误）
    if error_msg.contains("process") || error_msg.contains("command") {
        // 某些进程错误可能是临时性的
        if error_msg.contains("timeout") || error_msg.contains("signal") {
            return true;
        }
    }

    false
}

/// 带超时和重试执行操作
///
/// 在超时保护下执行操作，如果失败则重试。
/// 每次重试都会重新应用超时保护，确保每次尝试都有超时限制。
///
/// # 参数
///
/// * `timeout_config` - 超时配置（每次尝试的超时时间）
/// * `retry_config` - 重试配置
/// * `operation` - 要执行的操作（闭包）
/// * `operation_name` - 操作名称（用于日志输出）
///
/// # 返回
///
/// 返回操作的结果和重试信息。如果所有重试都失败，返回最后一次的错误。
///
/// # 注意
///
/// 由于 Rust 的生命周期限制，此函数要求操作是 `Fn()` 而不是 `FnMut()`。
/// 如果需要可变状态，请在闭包内部使用 `Arc<Mutex<T>>`。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::resilience::{
///     execute_with_timeout_and_retry, TimeoutConfig, RetryConfig,
///     default_extract_timeout,
/// };
/// use std::time::Duration;
///
/// # fn main() -> color_eyre::Result<()> {
/// let timeout_config = TimeoutConfig::new(default_extract_timeout()).with_platform_specific();
/// let retry_config = RetryConfig::platform_default();
///
/// let result = execute_with_timeout_and_retry(
///     timeout_config,
///     retry_config,
///     || -> color_eyre::Result<String> {
///         // 可能卡住或失败的操作
///         Ok("success".to_string())
///     },
///     "Extracting archive",
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn execute_with_timeout_and_retry<T, F>(
    timeout_config: super::timeout::TimeoutConfig,
    retry_config: RetryConfig,
    operation: F,
    operation_name: &str,
) -> Result<RetryResult<T>>
where
    T: Send + 'static,
    F: Fn() -> Result<T> + Send + Sync + 'static,
{
    use super::timeout::execute_with_timeout;
    use std::sync::Arc;
    use std::time::Instant;

    let mut delay = retry_config.retry_delay;
    let mut last_error: Option<color_eyre::eyre::Report> = None;
    let operation = Arc::new(operation);

    // 记录开始时间，用于计算总超时时间
    let start_time = Instant::now();
    let single_timeout = timeout_config.actual_timeout();

    // 计算总超时时间：单次超时 × (重试次数 + 1) + 重试延迟时间（估算）
    // 估算延迟时间：假设指数退避，总延迟 ≈ retry_delay × (2^(max_retries) - 1)
    let estimated_delay: Duration = if retry_config.exponential_backoff {
        let mut total_delay = Duration::from_secs(0);
        let mut current_delay = retry_config.retry_delay;
        for _ in 0..retry_config.max_retries {
            total_delay += current_delay;
            current_delay *= 2;
        }
        total_delay
    } else {
        retry_config.retry_delay * retry_config.max_retries as u32
    };
    let total_timeout = single_timeout * (retry_config.max_retries + 1) as u32 + estimated_delay;

    for attempt in 0..=retry_config.max_retries {
        // 检查总超时时间（防止总超时时间过长）
        if start_time.elapsed() > total_timeout {
            if let Some(err) = last_error {
                return Err(err.wrap_err(format!(
                    "{} exceeded total timeout of {:?} seconds (single timeout: {:?}, max retries: {})",
                    operation_name,
                    total_timeout.as_secs(),
                    single_timeout.as_secs(),
                    retry_config.max_retries
                )));
            } else {
                return Err(eyre!(
                    "{} exceeded total timeout of {:?} seconds (single timeout: {:?}, max retries: {})",
                    operation_name,
                    total_timeout.as_secs(),
                    single_timeout.as_secs(),
                    retry_config.max_retries
                ));
            }
        }

        let op = operation.clone();
        let timeout_cfg = timeout_config.clone();

        match execute_with_timeout(timeout_cfg, move || op()) {
            Ok(result) => {
                if attempt > 0 {
                    trace_info!(
                        "{} succeeded after {} retry attempts",
                        operation_name,
                        attempt
                    );
                }
                return Ok(RetryResult {
                    result,
                    retry_count: attempt as u32,
                    succeeded_on_first_attempt: attempt == 0,
                });
            }
            Err(e) => {
                let error = e;
                let error_desc = get_error_description(&error);
                last_error = Some(error);

                // 检查是否可重试
                if let Some(ref err) = last_error {
                    if !is_retryable_error(err) {
                        // 错误不可重试，立即返回（包括资源不足错误）
                        // 对于资源不足错误，无论是否是第一次尝试，都应该立即返回
                        // 输出完整的错误消息以便调试
                        let full_error_msg = err.to_string();
                        trace_warn!(
                            "{} failed: {} (not retryable, stopping immediately)",
                            operation_name,
                            error_desc
                        );
                        trace_warn!("Full error message: {}", full_error_msg);
                        return Err(last_error.ok_or_else(|| {
                            eyre!("No error available but retryable check failed")
                        })?);
                    }
                }

                // 如果还有重试机会
                if attempt < retry_config.max_retries {
                    trace_warn!(
                        "{} failed: {} (attempt {}/{})",
                        operation_name,
                        error_desc,
                        attempt + 1,
                        retry_config.max_retries + 1
                    );

                    // 交互式确认：询问用户是否继续重试
                    if retry_config.interactive && attempt > 0 {
                        let prompt = format!(
                            "是否在 {} 秒后重试？(尝试 {}/{})",
                            delay.as_secs(),
                            attempt + 1,
                            retry_config.max_retries + 1
                        );
                        match ConfirmDialog::new(&prompt).with_default(true).prompt() {
                            Ok(true) => {
                                // 用户选择继续，等待
                                std::thread::sleep(delay);
                            }
                            Ok(false) => {
                                // 用户选择取消
                                trace_warn!("User cancelled operation");
                                return Err(eyre!("User cancelled operation"));
                            }
                            Err(e) => {
                                // 交互失败，可能是非交互式终端，直接继续
                                trace_debug!("Failed to get user input, auto-continuing: {}", e);
                                std::thread::sleep(delay);
                            }
                        }
                    } else {
                        // 非交互模式，直接等待
                        std::thread::sleep(delay);
                    }

                    // 更新延迟（指数退避）
                    if retry_config.exponential_backoff {
                        delay *= 2;
                    }
                } else {
                    // 所有重试都失败了
                    trace_warn!(
                        "{} failed: {} (retried {} times)",
                        operation_name,
                        error_desc,
                        retry_config.max_retries
                    );
                }
            }
        }
    }

    // 所有重试都失败，返回最后一次的错误，并添加上下文信息
    let final_error =
        last_error.ok_or_else(|| eyre!("All retries failed but no error available"))?;
    Err(final_error.wrap_err(format!(
        "{} failed after {} retries",
        operation_name, retry_config.max_retries
    )))
}

/// 从错误中提取可读的错误描述
///
/// 尝试从错误中提取有用的信息，用于日志输出。
/// 优先显示底层错误（如资源不足错误），然后显示包装的错误。
///
/// # 参数
///
/// * `error` - 要提取描述的错误
///
/// # 返回
///
/// 返回错误的描述，优先显示底层错误信息。
fn get_error_description(error: &color_eyre::eyre::Report) -> String {
    // 首先尝试从 IO 错误中提取信息
    if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
        let error_code = io_error.raw_os_error();
        let kind = io_error.kind();

        // 如果是资源不足错误，显示详细信息
        #[cfg(target_os = "macos")]
        {
            if error_code == Some(35) {
                return format!(
                    "IO error: {} (os error 35 - Resource temporarily unavailable)",
                    kind
                );
            }
        }
        #[cfg(target_os = "linux")]
        {
            if error_code == Some(11) {
                return format!(
                    "IO error: {} (os error 11 - Resource temporarily unavailable)",
                    kind
                );
            }
        }
        #[cfg(target_os = "windows")]
        {
            if error_code == Some(10035) {
                return format!(
                    "IO error: {} (os error 10035 - Resource temporarily unavailable)",
                    kind
                );
            }
        }

        return format!("IO error: {}", kind);
    }

    // 获取完整的错误消息（包括所有包装的错误）
    let error_msg = error.to_string();

    // 检查是否包含资源不足相关的错误，优先显示这些信息
    let error_msg_lower = error_msg.to_lowercase();
    if error_msg_lower.contains("failed to create timeout thread")
        || error_msg_lower.contains("resource temporarily unavailable")
        || error_msg_lower.contains("os error 35")
        || error_msg_lower.contains("os error 11")
        || error_msg_lower.contains("error 10035")
    {
        // 提取包含资源不足错误的部分
        // 查找 "Failed to create timeout thread" 或 "Resource temporarily unavailable" 的位置
        if let Some(pos) = error_msg_lower.find("failed to create timeout thread") {
            // 从该位置开始提取，最多 200 个字符
            let start = pos;
            let end = (start + 200).min(error_msg.len());
            return error_msg[start..end].trim().to_string();
        }
        if let Some(pos) = error_msg_lower.find("resource temporarily unavailable") {
            // 从该位置开始提取，最多 200 个字符
            let start = pos.saturating_sub(50); // 向前提取一些上下文
            let end = (pos + 200).min(error_msg.len());
            return error_msg[start..end].trim().to_string();
        }
    }

    // 默认返回错误消息的前 150 个字符（增加长度以显示更多信息）
    if error_msg.len() > 150 {
        format!("{}...", &error_msg[..150])
    } else {
        error_msg
    }
}

// 注意：所有 public 方法的测试已迁移到 tests/base/resilience/retry.rs
