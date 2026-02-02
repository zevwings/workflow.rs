//! HTTP 重试逻辑

use std::time::{Duration, Instant};

use backon::{BlockingRetryable, ExponentialBuilder};

use crate::http::retry::config::{HttpRetryConfig, RetryResult};
use crate::http::retry::error::HttpRetryError;
use crate::http::HttpError;

/// HTTP 重试工具
///
/// 提供 HTTP 请求重试的功能，支持指数退避算法。
/// 专门针对 HTTP 请求的错误类型进行智能判断。
pub struct HttpRetry;

impl HttpRetry {
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
    fn is_retryable_error<E: std::error::Error + 'static>(error: &E) -> bool {
        // 尝试从错误链中找到 HttpError
        let mut current: Option<&dyn std::error::Error> = Some(error);
        while let Some(err) = current {
            if let Some(http_error) = err.downcast_ref::<HttpError>() {
                return match http_error {
                    // 客户端和网络错误
                    HttpError::Timeout { .. }
                    | HttpError::ConnectionFailed { .. }
                    | HttpError::RateLimitExceeded { .. } => true,
                    HttpError::CreateClientFailed(e) => {
                        e.is_timeout() || e.is_connect() || e.is_request()
                    }
                    HttpError::RequestFailed { source, .. } => {
                        source.is_timeout() || source.is_connect() || source.is_request()
                    }
                    // 响应处理错误
                    HttpError::ResponseFailed { status, .. } => *status >= 500 || *status == 429,
                    HttpError::HttpRequestFailed(status) => *status >= 500 || *status == 429,
                    _ => false,
                };
            }

            if let Some(reqwest_error) = err.downcast_ref::<reqwest::Error>() {
                if reqwest_error.is_timeout()
                    || reqwest_error.is_connect()
                    || reqwest_error.is_request()
                {
                    return true;
                }

                if let Some(status) = reqwest_error.status() {
                    return status.is_server_error() || status.as_u16() == 429;
                }
            }

            if let Some(io_error) = err.downcast_ref::<std::io::Error>() {
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

            current = err.source();
        }

        false
    }

    /// 从错误中提取可读的错误描述
    ///
    /// 尝试从错误中提取有用的信息，用于日志输出。
    fn get_error_description<E: std::error::Error + 'static>(error: &E) -> String {
        const MAX_ERROR_MSG_LENGTH: usize = 100;
        const ELLIPSIS: &str = "...";

        // 尝试从错误链中找到更具体的错误类型
        let mut current: Option<&dyn std::error::Error> = Some(error);
        while let Some(err) = current {
            if let Some(http_error) = err.downcast_ref::<HttpError>() {
                return http_error.to_string();
            }

            if let Some(reqwest_error) = err.downcast_ref::<reqwest::Error>() {
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

            if let Some(io_error) = err.downcast_ref::<std::io::Error>() {
                return format!("IO error: {}", io_error.kind());
            }

            current = err.source();
        }

        let error_msg = error.to_string();
        if error_msg.len() > MAX_ERROR_MSG_LENGTH {
            format!("{}{}", &error_msg[..MAX_ERROR_MSG_LENGTH], ELLIPSIS)
        } else {
            error_msg
        }
    }

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
    /// * `F` - 操作闭包类型，必须返回 `Result<T, E>`
    /// * `T` - 操作返回值的类型
    /// * `E` - 错误类型，必须实现 `std::error::Error`
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
    /// use toolkit::http::retry::{HttpRetry, HttpRetryConfig};
    /// use toolkit::http::HttpError;
    ///
    /// # fn main() -> Result<(), HttpError> {
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
    pub fn retry<F, T, E>(
        operation: F,
        config: &HttpRetryConfig,
        operation_name: &str,
    ) -> Result<RetryResult<T>, HttpRetryError>
    where
        F: Fn() -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
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

        // 如果 max_retries 为 0，直接执行操作，不进行重试
        if config.max_retries == 0 {
            let result = operation().map_err(|e| HttpRetryError::Other(e.to_string()))?;
            let duration = start.elapsed();

            tracing::info!(
                module = "http",
                operation = %operation_name,
                retry_count = 0,
                succeeded_on_first_attempt = true,
                http.retry.duration_ms = duration.as_millis(),
                "Operation succeeded"
            );

            return Ok(RetryResult {
                result,
                retry_count: 0,
                succeeded_on_first_attempt: true,
            });
        }

        // 跟踪重试次数
        let retry_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let retry_count_clone = retry_count.clone();
        let operation_name_clone = operation_name.to_string();

        // 配置指数退避策略
        let retry_policy = ExponentialBuilder::default()
            .with_max_times(config.max_retries as usize + 1) // +1 因为包含初始尝试
            .with_min_delay(Duration::from_secs(config.initial_delay))
            .with_max_delay(Duration::from_secs(config.max_delay))
            .with_factor(config.backoff_multiplier as f32)
            .with_jitter(); // 添加抖动以避免雷群效应

        // 包装操作以跟踪重试次数和记录日志
        let wrapped_operation = move || -> Result<T, E> {
            let attempt = retry_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

            let attempt_span = tracing::span!(
                tracing::Level::DEBUG,
                "http.retry.attempt",
                module = "http",
                operation = %operation_name_clone,
                attempt = attempt,
            );
            let _attempt_guard = attempt_span.enter();

            operation()
        };

        // 使用 backon 执行重试
        let operation_name_for_notify = operation_name.to_string();
        let result = wrapped_operation
            .retry(&retry_policy)
            .when(|err: &E| Self::is_retryable_error(err))
            .notify(move |err: &E, duration: Duration| {
                let error_desc = Self::get_error_description(err);
                let delay_seconds = duration.as_secs();

                tracing::warn!(
                    module = "http",
                    operation = %operation_name_for_notify,
                    error = %error_desc,
                    delay_seconds = delay_seconds,
                    "Operation failed, will retry"
                );
            })
            .call()
            .map_err(|e| {
                let final_retry_count = retry_count.load(std::sync::atomic::Ordering::SeqCst);
                let duration = start.elapsed();
                let error_desc = Self::get_error_description(&e);

                tracing::error!(
                    module = "http",
                    operation = %operation_name,
                    retry_count = final_retry_count.saturating_sub(1), // 减去初始尝试
                    error = %error_desc,
                    http.retry.duration_ms = duration.as_millis(),
                    "Operation failed after all retries"
                );

                HttpRetryError::OperationFailedAfterRetries {
                    operation: operation_name.to_string(),
                    retries: final_retry_count.saturating_sub(1),
                    error_message: e.to_string(),
                }
            })?;

        let final_retry_count = retry_count.load(std::sync::atomic::Ordering::SeqCst);
        let duration = start.elapsed();
        let actual_retry_count = final_retry_count.saturating_sub(1); // 减去初始尝试

        if actual_retry_count > 0 {
            tracing::info!(
                module = "http",
                operation = %operation_name,
                retry_count = actual_retry_count,
                succeeded_on_first_attempt = false,
                http.retry.duration_ms = duration.as_millis(),
                "Operation succeeded after retries"
            );
        } else {
            tracing::info!(
                module = "http",
                operation = %operation_name,
                retry_count = 0,
                succeeded_on_first_attempt = true,
                http.retry.duration_ms = duration.as_millis(),
                "Operation succeeded"
            );
        }

        Ok(RetryResult {
            result,
            retry_count: actual_retry_count,
            succeeded_on_first_attempt: actual_retry_count == 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::http::HttpError;

    #[test]
    fn test_retry_succeeds_on_first_attempt() {
        let config = HttpRetryConfig::new();
        let result = HttpRetry::retry(|| Ok::<i32, HttpError>(42), &config, "test");
        assert!(result.is_ok());
        let retry_result = result.unwrap();
        assert_eq!(retry_result.result, 42);
        assert_eq!(retry_result.retry_count, 0);
        assert!(retry_result.succeeded_on_first_attempt);
    }

    #[test]
    fn test_retry_succeeds_after_failures() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let mut config = HttpRetryConfig::new();
        config.max_retries = 3;
        let result = HttpRetry::retry(
            || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                if *count < 3 {
                    Err(HttpError::Timeout {
                        url: "test".to_string(),
                        method: "GET".to_string(),
                    })
                } else {
                    Ok(42)
                }
            },
            &config,
            "test",
        );

        assert!(result.is_ok());
        let retry_result = result.unwrap();
        assert_eq!(retry_result.result, 42);
        assert_eq!(retry_result.retry_count, 2);
        assert!(!retry_result.succeeded_on_first_attempt);
    }

    #[test]
    fn test_retry_fails_after_max_retries() {
        let config = HttpRetryConfig::new().with_max_retries(2);
        let result = HttpRetry::retry(
            || {
                Err::<i32, HttpError>(HttpError::Timeout {
                    url: "test".to_string(),
                    method: "GET".to_string(),
                })
            },
            &config,
            "test",
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            HttpRetryError::OperationFailedAfterRetries { .. }
        ));
    }

    #[test]
    fn test_non_retryable_error_not_retried() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let config = HttpRetryConfig::new().with_max_retries(3);
        let result = HttpRetry::retry(
            || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                Err::<i32, HttpError>(HttpError::HttpRequestFailed(404))
            },
            &config,
            "test",
        );

        assert!(result.is_err());
        // 应该只尝试一次，因为 404 不可重试
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_server_error_is_retryable() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let config = HttpRetryConfig::new().with_max_retries(2);
        let result = HttpRetry::retry(
            || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                if *count < 2 {
                    Err(HttpError::HttpRequestFailed(500))
                } else {
                    Ok(42)
                }
            },
            &config,
            "test",
        );

        assert!(result.is_ok());
        let retry_result = result.unwrap();
        assert_eq!(retry_result.result, 42);
        assert_eq!(retry_result.retry_count, 1);
    }

    #[test]
    fn test_rate_limit_is_retryable() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let config = HttpRetryConfig::new().with_max_retries(2);
        let result = HttpRetry::retry(
            || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                if *count < 2 {
                    Err(HttpError::HttpRequestFailed(429))
                } else {
                    Ok(42)
                }
            },
            &config,
            "test",
        );

        assert!(result.is_ok());
        let retry_result = result.unwrap();
        assert_eq!(retry_result.result, 42);
        assert_eq!(retry_result.retry_count, 1);
    }

    #[test]
    fn test_zero_max_retries() {
        let config = HttpRetryConfig::new().with_max_retries(0);
        let result = HttpRetry::retry(|| Ok::<i32, HttpError>(42), &config, "test");
        assert!(result.is_ok());
        let retry_result = result.unwrap();
        assert_eq!(retry_result.result, 42);
        assert_eq!(retry_result.retry_count, 0);
        assert!(retry_result.succeeded_on_first_attempt);
    }

    #[test]
    fn test_zero_max_retries_with_error() {
        let config = HttpRetryConfig::new().with_max_retries(0);
        let result = HttpRetry::retry(
            || {
                Err::<i32, HttpError>(HttpError::Timeout {
                    url: "test".to_string(),
                    method: "GET".to_string(),
                })
            },
            &config,
            "test",
        );
        assert!(result.is_err());
    }
}
