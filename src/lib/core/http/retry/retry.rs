//! HTTP 重试逻辑

use color_eyre::Result;
use std::time::{Duration, Instant};

use crate::core::http::retry::config::{HttpRetryConfig, RetryResult};
use crate::core::http::retry::error::HttpRetryError;

/// 判断错误是否可重试
///
/// 检查错误类型，判断是否应该重试。
/// 可重试的错误包括：
/// - 网络错误（超时、连接失败、请求中断）
/// - 5xx 服务器错误（500, 502, 503, 504）
/// - 429 Too Many Requests（需要特殊处理，使用 Retry-After header）
///
/// 不可重试的错误包括：
/// - 4xx 客户端错误（400, 401, 403, 404 等）
/// - 解析错误（JSON 解析失败、文件格式错误）
/// - 其他非网络错误
///
/// # 参数
///
/// * `error` - 要检查的错误
///
/// # 返回
///
/// 返回 `true` 如果错误可重试，否则返回 `false`。
fn is_retryable_error(error: &color_eyre::eyre::Report) -> bool {
    // 检查是否是 reqwest 网络错误
    if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
        // 检查是否是网络连接错误
        if reqwest_error.is_timeout() || reqwest_error.is_connect() || reqwest_error.is_request() {
            return true;
        }

        // 检查 HTTP 状态码
        if let Some(status) = reqwest_error.status() {
            // 5xx 服务器错误和 429 Too Many Requests 可重试
            return status.is_server_error() || status.as_u16() == 429;
        }
    }

    // 检查是否是标准库 IO 错误（可能是网络相关的）
    if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
        // 网络相关的 IO 错误可重试
        match io_error.kind() {
            std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::NotConnected
            | std::io::ErrorKind::BrokenPipe => return true,
            _ => {}
        }
    }

    false
}

/// 从错误中提取可读的错误描述
///
/// 尝试从错误中提取有用的信息，用于日志输出。
///
/// # 参数
///
/// * `error` - 要提取描述的错误
///
/// # 返回
///
/// 返回错误的简短描述。
fn get_error_description(error: &color_eyre::eyre::Report) -> String {
    const MAX_ERROR_MSG_LENGTH: usize = 100;
    const ELLIPSIS: &str = "...";

    // 尝试从 reqwest 错误中提取状态码
    if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
        if let Some(status) = reqwest_error.status() {
            return format!("HTTP {}", status.as_u16());
        }
        if reqwest_error.is_timeout() {
            return "Network timeout".to_string();
        }
        if reqwest_error.is_connect() {
            return "Connection failed".to_string();
        }
    }

    // 尝试从 IO 错误中提取信息
    if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
        return format!("IO error: {}", io_error.kind());
    }

    // 默认返回错误消息的前 N 个字符
    let error_msg = error.to_string();
    if error_msg.len() > MAX_ERROR_MSG_LENGTH {
        format!("{}{}", &error_msg[..MAX_ERROR_MSG_LENGTH], ELLIPSIS)
    } else {
        error_msg
    }
}

/// 从错误中提取 Retry-After header 的值
///
/// 对于 429 Too Many Requests 错误，尝试从响应中提取 Retry-After header。
/// Retry-After 可以是秒数（整数）或 HTTP 日期格式。
///
/// 注意：在 blocking 模式下，reqwest::Error 可能不包含响应信息。
/// 如果需要完整的 Retry-After 支持，建议在 HttpResponse 中检查 header。
///
/// # 参数
///
/// * `error` - 要检查的错误
///
/// # 返回
///
/// 如果找到 Retry-After header 且可以解析，返回秒数；否则返回 `None`。
fn extract_retry_after(error: &color_eyre::eyre::Report) -> Option<u64> {
    // 检查是否是 reqwest 错误，并且是 429 状态码
    if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
        if let Some(status) = reqwest_error.status() {
            if status.as_u16() == 429 {
                // 注意：在 blocking 模式下，reqwest::Error 可能不包含响应
                // 如果需要完整的 Retry-After 支持，建议在 HttpResponse 中检查
                // 这里返回 None，使用默认的指数退避
            }
        }
    }

    // 检查是否是 ClientHttpError::RateLimitExceeded
    // 注意：ClientHttpError 可能不包含响应信息，所以这里无法提取 Retry-After
    // 如果需要，可以在 ClientHttpError 中添加响应信息

    None
}

/// HTTP 重试工具
///
/// 提供 HTTP 请求重试的功能，支持指数退避算法。
/// 专门针对 HTTP 请求的错误类型进行智能判断。
pub struct HttpRetry;

impl HttpRetry {
    /// 使用指数退避算法重试 HTTP 操作
    ///
    /// 执行一个可能失败的 HTTP 操作，如果失败且错误可重试，则按照配置的重试策略进行重试。
    /// 使用指数退避算法，每次重试的延迟时间会逐渐增加，直到达到最大延迟。
    ///
    /// # 参数
    ///
    /// * `operation` - 要执行的操作（闭包）
    /// * `config` - 重试配置
    /// * `operation_name` - 操作名称（用于日志输出）
    ///
    /// # 类型参数
    ///
    /// * `F` - 操作闭包类型，必须返回 `Result<T>`
    /// * `T` - 操作返回值的类型
    ///
    /// # 返回
    ///
    /// 返回操作的结果和重试信息。如果所有重试都失败，返回最后一次的错误。
    ///
    /// # 错误处理
    ///
    /// - 如果错误不可重试（如 4xx 客户端错误），立即返回错误，不进行重试
    /// - 如果所有重试都失败，返回最后一次的错误，并附加重试信息
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::http::retry::{HttpRetry, HttpRetryConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = HttpRetryConfig::new();
    /// let result = HttpRetry::retry(
    ///     || {
    ///         // 执行可能失败的 HTTP 操作
    ///         Ok(42)
    ///     },
    ///     &config,
    ///     "获取数据",
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn retry<F, T>(
        operation: F,
        config: &HttpRetryConfig,
        operation_name: &str,
    ) -> Result<RetryResult<T>>
    where
        F: Fn() -> Result<T>,
    {
        let span = tracing::span!(
            tracing::Level::DEBUG,
            "http.retry",
            module = "http",
            operation = %operation_name,
            max_retries = config.max_retries,
        );
        let _guard = span.enter();
        let start = Instant::now();

        let mut delay = config.initial_delay;
        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            let attempt_span = tracing::span!(
                tracing::Level::DEBUG,
                "http.retry.attempt",
                module = "http",
                operation = %operation_name,
                attempt = attempt + 1,
                max_retries = config.max_retries + 1,
            );
            let _attempt_guard = attempt_span.enter();

            match operation() {
                Ok(result) => {
                    let duration = start.elapsed();
                    tracing::info!(
                        module = "http",
                        operation = %operation_name,
                        retry_count = attempt,
                        succeeded_on_first_attempt = attempt == 0,
                        http.retry.duration_ms = duration.as_millis(),
                        "Operation succeeded"
                    );
                    return Ok(RetryResult {
                        result,
                        retry_count: attempt,
                        succeeded_on_first_attempt: attempt == 0,
                    });
                }
                Err(e) => {
                    let error = e;
                    last_error = Some(error);

                    // 检查是否可重试
                    if let Some(ref err) = last_error {
                        if !is_retryable_error(err) {
                            // 错误不可重试，立即返回
                            tracing::warn!(
                                module = "http",
                                operation = %operation_name,
                                attempt = attempt + 1,
                                error = %err,
                                "Error is not retryable"
                            );
                            return Err(last_error.ok_or_else(
                                || -> color_eyre::eyre::Report {
                                    HttpRetryError::NoErrorAvailable.into()
                                },
                            )?);
                        }
                    }

                    // 如果还有重试机会
                    if attempt < config.max_retries {
                        if let Some(ref err) = last_error {
                            let error_desc = get_error_description(err);

                            // 检查是否是 429 错误，并尝试提取 Retry-After header
                            let retry_after = extract_retry_after(err);
                            let actual_delay = retry_after.unwrap_or(delay);

                            tracing::warn!(
                                module = "http",
                                operation = %operation_name,
                                attempt = attempt + 1,
                                max_retries = config.max_retries + 1,
                                error = %error_desc,
                                delay_seconds = actual_delay,
                                retry_after = retry_after.map(|d| d.to_string()),
                                "Operation failed, will retry"
                            );

                            // 等待后重试（使用 Retry-After 或指数退避延迟）
                            std::thread::sleep(Duration::from_secs(actual_delay));

                            // 如果使用了 Retry-After，下次仍然使用指数退避
                            // 否则继续指数退避
                            if retry_after.is_none() {
                                delay = ((delay as f64 * config.backoff_multiplier) as u64)
                                    .min(config.max_delay);
                            } else {
                                // 使用 Retry-After 后，重置延迟为初始延迟的倍数
                                delay = ((config.initial_delay as f64 * config.backoff_multiplier)
                                    as u64)
                                    .min(config.max_delay);
                            }
                        }
                    } else {
                        // 所有重试都失败了
                        if let Some(ref err) = last_error {
                            let error_desc = get_error_description(err);
                            tracing::error!(
                                module = "http",
                                operation = %operation_name,
                                retry_count = config.max_retries,
                                error = %error_desc,
                                "All retries exhausted"
                            );
                        }
                    }
                }
            }
        }

        // 所有重试都失败，返回最后一次的错误，并添加上下文信息
        let duration = start.elapsed();
        let final_error: color_eyre::eyre::Report =
            last_error.ok_or_else(|| -> color_eyre::eyre::Report {
                HttpRetryError::AllRetriesFailedNoError.into()
            })?;
        tracing::error!(
            module = "http",
            operation = %operation_name,
            retries = config.max_retries,
            http.retry.duration_ms = duration.as_millis(),
            "Operation failed after all retries"
        );
        Err(HttpRetryError::OperationFailedAfterRetries {
            operation: operation_name.to_string(),
            retries: config.max_retries,
            source: final_error,
        }
        .into())
    }
}
